use crate::commands::{
    CodeActionProvider, CodeLensProvider, ColorProvider, CompletionProvider, DefinitionProvider,
    DocumentFormattingProvider, DocumentHighlightProvider, DocumentSelector,
    DocumentSymbolsProvider, FoldingRangeProvider, HoverProvider, LanguageFeaturesRegistry,
    OnTypeFormattingProvider, RangeFormattingProvider,
    ReferencesProvider, RenameProvider, SelectionRangeProvider, SemanticTokensLegend,
    SemanticTokensProvider, SignatureHelpProvider, WorkspaceSymbolsProvider,
};
use serde_json::{json, Value};

pub(super) struct ProviderHandlerContext<'a> {
    pub payload: &'a Value,
    pub provider_type: &'a str,
}

pub(super) fn handle_register_provider(
    context: &ProviderHandlerContext, registry: &LanguageFeaturesRegistry,
) -> Result<Value, String> {
    let provider_id = uuid::Uuid::new_v4().to_string();
    let owner =
        context.payload.get("owner").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

    let selector = parse_document_selector(context.payload);
    let payload = context.payload;
    let provider_type = context.provider_type;

    match provider_type {
        "completion" => {
            let trigger_characters = payload
                .get("triggerCharacters")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let provider = CompletionProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                trigger_characters: trigger_characters.clone(),
            };
            registry.register_completion_provider(provider)?;
        }
        "hover" => {
            let provider = HoverProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_hover_provider(provider)?;
        }
        "definition" => {
            let provider = DefinitionProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_definition_provider(provider)?;
        }
        "references" => {
            let provider = ReferencesProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_references_provider(provider)?;
        }
        "codeAction" => {
            let code_action_kinds = payload
                .get("metadata")
                .and_then(|m| m.get("providedCodeActionKinds"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let provider = CodeActionProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                code_action_kinds,
            };
            registry.register_code_action_provider(provider)?;
        }
        "documentSymbol" => {
            let provider = DocumentSymbolsProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_document_symbols_provider(provider)?;
        }
        "formatting" => {
            let provider = DocumentFormattingProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_document_formatting_provider(provider)?;
        }
        "rename" => {
            let prepare_provider =
                payload.get("prepareProvider").and_then(|v| v.as_bool()).unwrap_or(false);
            let provider = RenameProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                prepare_provider,
            };
            registry.register_rename_provider(provider)?;
        }
        "signatureHelp" => {
            let trigger_characters: Vec<String> = payload
                .get("triggerCharacters")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let retrigger_characters: Vec<String> = payload
                .get("retriggerCharacters")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let provider = SignatureHelpProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                trigger_characters: trigger_characters.clone(),
                retrigger_characters,
            };
            registry.register_signature_help_provider(provider)?;
        }
        "codeLens" => {
            let provider = CodeLensProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_code_lens_provider(provider)?;
        }
        "documentHighlight" => {
            let provider = DocumentHighlightProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_document_highlight_provider(provider)?;
        }
        "foldingRange" => {
            let provider = FoldingRangeProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_folding_range_provider(provider)?;
        }
        "selectionRange" => {
            let provider = SelectionRangeProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_selection_range_provider(provider)?;
        }
        "color" => {
            let provider = ColorProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_color_provider(provider)?;
        }
        "rangeFormatting" => {
            let provider = RangeFormattingProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
            };
            registry.register_range_formatting_provider(provider)?;
        }
        "onTypeFormatting" => {
            let trigger_characters: Vec<String> = payload
                .get("triggerCharacters")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let provider = OnTypeFormattingProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                trigger_characters: trigger_characters.clone(),
            };
            registry.register_on_type_formatting_provider(provider)?;
        }
        "semanticTokens" => {
            let legend_val = payload.get("legend");
            let legend = legend_val.map(|l| SemanticTokensLegend {
                        token_types: l
                            .get("tokenTypes")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        token_modifiers: l
                            .get("tokenModifiers")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                .unwrap_or(SemanticTokensLegend {
                    token_types: Vec::new(),
                    token_modifiers: Vec::new(),
                });
            let provider = SemanticTokensProvider {
                id: provider_id.clone(),
                owner: owner.clone(),
                selector: selector.clone(),
                legend,
            };
            registry.register_semantic_tokens_provider(provider)?;
        }
        "workspaceSymbol" => {
            let provider =
                WorkspaceSymbolsProvider { id: provider_id.clone(), owner: owner.clone() };
            registry.register_workspace_symbols_provider(provider)?;
        }
        _ => {}
    }

    Ok(json!({
        "success": true,
        "providerId": provider_id,
    }))
}

fn parse_document_selector(payload: &Value) -> DocumentSelector {
    use crate::commands::DocumentFilter;

    let languages = payload
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    if let Some(lang) = item.as_str() {
                        Some(DocumentFilter {
                            language: Some(lang.to_string()),
                            scheme: None,
                            pattern: None,
                        })
                    } else {
                        let lang =
                            item.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());
                        let scheme =
                            item.get("scheme").and_then(|s| s.as_str()).map(|s| s.to_string());
                        let pattern =
                            item.get("pattern").and_then(|p| p.as_str()).map(|s| s.to_string());
                        if lang.is_some() || scheme.is_some() || pattern.is_some() {
                            Some(DocumentFilter { language: lang, scheme, pattern })
                        } else {
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    DocumentSelector { filters: languages }
}
