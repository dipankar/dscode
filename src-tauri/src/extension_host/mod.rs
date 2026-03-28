pub mod ipc;
pub mod manager;
pub mod path_validator;
pub mod permissions;
pub mod rate_limiter;
pub mod sandbox;
pub mod secrets;

pub use ipc::{ExtensionIpc, IncomingIpc, IncomingRequestHandler, IpcManager};
pub use manager::{ExtensionHostManager, ExtensionHostState};
pub use path_validator::PathValidator;
pub use permissions::{ExtensionPermissions, Permission};
pub use rate_limiter::RateLimiter;
pub use sandbox::{apply_sandbox, SandboxConfig};
pub use secrets::SecretStorage;
