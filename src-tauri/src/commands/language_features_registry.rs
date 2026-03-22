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

    /// Register a hover provider
    pub fn register_hover_provider(&self, provider: HoverProvider) -> Result<String, String> {
        let mut providers = self.hover_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered hover provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a definition provider
    pub fn register_definition_provider(
        &self,
        provider: DefinitionProvider,
    ) -> Result<String, String> {
        let mut providers = self.definition_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered definition provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a completion provider
    pub fn register_completion_provider(
        &self,
        provider: CompletionProvider,
    ) -> Result<String, String> {
        let mut providers = self.completion_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered completion provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a code action provider
    pub fn register_code_action_provider(
        &self,
        provider: CodeActionProvider,
    ) -> Result<String, String> {
        let mut providers = self.code_action_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered code action provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a signature help provider
    pub fn register_signature_help_provider(
        &self,
        provider: SignatureHelpProvider,
    ) -> Result<String, String> {
        let mut providers = self.signature_help_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered signature help provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a references provider
    pub fn register_references_provider(
        &self,
        provider: ReferencesProvider,
    ) -> Result<String, String> {
        let mut providers = self.references_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered references provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a code lens provider
    pub fn register_code_lens_provider(
        &self,
        provider: CodeLensProvider,
    ) -> Result<String, String> {
        let mut providers = self.code_lens_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered code lens provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a document highlight provider
    pub fn register_document_highlight_provider(
        &self,
        provider: DocumentHighlightProvider,
    ) -> Result<String, String> {
        let mut providers = self.document_highlight_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered document highlight provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a folding range provider
    pub fn register_folding_range_provider(
        &self,
        provider: FoldingRangeProvider,
    ) -> Result<String, String> {
        let mut providers = self.folding_range_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered folding range provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a rename provider
    pub fn register_rename_provider(
        &self,
        provider: RenameProvider,
    ) -> Result<String, String> {
        let mut providers = self.rename_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered rename provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a document symbols provider
    pub fn register_document_symbols_provider(
        &self,
        provider: DocumentSymbolsProvider,
    ) -> Result<String, String> {
        let mut providers = self.document_symbols_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered document symbols provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a workspace symbols provider
    pub fn register_workspace_symbols_provider(
        &self,
        provider: WorkspaceSymbolsProvider,
    ) -> Result<String, String> {
        let mut providers = self.workspace_symbols_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered workspace symbols provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a document formatting provider
    pub fn register_document_formatting_provider(
        &self,
        provider: DocumentFormattingProvider,
    ) -> Result<String, String> {
        let mut providers = self.document_formatting_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered document formatting provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a range formatting provider
    pub fn register_range_formatting_provider(
        &self,
        provider: RangeFormattingProvider,
    ) -> Result<String, String> {
        let mut providers = self.range_formatting_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered range formatting provider: {}",
            id
        );

        Ok(id)
    }

    /// Register an on-type formatting provider
    pub fn register_on_type_formatting_provider(
        &self,
        provider: OnTypeFormattingProvider,
    ) -> Result<String, String> {
        let mut providers = self.on_type_formatting_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered on-type formatting provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a semantic tokens provider
    pub fn register_semantic_tokens_provider(
        &self,
        provider: SemanticTokensProvider,
    ) -> Result<String, String> {
        let mut providers = self.semantic_tokens_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered semantic tokens provider: {}",
            id
        );

        Ok(id)
    }

    /// Register an inline values provider
    pub fn register_inline_values_provider(
        &self,
        provider: InlineValuesProvider,
    ) -> Result<String, String> {
        let mut providers = self.inline_values_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered inline values provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a color provider
    pub fn register_color_provider(
        &self,
        provider: ColorProvider,
    ) -> Result<String, String> {
        let mut providers = self.color_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered color provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a selection range provider
    pub fn register_selection_range_provider(
        &self,
        provider: SelectionRangeProvider,
    ) -> Result<String, String> {
        let mut providers = self.selection_range_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered selection range provider: {}",
            id
        );

        Ok(id)
    }

    /// Register a linked editing range provider
    pub fn register_linked_editing_range_provider(
        &self,
        provider: LinkedEditingRangeProvider,
    ) -> Result<String, String> {
        let mut providers = self.linked_editing_range_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!(
            "[LanguageFeatures] Registered linked editing range provider: {}",
            id
        );

        Ok(id)
    }

    /// Get hover providers for a document
    pub fn get_hover_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<HoverProvider> {
        let providers = self.hover_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get definition providers for a document
    pub fn get_definition_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<DefinitionProvider> {
        let providers = self.definition_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get completion providers for a document
    pub fn get_completion_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<CompletionProvider> {
        let providers = self.completion_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get code action providers for a document
    pub fn get_code_action_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<CodeActionProvider> {
        let providers = self.code_action_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get signature help providers for a document
    pub fn get_signature_help_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<SignatureHelpProvider> {
        let providers = self.signature_help_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get references providers for a document
    pub fn get_references_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<ReferencesProvider> {
        let providers = self.references_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get code lens providers for a document
    pub fn get_code_lens_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<CodeLensProvider> {
        let providers = self.code_lens_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get document highlight providers for a document
    pub fn get_document_highlight_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<DocumentHighlightProvider> {
        let providers = self.document_highlight_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get folding range providers for a document
    pub fn get_folding_range_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<FoldingRangeProvider> {
        let providers = self.folding_range_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get rename providers for a document
    pub fn get_rename_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<RenameProvider> {
        let providers = self.rename_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get document symbols providers for a document
    pub fn get_document_symbols_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<DocumentSymbolsProvider> {
        let providers = self.document_symbols_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get all workspace symbols providers
    pub fn get_workspace_symbols_providers(&self) -> Vec<WorkspaceSymbolsProvider> {
        let providers = self.workspace_symbols_providers.read().unwrap();
        providers.iter().cloned().collect()
    }

    /// Get document formatting providers for a document
    pub fn get_document_formatting_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<DocumentFormattingProvider> {
        let providers = self.document_formatting_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get range formatting providers for a document
    pub fn get_range_formatting_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<RangeFormattingProvider> {
        let providers = self.range_formatting_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get on-type formatting providers for a document
    pub fn get_on_type_formatting_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<OnTypeFormattingProvider> {
        let providers = self.on_type_formatting_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get semantic tokens providers for a document
    pub fn get_semantic_tokens_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<SemanticTokensProvider> {
        let providers = self.semantic_tokens_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get inline values providers for a document
    pub fn get_inline_values_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<InlineValuesProvider> {
        let providers = self.inline_values_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get color providers for a document
    pub fn get_color_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<ColorProvider> {
        let providers = self.color_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get selection range providers for a document
    pub fn get_selection_range_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<SelectionRangeProvider> {
        let providers = self.selection_range_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
    }

    /// Get linked editing range providers for a document
    pub fn get_linked_editing_range_providers(
        &self,
        language: &str,
        uri: &str,
    ) -> Vec<LinkedEditingRangeProvider> {
        let providers = self.linked_editing_range_providers.read().unwrap();
        providers
            .iter()
            .filter(|p| p.selector.matches(language, uri))
            .cloned()
            .collect()
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
        if let Err(e) = self.app_handle.emit("diagnostics-changed", &(uri, diagnostics)) {
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
            diagnostics.retain(|d| {
                d.source.as_ref().map(|s| s.as_str()) != Some(owner)
            });
            !diagnostics.is_empty()
        });

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("diagnostics-cleared", owner) {
            eprintln!("[LanguageFeatures] Failed to emit diagnostics cleared event: {}", e);
        }

        Ok(())
    }

    /// Clear all providers from a specific owner (for cleanup)
    pub fn clear_owner_providers(&self, owner: &str) -> Result<(), String> {
        {
            let mut hover_providers = self.hover_providers.write().map_err(|e| e.to_string())?;
            hover_providers.retain(|p| p.owner != owner);
        }

        {
            let mut definition_providers = self.definition_providers.write().map_err(|e| e.to_string())?;
            definition_providers.retain(|p| p.owner != owner);
        }

        {
            let mut completion_providers = self.completion_providers.write().map_err(|e| e.to_string())?;
            completion_providers.retain(|p| p.owner != owner);
        }

        {
            let mut code_action_providers = self.code_action_providers.write().map_err(|e| e.to_string())?;
            code_action_providers.retain(|p| p.owner != owner);
        }

        {
            let mut signature_help_providers = self.signature_help_providers.write().map_err(|e| e.to_string())?;
            signature_help_providers.retain(|p| p.owner != owner);
        }

        {
            let mut references_providers = self.references_providers.write().map_err(|e| e.to_string())?;
            references_providers.retain(|p| p.owner != owner);
        }

        {
            let mut code_lens_providers = self.code_lens_providers.write().map_err(|e| e.to_string())?;
            code_lens_providers.retain(|p| p.owner != owner);
        }

        {
            let mut document_highlight_providers = self.document_highlight_providers.write().map_err(|e| e.to_string())?;
            document_highlight_providers.retain(|p| p.owner != owner);
        }

        {
            let mut folding_range_providers = self.folding_range_providers.write().map_err(|e| e.to_string())?;
            folding_range_providers.retain(|p| p.owner != owner);
        }

        {
            let mut rename_providers = self.rename_providers.write().map_err(|e| e.to_string())?;
            rename_providers.retain(|p| p.owner != owner);
        }

        {
            let mut document_symbols_providers = self.document_symbols_providers.write().map_err(|e| e.to_string())?;
            document_symbols_providers.retain(|p| p.owner != owner);
        }

        {
            let mut workspace_symbols_providers = self.workspace_symbols_providers.write().map_err(|e| e.to_string())?;
            workspace_symbols_providers.retain(|p| p.owner != owner);
        }

        {
            let mut document_formatting_providers = self.document_formatting_providers.write().map_err(|e| e.to_string())?;
            document_formatting_providers.retain(|p| p.owner != owner);
        }

        {
            let mut range_formatting_providers = self.range_formatting_providers.write().map_err(|e| e.to_string())?;
            range_formatting_providers.retain(|p| p.owner != owner);
        }

        {
            let mut on_type_formatting_providers = self.on_type_formatting_providers.write().map_err(|e| e.to_string())?;
            on_type_formatting_providers.retain(|p| p.owner != owner);
        }

        {
            let mut semantic_tokens_providers = self.semantic_tokens_providers.write().map_err(|e| e.to_string())?;
            semantic_tokens_providers.retain(|p| p.owner != owner);
        }

        {
            let mut inline_values_providers = self.inline_values_providers.write().map_err(|e| e.to_string())?;
            inline_values_providers.retain(|p| p.owner != owner);
        }

        {
            let mut color_providers = self.color_providers.write().map_err(|e| e.to_string())?;
            color_providers.retain(|p| p.owner != owner);
        }

        {
            let mut selection_range_providers = self.selection_range_providers.write().map_err(|e| e.to_string())?;
            selection_range_providers.retain(|p| p.owner != owner);
        }

        {
            let mut linked_editing_range_providers = self.linked_editing_range_providers.write().map_err(|e| e.to_string())?;
            linked_editing_range_providers.retain(|p| p.owner != owner);
        }

        // Also clear diagnostics
        self.clear_diagnostics(owner)?;

        println!("[LanguageFeatures] Cleared all providers for owner: {}", owner);

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
