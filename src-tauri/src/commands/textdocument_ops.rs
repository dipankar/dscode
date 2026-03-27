use super::textdocument_registry::*;
use crate::extension_host::IpcManager;
use crate::session::SessionManager;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Register text document
#[tauri::command]
pub async fn register_text_document(
    document: TextDocument, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.register_text_document(document)
}

/// Unregister text document
#[tauri::command]
pub async fn unregister_text_document(
    uri: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.unregister_text_document(&uri)
}

/// Update text document and forward changes to extension host
#[tauri::command]
pub async fn update_text_document(
    uri: String, version: u64, content_changes: Vec<TextDocumentContentChange>,
    registry: State<'_, TextDocumentRegistry>, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    registry.update_text_document(&uri, version, content_changes.clone())?;

    let sm = session.read().await;
    let _ = sm.forward_document_change(&uri, version, &content_changes).await;

    Ok(())
}

/// Mark document as saved
#[tauri::command]
pub async fn mark_document_saved(
    uri: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.mark_document_saved(&uri)
}

/// Get text document
#[tauri::command]
pub async fn get_text_document(
    uri: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<TextDocument, String> {
    registry.get_text_document(&uri)
}

/// Get all text documents
#[tauri::command]
pub async fn get_all_text_documents(
    registry: State<'_, TextDocumentRegistry>,
) -> Result<Vec<TextDocument>, String> {
    Ok(registry.get_all_text_documents())
}

/// Register text editor
#[tauri::command]
pub async fn register_text_editor(
    editor: TextEditor, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.register_text_editor(editor)
}

/// Unregister text editor
#[tauri::command]
pub async fn unregister_text_editor(
    editor_id: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.unregister_text_editor(&editor_id)
}

/// Update editor selections
#[tauri::command]
pub async fn update_text_editor_selections(
    editor_id: String, selections: Vec<Selection>, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.update_editor_selections(&editor_id, selections)
}

/// Update editor visible ranges
#[tauri::command]
pub async fn update_text_editor_visible_ranges(
    editor_id: String, visible_ranges: Vec<Range>, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.update_editor_visible_ranges(&editor_id, visible_ranges)
}

/// Get text editor
#[tauri::command]
pub async fn get_text_editor(
    editor_id: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<TextEditor, String> {
    registry.get_text_editor(&editor_id)
}

/// Get all text editors
#[tauri::command]
pub async fn get_all_text_editors(
    registry: State<'_, TextDocumentRegistry>,
) -> Result<Vec<TextEditor>, String> {
    Ok(registry.get_all_text_editors())
}

/// Create decoration type
#[tauri::command]
pub async fn create_text_editor_decoration_type(
    decoration_type: DecorationType, registry: State<'_, TextDocumentRegistry>,
) -> Result<String, String> {
    registry.create_decoration_type(decoration_type)
}

/// Dispose decoration type
#[tauri::command]
pub async fn dispose_text_editor_decoration_type(
    decoration_type_id: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.dispose_decoration_type(&decoration_type_id)
}

/// Set editor decorations
#[tauri::command]
pub async fn set_text_editor_decorations(
    editor_id: String, decoration_type_id: String, ranges: Vec<Range>, owner: String,
    registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.set_editor_decorations(&editor_id, &decoration_type_id, ranges, &owner)
}

/// Get editor decorations
#[tauri::command]
pub async fn get_text_editor_decorations(
    editor_id: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<Vec<TextEditorDecoration>, String> {
    Ok(registry.get_editor_decorations(&editor_id))
}

/// Apply text edits to document
#[tauri::command]
pub async fn apply_text_edits(uri: String, edits: Vec<TextEdit>) -> Result<(), String> {
    // This would typically apply edits through Monaco
    // For now, we just validate and return success
    // The actual application happens in the frontend

    if edits.is_empty() {
        return Ok(());
    }

    println!("[TextDocument] Applying {} text edit(s) to: {}", edits.len(), uri);

    Ok(())
}

/// Clear text document data for owner
#[tauri::command]
pub async fn clear_textdocument_data(
    owner: String, registry: State<'_, TextDocumentRegistry>,
) -> Result<(), String> {
    registry.clear_textdocument_data(&owner);
    Ok(())
}
