//! Extension host sandbox example.
//!
//! Demonstrates path validation, permissions, rate limiting, and secret storage.

use dscode_extension_host::{
    ExtensionPermissions, PathValidator, Permission, RateLimiter, SecretStorage,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path validation
    let mut validator = PathValidator::new();
    validator.add_workspace_folder(std::env::current_dir()?);
    let safe = validator.validate_path("file:///workspace/src/main.rs")?;
    println!("Validated path: {}", safe.display());

    // Permissions
    let perms = ExtensionPermissions::from_manifest(
        "my-extension".to_string(),
        Some(vec!["fileSystem.read".to_string()]),
    )?;
    perms.check_permission(&Permission::FileSystemRead)?;
    println!("Permission check passed.");

    // Rate limiting
    let limiter = RateLimiter::new();
    limiter.check_rate_limit("my-extension")?;
    println!("Rate limit check passed.");

    // Secret storage
    let secrets = SecretStorage::new();
    secrets.set("my-extension", "api_key", "secret-value")?;
    let value = secrets.get("my-extension", "api_key")?;
    println!("Retrieved secret: {}", value.unwrap_or_default());

    Ok(())
}
