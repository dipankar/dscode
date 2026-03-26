# Comments API

DSCode implements the `vscode.comments` namespace, enabling extensions to add inline comment threads to source files. This API is commonly used for code review integrations, inline annotations, and collaborative feedback.

**Source:** `extension-host/src/api/comments.ts`

---

## Overview

The Comments API provides three main types:

| Type | Description |
|------|-------------|
| **CommentController** | Manages comment threads for a specific provider |
| **CommentThread** | A thread of comments attached to a file location |
| **Comment** | An individual comment within a thread |

---

## Creating a Comment Controller

```typescript
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const controller = vscode.comments.createCommentController(
        'myReview',       // Unique ID
        'Code Review'     // Display label
    );

    // Optional: define where comments can be added
    controller.commentingRangeProvider = {
        provideCommentingRanges(document, token) {
            // Allow comments on any line
            return [new vscode.Range(0, 0, document.lineCount - 1, 0)];
        }
    };

    context.subscriptions.push(controller);
}
```

---

## Creating Comment Threads

A comment thread is anchored to a specific location in a file:

```typescript
const thread = controller.createCommentThread(
    vscode.Uri.file('/path/to/file.ts'),
    new vscode.Range(10, 0, 10, 0),  // Line 10
    [
        {
            body: 'Consider extracting this logic into a helper function.',
            mode: vscode.CommentMode.Preview,
            author: {
                name: 'Alice',
                iconPath: vscode.Uri.parse('https://example.com/alice.png')
            },
            timestamp: new Date()
        }
    ]
);

// Configure the thread
thread.collapsibleState = vscode.CommentThreadCollapsibleState.Expanded;
thread.canReply = true;
thread.label = 'Review Comment';
```

---

## Comment Properties

Each comment has the following properties:

| Property | Type | Description |
|----------|------|-------------|
| `body` | `string` or `MarkdownString` | The comment text content |
| `mode` | `CommentMode` | `Editing` (0) or `Preview` (1) |
| `author` | `{ name, iconPath? }` | Comment author information |
| `label` | `string?` | Optional label (e.g., "Author", "Reviewer") |
| `timestamp` | `Date?` | When the comment was created |

---

## Thread Collapsible State

```typescript
enum CommentThreadCollapsibleState {
    Collapsed = 0,  // Thread is collapsed, showing only a marker
    Expanded = 1    // Thread is expanded, showing all comments
}
```

---

## Adding Replies

Add replies to a thread by updating its `comments` array:

```typescript
// Add a reply
thread.comments = [
    ...thread.comments,
    {
        body: 'Good point! I will refactor this.',
        mode: vscode.CommentMode.Preview,
        author: {
            name: 'Bob'
        }
    }
];
```

---

## Use Case: Code Review Integration

```typescript
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const controller = vscode.comments.createCommentController(
        'prReview',
        'PR Review'
    );

    controller.commentingRangeProvider = {
        provideCommentingRanges(document) {
            return [new vscode.Range(0, 0, document.lineCount - 1, 0)];
        }
    };

    // Command to add a new review comment
    context.subscriptions.push(
        vscode.commands.registerCommand('prReview.addComment', (uri, range) => {
            const thread = controller.createCommentThread(uri, range, []);
            thread.canReply = true;
            thread.collapsibleState = vscode.CommentThreadCollapsibleState.Expanded;
        })
    );

    // Command to submit a reply
    context.subscriptions.push(
        vscode.commands.registerCommand('prReview.reply', (thread) => {
            // Submit the reply to your review backend
            vscode.window.showInformationMessage('Comment submitted!');
        })
    );

    context.subscriptions.push(controller);
}
```

---

## Disposing

Comment threads and controllers should be disposed when no longer needed:

```typescript
// Dispose a single thread
thread.dispose();

// Dispose the controller (disposes all threads)
controller.dispose();
```

---

## See Also

- [VS Code API Reference](vscode-api-reference.md) -- full API documentation
- [VS Code Compatibility Matrix](../reference/vscode-compatibility-matrix.md) -- Comments API coverage
