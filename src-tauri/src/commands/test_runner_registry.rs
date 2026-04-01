use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Test runner registry for extension API testing
pub struct TestRunnerRegistry {
    test_suites: Arc<RwLock<HashMap<String, TestSuite>>>,
    test_results: Arc<RwLock<HashMap<String, TestRunResult>>>,
    coverage_data: Arc<RwLock<HashMap<String, CoverageData>>>,
    app_handle: AppHandle,
}

impl TestRunnerRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            test_suites: Arc::new(RwLock::new(HashMap::new())),
            test_results: Arc::new(RwLock::new(HashMap::new())),
            coverage_data: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    // ===== Test Suite Management =====

    /// Register test suite
    pub async fn register_test_suite(&self, suite: TestSuite) -> Result<String, String> {
        let mut suites = self.test_suites.write().await;

        let suite_id = suite.id.clone();
        suites.insert(suite_id.clone(), suite);

        info!("[TestRunner] Registered test suite: {}", suite_id);

        Ok(suite_id)
    }

    /// Get test suite
    pub async fn get_test_suite(&self, suite_id: &str) -> Result<TestSuite, String> {
        self.test_suites
            .read()
            .await
            .get(suite_id)
            .cloned()
            .ok_or_else(|| format!("Test suite '{}' not found", suite_id))
    }

    /// Get all test suites
    pub async fn get_all_test_suites(&self) -> Vec<TestSuite> {
        self.test_suites.read().await.values().cloned().collect()
    }

    /// Get test suites by extension
    pub async fn get_test_suites_by_extension(&self, extension_id: &str) -> Vec<TestSuite> {
        self.test_suites
            .read()
            .await
            .values()
            .filter(|s| s.extension_id == extension_id)
            .cloned()
            .collect()
    }

    // ===== Test Execution =====

    /// Start test run
    pub async fn start_test_run(&self, suite_id: &str, run_id: String) -> Result<(), String> {
        let suite = self.get_test_suite(suite_id).await?;

        let result = TestRunResult {
            run_id: run_id.clone(),
            suite_id: suite_id.to_string(),
            status: TestStatus::Running,
            tests: vec![],
            summary: TestSummary {
                total: suite.tests.len(),
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 0,
            },
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };

        let mut results = self.test_results.write().await;
        results.insert(run_id.clone(), result.clone());

        // Emit event
        if let Err(e) = self.app_handle.emit("test-run-started", &result) {
            error!("[TestRunner] Failed to emit test run started event: {}", e);
        }

        info!("[TestRunner] Started test run: {}", run_id);

        Ok(())
    }

    /// Update test result
    pub async fn update_test_result(
        &self, run_id: &str, test_result: TestResult,
    ) -> Result<(), String> {
        let mut results = self.test_results.write().await;

        if let Some(run_result) = results.get_mut(run_id) {
            run_result.tests.push(test_result.clone());

            // Update summary
            match test_result.status {
                TestResultStatus::Passed => run_result.summary.passed += 1,
                TestResultStatus::Failed => run_result.summary.failed += 1,
                TestResultStatus::Skipped => run_result.summary.skipped += 1,
            }

            // Emit event
            if let Err(e) = self.app_handle.emit("test-result-updated", &test_result) {
                error!("[TestRunner] Failed to emit test result updated event: {}", e);
            }

            Ok(())
        } else {
            Err(format!("Test run '{}' not found", run_id))
        }
    }

    /// Complete test run
    pub async fn complete_test_run(&self, run_id: &str, duration_ms: u64) -> Result<(), String> {
        let mut results = self.test_results.write().await;

        if let Some(run_result) = results.get_mut(run_id) {
            run_result.status =
                if run_result.summary.failed > 0 { TestStatus::Failed } else { TestStatus::Passed };
            run_result.summary.duration_ms = duration_ms;
            run_result.completed_at = Some(chrono::Utc::now().to_rfc3339());

            // Emit event
            if let Err(e) = self.app_handle.emit("test-run-completed", &*run_result) {
                error!("[TestRunner] Failed to emit test run completed event: {}", e);
            }

            info!("[TestRunner] Completed test run: {}", run_id);

            Ok(())
        } else {
            Err(format!("Test run '{}' not found", run_id))
        }
    }

    /// Get test run result
    pub async fn get_test_run_result(&self, run_id: &str) -> Result<TestRunResult, String> {
        self.test_results
            .read()
            .await
            .get(run_id)
            .cloned()
            .ok_or_else(|| format!("Test run '{}' not found", run_id))
    }

    /// Get all test results
    pub async fn get_all_test_results(&self) -> Vec<TestRunResult> {
        self.test_results.read().await.values().cloned().collect()
    }

    /// Get test results by suite
    pub async fn get_test_results_by_suite(&self, suite_id: &str) -> Vec<TestRunResult> {
        self.test_results
            .read()
            .await
            .values()
            .filter(|r| r.suite_id == suite_id)
            .cloned()
            .collect()
    }

    // ===== Coverage Tracking =====

    /// Update coverage data
    pub async fn update_coverage(
        &self, extension_id: String, coverage: CoverageData,
    ) -> Result<(), String> {
        let mut coverage_data = self.coverage_data.write().await;
        coverage_data.insert(extension_id.clone(), coverage);

        info!("[TestRunner] Updated coverage for: {}", extension_id);

        Ok(())
    }

    /// Get coverage data
    pub async fn get_coverage(&self, extension_id: &str) -> Option<CoverageData> {
        self.coverage_data.read().await.get(extension_id).cloned()
    }

    /// Get all coverage data
    pub async fn get_all_coverage(&self) -> HashMap<String, CoverageData> {
        self.coverage_data.read().await.clone()
    }

    // ===== API Validation =====

    /// Validate API usage
    pub async fn validate_api_usage(&self, validation: APIValidation) -> ValidationResult {
        let mut issues = Vec::new();

        // Check for deprecated API usage
        for api in &validation.apis_used {
            if self.is_deprecated_api(api) {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: format!("Deprecated API used: {}", api),
                    suggestion: self.get_api_replacement(api),
                });
            }
        }

        // Check for missing required APIs
        let required_apis = self.get_required_apis(&validation.extension_type);
        for required in required_apis {
            if !validation.apis_used.contains(&required) {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("Required API not implemented: {}", required),
                    suggestion: Some(format!(
                        "Implement {} for {} extensions",
                        required, validation.extension_type
                    )),
                });
            }
        }

        ValidationResult { valid: issues.iter().all(|i| i.severity != Severity::Error), issues }
    }

    fn is_deprecated_api(&self, api: &str) -> bool {
        matches!(api, "workspace.rootPath" | "window.onDidChangeActiveEditor")
    }

    fn get_api_replacement(&self, api: &str) -> Option<String> {
        match api {
            "workspace.rootPath" => Some("workspace.workspaceFolders".to_string()),
            "window.onDidChangeActiveEditor" => {
                Some("window.onDidChangeActiveTextEditor".to_string())
            }
            _ => None,
        }
    }

    fn get_required_apis(&self, extension_type: &str) -> Vec<String> {
        match extension_type {
            "language" => vec![
                "languages.registerHoverProvider".to_string(),
                "languages.registerCompletionItemProvider".to_string(),
            ],
            "debugger" => vec![
                "debug.registerDebugConfigurationProvider".to_string(),
                "debug.registerDebugAdapterDescriptorFactory".to_string(),
            ],
            _ => vec![],
        }
    }

    // ===== Cleanup =====

    /// Clear test data for extension
    pub async fn clear_test_data(&self, extension_id: &str) {
        let mut suites = self.test_suites.write().await;
        suites.retain(|_, s| s.extension_id != extension_id);

        let mut coverage_data = self.coverage_data.write().await;
        coverage_data.remove(extension_id);

        info!("[TestRunner] Cleared test data for: {}", extension_id);
    }

    /// Clear all test results
    pub async fn clear_all_results(&self) {
        let mut results = self.test_results.write().await;
        results.clear();

        info!("[TestRunner] Cleared all test results");
    }
}

// ===== Type Definitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSuite {
    pub id: String,
    pub extension_id: String,
    pub name: String,
    pub description: Option<String>,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub timeout_ms: Option<u64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunResult {
    pub run_id: String,
    pub suite_id: String,
    pub status: TestStatus,
    pub tests: Vec<TestResult>,
    pub summary: TestSummary,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub test_id: String,
    pub name: String,
    pub status: TestResultStatus,
    pub duration_ms: u64,
    pub error: Option<TestError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestResultStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestError {
    pub message: String,
    pub stack_trace: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageData {
    pub extension_id: String,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub percentage: f64,
    pub files: HashMap<String, FileCoverage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCoverage {
    pub path: String,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub percentage: f64,
    pub uncovered_ranges: Vec<LineRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIValidation {
    pub extension_id: String,
    pub extension_type: String,
    pub apis_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}
