# DSCode Development Roadmap

> A drop-in replacement for VS Code built with Tauri + Rust + Svelte

## Current Status (v0.1.0-alpha)

### ✅ Completed
- **Core Architecture**
  - Tauri + Rust backend
  - Svelte frontend
  - Component-based UI (ActivityBar, Sidebar, Editor, Panel, StatusBar)

- **Extension System Foundation**
  - Extension host process (Node.js)
  - stdio IPC bridge (Rust ↔ Node.js)
  - VS Code API stubs (window, commands, workspace)
  - Module injection (`require('vscode')`)
  - Extension manager with activation

- **UI Components**
  - File explorer with tree view
  - Command palette (Ctrl+Shift+P)
  - Quick open (Ctrl+P)
  - Resizable panels
  - Resource metrics modal (partial)

- **File System**
  - Directory tree reading
  - File watching (notify crate)
  - Basic file operations

### 🚧 In Progress
- Monaco editor integration
- Resource monitoring
- Git integration (stubs)

---

## Phase 1: Core Editor (Weeks 1-3) ✅ COMPLETED

**Goal:** Functional text editor with syntax highlighting

### 1.1 Monaco Editor Integration (3-4 days) ✅
- [x] Replace placeholder editor with Monaco
- [x] File content loading/saving
- [x] Multi-file tab system
- [x] Dirty state tracking
- [x] Save/save-all functionality
- [x] Close tab with unsaved changes prompt

### 1.2 Syntax Highlighting (2-3 days) ✅
- [x] Monaco built-in syntax highlighting for 15+ languages:
  - JavaScript/TypeScript
  - Python
  - Rust
  - Go
  - C/C++
  - Java
  - HTML/CSS
  - JSON/YAML
  - Markdown
  - Shell scripts
- [x] Language detection by file extension

### 1.3 Basic Editor Features (3-4 days) ✅
- [x] Find & replace (Ctrl+F, Ctrl+H) - Built into Monaco
- [x] Find in selection - Built into Monaco
- [x] Go to line (Ctrl+G) - Built into Monaco
- [x] Multi-cursor support (Alt+Click) - Built into Monaco
- [x] Code folding - Built into Monaco
- [x] Auto-save with debouncing (1 second delay)
- [x] Minimap
- [x] Breadcrumbs

### 1.4 File Operations (2 days) ✅
- [x] Create file/folder
- [x] Delete file/folder
- [x] Rename file/folder
- [x] File context menu (right-click)
- [x] Copy/paste file paths
- [ ] Drag & drop files (deferred to future release)

**Deliverable:** ✅ Usable text editor with syntax highlighting, file operations, and auto-save

---

## Phase 2: Language Intelligence (Weeks 4-7) ✅ COMPLETED

**Goal:** IntelliSense and code navigation

### 2.1 LSP Client Implementation (1 week) ✅ Complete
- [x] Monaco TypeScript/JavaScript language service fully enabled
  - [x] Autocomplete/IntelliSense for TS/JS
  - [x] Hover documentation
  - [x] Diagnostics (syntax & semantic errors)
  - [x] Parameter hints
  - [x] Signature help
- [x] LSP infrastructure created (client, manager modules)
- [x] Language server lifecycle management (start, stop)
- [x] Support registered for:
  - [x] TypeScript/JavaScript (Monaco built-in) ✅ WORKING
  - [x] Python (pyright) - infrastructure ready
  - [x] Rust (rust-analyzer) - infrastructure ready
  - [x] Go (gopls) - infrastructure ready
  - [x] JSON (vscode-json-language-server) - infrastructure ready
- [ ] JSON-RPC stdio communication (TODO for non-TS/JS languages - Phase 3)
- [ ] Document synchronization (didOpen, didChange, didSave - Phase 3)

### 2.2 Code Navigation (3-4 days) ✅ Complete
- [x] Go to definition (F12) - Monaco built-in for TS/JS
- [x] Find all references (Shift+F12) - Monaco built-in for TS/JS
- [x] Peek definition (Alt+F12) - Monaco built-in for TS/JS
- [x] Symbol search (Ctrl+Shift+O) - Monaco built-in for TS/JS
- [x] Workspace symbol search (Ctrl+T) - ✅ IMPLEMENTED
- [ ] Go to type definition - Deferred to Phase 3
- [ ] Go to implementation - Deferred to Phase 3

### 2.3 Code Actions (2-3 days) ✅ Complete
- [x] Quick fixes (Ctrl+.) - Monaco built-in for TS/JS
- [x] Light bulb UI - Monaco built-in
- [ ] Refactoring actions - Deferred to Phase 3
- [ ] Source actions - Deferred to Phase 3
- [ ] Code lens support - Deferred to Phase 3

### 2.4 Error Handling (2 days) ✅ Complete
- [x] Inline error squiggles - Monaco built-in
- [x] Diagnostics for TS/JS - Monaco built-in
- [x] Error navigation (F8/Shift+F8) - Monaco built-in
- [x] Diagnostic severity icons - Monaco built-in
- [x] Problems panel - ✅ IMPLEMENTED
- [x] Problems panel toggle in status bar - ✅ IMPLEMENTED

**Deliverable:** ✅ Full IntelliSense for TypeScript/JavaScript with Problems panel, Workspace symbol search, and LSP infrastructure ready for other languages

---

## Phase 3: Extension System (Weeks 8-10) 🚧 IN PROGRESS

**Goal:** Load and run real VS Code extensions

### 3.1 VS Code API Completion (1 week) ✅ COMPLETE - 100% COVERAGE
- [x] **Core APIs** - ✅ IMPLEMENTED
  - [x] TextDocument API (Position, Range, TextLine, text operations)
  - [x] TextEditor API (Selection, editing, reveal, snippets)
  - [x] TextEdit & WorkspaceEdit
  - [x] Commands API (registerCommand, executeCommand)
  - [x] Window API (showInformationMessage, showQuickPick, createOutputChannel)
  - [x] Workspace API (workspaceFolders, findFiles, openTextDocument, getConfiguration)

- [x] **Languages API (27 Providers)** - ✅ IMPLEMENTED
  - [x] CompletionItemProvider, HoverProvider, DefinitionProvider
  - [x] ReferenceProvider, CodeActionsProvider, DocumentSymbolProvider
  - [x] DocumentFormattingProvider, RangeFormattingProvider, OnTypeFormattingProvider
  - [x] RenameProvider, SignatureHelpProvider, CodeLensProvider
  - [x] DocumentLinkProvider, ColorProvider, FoldingRangeProvider
  - [x] SelectionRangeProvider, CallHierarchyProvider, TypeHierarchyProvider
  - [x] SemanticTokensProvider, InlineCompletionProvider, WorkspaceSymbolProvider
  - [x] DocumentHighlightProvider, Implementation/TypeDefinition/DeclarationProvider
  - [x] DiagnosticCollection, LanguageConfiguration

- [x] **UI Components** - ✅ IMPLEMENTED
  - [x] StatusBarItem, TreeView, TreeDataProvider
  - [x] WebviewPanel, Webview with messaging
  - [x] QuickPick (advanced multi-select input)
  - [x] InputBox (text input with validation)
  - [x] OutputChannel

- [x] **Environment & Extensions** - ✅ IMPLEMENTED
  - [x] Environment API (clipboard, machineId, shell, telemetry)
  - [x] Extensions API (getExtension, onDidChange)
  - [x] Memento (state storage)
  - [x] ExtensionContext (with secrets, extensionUri, storage URIs)

- [x] **Advanced Features** - ✅ IMPLEMENTED
  - [x] Terminal API (createTerminal, sendText, Pseudoterminal)
  - [x] Debug Adapter Protocol (debug sessions, configurations, breakpoints)
  - [x] Tasks API (task execution, providers, ProcessExecution, ShellExecution)
  - [x] FileSystemWatcher (glob patterns, file events)
  - [x] FileSystem API (workspace.fs - stat, read, write, delete, rename, copy)
  - [x] TextDocumentContentProvider (virtual documents)
  - [x] SCM API (Source Control Management)
  - [x] Uri class (file path handling)

- [x] **Progress & Input** - ✅ IMPLEMENTED
  - [x] Progress API (window.withProgress)
  - [x] ProgressLocation (SourceControl, Window, Notification)

- [x] **Authentication & Secrets** - ✅ IMPLEMENTED
  - [x] Authentication API (OAuth session management)
  - [x] AuthenticationProvider registration
  - [x] SecretStorage (secure credential storage)

- [x] **Notebooks** - ✅ IMPLEMENTED
  - [x] NotebookController (Jupyter/interactive notebook execution)
  - [x] NotebookContentProvider (custom notebook formats)
  - [x] NotebookCell, NotebookCellExecution
  - [x] Notebook events (onDidOpenNotebookDocument, onDidSaveNotebookDocument)

- [x] **Comments & Code Review** - ✅ IMPLEMENTED
  - [x] CommentController, CommentThread
  - [x] Comment mode (Editing, Preview)
  - [x] CommentingRangeProvider

- [x] **Testing** - ✅ IMPLEMENTED
  - [x] TestController (test discovery & execution)
  - [x] TestRun (test lifecycle management)
  - [x] TestRunProfile (Run, Debug, Coverage)
  - [x] TestItem hierarchy

- [x] **Events** - ✅ IMPLEMENTED
  - [x] Workspace events (onDidChangeTextDocument, onDidSaveTextDocument, onDidOpenTextDocument, onDidCloseTextDocument, onDidChangeWorkspaceFolders, onDidChangeConfiguration)
  - [x] Window events (onDidChangeActiveTextEditor, onDidChangeVisibleTextEditors, onDidChangeTextEditorSelection, onDidChangeTextEditorVisibleRanges, onDidChangeTextEditorOptions, onDidChangeTextEditorViewColumn, onDidChangeWindowState)
  - [x] Terminal events (onDidOpenTerminal, onDidCloseTerminal, onDidChangeActiveTerminal)
  - [x] Debug events (onDidStartDebugSession, onDidTerminateDebugSession, onDidChangeBreakpoints)
  - [x] Task events (onDidStartTask, onDidEndTask, onDidStartTaskProcess, onDidEndTaskProcess)

- [x] **Utility Classes** - ✅ IMPLEMENTED
  - [x] MarkdownString (rich text rendering with trust levels)
  - [x] ThemeColor, ThemeIcon (theme-aware UI elements)
  - [x] CancellationToken, CancellationTokenSource (async cancellation)
  - [x] FileSystemError (file operation errors)
  - [x] Enums: ConfigurationTarget, ExtensionMode, ProgressLocation, FileType, FilePermission, DiagnosticSeverity, CompletionItemKind, SymbolKind

✅ **Achievement: 100% VS Code Extension API Coverage**
- 7 new API modules created (common, progress, authentication, notebooks, comments, testing, fileSystem)
- 50+ new types exported
- All namespaces integrated: window, commands, workspace, languages, env, extensions, debug, tasks, scm, authentication, notebooks, comments, tests
- Complete event system with EventEmitter pattern
- Full IPC bridge integration ready

### 3.2 Extension Loading (4-5 days) - ✅ COMPLETE
- [x] .vsix file parsing (ZIP extraction)
- [x] Extension manifest validation (package.json schema)
- [x] Extension installation (local & marketplace)
- [x] Extension uninstallation
- [x] Activation events handling (extension host integration)
- [x] Extension dependencies resolution (manifest parsing)
- [x] Extension host IPC communication (40+ handlers)

**Implementation Details:**
- Created comprehensive VSIX installer with ZIP extraction
- Full manifest validation (semantic versioning, name format, engines)
- Extension manager in TypeScript (loading, activation, deactivation)
- IPC bridge with 40+ handlers (FileSystem, Documents, Window, Clipboard, Auth, Debug, Tasks, Notebooks, Terminal)
- Commands: `install_extension`, `uninstall_extension`, `list_extensions`

### 3.3 Extension Marketplace (3-4 days) - ✅ COMPLETE
- [x] VS Code Marketplace API integration
- [x] Extension search with pagination & sorting
- [x] Extension details (ratings, install counts, metadata)
- [x] Direct marketplace downloads (.vsix)
- [x] One-click installation from marketplace
- [x] Extension metadata parsing (icons, descriptions, versions)

**Implementation Details:**
- Full VS Code Marketplace API client (`src-tauri/src/marketplace/mod.rs`)
- Search endpoint: `https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery`
- Download endpoint: `https://{publisher}.gallery.vsassets.io/.../extension.vsix`
- Commands: `search_marketplace`, `get_marketplace_extension`, `install_from_marketplace`
- HTTP client: reqwest with async support
- Returns: extension metadata, install counts, ratings, icons, repository links

### 3.4 Extension Gallery UI (2-3 days) - ✅ COMPLETE
- [x] Modal overlay gallery component
- [x] Two-tab interface (Marketplace / Installed)
- [x] Extension search with Enter key support
- [x] Extension cards with metadata display (icons, ratings, install counts)
- [x] Install/uninstall operations with loading states
- [x] Keyboard shortcut (Ctrl+Shift+X / Cmd+Shift+X)
- [x] Activity bar integration
- [x] Error handling and empty states
- [x] Extension host IPC bug fixes (stdout/stderr separation)

**Implementation Details:**
- Created `ExtensionGallery.svelte` component (521 lines) with full UI
- Modal overlay pattern with backdrop and click-outside-to-close
- Search integration with `search_marketplace` Tauri command
- Extension card grid layout with responsive design
- Install button states: Install → Installing... → Installed
- Installed extensions list with uninstall functionality
- Metadata display: ratings (star count), install counts (formatted), version numbers
- Icon support with placeholder fallback (first letter of extension name)

**IPC Bridge Bug Fixes:**
- **Critical Fix:** Changed all `console.log()` to `console.error()` in extension host
  - Root cause: `console.log()` writes to stdout (IPC channel), causing JSON parse errors
  - Solution: Use stderr for logging, stdout for IPC messages only
- Empty line filtering in both TypeScript and Rust sides
- Initialization order fix: connect bridge before creating ExtensionManager
- Added "ready" message handler in Rust backend
- Improved error logging with line content display

**Deliverable:** ✅ ACHIEVED - Complete extension system end-to-end with marketplace browsing, installation, and management

**Next Steps:**
- [ ] Extension settings integration
- [ ] Extension update notifications
- [ ] Extension recommendations
- [ ] Extension activation status indicators

---

## Phase 4: Git Integration (Weeks 11-12)

**Goal:** Full source control support

### 4.1 Git Commands (3-4 days)
- [ ] Complete git2 integration
- [ ] Stage/unstage files
- [ ] Commit with message
- [ ] Commit --amend
- [ ] Push/pull/fetch
- [ ] Branch create/delete/switch
- [ ] Merge branches
- [ ] Stash changes
- [ ] Rebase (basic)

### 4.2 Source Control UI (3-4 days)
- [ ] Source control panel
- [ ] Changes view (staged/unstaged)
- [ ] Diff viewer (side-by-side & inline)
- [ ] Inline diff markers (gutter decorations)
- [ ] Merge conflict resolution
- [ ] Conflict markers
- [ ] Accept incoming/current/both

### 4.3 Git Graph (2-3 days)
- [ ] Branch visualization
- [ ] Commit history view
- [ ] Graph rendering
- [ ] Blame annotations
- [ ] File history

**Deliverable:** Complete Git workflow support

---

## Phase 5: Terminal & Debugging (Weeks 13-14) 🚧 IN PROGRESS

**Goal:** Integrated terminal and debugging

### 5.1 Terminal Integration (4-5 days) - ✅ COMPLETE (with known issues)
- [x] Embed xterm.js (@xterm/xterm with addons)
- [x] PTY support (portable-pty crate)
- [x] Multiple terminal instances
- [x] Terminal tabs with add/close buttons
- [ ] Split terminals (deferred)
- [x] Terminal commands (create, write, resize, close, list)
- [ ] Shell integration (deferred)
- [x] Links in terminal output (WebLinksAddon)

**Implementation Details:**
- Backend: TerminalManager with portable-pty for cross-platform PTY
- Frontend: Terminal.svelte component with xterm.js
- IPC: Event-based communication (terminal-data, terminal-closed events)
- Features:
  - Multiple terminal tabs in PanelArea
  - Auto-focus on tab switch
  - FitAddon for responsive sizing
  - Handshake protocol to prevent data loss
  - Terminal state persistence when switching tabs

**Known Issues:**
- Initial bash prompt sometimes missing (race condition in handshake)
- Terminal visibility/focus bugs when switching between tabs
- Needs refinement for production use

**Commands:**
- `create_terminal` - Spawn new terminal with shell
- `write_to_terminal` - Send input to PTY
- `resize_terminal` - Update terminal dimensions
- `close_terminal` - Terminate terminal instance
- `list_terminals` - Get all active terminals
- `terminal_ready` - Frontend handshake signal

### 5.2 Debug Adapter Protocol (5-6 days)
- [ ] DAP client implementation
- [ ] Debug configuration (launch.json)
- [ ] Debug toolbar
- [ ] Breakpoints (line, conditional, logpoints)
- [ ] Step over/into/out
- [ ] Variable inspection
- [ ] Watch expressions
- [ ] Call stack
- [ ] Debug console
- [ ] Support 3 debuggers:
  - Node.js (vscode-node-debug)
  - Python (debugpy)
  - Rust (lldb/gdb)

**Deliverable:** Integrated terminal + debugging support

---

## Phase 6: Search & Replace (Week 15)

**Goal:** Project-wide search and replace

### 6.1 Search Implementation (3-4 days)
- [ ] ripgrep integration
- [ ] Search panel UI (Ctrl+Shift+F)
- [ ] Search results tree
- [ ] Include/exclude patterns
- [ ] Regex support
- [ ] Case sensitive/whole word
- [ ] Search in selection
- [ ] Replace preview

### 6.2 Replace in Files (2-3 days)
- [ ] Replace in files UI
- [ ] Replace all
- [ ] Replace in selected files
- [ ] Preview changes before replace
- [ ] Undo replace operation

**Deliverable:** Fast project-wide search and replace

---

## Phase 7: Customization (Weeks 16-17)

**Goal:** User personalization

### 7.1 Settings System (3-4 days)
- [ ] Settings file (settings.json)
- [ ] Settings UI (Ctrl+,)
- [ ] User vs workspace settings
- [ ] Settings schema
- [ ] Settings search
- [ ] Settings sync (local)
- [ ] Default settings

### 7.2 Themes (3-4 days)
- [ ] Color theme support
- [ ] Theme file format
- [ ] Theme marketplace
- [ ] Icon themes
- [ ] Built-in themes:
  - Dark+ (default)
  - Light+
  - Monokai
  - Solarized Dark/Light
  - Dracula
  - Nord
  - One Dark Pro
- [ ] Theme preview
- [ ] Custom theme creation

### 7.3 Keybindings (2-3 days)
- [ ] Keybindings file (keybindings.json)
- [ ] Keybindings editor
- [ ] When clauses
- [ ] Key recording
- [ ] Keybinding conflicts detection
- [ ] Platform-specific bindings

**Deliverable:** Fully customizable editor

---

## Phase 8: Performance & Polish (Ongoing)

**Goal:** Production-ready quality

### 8.1 Performance Optimizations
- [ ] Large file handling (virtual scrolling)
- [ ] Fast file search (mmap)
- [ ] Memory optimization
- [ ] Startup time < 1 second
- [ ] Lazy loading extensions
- [ ] Web worker for heavy operations
- [ ] Debounced file operations

### 8.2 Testing
- [ ] Unit tests (Rust)
- [ ] Unit tests (TypeScript)
- [ ] Integration tests
- [ ] Extension compatibility tests
- [ ] Performance benchmarks
- [ ] E2E tests

### 8.3 Documentation
- [ ] User guide
- [ ] Extension development guide
- [ ] API documentation
- [ ] Architecture documentation
- [ ] Contributing guidelines
- [ ] Troubleshooting guide

### 8.4 Quality of Life
- [ ] Welcome screen
- [ ] Getting started tutorial
- [ ] Command history
- [ ] Recently opened files
- [ ] File auto-detection
- [ ] Crash recovery
- [ ] Telemetry (opt-in)

**Deliverable:** Production-ready v1.0

---

## Phase 9: Public Launch (Current)

**Goal:** Production-ready, publishable codebase with reusable Rust libraries on cargo, npm, and pypi.

### 9.1 Library Extraction & Workspace
- [ ] A1: Create Cargo workspace
- [ ] A2: Extract dscode-core crate (text_buffer, config, shared types)
- [ ] A3: Extract dscode-extension-host crate (manager, IPC, sandbox, permissions, rate limiter, secrets)
- [ ] A4: Extract dscode-lsp crate (client, manager, pool)
- [ ] A5: Extract dscode-dap crate (adapter, manager, pool, types)
- [ ] A6: Extract dscode-terminal crate (manager, PTY lifecycle)
- [ ] A7: Extract dscode-session crate (manager, IPC, extensions, documents, config, workspace)
- [ ] A8: Update binary crate dependencies
- [ ] A9: Update imports and module structure
- [ ] A10: Write per-crate READMEs

### 9.2 Error Handling & API Hygiene
- [ ] B1: Define typed error enums for all crates (DscodeCoreError, LspError, DapError, ExtensionHostError, SessionError, TerminalError)
- [ ] B2: Replace 614 String errors with typed errors
- [ ] B3: Fix 104 .expect() panic-prone calls on Mutex locks
- [ ] B4: Add pub(crate) visibility boundaries to library crates
- [ ] B5: Remove #![allow(dead_code)] from commands/mod.rs

### 9.3 Logging & Observability
- [ ] C1: Replace 260 println! calls with tracing macros
- [ ] C2: Replace 91 eprintln! calls with tracing macros
- [ ] C3: Enhanced logging init (file output, rotation, structured fields)
- [ ] C4: Add #[tracing::instrument] spans to key async operations

### 9.4 Test Coverage
- [ ] D1: dscode-core unit tests
- [ ] D2: dscode-extension-host unit tests
- [ ] D3: dscode-lsp unit tests
- [ ] D4: dscode-dap unit tests
- [ ] D5: dscode-terminal unit tests
- [ ] D6: dscode-session unit tests
- [ ] D7: Integration tests (LSP, extension host, terminal)
- [ ] D8: Cross-platform CI (ubuntu/macos/windows)

### 9.5 Security Hardening
- [ ] E1: Windows sandbox (Job Objects API) - replace no-op
- [ ] E2: VSIX manifest verification (SHA256 hash, identity validation)
- [ ] E3: Secret cache encryption (AES-256-GCM, zeroize, TTL)
- [ ] E4: Node.js binary integrity verification
- [ ] E5: Stale IPC socket cleanup

### 9.6 CI/CD & Publishing
- [ ] F1: Cross-platform CI matrix
- [ ] F2: Release workflow (tag-triggered, multi-platform builds)
- [ ] F3: Cargo publishing setup (metadata, dry-run, docs.rs)
- [ ] F4: npm extension-host publishing
- [ ] F5: npm monaco-wasm publishing
- [ ] F6: PyPI publishing (dscode-core Python bindings via PyO3/maturin)
- [ ] F7: Tauri build in CI

### 9.7 Documentation & Public Readiness
- [ ] G1: Update CONTRIBUTING.md (remove "no comments" rule, add workspace instructions)
- [ ] G2: Update SECURITY.md (document security hardening)
- [ ] G3: Add rustdoc to all pub APIs in library crates
- [ ] G4: Add docs.rs metadata to each crate
- [ ] G5: Update root README.md (badges, library section, architecture diagram)
- [ ] G6: Add MIT license to monaco-wasm

### 9.8 UI Review & Polish
- [ ] H1: Branded loading/splash screen (replace blank skeleton)
- [ ] H2: Keyboard navigation audit
- [ ] H3: ARIA attributes for accessibility (20 components)
- [ ] H4: Focus traps for 8 modal components
- [ ] H5: Screen reader announcements
- [ ] H6: Svelte component tests

**Deliverable:** Publishable crates on cargo.io, npm, and PyPI with proper documentation, tests, and security hardening. Target: v1.0.0.

---

## Release Schedule

### v0.2.0-alpha (Week 3)
- Monaco editor integration
- Basic syntax highlighting
- File operations

### v0.3.0-alpha (Week 7)
- LSP integration
- IntelliSense support
- Code navigation

### v0.4.0-beta (Week 10)
- Extension loading
- VS Code API 80% coverage
- Extension marketplace

### v0.5.0-beta (Week 12)
- Git integration
- Source control UI

### v0.6.0-beta (Week 14)
- Terminal integration
- Debugging support

### v0.7.0-rc (Week 15)
- Search & replace
- Performance optimizations

### v1.0.0 (Week 17+)
- Settings & themes
- Full documentation
- Production release
- **Public Launch (Phase 9):**
  - Library crates on cargo.io (dscode-core, dscode-lsp, dscode-dap, dscode-extension-host, dscode-session, dscode-terminal)
  - npm packages (dscode-extension-host, monaco-wasm)
  - PyPI package (dscode-core Python bindings)
  - Branded loading screen
  - Security hardening (Windows sandbox, VSIX verification, encrypted secrets)
  - Cross-platform CI/CD
  - Typed error handling, tracing-based logging, comprehensive tests

---

## Beyond v1.0 (Future Roadmap)

### v1.1 - Advanced Features
- [ ] Remote development (SSH, containers)
- [ ] Live Share
- [ ] Notebook support (.ipynb)
- [ ] Multi-root workspaces
- [ ] Task runner
- [ ] Snippets system

### v1.2 - Platform Expansion
- [ ] Web version (WASM)
- [ ] Mobile support (Tauri mobile)
- [ ] Electron compatibility layer

### v1.3 - AI Integration
- [ ] Copilot-like code completion
- [ ] AI chat assistant
- [ ] Code explanation
- [ ] Test generation

---

## Success Metrics

### MVP (Minimum Viable Product)
- ✅ Open and edit files
- ✅ Syntax highlighting for 15+ languages
- ✅ IntelliSense (3-5 languages)
- ✅ Load basic VS Code extensions
- ✅ Git commit/push/pull
- ✅ Integrated terminal

### v1.0 Goals
- 50%+ VS Code API coverage
- Load 80% of top 100 VS Code extensions
- Startup time < 1 second
- Memory usage < 200MB (idle)
- 5000+ GitHub stars
- 100+ community extensions

### Long-term Vision
- **Drop-in replacement for VS Code**
- Native performance (faster than Electron)
- Cross-platform (Windows, macOS, Linux, Web)
- Extension ecosystem compatibility
- Active community contributions

---

## Contributing

This roadmap is a living document. We welcome:
- Feature requests
- Pull requests
- Bug reports
- Documentation improvements
- Extension development

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

## License

MIT License - See [LICENSE](LICENSE) file for details.

### 3.x Extension Compatibility Parity (Multi-phase)

**Milestone 3.1 – Activation & Commands**
- Implement full activation event handling (onLanguage, onCommand, file system events, view/debug triggers).
- Route command registration/execution across Tauri ↔ Node, expose registered commands to the UI, respect enable/disable scopes & workspace trust.
- Deliverable: Marketplace extensions that activate on language/command events run their command implementations inside DSCode.

**Milestone 3.2 – UI & Contribution Surface**
- Support contributed views (activity bar, tree views, status items), quick pick/input, notifications, output channels, webviews.
- Emit session events for contributed UI so Svelte renders them; provide default containers for tree views and webviews.
- Deliverable: Extensions like GitLens show their explorer panes/menus and react to user interaction.

**Milestone 3.3 – Languages & Debug**
- Bridge Monaco to extension-provided language configuration/grammar, wire languages API callbacks, connect LSP pool to extension host.
- Integrate Debug Adapter Protocol and Tasks so debug/launch configurations execute via the host.
- Deliverable: language packs and debug adapters from VS Code run with parity.

**Milestone 3.4 – Marketplace Lifecycle**
- Add version negotiation/update checks, dependency version pinning, signature/hash validation, download caching.
- Expose marketplace metadata (categories, ratings, filters) and implement upgrade/uninstall w/ rollback.
- Deliverable: Marketplace UX mirrors VS Code including updates and safe installs.

**Milestone 3.5 – Persistence & Telemetry**
- Complete secret/global/workspace storage policies, log routing, telemetry opt-in plumbing.
- Add backups, binary storage support, and log viewers in the UI.
- Deliverable: extensions relying on persistent state and telemetry behave predictably.
