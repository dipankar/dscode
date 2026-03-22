# Testing Guide

This document describes the test infrastructure for DSCode.

## Overview

The codebase now includes comprehensive tests for the critical Extension Host IPC infrastructure:

- **TypeScript tests**: Extension Host bridge and workspace API
- **Rust tests**: Extension Host Pool and IPC communication

## Running Tests

### TypeScript Tests (Extension Host)

```bash
cd extension-host

# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run with coverage report
npm run test:coverage
```

### Rust Tests (Tauri Backend)

```bash
cd src-tauri

# Run all tests
cargo test

# Run specific test file
cargo test --test extension_host_pool_tests

# Run with output
cargo test -- --nocapture
```

## Test Coverage

### TypeScript Tests (`extension-host/src/__tests__/workspace.test.ts`)

Tests the Workspace API which had the `folders.map is not a function` bug:

- ✅ Handles array responses correctly
- ✅ Handles empty arrays
- ✅ Handles non-array responses gracefully (the bug we fixed)
- ✅ Handles null/undefined responses
- ✅ Handles connection errors
- ✅ Correctly maps folder paths to WorkspaceFolder objects
- ✅ File finding operations
- ✅ Configuration API
- ✅ Workspace properties (name, rootPath)

**15 tests, all passing**

### Rust Tests (`src-tauri/tests/extension_host_pool_tests.rs`)

Tests the Extension Host Pool infrastructure:

- ✅ Extension Host path resolution (validates the fix where we stopped using extension directory path)
- ✅ Workspace folders returns array (not status object)
- ✅ Extensions directory path format
- ✅ IPC URL generation and format
- ✅ Response serialization formats

**6 tests, all passing**

## What These Tests Validate

### Bug Fixes Covered

1. **Workspace API TypeError**
   - Test: `should handle non-array response gracefully`
   - Validates defensive check for array responses

2. **Extension Host Path Confusion**
   - Test: `test_extension_host_path_resolution`
   - Ensures Extension Host main.js path is used, not extension directory

3. **Generic Status Responses**
   - Tests: `test_workspace_folders_returns_array`, `test_extensions_dir_path`
   - Validates proper response formats, not generic `{status: "ok"}`

### Architecture Validated

- Bidirectional NNG IPC URL generation
- Response format consistency
- Error handling and graceful degradation
- Path resolution logic

## Adding New Tests

### TypeScript Tests

Create test files in `extension-host/src/__tests__/`:

```typescript
import { YourClass } from '../your-module';

describe('YourClass', () => {
  it('should do something', () => {
    // Test implementation
  });
});
```

### Rust Tests

Create test files in `src-tauri/tests/`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_async_something() {
        // Async test implementation
    }
}
```

## Coverage Goals

Current coverage thresholds (set in `jest.config.js`):

- **Branches**: 70%
- **Functions**: 70%
- **Lines**: 70%
- **Statements**: 70%

## CI/CD Integration

To integrate with CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run TypeScript tests
  run: |
    cd extension-host
    npm test

- name: Run Rust tests
  run: |
    cd src-tauri
    cargo test
```

## Future Test Additions

Recommended areas for additional test coverage:

1. **NNG IPC Integration Tests**: Full bidirectional communication tests
2. **Extension Loading**: End-to-end extension activation tests
3. **LSP/DAP Pools**: Language server and debug adapter tests
4. **Session Manager**: Extension lifecycle management tests
5. **Marketplace**: Extension installation and updates tests

## Troubleshooting

### TypeScript Tests Failing

```bash
# Clear Jest cache
npm test -- --clearCache

# Update snapshots if needed
npm test -- --updateSnapshot
```

### Rust Tests Failing

```bash
# Clean build artifacts
cargo clean

# Rebuild and test
cargo test
```

## Test Structure

```
dscode/
├── extension-host/
│   ├── src/
│   │   └── __tests__/
│   │       └── workspace.test.ts      # Workspace API tests
│   ├── jest.config.js                 # Jest configuration
│   └── package.json                   # Test scripts
└── src-tauri/
    └── tests/
        └── extension_host_pool_tests.rs  # Extension Host Pool tests
```
