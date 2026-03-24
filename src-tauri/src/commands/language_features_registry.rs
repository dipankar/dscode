use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSelector {
    pub filters: Vec<DocumentFilter>,
}

impl DocumentSelector {
    pub fn matches(&self, language: &str, uri: &str) -> bool {
        self.filters.iter().any(|filter| {
            // Check language match
            if let Some(lang) = &filter.language {
                if lang != language && lang != "*" {
                    return false;
                }
            }

            // Check scheme match
            if let Some(scheme) = &filter.scheme {
                if !uri.starts_with(&format!("{}:", scheme)) {
                    return false;
                }
            }

            // Check pattern match (simple glob for now)
            if let Some(pattern) = &filter.pattern {
                if !simple_glob_match(uri, pattern) {
                    return false;
                }
            }

            true
        })
    }
}

fn simple_glob_match(text: &str, pattern: &str) -> bool {
    // Simple glob matching - just handle * for now
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return text.starts_with(prefix) && text.ends_with(suffix);
        }
    }

    text == pattern
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub uri: String,
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub code_action_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub trigger_characters: Vec<String>,
    pub retrigger_characters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencesProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLensProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHighlightProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldingRangeProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub prepare_provider: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbolsProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSymbolsProvider {
    pub id: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFormattingProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeFormattingProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnTypeFormattingProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
    pub legend: SemanticTokensLegend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTokensLegend {
    pub token_types: Vec<String>,
    pub token_modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineValuesProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRangeProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedEditingRangeProvider {
    pub id: String,
    pub owner: String,
    pub selector: DocumentSelector,
}

trait OwnedProvider {
    fn id(&self) -> &str;
    fn owner(&self) -> &str;
}

trait SelectableProvider: OwnedProvider {
    fn matches_document(&self, language: &str, uri: &str) -> bool;
}

macro_rules! impl_owned_provider {
    ($($provider:ty),+ $(,)?) => {
        $(
            impl OwnedProvider for $provider {
                fn id(&self) -> &str {
                    &self.id
                }

                fn owner(&self) -> &str {
                    &self.owner
                }
            }
        )+
    };
}

macro_rules! impl_selectable_provider {
    ($($provider:ty),+ $(,)?) => {
        $(
            impl SelectableProvider for $provider {
                fn matches_document(&self, language: &str, uri: &str) -> bool {
                    self.selector.matches(language, uri)
                }
            }
        )+
    };
}

impl_owned_provider!(
    HoverProvider,
    DefinitionProvider,
    CompletionProvider,
    CodeActionProvider,
    SignatureHelpProvider,
    ReferencesProvider,
    CodeLensProvider,
    DocumentHighlightProvider,
    FoldingRangeProvider,
    RenameProvider,
    DocumentSymbolsProvider,
    WorkspaceSymbolsProvider,
    DocumentFormattingProvider,
    RangeFormattingProvider,
    OnTypeFormattingProvider,
    SemanticTokensProvider,
    InlineValuesProvider,
    ColorProvider,
    SelectionRangeProvider,
    LinkedEditingRangeProvider,
);

impl_selectable_provider!(
    HoverProvider,
    DefinitionProvider,
    CompletionProvider,
    CodeActionProvider,
    SignatureHelpProvider,
    ReferencesProvider,
    CodeLensProvider,
    DocumentHighlightProvider,
    FoldingRangeProvider,
    RenameProvider,
    DocumentSymbolsProvider,
    DocumentFormattingProvider,
    RangeFormattingProvider,
    OnTypeFormattingProvider,
    SemanticTokensProvider,
    InlineValuesProvider,
    ColorProvider,
    SelectionRangeProvider,
    LinkedEditingRangeProvider,
);

macro_rules! register_provider_method {
    ($fn_name:ident, $field:ident, $provider_type:ty, $label:literal) => {
        pub fn $fn_name(&self, provider: $provider_type) -> Result<String, String> {
            self.register_provider(&self.$field, provider, $label)
        }
    };
}

macro_rules! get_matching_provider_method {
    ($fn_name:ident, $field:ident, $provider_type:ty) => {
        pub fn $fn_name(&self, language: &str, uri: &str) -> Vec<$provider_type> {
            self.get_matching_providers(&self.$field, language, uri)
        }
    };
}

/// Registry for language features contributed by extensions
pub struct LanguageFeaturesRegistry {
    hover_providers: Arc<RwLock<Vec<HoverProvider>>>,
    definition_providers: Arc<RwLock<Vec<DefinitionProvider>>>,
    completion_providers: Arc<RwLock<Vec<CompletionProvider>>>,
    code_action_providers: Arc<RwLock<Vec<CodeActionProvider>>>,
    signature_help_providers: Arc<RwLock<Vec<SignatureHelpProvider>>>,
    references_providers: Arc<RwLock<Vec<ReferencesProvider>>>,
    code_lens_providers: Arc<RwLock<Vec<CodeLensProvider>>>,
    document_highlight_providers: Arc<RwLock<Vec<DocumentHighlightProvider>>>,
    folding_range_providers: Arc<RwLock<Vec<FoldingRangeProvider>>>,
    rename_providers: Arc<RwLock<Vec<RenameProvider>>>,
    document_symbols_providers: Arc<RwLock<Vec<DocumentSymbolsProvider>>>,
    workspace_symbols_providers: Arc<RwLock<Vec<WorkspaceSymbolsProvider>>>,
    document_formatting_providers: Arc<RwLock<Vec<DocumentFormattingProvider>>>,
    range_formatting_providers: Arc<RwLock<Vec<RangeFormattingProvider>>>,
    on_type_formatting_providers: Arc<RwLock<Vec<OnTypeFormattingProvider>>>,
    semantic_tokens_providers: Arc<RwLock<Vec<SemanticTokensProvider>>>,
    inline_values_providers: Arc<RwLock<Vec<InlineValuesProvider>>>,
    color_providers: Arc<RwLock<Vec<ColorProvider>>>,
    selection_range_providers: Arc<RwLock<Vec<SelectionRangeProvider>>>,
    linked_editing_range_providers: Arc<RwLock<Vec<LinkedEditingRangeProvider>>>,
    diagnostics: Arc<RwLock<HashMap<String, Vec<Diagnostic>>>>, // uri -> diagnostics
    app_handle: AppHandle,
}

impl LanguageFeaturesRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            hover_providers: Arc::new(RwLock::new(Vec::new())),
            definition_providers: Arc::new(RwLock::new(Vec::new())),
            completion_providers: Arc::new(RwLock::new(Vec::new())),
            code_action_providers: Arc::new(RwLock::new(Vec::new())),
            signature_help_providers: Arc::new(RwLock::new(Vec::new())),
            references_providers: Arc::new(RwLock::new(Vec::new())),
            code_lens_providers: Arc::new(RwLock::new(Vec::new())),
            document_highlight_providers: Arc::new(RwLock::new(Vec::new())),
            folding_range_providers: Arc::new(RwLock::new(Vec::new())),
            rename_providers: Arc::new(RwLock::new(Vec::new())),
            document_symbols_providers: Arc::new(RwLock::new(Vec::new())),
            workspace_symbols_providers: Arc::new(RwLock::new(Vec::new())),
            document_formatting_providers: Arc::new(RwLock::new(Vec::new())),
            range_formatting_providers: Arc::new(RwLock::new(Vec::new())),
            on_type_formatting_providers: Arc::new(RwLock::new(Vec::new())),
            semantic_tokens_providers: Arc::new(RwLock::new(Vec::new())),
            inline_values_providers: Arc::new(RwLock::new(Vec::new())),
            color_providers: Arc::new(RwLock::new(Vec::new())),
            selection_range_providers: Arc::new(RwLock::new(Vec::new())),
            linked_editing_range_providers: Arc::new(RwLock::new(Vec::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    fn register_provider<T: OwnedProvider>(
        &self,
        providers: &Arc<RwLock<Vec<T>>>,
        provider: T,
        label: &str,
    ) -> Result<String, String> {
        let id = provider.id().to_string();
        let mut providers = providers.write().map_err(|e| e.to_string())?;
        providers.push(provider);

        println!("[LanguageFeatures] Registered {} provider: {}", label, id);

        Ok(id)
    }

    fn get_matching_providers<T: SelectableProvider + Clone>(
        &self,
        providers: &Arc<RwLock<Vec<T>>>,
        language: &str,
        uri: &str,
    ) -> Vec<T> {
        let providers = providers.read().unwrap();
        providers
            .iter()
            .filter(|provider| provider.matches_document(language, uri))
            .cloned()
            .collect()
    }

    fn get_all_providers<T: Clone>(&self, providers: &Arc<RwLock<Vec<T>>>) -> Vec<T> {
        let providers = providers.read().unwrap();
        providers.iter().cloned().collect()
    }

    fn clear_owned_provider_list<T: OwnedProvider>(
        &self,
        providers: &Arc<RwLock<Vec<T>>>,
        owner: &str,
    ) -> Result<(), String> {
        let mut providers = providers.write().map_err(|e| e.to_string())?;
        providers.retain(|provider| provider.owner() != owner);
        Ok(())
    }

    register_provider_method!(
        register_hover_provider,
        hover_providers,
        HoverProvider,
        "hover"
    );
    register_provider_method!(
        register_definition_provider,
        definition_providers,
        DefinitionProvider,
        "definition"
    );
    register_provider_method!(
        register_completion_provider,
        completion_providers,
        CompletionProvider,
        "completion"
    );
    register_provider_method!(
        register_code_action_provider,
        code_action_providers,
        CodeActionProvider,
        "code action"
    );
    register_provider_method!(
        register_signature_help_provider,
        signature_help_providers,
        SignatureHelpProvider,
        "signature help"
    );
    register_provider_method!(
        register_references_provider,
        references_providers,
        ReferencesProvider,
        "references"
    );
    register_provider_method!(
        register_code_lens_provider,
        code_lens_providers,
        CodeLensProvider,
        "code lens"
    );
    register_provider_method!(
        register_document_highlight_provider,
        document_highlight_providers,
        DocumentHighlightProvider,
        "document highlight"
    );
    register_provider_method!(
        register_folding_range_provider,
        folding_range_providers,
        FoldingRangeProvider,
        "folding range"
    );
    register_provider_method!(
        register_rename_provider,
        rename_providers,
        RenameProvider,
        "rename"
    );
    register_provider_method!(
        register_document_symbols_provider,
        document_symbols_providers,
        DocumentSymbolsProvider,
        "document symbols"
    );
    register_provider_method!(
        register_workspace_symbols_provider,
        workspace_symbols_providers,
        WorkspaceSymbolsProvider,
        "workspace symbols"
    );
    register_provider_method!(
        register_document_formatting_provider,
        document_formatting_providers,
        DocumentFormattingProvider,
        "document formatting"
    );
    register_provider_method!(
        register_range_formatting_provider,
        range_formatting_providers,
        RangeFormattingProvider,
        "range formatting"
    );
    register_provider_method!(
        register_on_type_formatting_provider,
        on_type_formatting_providers,
        OnTypeFormattingProvider,
        "on-type formatting"
    );
    register_provider_method!(
        register_semantic_tokens_provider,
        semantic_tokens_providers,
        SemanticTokensProvider,
        "semantic tokens"
    );
    register_provider_method!(
        register_inline_values_provider,
        inline_values_providers,
        InlineValuesProvider,
        "inline values"
    );
    register_provider_method!(
        register_color_provider,
        color_providers,
        ColorProvider,
        "color"
    );
    register_provider_method!(
        register_selection_range_provider,
        selection_range_providers,
        SelectionRangeProvider,
        "selection range"
    );
    register_provider_method!(
        register_linked_editing_range_provider,
        linked_editing_range_providers,
        LinkedEditingRangeProvider,
        "linked editing range"
    );

    get_matching_provider_method!(get_hover_providers, hover_providers, HoverProvider);
    get_matching_provider_method!(
        get_definition_providers,
        definition_providers,
        DefinitionProvider
    );
    get_matching_provider_method!(
        get_completion_providers,
        completion_providers,
        CompletionProvider
    );
    get_matching_provider_method!(
        get_code_action_providers,
        code_action_providers,
        CodeActionProvider
    );
    get_matching_provider_method!(
        get_signature_help_providers,
        signature_help_providers,
        SignatureHelpProvider
    );
    get_matching_provider_method!(
        get_references_providers,
        references_providers,
        ReferencesProvider
    );
    get_matching_provider_method!(
        get_code_lens_providers,
        code_lens_providers,
        CodeLensProvider
    );
    get_matching_provider_method!(
        get_document_highlight_providers,
        document_highlight_providers,
        DocumentHighlightProvider
    );
    get_matching_provider_method!(
        get_folding_range_providers,
        folding_range_providers,
        FoldingRangeProvider
    );
    get_matching_provider_method!(get_rename_providers, rename_providers, RenameProvider);
    get_matching_provider_method!(
        get_document_symbols_providers,
        document_symbols_providers,
        DocumentSymbolsProvider
    );
    get_matching_provider_method!(
        get_document_formatting_providers,
        document_formatting_providers,
        DocumentFormattingProvider
    );
    get_matching_provider_method!(
        get_range_formatting_providers,
        range_formatting_providers,
        RangeFormattingProvider
    );
    get_matching_provider_method!(
        get_on_type_formatting_providers,
        on_type_formatting_providers,
        OnTypeFormattingProvider
    );
    get_matching_provider_method!(
        get_semantic_tokens_providers,
        semantic_tokens_providers,
        SemanticTokensProvider
    );
    get_matching_provider_method!(
        get_inline_values_providers,
        inline_values_providers,
        InlineValuesProvider
    );
    get_matching_provider_method!(get_color_providers, color_providers, ColorProvider);
    get_matching_provider_method!(
        get_selection_range_providers,
        selection_range_providers,
        SelectionRangeProvider
    );
    get_matching_provider_method!(
        get_linked_editing_range_providers,
        linked_editing_range_providers,
        LinkedEditingRangeProvider
    );

    pub fn get_workspace_symbols_providers(&self) -> Vec<WorkspaceSymbolsProvider> {
        self.get_all_providers(&self.workspace_symbols_providers)
    }

    /// Publish diagnostics for a document
    pub fn publish_diagnostics(
        &self,
        uri: String,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<(), String> {
        {
            let mut all_diagnostics = self.diagnostics.write().map_err(|e| e.to_string())?;

            if diagnostics.is_empty() {
                all_diagnostics.remove(&uri);
            } else {
                all_diagnostics.insert(uri.clone(), diagnostics.clone());
            }
        }

        // Emit event to frontend
        if let Err(e) = self
            .app_handle
            .emit("diagnostics-changed", &(uri, diagnostics))
        {
            eprintln!("[LanguageFeatures] Failed to emit diagnostics event: {}", e);
        }

        Ok(())
    }

    /// Get diagnostics for a document
    pub fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        let all_diagnostics = self.diagnostics.read().unwrap();
        all_diagnostics.get(uri).cloned().unwrap_or_default()
    }

    /// Get all diagnostics
    pub fn get_all_diagnostics(&self) -> HashMap<String, Vec<Diagnostic>> {
        let all_diagnostics = self.diagnostics.read().unwrap();
        all_diagnostics.clone()
    }

    /// Clear diagnostics for a specific owner
    pub fn clear_diagnostics(&self, owner: &str) -> Result<(), String> {
        let mut all_diagnostics = self.diagnostics.write().map_err(|e| e.to_string())?;

        // Remove diagnostics that belong to this owner
        all_diagnostics.retain(|_, diagnostics| {
            diagnostics.retain(|d| d.source.as_ref().map(|s| s.as_str()) != Some(owner));
            !diagnostics.is_empty()
        });

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("diagnostics-cleared", owner) {
            eprintln!(
                "[LanguageFeatures] Failed to emit diagnostics cleared event: {}",
                e
            );
        }

        Ok(())
    }

    /// Clear all providers from a specific owner (for cleanup)
    pub fn clear_owner_providers(&self, owner: &str) -> Result<(), String> {
        self.clear_owned_provider_list(&self.hover_providers, owner)?;
        self.clear_owned_provider_list(&self.definition_providers, owner)?;
        self.clear_owned_provider_list(&self.completion_providers, owner)?;
        self.clear_owned_provider_list(&self.code_action_providers, owner)?;
        self.clear_owned_provider_list(&self.signature_help_providers, owner)?;
        self.clear_owned_provider_list(&self.references_providers, owner)?;
        self.clear_owned_provider_list(&self.code_lens_providers, owner)?;
        self.clear_owned_provider_list(&self.document_highlight_providers, owner)?;
        self.clear_owned_provider_list(&self.folding_range_providers, owner)?;
        self.clear_owned_provider_list(&self.rename_providers, owner)?;
        self.clear_owned_provider_list(&self.document_symbols_providers, owner)?;
        self.clear_owned_provider_list(&self.workspace_symbols_providers, owner)?;
        self.clear_owned_provider_list(&self.document_formatting_providers, owner)?;
        self.clear_owned_provider_list(&self.range_formatting_providers, owner)?;
        self.clear_owned_provider_list(&self.on_type_formatting_providers, owner)?;
        self.clear_owned_provider_list(&self.semantic_tokens_providers, owner)?;
        self.clear_owned_provider_list(&self.inline_values_providers, owner)?;
        self.clear_owned_provider_list(&self.color_providers, owner)?;
        self.clear_owned_provider_list(&self.selection_range_providers, owner)?;
        self.clear_owned_provider_list(&self.linked_editing_range_providers, owner)?;

        // Also clear diagnostics
        self.clear_diagnostics(owner)?;

        println!(
            "[LanguageFeatures] Cleared all providers for owner: {}",
            owner
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_selector_matches() {
        let selector = DocumentSelector {
            filters: vec![DocumentFilter {
                language: Some("rust".to_string()),
                scheme: Some("file".to_string()),
                pattern: Some("*.rs".to_string()),
            }],
        };

        assert!(selector.matches("rust", "file:///path/to/file.rs"));
        assert!(!selector.matches("python", "file:///path/to/file.rs"));
        assert!(!selector.matches("rust", "http://example.com/file.rs"));
    }

    #[test]
    fn test_simple_glob_match() {
        assert!(simple_glob_match("test.rs", "*.rs"));
        assert!(simple_glob_match("file.rs", "file.rs"));
        assert!(simple_glob_match("anything", "*"));
        assert!(!simple_glob_match("test.py", "*.rs"));
    }
}
