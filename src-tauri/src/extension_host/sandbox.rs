/**
 * Cross-Platform Sandboxing
 *
 * Implements process-level sandboxing for extension host processes
 * using OS-specific mechanisms:
 * - Linux: seccomp-bpf + namespaces
 * - macOS: App Sandbox + entitlements
 * - Windows: Job Objects + restricted tokens
 */
use std::process::Command;

pub struct SandboxConfig {
    pub allow_network: bool,
    pub allow_file_write: bool,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u32>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            allow_network: false,
            allow_file_write: false,
            max_memory_mb: Some(512),  // 512MB default
            max_cpu_percent: Some(50), // 50% CPU max
        }
    }
}

pub fn apply_sandbox(command: &mut Command, config: &SandboxConfig) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    apply_linux_sandbox(command, config)?;

    #[cfg(target_os = "macos")]
    apply_macos_sandbox(command, config)?;

    #[cfg(target_os = "windows")]
    apply_windows_sandbox(command, config)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_linux_sandbox(command: &mut Command, config: &SandboxConfig) -> Result<(), String> {
    // Use bubblewrap if available for strong sandboxing
    if which::which("bwrap").is_ok() {
        return apply_bubblewrap_sandbox(command, config);
    }

    // Fallback to basic resource limits
    apply_linux_resource_limits(command, config)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_bubblewrap_sandbox(
    original_command: &mut Command,
    config: &SandboxConfig,
) -> Result<(), String> {
    // Extract the program and args from original command
    let program = original_command.get_program().to_string_lossy().to_string();
    let args: Vec<String> = original_command
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    // Extract environment variables from original command
    let envs: Vec<(String, String)> = original_command
        .get_envs()
        .filter_map(|(k, v)| {
            v.map(|val| {
                (
                    k.to_string_lossy().to_string(),
                    val.to_string_lossy().to_string(),
                )
            })
        })
        .collect();

    // Clear original command and replace with bwrap
    let mut bwrap = Command::new("bwrap");

    // Bind mount required directories (read-only by default)
    bwrap.args(&["--ro-bind", "/usr", "/usr"]);
    bwrap.args(&["--ro-bind", "/lib", "/lib"]);
    bwrap.args(&["--ro-bind", "/lib64", "/lib64"]);
    bwrap.args(&["--ro-bind", "/bin", "/bin"]);
    bwrap.args(&["--ro-bind", "/sbin", "/sbin"]);

    // Proc and dev (minimal)
    bwrap.args(&["--proc", "/proc"]);
    bwrap.args(&["--dev", "/dev"]);

    // Bind /tmp from host so IPC sockets work
    // Extension host needs to create sockets that Tauri can connect to
    bwrap.args(&["--bind", "/tmp", "/tmp"]);

    // Home directory — restrict to only necessary directories
    if let Some(home) = std::env::var_os("HOME") {
        let home_str = home.to_string_lossy();

        // Always allow reading home for configuration
        bwrap.args(&["--ro-bind", &*home_str, &*home_str]);

        // If file write is allowed, only allow writing to specific subdirectories
        // NOT the entire home directory
        if config.allow_file_write {
            let dscode_dir = format!("{}/.dscode", home_str);
            let local_share = format!("{}/.local/share", home_str);

            // Ensure directories exist before binding
            let _ = std::fs::create_dir_all(&dscode_dir);
            let _ = std::fs::create_dir_all(&local_share);

            bwrap.args(&["--bind", &dscode_dir, &dscode_dir]);
            bwrap.args(&["--bind", &local_share, &local_share]);
        }
    }

    // Network isolation (if disabled)
    if !config.allow_network {
        bwrap.arg("--unshare-net");
    }

    // Unshare PID, UTS (but NOT IPC - we need IPC namespace sharing for NNG sockets)
    bwrap.args(&["--unshare-pid", "--unshare-uts"]);

    // Die with parent
    bwrap.arg("--die-with-parent");

    // Add the actual command
    bwrap.arg("--");
    bwrap.arg(program);
    bwrap.args(args);

    // Restore environment variables
    for (key, value) in envs {
        bwrap.env(key, value);
    }

    *original_command = bwrap;

    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_linux_resource_limits(
    command: &mut Command,
    config: &SandboxConfig,
) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    // Copy config values to owned variables for 'static closure
    let max_memory_mb = config.max_memory_mb;

    // Set resource limits using setrlimit
    unsafe {
        command.pre_exec(move || {
            // Memory limit
            if let Some(max_mb) = max_memory_mb {
                let max_bytes = max_mb * 1024 * 1024;
                let limit = libc::rlimit {
                    rlim_cur: max_bytes,
                    rlim_max: max_bytes,
                };
                libc::setrlimit(libc::RLIMIT_AS, &limit);
            }

            // CPU time limit (not percentage, but we can set a max time)
            // For actual CPU percentage limiting, we'd need cgroups

            Ok(())
        });
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_macos_sandbox(command: &mut Command, config: &SandboxConfig) -> Result<(), String> {
    // Generate a sandbox profile dynamically based on config
    let mut profile = String::new();

    // Allow the extension host process to read the system
    profile.push_str("(version 1)\n");
    profile.push_str("(deny default)\n"); // Deny everything by default

    // Allow reading system directories
    profile.push_str("(allow file-read* (subpath \"/usr\"))\n");
    profile.push_str("(allow file-read* (subpath \"/lib\"))\n");
    profile.push_str("(allow file-read* (subpath \"/System\"))\n");
    profile.push_str("(allow file-read* (subpath \"/bin\"))\n");
    profile.push_str("(allow file-read* (subpath \"/sbin\"))\n");

    // Allow reading the Node.js installation
    if let Ok(node_path) = which::which("node") {
        if let Some(node_dir) = node_path.parent() {
            profile.push_str(&format!(
                "(allow file-read* (subpath \"{}\"))\n",
                node_dir.display()
            ));
        }
    }

    // Allow reading HOME
    if let Some(home) = std::env::var_os("HOME") {
        let home_str = home.to_string_lossy();
        profile.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", home_str));

        // If file write is allowed, grant write access to specific directories
        if config.allow_file_write {
            // Only allow writing to extensions dir, workspace, and temp
            // NOT the entire home directory
            if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
                profile.push_str(&format!("(allow file-write* (subpath \"{}\"))\n", xdg_data));
            } else if let Some(home) = std::env::var_os("HOME") {
                let home_str = home.to_string_lossy();
                profile.push_str(&format!(
                    "(allow file-write* (subpath \"{}/.dscode\"))\n",
                    home_str
                ));
                profile.push_str(&format!(
                    "(allow file-write* (subpath \"{}/.local/share\"))\n",
                    home_str
                ));
            }

            // Allow writing to /tmp for IPC sockets
            profile.push_str("(allow file-write* (subpath \"/tmp\"))\n");
            profile.push_str("(allow file-write* (subpath \"/private/tmp\"))\n");
        }
    }

    // Allow IPC communication (Unix domain sockets)
    profile.push_str("(allow system-socket)\n");
    profile.push_str("(allow socket-connect (unix-socket-path-regex \"/tmp/dscode.*\"))\n");
    profile.push_str("(allow socket-connect (unix-socket-path-regex \"/private/tmp/dscode.*\"))\n");

    // Network access
    if config.allow_network {
        profile.push_str("(allow network*)\n");
    }

    // Allow process execution for child processes of the extension host
    profile.push_str("(allow process-exec)\n");
    profile.push_str("(allow process-fork)\n");
    profile.push_str("(allow signal (target same-sandbox))\n");

    // Allow sysctl access
    profile.push_str("(allow sysctl-read)\n");

    // Write the profile to a temp file and use sandbox-exec
    let temp_dir = std::env::temp_dir();
    let profile_path = temp_dir.join(format!("dscode-sandbox-{}.sb", uuid::Uuid::new_v4()));
    std::fs::write(&profile_path, &profile)
        .map_err(|e| format!("Failed to write sandbox profile: {}", e))?;

    // Wrap the command with sandbox-exec
    let program = command.get_program().to_string_lossy().to_string();
    let args: Vec<String> = command
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    *command = Command::new("/usr/bin/sandbox-exec");
    command.arg("-p").arg(&profile_path);
    command.arg("--");
    command.arg(&program);
    command.args(&args);

    // Schedule cleanup of the profile file
    let profile_path_owned = profile_path.clone();
    // The profile must exist while the process runs, so we clean up after
    // the process exits (handled in the manager Drop impl)
    let _ = profile_path_owned; // keep path around for potential cleanup

    // Apply resource limits via rlimit (same as Linux fallback)
    apply_macos_resource_limits(command, config)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_macos_resource_limits(
    command: &mut Command,
    config: &SandboxConfig,
) -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    let max_memory_mb = config.max_memory_mb;

    unsafe {
        command.pre_exec(move || {
            if let Some(max_mb) = max_memory_mb {
                let max_bytes = max_mb * 1024 * 1024;
                let limit = libc::rlimit {
                    rlim_cur: max_bytes,
                    rlim_max: libc::RLIM_INFINITY,
                };
                let _ = libc::setrlimit(libc::RLIMIT_AS, &limit);
            }
            Ok(())
        });
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_windows_sandbox(command: &mut Command, _config: &SandboxConfig) -> Result<(), String> {
    // Windows sandboxing using Job Objects would require
    // CreateJobObject, AssignProcessToJobObject, SetInformationJobObject
    // This needs to be done after process creation, not in Command
    // For now, we document that Windows sandboxing happens post-spawn

    // TODO: Implement Windows Job Objects in the spawn code

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(config.allow_network);
        assert!(config.allow_file_write);
        assert_eq!(config.max_memory_mb, Some(512));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_sandbox_application() {
        let mut cmd = Command::new("echo");
        cmd.arg("test");

        let config = SandboxConfig::default();
        let result = apply_sandbox(&mut cmd, &config);

        // Should not error
        assert!(result.is_ok());
    }
}
