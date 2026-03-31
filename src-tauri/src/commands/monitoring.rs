use crate::monitoring::{ResourceMetrics, ResourceMonitor};
use tauri::State;
use tracing::debug;

#[tauri::command]
pub fn get_resource_metrics(monitor: State<ResourceMonitor>) -> Result<ResourceMetrics, String> {
    debug!("[Metrics] Command called");
    let metrics = monitor.get_metrics();
    debug!("[Metrics] Metrics retrieved successfully");
    Ok(metrics)
}
