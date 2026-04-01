use dscode_core::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

/// Text document representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocument {
    pub uri: String,
    pub language_id: String,
    pub version: u64,
    pub line_count: usize,
    pub is_untitled: bool,
    pub is_dirty: bool,
}

/// Text editor representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditor {
    pub id: String,
    pub document: TextDocument,
    pub selections: Vec<Selection>,
    pub visible_ranges: Vec<Range>,
    pub options: TextEditorOptions,
}

/// Text editor options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorOptions {
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub cursor_style: CursorStyle,
    pub line_numbers: LineNumbersStyle,
}

/// Cursor style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Line,
    Block,
    Underline,
}

/// Line numbers style
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineNumbersStyle {
    Off,
    On,
    Relative,
}

/// Position in a text document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in a text document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Selection in a text editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub anchor: Position,
    pub active: Position,
}

/// Text edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Text document change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentChangeEvent {
    pub document: TextDocument,
    pub content_changes: Vec<TextDocumentContentChange>,
}

/// Text document content change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChange {
    pub range: Option<Range>,
    pub range_offset: Option<u32>,
    pub range_length: Option<u32>,
    pub text: String,
}

/// Text editor decoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorDecoration {
    pub id: String,
    pub owner: String,
    pub decoration_type: String,
    pub ranges: Vec<Range>,
}

/// Decoration render options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationRenderOptions {
    pub background_color: Option<String>,
    pub border: Option<String>,
    pub border_color: Option<String>,
    pub border_radius: Option<String>,
    pub border_width: Option<String>,
    pub color: Option<String>,
    pub cursor: Option<String>,
    pub text_decoration: Option<String>,
    pub outline: Option<String>,
    pub outline_color: Option<String>,
    pub outline_width: Option<String>,
}

/// Decoration type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationType {
    pub id: String,
    pub owner: String,
    pub options: DecorationRenderOptions,
}

/// Text document registry
pub struct TextDocumentRegistry {
    documents: Arc<RwLock<HashMap<String, TextDocument>>>,
    editors: Arc<RwLock<HashMap<String, TextEditor>>>,
    decorations: Arc<RwLock<HashMap<String, Vec<TextEditorDecoration>>>>,
    decoration_types: Arc<RwLock<HashMap<String, DecorationType>>>,
    app_handle: AppHandle,
}

impl TextDocumentRegistry {
    /// Create a new text document registry
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            editors: Arc::new(RwLock::new(HashMap::new())),
            decorations: Arc::new(RwLock::new(HashMap::new())),
            decoration_types: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    /// Register a text document
    pub fn register_text_document(&self, document: TextDocument) -> Result<(), CoreError> {
        let mut documents = self.documents.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let uri = document.uri.clone();
        documents.insert(uri.clone(), document.clone());

        info!("Registered document: {}", uri);

        // Emit event
        if let Err(e) = self.app_handle.emit("text-document-opened", &document) {
            error!("Failed to emit document opened event: {}", e);
        }

        Ok(())
    }

    /// Unregister a text document
    pub fn unregister_text_document(&self, uri: &str) -> Result<(), CoreError> {
        let mut documents = self.documents.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let document = documents
            .remove(uri)
            .ok_or_else(|| CoreError::Config(format!("Text document not found: {}", uri)))?;

        info!("Unregistered document: {}", uri);

        // Emit event
        if let Err(e) = self.app_handle.emit("text-document-closed", &document) {
            error!("Failed to emit document closed event: {}", e);
        }

        Ok(())
    }

    /// Update text document
    pub fn update_text_document(
        &self, uri: &str, version: u64, content_changes: Vec<TextDocumentContentChange>,
    ) -> Result<(), CoreError> {
        let mut documents = self.documents.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let document = documents
            .get_mut(uri)
            .ok_or_else(|| CoreError::Config(format!("Text document not found: {}", uri)))?;

        // Update version
        document.version = version;

        let event = TextDocumentChangeEvent { document: document.clone(), content_changes };

        // Emit event
        if let Err(e) = self.app_handle.emit("text-document-changed", &event) {
            error!("Failed to emit document changed event: {}", e);
        }

        Ok(())
    }

    /// Mark document as saved
    pub fn mark_document_saved(&self, uri: &str) -> Result<(), CoreError> {
        let mut documents = self.documents.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let document = documents
            .get_mut(uri)
            .ok_or_else(|| CoreError::Config(format!("Text document not found: {}", uri)))?;

        document.is_dirty = false;

        // Emit event
        if let Err(e) = self.app_handle.emit("text-document-saved", &document.clone()) {
            error!("Failed to emit document saved event: {}", e);
        }

        Ok(())
    }

    /// Get text document
    pub fn get_text_document(&self, uri: &str) -> Result<TextDocument, CoreError> {
        let documents = self.documents.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        documents
            .get(uri)
            .cloned()
            .ok_or_else(|| CoreError::Config(format!("Text document not found: {}", uri)))
    }

    /// Get all text documents
    pub fn get_all_text_documents(&self) -> Vec<TextDocument> {
        let documents = self.documents.read().unwrap_or_else(|e| {
            warn!("documents read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        documents.values().cloned().collect()
    }

    /// Register text editor
    pub fn register_text_editor(&self, editor: TextEditor) -> Result<(), CoreError> {
        let mut editors = self.editors.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let id = editor.id.clone();
        editors.insert(id.clone(), editor.clone());

        info!("Registered editor: {}", id);

        // Emit event
        if let Err(e) = self.app_handle.emit("text-editor-opened", &editor) {
            error!("Failed to emit editor opened event: {}", e);
        }

        Ok(())
    }

    /// Unregister text editor
    pub fn unregister_text_editor(&self, editor_id: &str) -> Result<(), CoreError> {
        let mut editors = self.editors.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        editors
            .remove(editor_id)
            .ok_or_else(|| CoreError::Config(format!("Text editor not found: {}", editor_id)))?;

        info!("Unregistered editor: {}", editor_id);

        Ok(())
    }

    /// Update editor selections
    pub fn update_editor_selections(
        &self, editor_id: &str, selections: Vec<Selection>,
    ) -> Result<(), CoreError> {
        let mut editors = self.editors.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let editor = editors
            .get_mut(editor_id)
            .ok_or_else(|| CoreError::Config(format!("Text editor not found: {}", editor_id)))?;

        editor.selections = selections;

        // Emit event
        if let Err(e) = self.app_handle.emit("text-editor-selection-changed", &editor.clone()) {
            error!("Failed to emit selection changed event: {}", e);
        }

        Ok(())
    }

    /// Update editor visible ranges
    pub fn update_editor_visible_ranges(
        &self, editor_id: &str, visible_ranges: Vec<Range>,
    ) -> Result<(), CoreError> {
        let mut editors = self.editors.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let editor = editors
            .get_mut(editor_id)
            .ok_or_else(|| CoreError::Config(format!("Text editor not found: {}", editor_id)))?;

        editor.visible_ranges = visible_ranges;

        // Emit event
        if let Err(e) = self.app_handle.emit("text-editor-visible-ranges-changed", &editor.clone())
        {
            error!("Failed to emit visible ranges changed event: {}", e);
        }

        Ok(())
    }

    /// Get text editor
    pub fn get_text_editor(&self, editor_id: &str) -> Result<TextEditor, CoreError> {
        let editors = self.editors.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        editors
            .get(editor_id)
            .cloned()
            .ok_or_else(|| CoreError::Config(format!("Text editor not found: {}", editor_id)))
    }

    /// Get all text editors
    pub fn get_all_text_editors(&self) -> Vec<TextEditor> {
        let editors = self.editors.read().unwrap_or_else(|e| {
            warn!("editors read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        editors.values().cloned().collect()
    }

    /// Create decoration type
    pub fn create_decoration_type(
        &self, decoration_type: DecorationType,
    ) -> Result<String, CoreError> {
        let mut decoration_types = self.decoration_types.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let id = decoration_type.id.clone();
        decoration_types.insert(id.clone(), decoration_type);

        info!("Created decoration type: {}", id);

        Ok(id)
    }

    /// Dispose decoration type
    pub fn dispose_decoration_type(&self, decoration_type_id: &str) -> Result<(), CoreError> {
        let mut decoration_types = self.decoration_types.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        decoration_types.remove(decoration_type_id).ok_or_else(|| {
            CoreError::Config(format!("Decoration type not found: {}", decoration_type_id))
        })?;

        // Also remove all decorations of this type
        let mut decorations = self.decorations.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
        for decoration_list in decorations.values_mut() {
            decoration_list.retain(|d| d.decoration_type != decoration_type_id);
        }

        info!("Disposed decoration type: {}", decoration_type_id);

        Ok(())
    }

    /// Set editor decorations
    pub fn set_editor_decorations(
        &self, editor_id: &str, decoration_type_id: &str, ranges: Vec<Range>, owner: &str,
    ) -> Result<(), CoreError> {
        let mut decorations = self.decorations.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let editor_decorations = decorations.entry(editor_id.to_string()).or_insert_with(Vec::new);

        // Remove existing decorations of this type
        editor_decorations.retain(|d| d.decoration_type != decoration_type_id);

        // Add new decorations
        if !ranges.is_empty() {
            editor_decorations.push(TextEditorDecoration {
                id: format!("{}:{}", decoration_type_id, editor_id),
                owner: owner.to_string(),
                decoration_type: decoration_type_id.to_string(),
                ranges,
            });
        }

        // Emit event
        if let Err(e) =
            self.app_handle.emit(&format!("editor-decorations-changed:{}", editor_id), &editor_id)
        {
            error!("Failed to emit decorations changed event: {}", e);
        }

        Ok(())
    }

    /// Get editor decorations
    pub fn get_editor_decorations(&self, editor_id: &str) -> Vec<TextEditorDecoration> {
        let decorations = self.decorations.read().unwrap_or_else(|e| {
            warn!("decorations read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        decorations.get(editor_id).cloned().unwrap_or_default()
    }

    /// Clear all text document data for an owner
    pub fn clear_textdocument_data(&self, owner: &str) {
        // Clear decoration types
        {
            let mut decoration_types =
                self.decoration_types.write().unwrap_or_else(|e| {
                    warn!("decoration_types write lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
            let before = decoration_types.len();
            decoration_types.retain(|_, dt| dt.owner != owner);
            let removed = before - decoration_types.len();
            if removed > 0 {
                info!(
                    "Cleared {} decoration type(s) for owner: {}",
                    removed, owner
                );
            }
        }

        // Clear decorations
        {
            let mut decorations =
                self.decorations.write().unwrap_or_else(|e| {
                    warn!("decorations write lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
            let mut total_removed = 0;
            for decoration_list in decorations.values_mut() {
                let before = decoration_list.len();
                decoration_list.retain(|d| d.owner != owner);
                total_removed += before - decoration_list.len();
            }
            if total_removed > 0 {
                info!(
                    "Cleared {} decoration(s) for owner: {}",
                    total_removed, owner
                );
            }
        }
    }
}
