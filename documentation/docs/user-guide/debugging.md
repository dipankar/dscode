# Debugging

DSCode provides an integrated debugging experience through the Debug Adapter Protocol (DAP). The Rust backend (`debug_ops` and `DebugManager`) manages debug sessions, while the frontend components -- `DebugView.svelte`, `DebugToolbar.svelte`, `DebugConsole.svelte`, `VariablesPanel.svelte`, and `CallStackPanel.svelte` -- provide the visual interface.

---

## Debug View

Open the Debug view by clicking the bug icon in the Activity Bar, or press ++ctrl+shift+d++.

The Debug view contains:

- **Launch configuration selector** -- dropdown to choose or create launch configurations
- **Start button** -- green play button to begin debugging
- **Breakpoints panel** -- list of all set breakpoints across files
- **Variables panel** -- inspect variable values during a debug session
- **Watch panel** -- track custom expressions
- **Call Stack panel** -- view the execution stack

---

## Launch Configurations

Debug launch configurations are defined in a `launch.json` file inside the `.vscode` folder of your workspace.

### Creating a Launch Configuration

1. Open the Debug view (++ctrl+shift+d++).
2. Click **create a launch.json file** if one does not exist, or click the gear icon to edit the existing one.
3. Select a debug environment (e.g., Node.js, Python, Rust) from the list.
4. DSCode generates a template configuration.

### Example Configurations

=== "Node.js"

    ```json
    {
      "version": "0.2.0",
      "configurations": [
        {
          "type": "node",
          "request": "launch",
          "name": "Launch Program",
          "program": "${workspaceFolder}/src/index.js",
          "outFiles": ["${workspaceFolder}/dist/**/*.js"]
        }
      ]
    }
    ```

=== "Python"

    ```json
    {
      "version": "0.2.0",
      "configurations": [
        {
          "type": "python",
          "request": "launch",
          "name": "Python: Current File",
          "program": "${file}",
          "console": "integratedTerminal"
        }
      ]
    }
    ```

=== "Rust (lldb)"

    ```json
    {
      "version": "0.2.0",
      "configurations": [
        {
          "type": "lldb",
          "request": "launch",
          "name": "Debug Rust",
          "cargo": {
            "args": ["build", "--bin=myapp"],
            "filter": {
              "name": "myapp",
              "kind": "bin"
            }
          },
          "args": [],
          "cwd": "${workspaceFolder}"
        }
      ]
    }
    ```

=== "C/C++ (GDB)"

    ```json
    {
      "version": "0.2.0",
      "configurations": [
        {
          "type": "cppdbg",
          "request": "launch",
          "name": "Debug C++",
          "program": "${workspaceFolder}/build/myapp",
          "args": [],
          "stopAtEntry": false,
          "cwd": "${workspaceFolder}",
          "MIMode": "gdb"
        }
      ]
    }
    ```

### Configuration Variables

Use these predefined variables in your launch configurations:

| Variable | Value |
|---|---|
| `${workspaceFolder}` | Root folder of your workspace |
| `${file}` | Currently open file |
| `${fileBasename}` | File name without path |
| `${fileDirname}` | Directory of the current file |
| `${fileExtname}` | Extension of the current file |
| `${lineNumber}` | Current cursor line number |

!!! tip
    You can define multiple configurations in `launch.json` and switch between them using the dropdown at the top of the Debug view. This is useful when your project has different entry points or test configurations.

---

## Setting Breakpoints

Breakpoints pause program execution at a specific line, allowing you to inspect state.

### Adding Breakpoints

- **Click the gutter** to the left of a line number to toggle a breakpoint (a red dot appears).
- **Press** ++f9++ to toggle a breakpoint on the current line.
- **Conditional breakpoint**: Right-click the gutter and select **Add Conditional Breakpoint...**. Enter a condition expression that must evaluate to `true` for the breakpoint to trigger.
- **Logpoint**: Right-click the gutter and select **Add Logpoint...**. Enter a message template (using `{}` for expressions). The program logs the message without pausing.

### Managing Breakpoints

The Breakpoints panel in the Debug view lists all breakpoints. From here you can:

- **Enable/disable** individual breakpoints with the checkbox.
- **Remove** a breakpoint by clicking the `x` icon.
- **Edit** a breakpoint's condition by right-clicking it.
- **Enable/disable all** breakpoints with the toolbar buttons.

The backend stores breakpoints as `SourceBreakpoint` structures:

```rust
pub struct SourceBreakpoint {
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
    pub hit_condition: Option<String>,
    pub log_message: Option<String>,
}
```

---

## Stepping Through Code

Once execution is paused at a breakpoint, use the Debug Toolbar (`DebugToolbar.svelte`) to control execution flow:

| Action | Shortcut | Description |
|---|---|---|
| Continue | ++f5++ | Resume execution until the next breakpoint |
| Step Over | ++f10++ | Execute the current line, stepping over function calls |
| Step Into | ++f11++ | Step into the function call on the current line |
| Step Out | ++shift+f11++ | Execute until the current function returns |
| Restart | ++ctrl+shift+f5++ | Stop and restart the debug session |
| Stop | ++shift+f5++ | Terminate the debug session |
| Pause | ++f6++ | Pause a running program |

The Rust backend manages session state transitions:

```rust
pub enum DebugState {
    Running,
    Paused,
    Stopped,
}
```

!!! note
    Step Into will enter library or framework code. If you want to skip over external code, configure `skipFiles` in your launch configuration:

    ```json
    {
      "skipFiles": ["<node_internals>/**", "node_modules/**"]
    }
    ```

---

## Variables Panel

The Variables panel (`VariablesPanel.svelte`) displays the values of variables in the current scope when execution is paused.

Variables are organized into scopes:

- **Local** -- variables in the current function
- **Closure** -- variables captured from enclosing scopes
- **Global** -- global/module-level variables

### Inspecting Variables

- **Expand** objects and arrays to see their properties.
- **Hover** over a variable name in the editor to see its current value in a tooltip.
- **Set value**: Right-click a variable in the panel and select **Set Value** to modify it during the debug session.

---

## Watch Expressions

The Watch panel lets you evaluate arbitrary expressions in the context of the paused program.

### Adding Watch Expressions

1. Click the `+` icon in the Watch panel.
2. Type an expression (e.g., `user.name`, `items.length`, `x + y`).
3. Press ++enter++.

The expression is re-evaluated each time execution pauses. Watch expressions persist across debug sessions.

!!! tip
    You can select any expression in the editor, right-click, and choose **Add to Watch** to add it to the Watch panel quickly.

---

## Call Stack

The Call Stack panel (`CallStackPanel.svelte`) shows the chain of function calls that led to the current execution point.

- Each frame displays the **function name**, **file name**, and **line number**.
- Click a frame to navigate to that location in the editor and see the variables in that frame's scope.
- If your program uses multiple threads, each thread appears as a separate section in the call stack.

---

## Debug Console

The Debug Console (`DebugConsole.svelte`) provides an interactive REPL for evaluating expressions during a debug session.

- Open it with ++ctrl+shift+y++ or from the Panel Area tabs.
- Type expressions at the prompt and press ++enter++ to evaluate them.
- The console also shows debug output (e.g., `console.log` output for Node.js, `print` output for Python).

```
> user.email
"alice@example.com"
> items.filter(i => i.active).length
5
> JSON.stringify(config, null, 2)
"{\n  \"debug\": true\n}"
```

!!! note
    The Debug Console evaluation context depends on the current stack frame selected in the Call Stack panel. Switch frames to evaluate expressions in different scopes.

---

## Debug Adapter Protocol (DAP) Support

DSCode implements the [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/), the same protocol used by VS Code. This means DSCode can work with any DAP-compliant debug adapter.

### Supported Debug Adapters

Through extensions, DSCode supports debuggers for many languages:

| Language | Debug Adapter | Extension |
|---|---|---|
| JavaScript / TypeScript | Node.js debugger | Built-in |
| Python | debugpy | Python extension |
| Rust | CodeLLDB (lldb) | CodeLLDB extension |
| C / C++ | cppdbg (GDB/LLDB) | C/C++ extension |
| Go | Delve | Go extension |
| Java | Java Debug Server | Java extension |
| C# / .NET | OmniSharp | C# extension |

### How DAP Works in DSCode

1. The Rust `DebugManager` creates a `DebugSession` and launches the appropriate debug adapter process.
2. DSCode communicates with the adapter using the DAP JSON protocol over stdin/stdout.
3. Frontend components display session state (breakpoints, variables, call stack) received from the adapter.
4. User actions (continue, step, etc.) are translated into DAP requests and sent to the adapter.

```rust
pub fn create_debug_session(
    debug_manager: State<Mutex<DebugManager>>,
    name: String,
    adapter_type: String,
) -> Result<String, String>
```

---

## Debugging Shortcuts Reference

| Action | Shortcut |
|---|---|
| Open Debug view | ++ctrl+shift+d++ |
| Start / Continue debugging | ++f5++ |
| Stop debugging | ++shift+f5++ |
| Restart debugging | ++ctrl+shift+f5++ |
| Step Over | ++f10++ |
| Step Into | ++f11++ |
| Step Out | ++shift+f11++ |
| Pause | ++f6++ |
| Toggle breakpoint | ++f9++ |
| Open Debug Console | ++ctrl+shift+y++ |

!!! warning
    Ensure the appropriate debug adapter extension is installed before starting a debug session. If no adapter is available for your language, DSCode will display an error prompting you to install one from the extension marketplace.
