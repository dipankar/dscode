/**
 * Resource Monitor
 *
 * Tracks system resources used by the application and extension host
 */
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::{Pid, System};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub timestamp: u64,
    pub main_process: ProcessMetrics,
    pub extension_host: Option<ProcessMetrics>,
    pub system: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_cpu_percent: f32,
    pub total_memory_mb: f64,
    pub used_memory_mb: f64,
    pub cpu_count: usize,
}

pub struct ResourceMonitor {
    main_pid: Pid,
    extension_host_pid: Arc<Mutex<Option<Pid>>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        let main_pid = Pid::from(std::process::id() as usize);

        Self { main_pid, extension_host_pid: Arc::new(Mutex::new(None)) }
    }

    /**
     * Set the extension host PID for tracking
     */
    pub fn set_extension_host_pid(&self, pid: u32) {
        let mut ext_pid = self.extension_host_pid.lock().unwrap_or_else(|e| {
            warn!("extension_host_pid lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        *ext_pid = Some(Pid::from(pid as usize));
        info!("[ResourceMonitor] Tracking extension host PID: {}", pid);
    }

    /**
     * Get current resource metrics
     */
    pub fn get_metrics(&self) -> ResourceMetrics {
        debug!("[ResourceMonitor] get_metrics called");

        // Create a minimal System instance - don't call new_all() which crashes
        let mut system = System::new();

        // Only refresh what we need
        system.refresh_memory();
        system.refresh_cpu_usage();
        system.refresh_process(self.main_pid);

        let ext_host_pid =
            self.extension_host_pid.lock().unwrap_or_else(|e| {
                warn!("extension_host_pid lock poisoned, recovering: {}", e);
                e.into_inner()
            });
        if let Some(pid) = *ext_host_pid {
            system.refresh_process(pid);
        }
        drop(ext_host_pid);

        debug!("[ResourceMonitor] System info refreshed");

        // Get main process metrics
        let main_process = system
            .process(self.main_pid)
            .map(|p| ProcessMetrics {
                cpu_percent: p.cpu_usage(),
                memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
                pid: self.main_pid.as_u32(),
            })
            .unwrap_or(ProcessMetrics {
                cpu_percent: 0.0,
                memory_mb: 0.0,
                pid: self.main_pid.as_u32(),
            });

        // Get extension host metrics if available
        let ext_host_pid =
            self.extension_host_pid.lock().unwrap_or_else(|e| {
                warn!("extension_host_pid lock poisoned, recovering: {}", e);
                e.into_inner()
            });
        let extension_host = ext_host_pid.and_then(|pid| {
            system.process(pid).map(|p| ProcessMetrics {
                cpu_percent: p.cpu_usage(),
                memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
                pid: pid.as_u32(),
            })
        });

        // Get system metrics
        let system_metrics = SystemMetrics {
            total_cpu_percent: system.global_cpu_info().cpu_usage(),
            total_memory_mb: system.total_memory() as f64 / 1024.0 / 1024.0,
            used_memory_mb: system.used_memory() as f64 / 1024.0 / 1024.0,
            cpu_count: system.cpus().len(),
        };

        ResourceMetrics {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            main_process,
            extension_host,
            system: system_metrics,
        }
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
