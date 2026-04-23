//! Language feature provider registration from extension host IPC.
//!
//! Handles registration of completion, hover, definition, and other language
//! providers from extension contributions.
//!
//! **This module requires the `tauri` feature flag.**

use serde_json::{json, Value};
use uuid::Uuid;

/// Context for handling a provider registration request.
#[allow(dead_code)]
pub(super) struct ProviderHandlerContext<'a> {
    pub payload: &'a Value,
    pub provider_type: &'a str,
}

/// Register a language feature provider based on the IPC request type.
///
/// Returns a JSON response with `success: true` and `providerId` fields.
#[allow(dead_code)]
pub(super) fn handle_register_provider(
    _context: &ProviderHandlerContext,
    _registry: &(), // Placeholder: will use LanguageFeaturesRegistry when integrated
) -> Result<Value, String> {
    let provider_id = Uuid::new_v4().to_string();

    // Note: In the full integration, this function will:
    // 1. Parse the document selector from the payload
    // 2. Create the appropriate provider type (Completion, Hover, etc.)
    // 3. Register it with the LanguageFeaturesRegistry
    // 4. Return the provider ID

    Ok(json!({
        "success": true,
        "providerId": provider_id,
    }))
}
