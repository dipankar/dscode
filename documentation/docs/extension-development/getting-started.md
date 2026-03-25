---
title: Getting Started with Extension Development
description: Build your first DSCode extension -- from scaffolding to packaging. A step-by-step Hello World tutorial.
---

# Getting Started with Extension Development

Build your first DSCode extension in under 15 minutes. This tutorial walks you through scaffolding a project, writing a simple command, running it inside DSCode, and packaging it for distribution.

---

## Prerequisites

Before you begin, make sure you have the following installed:

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| **Node.js** | 18.0+ | Extension host runtime |
| **npm** | 9.0+ | Package management |
| **DSCode** | 0.1.0+ | Editor to test in |

!!! tip "Check your versions"
    ```bash
    node --version   # v18.0.0 or higher
    npm --version    # 9.0.0 or higher
    ```

You will also need the Yeoman generator and the VS Code Extension packaging tool:

```bash
npm install -g yo generator-code @vscode/vsce
```

---

## Scaffold a New Extension

Use the Yeoman generator to create a new extension project:

```bash
npx yo code
```

You will be prompted with a series of questions. For this tutorial, choose:

| Prompt | Value |
|--------|-------|
| What type of extension? | **New Extension (TypeScript)** |
| Extension name | `hello-dscode` |
| Identifier | `hello-dscode` |
| Description | `My first DSCode extension` |
| Initialize git repo? | **Yes** |
| Bundle with webpack? | **No** (keep it simple for now) |
| Package manager | **npm** |

This creates a `hello-dscode/` directory with the following structure:

```
hello-dscode/
├── .vscode/
│   ├── launch.json          # Debug configuration
│   └── tasks.json           # Build tasks
├── src/
│   └── extension.ts         # Extension entry point
├── package.json             # Extension manifest
├── tsconfig.json            # TypeScript configuration
└── README.md
```

---

## Understanding the Extension Manifest

The `package.json` file is the heart of every extension. It declares metadata, activation events, and contributions.

```json title="package.json"
{
  "name": "hello-dscode",
  "displayName": "Hello DSCode",
  "description": "My first DSCode extension",
  "version": "0.0.1",
  "publisher": "your-publisher-id",
  "engines": {
    "vscode": "^1.85.0" // (1)!
  },
  "categories": ["Other"],
  "activationEvents": [], // (2)!
  "main": "./out/extension.js", // (3)!
  "contributes": {
    "commands": [ // (4)!
      {
        "command": "hello-dscode.helloWorld",
        "title": "Hello World",
        "category": "Hello"
      }
    ]
  },
  "scripts": {
    "vscode:prepublish": "npm run compile",
    "compile": "tsc -p ./",
    "watch": "tsc -watch -p ./"
  },
  "devDependencies": {
    "@types/vscode": "^1.85.0",
    "typescript": "^5.3.0"
  }
}
```

1. DSCode uses the same engine compatibility range as VS Code. Extensions targeting `^1.85.0` work with DSCode 0.1.0+.
2. When `activationEvents` is empty, DSCode infers events from `contributes`. Here, it auto-generates `onCommand:hello-dscode.helloWorld`.
3. Points to the compiled JavaScript entry file. DSCode's extension host loads this when the extension activates.
4. The `contributes.commands` array registers commands that appear in the Command Palette.

!!! info "Key manifest fields"
    | Field | Required | Description |
    |-------|----------|-------------|
    | `name` | Yes | Unique extension identifier (lowercase, no spaces) |
    | `publisher` | Yes | Your publisher ID on the registry |
    | `engines.vscode` | Yes | Minimum API version required |
    | `main` | Yes | Path to the compiled entry point |
    | `activationEvents` | No | When to load the extension (auto-inferred if empty) |
    | `contributes` | No | Static contributions (commands, menus, keybindings, themes, etc.) |

---

## The Extension Entry Point

Open `src/extension.ts`. This is where your extension logic lives. Every extension must export two functions: `activate` and `deactivate`.

```typescript title="src/extension.ts"
import * as vscode from 'vscode';

/**
 * Called when the extension is activated.
 * Activation happens based on the events declared in package.json.
 */
export function activate(context: vscode.ExtensionContext) {
    console.log('Extension "hello-dscode" is now active!');

    // Register a command
    const disposable = vscode.commands.registerCommand(
        'hello-dscode.helloWorld',
        () => {
            // Show an information message in the editor
            vscode.window.showInformationMessage(
                'Hello from DSCode! 🚀'
            );
        }
    );

    // Add to subscriptions so it is disposed when the extension deactivates
    context.subscriptions.push(disposable);
}

/**
 * Called when the extension is deactivated.
 * Use this to clean up resources.
 */
export function deactivate() {
    // Nothing to clean up for this simple extension
}
```

!!! note "How it works under the hood"
    When the user runs the **Hello World** command from the Command Palette:

    1. DSCode's extension host receives the `onCommand:hello-dscode.helloWorld` activation event
    2. The host calls your `activate()` function
    3. The registered command handler runs and calls `showInformationMessage`
    4. The message is sent over NNG IPC from the Node.js extension host to the Rust backend
    5. The Rust backend forwards it to the Svelte frontend, which renders the notification toast

---

## Adding More Contributions

Let us enhance the extension with a keybinding and a status bar item.

### Keybinding

Add a keybinding in `package.json` so users can trigger the command with a keyboard shortcut:

```json title="package.json (contributes section)"
"contributes": {
    "commands": [
        {
            "command": "hello-dscode.helloWorld",
            "title": "Hello World",
            "category": "Hello"
        }
    ],
    "keybindings": [
        {
            "command": "hello-dscode.helloWorld",
            "key": "ctrl+shift+h",
            "mac": "cmd+shift+h",
            "when": "editorTextFocus"
        }
    ]
}
```

### Status Bar Item

Add a status bar item that shows a greeting and triggers the command when clicked:

```typescript title="src/extension.ts"
export function activate(context: vscode.ExtensionContext) {
    // Register command
    const disposable = vscode.commands.registerCommand(
        'hello-dscode.helloWorld',
        () => {
            vscode.window.showInformationMessage('Hello from DSCode!');
        }
    );

    // Create a status bar item
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(smiley) Hello';
    statusBarItem.tooltip = 'Click to say hello';
    statusBarItem.command = 'hello-dscode.helloWorld';
    statusBarItem.show();

    context.subscriptions.push(disposable, statusBarItem);
}
```

---

## Running and Debugging

### Method 1: Launch from DSCode

1. Open the `hello-dscode` folder in DSCode
2. Press ++f5++ or go to **Run > Start Debugging**
3. A new DSCode window opens with your extension loaded (the **Extension Development Host**)
4. Open the Command Palette (++ctrl+shift+p++ / ++cmd+shift+p++) and type `Hello World`
5. You should see the information message appear

### Method 2: Command Line

Compile the extension and load it directly:

```bash
# Compile TypeScript
cd hello-dscode
npm run compile

# Launch DSCode with the extension loaded
dscode --extensionDevelopmentPath=$(pwd)
```

### Debugging Tips

!!! tip "Setting breakpoints"
    - Set breakpoints in `.ts` files -- DSCode supports source maps out of the box
    - Use the **Debug Console** to evaluate expressions in the extension host context
    - Check the **Output** panel (channel: `Extension Host`) for `console.log` output

!!! warning "Reloading changes"
    After making code changes, you must reload the Extension Development Host window:

    - Press ++ctrl+shift+f5++ / ++cmd+shift+f5++ to restart
    - Or run **Developer: Reload Window** from the Command Palette

---

## Watching for Changes

For a faster development loop, run the TypeScript compiler in watch mode:

```bash
npm run watch
```

This recompiles on every save. Combine it with the reload shortcut for a tight feedback cycle.

---

## Packaging the Extension

When your extension is ready for distribution, package it as a `.vsix` file:

```bash
# Install vsce if you haven't already
npm install -g @vscode/vsce

# Package the extension
vsce package
```

This produces a file like `hello-dscode-0.0.1.vsix`. You can install it in DSCode:

```bash
dscode --install-extension hello-dscode-0.0.1.vsix
```

Or share it with others for manual installation via **Extensions > Install from VSIX...**.

!!! info "Before packaging"
    Make sure you have:

    - A valid `publisher` field in `package.json`
    - A `README.md` file (required by `vsce`)
    - A `LICENSE` file (recommended)
    - Run `npm run compile` to ensure a clean build

---

## Project Structure Reference

Here is the recommended structure for a production-quality extension:

```
my-extension/
├── .vscode/
│   ├── launch.json              # Debug configurations
│   └── tasks.json               # Build tasks
├── src/
│   ├── extension.ts             # Entry point
│   ├── commands/                # Command handlers
│   │   └── helloWorld.ts
│   ├── providers/               # Language providers, tree views, etc.
│   └── utils/                   # Shared utilities
├── test/
│   ├── suite/                   # Integration tests
│   │   └── extension.test.ts
│   └── runTest.ts               # Test runner
├── resources/                   # Icons, images, static files
├── package.json                 # Extension manifest
├── tsconfig.json                # TypeScript config
├── .vscodeignore                # Files to exclude from .vsix
├── CHANGELOG.md                 # Version history
├── README.md                    # Extension documentation
└── LICENSE                      # License file
```

---

## Next Steps

You now have a working extension. Here is where to go next:

- **[VS Code API Reference](vscode-api-reference.md)** -- Explore the full API surface available in DSCode
- **[Activation Events](activation-events.md)** -- Control when your extension loads
- **[Language Providers](language-providers.md)** -- Add IntelliSense, hover info, and diagnostics
- **[Webview Extensions](webview-extensions.md)** -- Build custom UIs with HTML/CSS/JS
- **[Native Rust Plugins](native-rust-plugins.md)** -- DSCode-exclusive native plugin system
- **[Publishing](publishing.md)** -- Publish to the Open VSX registry
