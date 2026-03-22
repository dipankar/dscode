use super::test_runner_registry::*;
use tauri::State;

// ===== Test Suite Management =====

/// Register test suite
#[tauri::command]
pub async fn register_test_suite(
    suite: TestSuite,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<String, String> {
    registry.register_test_suite(suite).await
}

/// Get test suite
#[tauri::command]
pub async fn get_test_suite(
    suite_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<TestSuite, String> {
    registry.get_test_suite(&suite_id).await
}

/// Get all test suites
#[tauri::command]
pub async fn get_all_test_suites(
    registry: State<'_, TestRunnerRegistry>,
) -> Result<Vec<TestSuite>, String> {
    Ok(registry.get_all_test_suites().await)
}

/// Get test suites by extension
#[tauri::command]
pub async fn get_test_suites_by_extension(
    extension_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<Vec<TestSuite>, String> {
    Ok(registry.get_test_suites_by_extension(&extension_id).await)
}

// ===== Test Execution =====

/// Start test run
#[tauri::command]
pub async fn start_test_run(
    suite_id: String,
    run_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.start_test_run(&suite_id, run_id).await
}

/// Update test result
#[tauri::command]
pub async fn update_test_result(
    run_id: String,
    test_result: TestResult,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.update_test_result(&run_id, test_result).await
}

/// Complete test run
#[tauri::command]
pub async fn complete_test_run(
    run_id: String,
    duration_ms: u64,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.complete_test_run(&run_id, duration_ms).await
}

/// Get test run result
#[tauri::command]
pub async fn get_test_run_result(
    run_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<TestRunResult, String> {
    registry.get_test_run_result(&run_id).await
}

/// Get all test results
#[tauri::command]
pub async fn get_all_test_results(
    registry: State<'_, TestRunnerRegistry>,
) -> Result<Vec<TestRunResult>, String> {
    Ok(registry.get_all_test_results().await)
}

/// Get test results by suite
#[tauri::command]
pub async fn get_test_results_by_suite(
    suite_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<Vec<TestRunResult>, String> {
    Ok(registry.get_test_results_by_suite(&suite_id).await)
}

// ===== Coverage Tracking =====

/// Update coverage data
#[tauri::command]
pub async fn update_coverage(
    extension_id: String,
    coverage: CoverageData,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.update_coverage(extension_id, coverage).await
}

/// Get coverage data
#[tauri::command]
pub async fn get_coverage(
    extension_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<Option<CoverageData>, String> {
    Ok(registry.get_coverage(&extension_id).await)
}

/// Get all coverage data
#[tauri::command]
pub async fn get_all_coverage(
    registry: State<'_, TestRunnerRegistry>,
) -> Result<std::collections::HashMap<String, CoverageData>, String> {
    Ok(registry.get_all_coverage().await)
}

// ===== API Validation =====

/// Validate API usage
#[tauri::command]
pub async fn validate_api_usage(
    validation: APIValidation,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<ValidationResult, String> {
    Ok(registry.validate_api_usage(validation).await)
}

// ===== Cleanup =====

/// Clear test data for extension
#[tauri::command]
pub async fn clear_test_data(
    extension_id: String,
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.clear_test_data(&extension_id).await;
    Ok(())
}

/// Clear all test results
#[tauri::command]
pub async fn clear_all_test_results(
    registry: State<'_, TestRunnerRegistry>,
) -> Result<(), String> {
    registry.clear_all_results().await;
    Ok(())
}
