# DSCode Documentation

Welcome to the DSCode documentation! This guide will help you navigate through all available documentation resources.

## Quick Links

- [Main Project README](../README.md)
- [Project Roadmap](roadmap.md)

## Getting Started

New to DSCode? Start here:

- **[Installation Guide](getting-started/installation.md)** - System requirements and installation instructions
- **[Quick Start](getting-started/quick-start.md)** - Get up and running in 5 minutes
- **[Getting Started Guide](getting-started/getting-started.md)** - Comprehensive introduction
- **[Features Overview](getting-started/features.md)** - Key features and capabilities

## Architecture Documentation

Deep dive into DSCode's technical architecture:

- **[Architecture Overview](architecture/overview.md)** - High-level system design and Tauri + Rust backend
- **[IPC Design](architecture/ipc-design.md)** - Tauri IPC and nng communication patterns
- **[Extension System](architecture/extension-system.md)** - Node.js runtime and VS Code API compatibility
- **[UI System](architecture/ui-system.md)** - Monaco editor and component architecture
- **[LSP Integration](architecture/lsp-integration.md)** - Language Server Protocol implementation
- **[Remote Development](architecture/remote-development.md)** - SSH, WSL, and container support
- **[Native API](architecture/native-api.md)** - Rust plugin API for native extensions
- **[Telemetry](architecture/telemetry.md)** - Privacy-first analytics and monitoring
- **[VS Code Compatibility](architecture/vscode-compat.md)** - Extension compatibility and API parity

## Development Guides

For contributors and developers:

- **[Extension Compatibility Plan](development/extension-compatibility-plan.md)** - 🎯 **NEW: Detailed 6-month plan to achieve 100% VS Code extension compatibility**
- **[Implementation Timeline](development/implementation-timeline.md)** - Detailed month-by-month development plan
- **[Testing Guide](development/testing.md)** - Testing strategies and how to run tests
- **[Session Manager Architecture](development/session-manager.md)** - Session management implementation
- **[Performance Optimization](development/performance.md)** - Performance best practices and benchmarks
- **[WASM Optimization](development/wasm-optimization.md)** - WebAssembly optimization techniques

## Project Planning

- **[Roadmap](roadmap.md)** - Current development status and future plans
- **[Implementation Timeline](development/implementation-timeline.md)** - Detailed month-by-month breakdown

## Documentation Structure

```
docs/
├── README.md                           # This file - documentation index
├── roadmap.md                          # Project roadmap and current status
│
├── getting-started/                    # User-facing documentation
│   ├── installation.md                 # Installation guide
│   ├── quick-start.md                  # Quick start tutorial
│   ├── getting-started.md              # Comprehensive getting started guide
│   └── features.md                     # Features overview
│
├── architecture/                       # Technical architecture docs
│   ├── overview.md                     # Architecture overview
│   ├── ipc-design.md                   # IPC system design
│   ├── extension-system.md             # Extension architecture
│   ├── ui-system.md                    # UI architecture
│   ├── lsp-integration.md              # LSP implementation
│   ├── remote-development.md           # Remote dev architecture
│   ├── native-api.md                   # Native plugin API
│   ├── telemetry.md                    # Telemetry system
│   └── vscode-compat.md                # VS Code compatibility
│
└── development/                        # Developer guides
    ├── implementation-timeline.md      # Development timeline
    ├── testing.md                      # Testing guide
    ├── session-manager.md              # Session manager docs
    ├── performance.md                  # Performance guide
    └── wasm-optimization.md            # WASM optimization
```

## Contributing

Want to contribute to DSCode? Check out:

1. [Main README](../README.md) - Project overview and setup
2. [Architecture Overview](architecture/overview.md) - Understand the codebase
3. [Testing Guide](development/testing.md) - How to test your changes
4. CONTRIBUTING.md (coming soon) - Contribution guidelines

## Additional Resources

- **GitHub Repository**: [DSCode on GitHub](https://github.com/yourusername/dscode)
- **Issue Tracker**: Report bugs and request features
- **Discussions**: Ask questions and share ideas
- **Discord**: Join our community (coming soon)

## Documentation Updates

This documentation is actively maintained. Last major update: November 2025

Found an issue or want to improve the docs? Contributions are welcome!
