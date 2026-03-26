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
            max_memory_mb: Some(512),
            max_cpu_percent: Some(50),
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
    if which::which("bwrap").is_ok() {
        return apply_bubblewrap_sandbox(command, config);
    }
    apply_linux_resource_limits(command, config)?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_bubblewrap_sandbox(
    original_command: &mut Command,
    config: &SandboxConfig,
) -> Result<(), String> {
    let program = original_command.get_program().to_string_lossy().to_string();
    let args: Vec<String> = original_command
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

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

    let mut bwrap = Command::new("bwrap");

    bwrap.args(&["--ro-bind", "/usr", "/usr"]);
    bwrap.args(&["--ro-bind", "/lib", "/lib"]);
    bwrap.args(&["--ro-bind", "/lib64", "/lib64"]);
    bwrap.args(&["--ro-bind", "/bin", "/bin"]);
    bwrap.args(&["--ro-bind", "/sbin", "/sbin"]);
    bwrap.args(&["--proc", "/proc"]);
    bwrap.args(&["--dev", "/dev"]);
    bwrap.args(&["--bind", "/tmp", "/tmp"]);

    if let Some(home) = std::env::var_os("HOME") {
        let home_str = home.to_string_lossy();
        bwrap.args(&["--ro-bind", &*home_str, &*home_str]);

        if config.allow_file_write {
            let dscode_dir = format!("{}/.dscode", home_str);
            let local_share = format!("{}/.local/share", home_str);
            let _ = std::fs::create_dir_all(&dscode_dir);
            let _ = std::fs::create_dir_all(&local_share);
            bwrap.args(&["--bind", &dscode_dir, &dscode_dir]);
            bwrap.args(&["--bind", &local_share, &local_share]);
        }
    }

    if !config.allow_network {
        bwrap.arg("--unshare-net");
    }

    bwrap.args(&["--unshare-pid", "--unshare-uts"]);
    bwrap.arg("--die-with-parent");
    bwrap.arg("--");
    bwrap.arg(program);
    bwrap.args(args);

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

    let max_memory_mb = config.max_memory_mb;

    unsafe {
        command.pre_exec(move || {
            if let Some(max_mb) = max_memory_mb {
                let max_bytes = max_mb * 1024 * 1024;
                let limit = libc::rlimit {
                    rlim_cur: max_bytes,
                    rlim_max: max_bytes,
                };
                libc::setrlimit(libc::RLIMIT_AS, &limit);
            }
            Ok(())
        });
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_macos_sandbox(command: &mut Command, config: &SandboxConfig) -> Result<(), String> {
    command.env("NODE_ENV", "production");
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.allow_network);
        assert!(!config.allow_file_write);
        assert_eq!(config.max_memory_mb, Some(512));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_sandbox_application() {
        let mut cmd = Command::new("echo");
        cmd.arg("test");

        let config = SandboxConfig::default();
        let result = apply_sandbox(&mut cmd, &config);

        assert!(result.is_ok());
    }
}
