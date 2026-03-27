use crate::monitoring::{ResourceMetrics, ResourceMonitor};
use tauri::State;

#[tauri::command]
pub fn get_resource_metrics(monitor: State<ResourceMonitor>) -> Result<ResourceMetrics, String> {
    println!("[Metrics] Command called");
    let metrics = monitor.get_metrics();
    println!("[Metrics] Metrics retrieved successfully");
    Ok(metrics)
}
