import { invoke } from '@tauri-apps/api/core';
import type {
  CodeActionProvider,
  CodeLensProvider,
  ColorProvider,
  CompletionProvider,
  DefinitionProvider,
  Diagnostic,
  DocumentFormattingProvider,
  DocumentHighlightProvider,
  DocumentSymbolsProvider,
  FoldingRangeProvider,
  HoverProvider,
  LinkedEditingRangeProvider,
  OnTypeFormattingProvider,
  RangeFormattingProvider,
  ReferencesProvider,
  RenameProvider,
  SelectionRangeProvider,
  SemanticTokensProvider,
  SignatureHelpProvider,
  WorkspaceSymbolsProvider,
} from './types';

export function registerHoverProvider(provider: HoverProvider) {
  return invoke<string>('register_hover_provider', { provider });
}

export function registerDefinitionProvider(provider: DefinitionProvider) {
  return invoke<string>('register_definition_provider', { provider });
}

export function registerCompletionProvider(provider: CompletionProvider) {
  return invoke<string>('register_completion_provider', { provider });
}

export function registerSignatureHelpProvider(provider: SignatureHelpProvider) {
  return invoke<string>('register_signature_help_provider', { provider });
}

export function registerReferencesProvider(provider: ReferencesProvider) {
  return invoke<string>('register_references_provider', { provider });
}

export function registerCodeLensProvider(provider: CodeLensProvider) {
  return invoke<string>('register_code_lens_provider', { provider });
}

export function registerDocumentHighlightProvider(provider: DocumentHighlightProvider) {
  return invoke<string>('register_document_highlight_provider', { provider });
}

export function registerFoldingRangeProvider(provider: FoldingRangeProvider) {
  return invoke<string>('register_folding_range_provider', { provider });
}

export function registerRenameProvider(provider: RenameProvider) {
  return invoke<string>('register_rename_provider', { provider });
}

export function registerDocumentSymbolsProvider(provider: DocumentSymbolsProvider) {
  return invoke<string>('register_document_symbols_provider', { provider });
}

export function registerWorkspaceSymbolsProvider(provider: WorkspaceSymbolsProvider) {
  return invoke<string>('register_workspace_symbols_provider', { provider });
}

export function registerDocumentFormattingProvider(provider: DocumentFormattingProvider) {
  return invoke<string>('register_document_formatting_provider', { provider });
}

export function registerRangeFormattingProvider(provider: RangeFormattingProvider) {
  return invoke<string>('register_range_formatting_provider', { provider });
}

export function registerOnTypeFormattingProvider(provider: OnTypeFormattingProvider) {
  return invoke<string>('register_on_type_formatting_provider', { provider });
}

export function registerSemanticTokensProvider(provider: SemanticTokensProvider) {
  return invoke<string>('register_semantic_tokens_provider', { provider });
}

export function registerColorProvider(provider: ColorProvider) {
  return invoke<string>('register_color_provider', { provider });
}

export function registerSelectionRangeProvider(provider: SelectionRangeProvider) {
  return invoke<string>('register_selection_range_provider', { provider });
}

export function registerLinkedEditingRangeProvider(provider: LinkedEditingRangeProvider) {
  return invoke<string>('register_linked_editing_range_provider', { provider });
}

export function publishDiagnostics(uri: string, diagnostics: Diagnostic[]) {
  return invoke<void>('publish_diagnostics', { uri, diagnostics });
}

export function getDiagnostics(uri: string) {
  return invoke<Diagnostic[]>('get_diagnostics', { uri });
}

export function clearDiagnostics(owner: string) {
  return invoke<void>('clear_diagnostics', { owner });
}

export function executeProviderCommand<T = unknown>(
  providerId: string,
  method: string,
  args: unknown[] = [],
) {
  return invoke<T>('extension_execute_command', {
    command: `${providerId}.${method}`,
    args,
  });
}
