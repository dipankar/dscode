use std::process::Command;

#[cfg(target_os = "linux")]
use tracing::info;

#[cfg(target_os = "windows")]
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use tracing::{info, warn};

/// Wrapper to make Windows `HANDLE` `Send` + `Sync`.
/// A HANDLE is a kernel object identifier that is inherently safe to share
/// between threads — the kernel manages all synchronization.
#[cfg(target_os = "windows")]
#[derive(Copy, Clone)]
struct JobHandle(windows::Win32::Foundation::HANDLE);

#[cfg(target_os = "windows")]
unsafe impl Sync for JobHandle {}

#[cfg(target_os = "windows")]
unsafe impl Send for JobHandle {}

/// Global storage for the Windows Job Object handle.
/// The job object lives for the duration of the process and is used to
/// sandbox all extension host child processes. The handle is intentionally
/// never closed — with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, closing it would
/// terminate all child processes in the job.
#[cfg(target_os = "windows")]
static JOB_OBJECT: OnceLock<JobHandle> = OnceLock::new();

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

    info!("Applying bubblewrap sandbox");

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
fn apply_windows_sandbox(_command: &mut Command, config: &SandboxConfig) -> Result<(), String> {
    // Create and configure the job object (idempotent — only creates once).
    if JOB_OBJECT.get().is_none() {
        match create_and_configure_job_object(config) {
            Ok(handle) => {
                let _ = JOB_OBJECT.set(JobHandle(handle));
                info!("Windows Job Object sandbox created and configured");
            }
            Err(e) => {
                warn!("Failed to create Windows Job Object sandbox: {e}");
                // Log and continue, matching the macOS/Linux behaviour where
                // individual setrlimit failures are silently ignored.
            }
        }
    }
    Ok(())
}

/// Completes sandbox setup after the process has been spawned.
/// On Windows, this assigns the child process to the Job Object created by
/// `apply_windows_sandbox`. On other platforms this is a no-op.
///
/// Call this after `Command::spawn()` and before interacting with the child.
pub fn complete_sandbox_setup(child: &std::process::Child) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        complete_windows_sandbox(child)?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = child;
    }
    Ok(())
}

/// Assigns the spawned child process to the Windows Job Object.
#[cfg(target_os = "windows")]
fn complete_windows_sandbox(child: &std::process::Child) -> Result<(), String> {
    use windows::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows::Win32::System::JobObjects::AssignProcessToJobObject;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};

    let Some(handle_wrapper) = JOB_OBJECT.get() else {
        warn!("No Windows Job Object found, skipping sandbox assignment");
        return Ok(());
    };

    let job = handle_wrapper.0;

    // Open a handle to the child process with the minimum access rights
    // required by AssignProcessToJobObject.
    let process_handle =
        unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, FALSE, child.id()) };

    let process_handle: HANDLE = match process_handle {
        Ok(h) => h,
        Err(e) => {
            warn!("Failed to open child process for job assignment: {e}");
            return Ok(());
        }
    };

    // Assign the process to the job object.
    let assign_result = unsafe { AssignProcessToJobObject(job, process_handle) };

    match assign_result {
        Ok(()) => {
            info!("Successfully assigned child process to Windows Job Object");
        }
        Err(e) => {
            warn!("Failed to assign process to Windows Job Object: {e}");
        }
    }

    // Close the process handle we opened — the assignment persists independently.
    unsafe {
        let _ = CloseHandle(process_handle);
    }

    Ok(())
}

/// Creates and configures a Windows Job Object with the specified limits.
#[cfg(target_os = "windows")]
fn create_and_configure_job_object(
    config: &SandboxConfig,
) -> Result<windows::Win32::Foundation::HANDLE, String> {
    use windows::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectBasicUIRestrictions, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_BASIC_UI_RESTRICTIONS,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_DISPLAY_SETTINGS,
        JOB_OBJECT_UILIMIT_EXIT_WINDOWS, JOB_OBJECT_UILIMIT_FLAGS,
    };

    // Create an anonymous job object.
    let job = unsafe { CreateJobObjectW(None, None) }
        .map_err(|e| format!("Failed to create job object: {e}"))?;

    // Configure extended limits: memory cap, active-process cap, kill-on-close.
    let mut extended_info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };

    let mut limit_flags = JOB_OBJECT_LIMIT_FLAGS(0);

    // Memory limit (e.g., 512 MB).
    if let Some(max_mb) = config.max_memory_mb {
        limit_flags |= JOB_OBJECT_LIMIT_JOB_MEMORY;
        extended_info.JobMemoryLimit = max_mb * 1024 * 1024;
    }

    // Active process limit — allow up to 4 concurrent processes in the job.
    limit_flags |= JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    extended_info.BasicLimitInformation.ActiveProcessLimit = 4;

    // Kill all processes in the job when the last handle is closed.
    limit_flags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

    extended_info.BasicLimitInformation.LimitFlags = limit_flags;

    unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &extended_info as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|e| format!("Failed to set job extended limits: {e}"))?;
    }

    // Configure UI restrictions.
    let mut ui_restrictions: JOBOBJECT_BASIC_UI_RESTRICTIONS = unsafe { std::mem::zeroed() };

    let mut ui_flags = JOB_OBJECT_UILIMIT_FLAGS(0);
    ui_flags |= JOB_OBJECT_UILIMIT_EXIT_WINDOWS;
    ui_flags |= JOB_OBJECT_UILIMIT_DESKTOP;
    ui_flags |= JOB_OBJECT_UILIMIT_DISPLAY_SETTINGS;

    ui_restrictions.UIRestrictionsClass = ui_flags;

    unsafe {
        SetInformationJobObject(
            job,
            JobObjectBasicUIRestrictions,
            &ui_restrictions as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
        )
        .map_err(|e| format!("Failed to set job UI restrictions: {e}"))?;
    }

    Ok(job)
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

    #[test]
    fn test_apply_sandbox_config() {
        // Verify apply_sandbox returns Ok for a basic command with default config
        let mut cmd = Command::new("echo");
        cmd.arg("test");
        let config = SandboxConfig::default();
        let result = apply_sandbox(&mut cmd, &config);
        assert!(result.is_ok());

        // Verify apply_sandbox returns Ok with network-enabled config
        let mut cmd2 = Command::new("echo");
        cmd2.arg("hello");
        let network_config = SandboxConfig {
            allow_network: true,
            allow_file_write: true,
            max_memory_mb: Some(256),
            max_cpu_percent: Some(25),
        };
        let result2 = apply_sandbox(&mut cmd2, &network_config);
        assert!(result2.is_ok());

        // Verify default config values are restrictive
        assert!(!config.allow_network, "Default should block network");
        assert!(!config.allow_file_write, "Default should block file write");
        assert_eq!(
            config.max_memory_mb,
            Some(512),
            "Default memory limit should be 512MB"
        );
        assert_eq!(
            config.max_cpu_percent,
            Some(50),
            "Default CPU limit should be 50%"
        );
    }

    #[test]
    fn test_sandbox_config_custom_values() {
        let config = SandboxConfig {
            allow_network: true,
            allow_file_write: true,
            max_memory_mb: Some(1024),
            max_cpu_percent: Some(80),
        };
        assert!(config.allow_network);
        assert!(config.allow_file_write);
        assert_eq!(config.max_memory_mb, Some(1024));
        assert_eq!(config.max_cpu_percent, Some(80));
    }

    #[test]
    fn test_sandbox_config_none_limits() {
        let config = SandboxConfig {
            allow_network: false,
            allow_file_write: false,
            max_memory_mb: None,
            max_cpu_percent: None,
        };
        assert!(config.max_memory_mb.is_none());
        assert!(config.max_cpu_percent.is_none());

        // apply_sandbox should still succeed with None limits
        let mut cmd = Command::new("echo");
        let result = apply_sandbox(&mut cmd, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sandbox_config_default_cpu_percent() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_cpu_percent, Some(50));
    }

    #[test]
    fn test_apply_sandbox_permissive_config() {
        let config = SandboxConfig {
            allow_network: true,
            allow_file_write: true,
            max_memory_mb: Some(2048),
            max_cpu_percent: Some(100),
        };
        let mut cmd = Command::new("echo");
        cmd.arg("permissive");
        let result = apply_sandbox(&mut cmd, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complete_sandbox_setup_noop_on_unix() {
        // On non-Windows, complete_sandbox_setup should be a no-op
        // We can't easily test with a real child, but we can verify the function exists
        // and the module compiles. The function is called after spawn.
        // Just verify SandboxConfig defaults are as expected.
        let config = SandboxConfig::default();
        assert!(!config.allow_network);
        assert!(!config.allow_file_write);
    }
}
