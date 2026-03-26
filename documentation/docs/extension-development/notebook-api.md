# Notebook API

DSCode implements the `vscode.notebooks` namespace in the extension host, enabling extensions to provide interactive notebook experiences similar to Jupyter notebooks. Extensions can define notebook types, execute cells, and produce rich outputs.

**Source:** `extension-host/src/api/notebooks.ts`

!!! warning "Frontend rendering incomplete"
    The extension host API for notebooks is implemented, but the **frontend notebook renderer** in DSCode is not yet complete. Extensions can register notebook controllers and process cell executions, but visual rendering of notebook documents in the editor is limited. Full notebook rendering is planned for a future release.

---

## Core Concepts

| Concept | Description |
|---------|-------------|
| **NotebookController** | Handles cell execution for a notebook type |
| **NotebookCellKind** | `Markup` (markdown) or `Code` (executable) |
| **NotebookCellOutput** | Output produced by executing a cell |
| **NotebookCellExecution** | Manages the lifecycle of a single cell execution |

---

## Cell Kinds

```typescript
enum NotebookCellKind {
    Markup = 1,  // Markdown or rich text cells
    Code = 2     // Executable code cells
}
```

---

## Creating a Notebook Controller

A notebook controller handles execution for a specific notebook type:

```typescript
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const controller = vscode.notebooks.createNotebookController(
        'myKernel',       // Unique ID
        'jupyter-notebook', // Notebook type to handle
        'My Kernel'        // Display label
    );

    controller.supportedLanguages = ['python', 'javascript'];
    controller.supportsExecutionOrder = true;

    controller.executeHandler = async (cells, notebook, ctrl) => {
        for (const cell of cells) {
            await executeCell(ctrl, cell);
        }
    };

    context.subscriptions.push(controller);
}
```

---

## Cell Execution

Use `createNotebookCellExecution()` to manage a cell's execution lifecycle:

```typescript
async function executeCell(
    controller: vscode.NotebookController,
    cell: vscode.NotebookCell
) {
    const execution = controller.createNotebookCellExecution(cell);
    execution.executionOrder = ++executionCount;
    execution.start(Date.now());

    try {
        // Execute the cell code
        const result = await runCode(cell.document.getText());

        // Replace output with results
        await execution.replaceOutput([
            new vscode.NotebookCellOutput([
                vscode.NotebookCellOutputItem.text(result, 'text/plain')
            ])
        ]);

        execution.end(true, Date.now());
    } catch (error) {
        await execution.replaceOutput([
            new vscode.NotebookCellOutput([
                vscode.NotebookCellOutputItem.error(error as Error)
            ])
        ]);

        execution.end(false, Date.now());
    }
}
```

---

## Cell Output Types

The `NotebookCellOutputItem` class provides factory methods for common output types:

| Factory Method | MIME Type | Use Case |
|---------------|-----------|----------|
| `text(value, mime?)` | `text/plain` | Plain text output |
| `json(value, mime?)` | `application/json` | Structured data |
| `error(value)` | `application/vnd.code.notebook.error` | Error with stack trace |
| `stdout(value)` | `application/vnd.code.notebook.stdout` | Standard output |
| `stderr(value)` | `application/vnd.code.notebook.stderr` | Standard error |

```typescript
// Rich output example
const output = new vscode.NotebookCellOutput([
    // Multiple representations of the same output
    vscode.NotebookCellOutputItem.text('<table>...</table>', 'text/html'),
    vscode.NotebookCellOutputItem.text('| col1 | col2 |', 'text/plain'),
    vscode.NotebookCellOutputItem.json({ rows: [...] }, 'application/json')
]);
```

---

## Execution State

Cell execution progresses through states:

```typescript
enum NotebookCellExecutionState {
    Idle = 1,       // Not executing
    Pending = 2,    // Queued for execution
    Executing = 3   // Currently executing
}
```

---

## Notebook Document Events

Extensions can listen for notebook lifecycle events:

```typescript
vscode.notebooks.onDidOpenNotebookDocument(notebook => {
    console.log(`Opened: ${notebook.uri}`);
});

vscode.notebooks.onDidCloseNotebookDocument(notebook => {
    console.log(`Closed: ${notebook.uri}`);
});

vscode.notebooks.onDidSaveNotebookDocument(notebook => {
    console.log(`Saved: ${notebook.uri}`);
});
```

---

## Current Limitations

| Feature | Status |
|---------|--------|
| Controller registration | Implemented |
| Cell execution lifecycle | Implemented |
| Output item creation | Implemented |
| Document events | Implemented |
| Frontend cell rendering | **Not yet implemented** |
| Notebook serialization | Implemented |
| Custom renderers | **Not yet implemented** |
| Notebook workspace edit | **Not yet implemented** |

---

## See Also

- [VS Code API Reference](vscode-api-reference.md) -- full API documentation
- [VS Code Compatibility Matrix](../reference/vscode-compatibility-matrix.md) -- Notebook API coverage
