use crate::commands::{
    CodeActionProvider, CodeLensProvider, ColorProvider, CompletionProvider, DefinitionProvider,
    Diagnostic, DocumentFormattingProvider, DocumentHighlightProvider, DocumentSymbolsProvider,
    FoldingRangeProvider, HoverProvider, InlineValuesProvider, LanguageFeaturesRegistry,
    LinkedEditingRangeProvider, OnTypeFormattingProvider, RangeFormattingProvider,
    ReferencesProvider, RenameProvider, SelectionRangeProvider, SemanticTokensProvider,
    SignatureHelpProvider, WorkspaceSymbolsProvider,
};
use crate::session::SessionManager;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Position in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Hover result from extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResult {
    pub contents: serde_json::Value,
    pub range: Option<Range>,
}

/// Definition result from extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionResult {
    pub definitions: Vec<LocationResult>,
}

/// Location result (used for definitions, references)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationResult {
    pub uri: serde_json::Value,
    pub range: Range,
}

/// Code action result from extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionResult {
    pub title: String,
    pub kind: Option<String>,
    pub edit: Option<serde_json::Value>,
    pub command: Option<serde_json::Value>,
    #[serde(rename = "isPreferred")]
    pub is_preferred: Option<bool>,
}

/// Document symbol result from extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbolResult {
    pub name: String,
    pub detail: String,
    pub kind: u32,
    pub range: Range,
    #[serde(rename = "selectionRange")]
    pub selection_range: Range,
    pub children: Vec<DocumentSymbolResult>,
}

/// Text edit result from formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditResult {
    pub range: Range,
    #[serde(rename = "newText")]
    pub new_text: String,
}

/// Register a hover provider
#[tauri::command]
pub async fn register_hover_provider(
    provider: HoverProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_hover_provider(provider)
}

/// Register a definition provider
#[tauri::command]
pub async fn register_definition_provider(
    provider: DefinitionProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_definition_provider(provider)
}

/// Register a completion provider
#[tauri::command]
pub async fn register_completion_provider(
    provider: CompletionProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_completion_provider(provider)
}

/// Register a code action provider
#[tauri::command]
pub async fn register_code_action_provider(
    provider: CodeActionProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_code_action_provider(provider)
}

/// Get hover providers for a document
#[tauri::command]
pub async fn get_hover_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<HoverProvider>, String> {
    Ok(registry.get_hover_providers(&language, &uri))
}

/// Get definition providers for a document
#[tauri::command]
pub async fn get_definition_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<DefinitionProvider>, String> {
    Ok(registry.get_definition_providers(&language, &uri))
}

/// Get completion providers for a document
#[tauri::command]
pub async fn get_completion_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<CompletionProvider>, String> {
    Ok(registry.get_completion_providers(&language, &uri))
}

/// Get code action providers for a document
#[tauri::command]
pub async fn get_code_action_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<CodeActionProvider>, String> {
    Ok(registry.get_code_action_providers(&language, &uri))
}

/// Register a signature help provider
#[tauri::command]
pub async fn register_signature_help_provider(
    provider: SignatureHelpProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_signature_help_provider(provider)
}

/// Register a references provider
#[tauri::command]
pub async fn register_references_provider(
    provider: ReferencesProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_references_provider(provider)
}

/// Register a code lens provider
#[tauri::command]
pub async fn register_code_lens_provider(
    provider: CodeLensProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_code_lens_provider(provider)
}

/// Register a document highlight provider
#[tauri::command]
pub async fn register_document_highlight_provider(
    provider: DocumentHighlightProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_document_highlight_provider(provider)
}

/// Register a folding range provider
#[tauri::command]
pub async fn register_folding_range_provider(
    provider: FoldingRangeProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_folding_range_provider(provider)
}

/// Register a rename provider
#[tauri::command]
pub async fn register_rename_provider(
    provider: RenameProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_rename_provider(provider)
}

/// Get signature help providers for a document
#[tauri::command]
pub async fn get_signature_help_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<SignatureHelpProvider>, String> {
    Ok(registry.get_signature_help_providers(&language, &uri))
}

/// Get references providers for a document
#[tauri::command]
pub async fn get_references_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<ReferencesProvider>, String> {
    Ok(registry.get_references_providers(&language, &uri))
}

/// Get code lens providers for a document
#[tauri::command]
pub async fn get_code_lens_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<CodeLensProvider>, String> {
    Ok(registry.get_code_lens_providers(&language, &uri))
}

/// Get document highlight providers for a document
#[tauri::command]
pub async fn get_document_highlight_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<DocumentHighlightProvider>, String> {
    Ok(registry.get_document_highlight_providers(&language, &uri))
}

/// Get folding range providers for a document
#[tauri::command]
pub async fn get_folding_range_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<FoldingRangeProvider>, String> {
    Ok(registry.get_folding_range_providers(&language, &uri))
}

/// Get rename providers for a document
#[tauri::command]
pub async fn get_rename_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<RenameProvider>, String> {
    Ok(registry.get_rename_providers(&language, &uri))
}

/// Publish diagnostics for a document
#[tauri::command]
pub async fn publish_diagnostics(
    uri: String, diagnostics: Vec<Diagnostic>, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<(), String> {
    registry.publish_diagnostics(uri, diagnostics)
}

/// Get diagnostics for a document
#[tauri::command]
pub async fn get_diagnostics(
    uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<Diagnostic>, String> {
    Ok(registry.get_diagnostics(&uri))
}

/// Get all diagnostics
#[tauri::command]
pub async fn get_all_diagnostics(
    registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<HashMap<String, Vec<Diagnostic>>, String> {
    Ok(registry.get_all_diagnostics())
}

/// Clear diagnostics for a specific owner
#[tauri::command]
pub async fn clear_diagnostics(
    owner: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<(), String> {
    registry.clear_diagnostics(&owner)
}

/// Register a document symbols provider
#[tauri::command]
pub async fn register_document_symbols_provider(
    provider: DocumentSymbolsProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_document_symbols_provider(provider)
}

/// Register a workspace symbols provider
#[tauri::command]
pub async fn register_workspace_symbols_provider(
    provider: WorkspaceSymbolsProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_workspace_symbols_provider(provider)
}

/// Register a document formatting provider
#[tauri::command]
pub async fn register_document_formatting_provider(
    provider: DocumentFormattingProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_document_formatting_provider(provider)
}

/// Register a range formatting provider
#[tauri::command]
pub async fn register_range_formatting_provider(
    provider: RangeFormattingProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_range_formatting_provider(provider)
}

/// Register an on-type formatting provider
#[tauri::command]
pub async fn register_on_type_formatting_provider(
    provider: OnTypeFormattingProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_on_type_formatting_provider(provider)
}

/// Get document symbols providers for a document
#[tauri::command]
pub async fn get_document_symbols_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<DocumentSymbolsProvider>, String> {
    Ok(registry.get_document_symbols_providers(&language, &uri))
}

/// Get workspace symbols providers
#[tauri::command]
pub async fn get_workspace_symbols_providers(
    registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<WorkspaceSymbolsProvider>, String> {
    Ok(registry.get_workspace_symbols_providers())
}

/// Get document formatting providers for a document
#[tauri::command]
pub async fn get_document_formatting_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<DocumentFormattingProvider>, String> {
    Ok(registry.get_document_formatting_providers(&language, &uri))
}

/// Get range formatting providers for a document
#[tauri::command]
pub async fn get_range_formatting_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<RangeFormattingProvider>, String> {
    Ok(registry.get_range_formatting_providers(&language, &uri))
}

/// Get on-type formatting providers for a document
#[tauri::command]
pub async fn get_on_type_formatting_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<OnTypeFormattingProvider>, String> {
    Ok(registry.get_on_type_formatting_providers(&language, &uri))
}

/// Register a semantic tokens provider
#[tauri::command]
pub async fn register_semantic_tokens_provider(
    provider: SemanticTokensProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_semantic_tokens_provider(provider)
}

/// Register an inline values provider
#[tauri::command]
pub async fn register_inline_values_provider(
    provider: InlineValuesProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_inline_values_provider(provider)
}

/// Register a color provider
#[tauri::command]
pub async fn register_color_provider(
    provider: ColorProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_color_provider(provider)
}

/// Register a selection range provider
#[tauri::command]
pub async fn register_selection_range_provider(
    provider: SelectionRangeProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_selection_range_provider(provider)
}

/// Register a linked editing range provider
#[tauri::command]
pub async fn register_linked_editing_range_provider(
    provider: LinkedEditingRangeProvider, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<String, String> {
    registry.register_linked_editing_range_provider(provider)
}

/// Get semantic tokens providers for a document
#[tauri::command]
pub async fn get_semantic_tokens_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<SemanticTokensProvider>, String> {
    Ok(registry.get_semantic_tokens_providers(&language, &uri))
}

/// Get inline values providers for a document
#[tauri::command]
pub async fn get_inline_values_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<InlineValuesProvider>, String> {
    Ok(registry.get_inline_values_providers(&language, &uri))
}

/// Get color providers for a document
#[tauri::command]
pub async fn get_color_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<ColorProvider>, String> {
    Ok(registry.get_color_providers(&language, &uri))
}

/// Get selection range providers for a document
#[tauri::command]
pub async fn get_selection_range_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<SelectionRangeProvider>, String> {
    Ok(registry.get_selection_range_providers(&language, &uri))
}

/// Get linked editing range providers for a document
#[tauri::command]
pub async fn get_linked_editing_range_providers(
    language: String, uri: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<Vec<LinkedEditingRangeProvider>, String> {
    Ok(registry.get_linked_editing_range_providers(&language, &uri))
}

/// Clear all providers from a specific owner
#[tauri::command]
pub async fn clear_language_providers(
    owner: String, registry: State<'_, LanguageFeaturesRegistry>,
) -> Result<(), String> {
    registry.clear_owner_providers(&owner)
}

// ==================== Provider Invocation Commands ====================
// These commands invoke the extension host to get language feature results

const EXTENSION_HOST_ID: &str = "main";

fn document_payload(language_id: &str, uri: &str) -> serde_json::Value {
    json!({
        "uri": uri,
        "languageId": language_id,
    })
}

fn position_payload(line: u32, character: u32) -> serde_json::Value {
    json!({
        "line": line,
        "character": character,
    })
}

async fn request_language_feature(
    session: Arc<RwLock<SessionManager>>, request_name: &str, operation_name: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let session = session.read().await;
    session
        .ipc_manager()
        .request(EXTENSION_HOST_ID, request_name, payload)
        .await
        .map_err(|e| format!("Failed to invoke {}: {}", operation_name, e))
}

fn parse_result<T: DeserializeOwned>(
    response: serde_json::Value, result_name: &str,
) -> Result<T, String> {
    serde_json::from_value(response)
        .map_err(|e| format!("Failed to parse {} result: {}", result_name, e))
}

fn parse_result_field<T: DeserializeOwned>(
    response: serde_json::Value, field: &str, result_name: &str,
) -> Result<Vec<T>, String> {
    let values = response
        .get(field)
        .cloned()
        .ok_or_else(|| format!("Missing {} field in {} result", field, result_name))?;

    serde_json::from_value(values)
        .map_err(|e| format!("Failed to parse {} result: {}", result_name, e))
}

/// Invoke hover provider via extension host
#[tauri::command]
pub async fn invoke_hover_provider(
    language_id: String, uri: String, line: u32, character: u32,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Option<HoverResult>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "position": position_payload(line, character),
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideHover",
        "hover provider",
        payload,
    )
    .await?;

    if response.is_null() {
        Ok(None)
    } else {
        parse_result(response, "hover").map(Some)
    }
}

/// Invoke definition provider via extension host
#[tauri::command]
pub async fn invoke_definition_provider(
    language_id: String, uri: String, line: u32, character: u32,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<DefinitionResult, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "position": position_payload(line, character),
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideDefinition",
        "definition provider",
        payload,
    )
    .await?;

    parse_result(response, "definition")
}

/// Invoke references provider via extension host
#[tauri::command]
pub async fn invoke_references_provider(
    language_id: String, uri: String, line: u32, character: u32, include_declaration: bool,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<LocationResult>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "position": position_payload(line, character),
        "includeDeclaration": include_declaration
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideReferences",
        "references provider",
        payload,
    )
    .await?;

    parse_result_field(response, "references", "references")
}

/// Invoke code actions provider via extension host
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn invoke_code_actions_provider(
    language_id: String, uri: String, start_line: u32, start_character: u32, end_line: u32,
    end_character: u32, diagnostics: Vec<serde_json::Value>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<CodeActionResult>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "range": {
            "start": position_payload(start_line, start_character),
            "end": position_payload(end_line, end_character),
        },
        "diagnostics": diagnostics
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideCodeActions",
        "code actions provider",
        payload,
    )
    .await?;

    parse_result_field(response, "actions", "code actions")
}

/// Invoke document symbols provider via extension host
#[tauri::command]
pub async fn invoke_document_symbols_provider(
    language_id: String, uri: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<DocumentSymbolResult>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideDocumentSymbols",
        "document symbols provider",
        payload,
    )
    .await?;

    parse_result_field(response, "symbols", "document symbols")
}

/// Invoke document formatting provider via extension host
#[tauri::command]
pub async fn invoke_document_formatting_provider(
    language_id: String, uri: String, tab_size: u32, insert_spaces: bool,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<TextEditResult>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "options": {
            "tabSize": tab_size,
            "insertSpaces": insert_spaces
        }
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideDocumentFormatting",
        "formatting provider",
        payload,
    )
    .await?;

    parse_result_field(response, "edits", "formatting")
}

/// Invoke completion provider via extension host
#[tauri::command]
pub async fn invoke_completion_provider(
    language_id: String, uri: String, line: u32, character: u32, trigger_character: Option<String>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<serde_json::Value>, String> {
    let payload = json!({
        "languageId": language_id,
        "document": document_payload(&language_id, &uri),
        "position": position_payload(line, character),
        "context": {
            "triggerKind": if trigger_character.is_some() { 2 } else { 1 },
            "triggerCharacter": trigger_character
        }
    });

    let response = request_language_feature(
        session.inner().clone(),
        "provideCompletion",
        "completion provider",
        payload,
    )
    .await?;

    if response.is_array() {
        parse_result(response, "completion")
    } else {
        parse_result_field(response, "items", "completion")
    }
}

/// Trigger language activation event when a file is opened
#[tauri::command]
pub async fn trigger_language_activation(
    language_id: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let payload = json!({
        "languageId": language_id
    });

    request_language_feature(
        session.inner().clone(),
        "trigger-on-language",
        "language activation",
        payload,
    )
    .await?;

    Ok(())
}
