import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ===== Type Definitions =====

export interface TestSuite {
  id: string;
  extensionId: string;
  name: string;
  description?: string;
  tests: TestCase[];
}

export interface TestCase {
  id: string;
  name: string;
  description?: string;
  timeoutMs?: number;
  tags: string[];
}

export interface TestRunResult {
  runId: string;
  suiteId: string;
  status: TestStatus;
  tests: TestResult[];
  summary: TestSummary;
  startedAt: string;
  completedAt?: string;
}

export type TestStatus = 'running' | 'passed' | 'failed';

export interface TestResult {
  testId: string;
  name: string;
  status: TestResultStatus;
  durationMs: number;
  error?: TestError;
}

export type TestResultStatus = 'passed' | 'failed' | 'skipped';

export interface TestError {
  message: string;
  stackTrace?: string;
  expected?: string;
  actual?: string;
}

export interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  durationMs: number;
}

export interface CoverageData {
  extensionId: string;
  totalLines: number;
  coveredLines: number;
  percentage: number;
  files: Record<string, FileCoverage>;
}

export interface FileCoverage {
  path: string;
  totalLines: number;
  coveredLines: number;
  percentage: number;
  uncoveredRanges: LineRange[];
}

export interface LineRange {
  start: number;
  end: number;
}

export interface APIValidation {
  extensionId: string;
  extensionType: string;
  apisUsed: string[];
}

export interface ValidationResult {
  valid: boolean;
  issues: ValidationIssue[];
}

export interface ValidationIssue {
  severity: Severity;
  message: string;
  suggestion?: string;
}

export type Severity = 'error' | 'warning' | 'info';

export interface TestRunStartedEvent {
  result: TestRunResult;
}

export interface TestResultUpdatedEvent {
  result: TestResult;
}

export interface TestRunCompletedEvent {
  result: TestRunResult;
}

/**
 * Test Runner Manager for Extension API Testing
 */
export class TestRunnerManager {
  private onTestRunStartedCallbacks: Array<(event: TestRunStartedEvent) => void> = [];
  private onTestResultUpdatedCallbacks: Array<(event: TestResultUpdatedEvent) => void> = [];
  private onTestRunCompletedCallbacks: Array<(event: TestRunCompletedEvent) => void> = [];

  constructor() {}

  /**
   * Initialize test runner manager
   */
  async initialize(): Promise<void> {
    // Listen for test lifecycle events
    await listen<TestRunResult>('test-run-started', (event) => {
      this.notifyTestRunStarted({ result: event.payload });
    });

    await listen<TestResult>('test-result-updated', (event) => {
      this.notifyTestResultUpdated({ result: event.payload });
    });

    await listen<TestRunResult>('test-run-completed', (event) => {
      this.notifyTestRunCompleted({ result: event.payload });
    });

    console.log('[TestRunner] Initialized');
  }

  // ===== Test Suite Management =====

  /**
   * Register test suite
   */
  async registerTestSuite(suite: TestSuite): Promise<string> {
    return await invoke<string>('register_test_suite', { suite });
  }

  /**
   * Get test suite by ID
   */
  async getTestSuite(suiteId: string): Promise<TestSuite> {
    return await invoke<TestSuite>('get_test_suite', { suiteId });
  }

  /**
   * Get all test suites
   */
  async getAllTestSuites(): Promise<TestSuite[]> {
    return await invoke<TestSuite[]>('get_all_test_suites');
  }

  /**
   * Get test suites by extension
   */
  async getTestSuitesByExtension(extensionId: string): Promise<TestSuite[]> {
    return await invoke<TestSuite[]>('get_test_suites_by_extension', { extensionId });
  }

  // ===== Test Execution =====

  /**
   * Start test run
   */
  async startTestRun(suiteId: string, runId: string): Promise<void> {
    await invoke('start_test_run', { suiteId, runId });
  }

  /**
   * Update test result
   */
  async updateTestResult(runId: string, testResult: TestResult): Promise<void> {
    await invoke('update_test_result', { runId, testResult });
  }

  /**
   * Complete test run
   */
  async completeTestRun(runId: string, durationMs: number): Promise<void> {
    await invoke('complete_test_run', { runId, durationMs });
  }

  /**
   * Get test run result
   */
  async getTestRunResult(runId: string): Promise<TestRunResult> {
    return await invoke<TestRunResult>('get_test_run_result', { runId });
  }

  /**
   * Get all test results
   */
  async getAllTestResults(): Promise<TestRunResult[]> {
    return await invoke<TestRunResult[]>('get_all_test_results');
  }

  /**
   * Get test results by suite
   */
  async getTestResultsBySuite(suiteId: string): Promise<TestRunResult[]> {
    return await invoke<TestRunResult[]>('get_test_results_by_suite', { suiteId });
  }

  // ===== Coverage Tracking =====

  /**
   * Update coverage data
   */
  async updateCoverage(extensionId: string, coverage: CoverageData): Promise<void> {
    await invoke('update_coverage', { extensionId, coverage });
  }

  /**
   * Get coverage data for extension
   */
  async getCoverage(extensionId: string): Promise<CoverageData | null> {
    return await invoke<CoverageData | null>('get_coverage', { extensionId });
  }

  /**
   * Get all coverage data
   */
  async getAllCoverage(): Promise<Record<string, CoverageData>> {
    return await invoke<Record<string, CoverageData>>('get_all_coverage');
  }

  // ===== API Validation =====

  /**
   * Validate API usage
   */
  async validateApiUsage(validation: APIValidation): Promise<ValidationResult> {
    return await invoke<ValidationResult>('validate_api_usage', { validation });
  }

  // ===== Cleanup =====

  /**
   * Clear test data for extension
   */
  async clearTestData(extensionId: string): Promise<void> {
    await invoke('clear_test_data', { extensionId });
  }

  /**
   * Clear all test results
   */
  async clearAllTestResults(): Promise<void> {
    await invoke('clear_all_test_results');
  }

  // ===== Utility Methods =====

  /**
   * Create test suite helper
   */
  createTestSuite(
    id: string,
    extensionId: string,
    name: string,
    tests: TestCase[],
    description?: string
  ): TestSuite {
    return {
      id,
      extensionId,
      name,
      description,
      tests,
    };
  }

  /**
   * Create test case helper
   */
  createTestCase(
    id: string,
    name: string,
    options: {
      description?: string;
      timeoutMs?: number;
      tags?: string[];
    } = {}
  ): TestCase {
    return {
      id,
      name,
      description: options.description,
      timeoutMs: options.timeoutMs,
      tags: options.tags || [],
    };
  }

  /**
   * Create test result helper
   */
  createTestResult(
    testId: string,
    name: string,
    status: TestResultStatus,
    durationMs: number,
    error?: TestError
  ): TestResult {
    return {
      testId,
      name,
      status,
      durationMs,
      error,
    };
  }

  /**
   * Create test error helper
   */
  createTestError(
    message: string,
    options: {
      stackTrace?: string;
      expected?: string;
      actual?: string;
    } = {}
  ): TestError {
    return {
      message,
      stackTrace: options.stackTrace,
      expected: options.expected,
      actual: options.actual,
    };
  }

  /**
   * Generate unique run ID
   */
  generateRunId(): string {
    return `run-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Calculate coverage percentage
   */
  calculateCoveragePercentage(coveredLines: number, totalLines: number): number {
    if (totalLines === 0) return 0;
    return (coveredLines / totalLines) * 100;
  }

  /**
   * Format test duration
   */
  formatDuration(ms: number): string {
    if (ms < 1000) {
      return `${ms}ms`;
    } else if (ms < 60000) {
      return `${(ms / 1000).toFixed(2)}s`;
    } else {
      const minutes = Math.floor(ms / 60000);
      const seconds = ((ms % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  }

  /**
   * Format coverage percentage
   */
  formatCoveragePercentage(percentage: number): string {
    return `${percentage.toFixed(2)}%`;
  }

  /**
   * Get test status icon
   */
  getTestStatusIcon(status: TestStatus | TestResultStatus): string {
    switch (status) {
      case 'passed':
        return '✓';
      case 'failed':
        return '✗';
      case 'skipped':
        return '○';
      case 'running':
        return '⟳';
      default:
        return '?';
    }
  }

  /**
   * Get test status color
   */
  getTestStatusColor(status: TestStatus | TestResultStatus): string {
    switch (status) {
      case 'passed':
        return '#4caf50';
      case 'failed':
        return '#f44336';
      case 'skipped':
        return '#ff9800';
      case 'running':
        return '#2196f3';
      default:
        return '#757575';
    }
  }

  /**
   * Get severity icon
   */
  getSeverityIcon(severity: Severity): string {
    switch (severity) {
      case 'error':
        return '✗';
      case 'warning':
        return '⚠';
      case 'info':
        return 'ℹ';
      default:
        return '?';
    }
  }

  /**
   * Get severity color
   */
  getSeverityColor(severity: Severity): string {
    switch (severity) {
      case 'error':
        return '#f44336';
      case 'warning':
        return '#ff9800';
      case 'info':
        return '#2196f3';
      default:
        return '#757575';
    }
  }

  /**
   * Calculate test pass rate
   */
  getPassRate(summary: TestSummary): number {
    if (summary.total === 0) return 0;
    return (summary.passed / summary.total) * 100;
  }

  /**
   * Check if all tests passed
   */
  allTestsPassed(summary: TestSummary): boolean {
    return summary.failed === 0 && summary.passed === summary.total - summary.skipped;
  }

  /**
   * Get coverage quality rating
   */
  getCoverageQuality(percentage: number): 'excellent' | 'good' | 'fair' | 'poor' {
    if (percentage >= 90) return 'excellent';
    if (percentage >= 75) return 'good';
    if (percentage >= 50) return 'fair';
    return 'poor';
  }

  /**
   * Get coverage quality color
   */
  getCoverageQualityColor(percentage: number): string {
    const quality = this.getCoverageQuality(percentage);
    switch (quality) {
      case 'excellent':
        return '#4caf50';
      case 'good':
        return '#8bc34a';
      case 'fair':
        return '#ff9800';
      case 'poor':
        return '#f44336';
    }
  }

  /**
   * Filter tests by tag
   */
  filterTestsByTag(suite: TestSuite, tag: string): TestCase[] {
    return suite.tests.filter((test) => test.tags.includes(tag));
  }

  /**
   * Get all unique tags from suite
   */
  getAllTags(suite: TestSuite): string[] {
    const tags = new Set<string>();
    for (const test of suite.tests) {
      for (const tag of test.tags) {
        tags.add(tag);
      }
    }
    return Array.from(tags).sort();
  }

  /**
   * Group test results by status
   */
  groupResultsByStatus(results: TestResult[]): Record<TestResultStatus, TestResult[]> {
    return {
      passed: results.filter((r) => r.status === 'passed'),
      failed: results.filter((r) => r.status === 'failed'),
      skipped: results.filter((r) => r.status === 'skipped'),
    };
  }

  /**
   * Get slowest tests
   */
  getSlowestTests(results: TestResult[], limit: number = 10): TestResult[] {
    return [...results].sort((a, b) => b.durationMs - a.durationMs).slice(0, limit);
  }

  /**
   * Get uncovered files
   */
  getUncoveredFiles(coverage: CoverageData): FileCoverage[] {
    return Object.values(coverage.files).filter((file) => file.percentage < 100);
  }

  /**
   * Get files with low coverage
   */
  getLowCoverageFiles(coverage: CoverageData, threshold: number = 50): FileCoverage[] {
    return Object.values(coverage.files).filter((file) => file.percentage < threshold);
  }

  /**
   * Calculate total uncovered lines
   */
  getTotalUncoveredLines(coverage: CoverageData): number {
    return coverage.totalLines - coverage.coveredLines;
  }

  /**
   * Get validation errors
   */
  getValidationErrors(result: ValidationResult): ValidationIssue[] {
    return result.issues.filter((issue) => issue.severity === 'error');
  }

  /**
   * Get validation warnings
   */
  getValidationWarnings(result: ValidationResult): ValidationIssue[] {
    return result.issues.filter((issue) => issue.severity === 'warning');
  }

  /**
   * Check if validation has errors
   */
  hasValidationErrors(result: ValidationResult): boolean {
    return result.issues.some((issue) => issue.severity === 'error');
  }

  /**
   * Format validation summary
   */
  formatValidationSummary(result: ValidationResult): string {
    const errorCount = this.getValidationErrors(result).length;
    const warningCount = this.getValidationWarnings(result).length;

    if (errorCount === 0 && warningCount === 0) {
      return 'No issues found';
    }

    const parts: string[] = [];
    if (errorCount > 0) {
      parts.push(`${errorCount} ${errorCount === 1 ? 'error' : 'errors'}`);
    }
    if (warningCount > 0) {
      parts.push(`${warningCount} ${warningCount === 1 ? 'warning' : 'warnings'}`);
    }

    return parts.join(', ');
  }

  /**
   * Create API validation helper
   */
  createAPIValidation(
    extensionId: string,
    extensionType: string,
    apisUsed: string[]
  ): APIValidation {
    return {
      extensionId,
      extensionType,
      apisUsed,
    };
  }

  /**
   * Merge coverage data
   */
  mergeCoverage(coverageList: CoverageData[]): CoverageData | null {
    if (coverageList.length === 0) return null;
    if (coverageList.length === 1) return coverageList[0];

    const merged: CoverageData = {
      extensionId: coverageList[0].extensionId,
      totalLines: 0,
      coveredLines: 0,
      percentage: 0,
      files: {},
    };

    // Merge all files
    for (const coverage of coverageList) {
      merged.totalLines += coverage.totalLines;
      merged.coveredLines += coverage.coveredLines;

      for (const [path, file] of Object.entries(coverage.files)) {
        if (!merged.files[path]) {
          merged.files[path] = { ...file };
        } else {
          // If file appears in multiple coverage reports, take the max coverage
          const existing = merged.files[path];
          if (file.percentage > existing.percentage) {
            merged.files[path] = { ...file };
          }
        }
      }
    }

    merged.percentage = this.calculateCoveragePercentage(merged.coveredLines, merged.totalLines);
    return merged;
  }

  // ===== Event Subscriptions =====

  /**
   * Subscribe to test run started events
   */
  onTestRunStarted(callback: (event: TestRunStartedEvent) => void): () => void {
    this.onTestRunStartedCallbacks.push(callback);

    return () => {
      const index = this.onTestRunStartedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTestRunStartedCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to test result updated events
   */
  onTestResultUpdated(callback: (event: TestResultUpdatedEvent) => void): () => void {
    this.onTestResultUpdatedCallbacks.push(callback);

    return () => {
      const index = this.onTestResultUpdatedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTestResultUpdatedCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to test run completed events
   */
  onTestRunCompleted(callback: (event: TestRunCompletedEvent) => void): () => void {
    this.onTestRunCompletedCallbacks.push(callback);

    return () => {
      const index = this.onTestRunCompletedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onTestRunCompletedCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Internal Methods =====

  private notifyTestRunStarted(event: TestRunStartedEvent): void {
    for (const callback of this.onTestRunStartedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[TestRunner] Error in test run started callback:', error);
      }
    }
  }

  private notifyTestResultUpdated(event: TestResultUpdatedEvent): void {
    for (const callback of this.onTestResultUpdatedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[TestRunner] Error in test result updated callback:', error);
      }
    }
  }

  private notifyTestRunCompleted(event: TestRunCompletedEvent): void {
    for (const callback of this.onTestRunCompletedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[TestRunner] Error in test run completed callback:', error);
      }
    }
  }
}

// Export singleton instance
export const testRunner = new TestRunnerManager();
