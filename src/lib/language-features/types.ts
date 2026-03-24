export interface DocumentFilter {
  language?: string;
  scheme?: string;
  pattern?: string;
}

export interface DocumentSelector {
  filters: DocumentFilter[];
}

export interface HoverProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface DefinitionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface CompletionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
}

export interface CodeActionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  code_action_kinds: string[];
}

export interface SignatureHelpProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
  retrigger_characters: string[];
}

export interface ReferencesProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface CodeLensProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface DocumentHighlightProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface FoldingRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface RenameProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  prepare_provider: boolean;
}

export interface DocumentSymbolsProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface WorkspaceSymbolsProvider {
  id: string;
  owner: string;
}

export interface DocumentFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface RangeFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface OnTypeFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
}

export interface SemanticTokensProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  legend: SemanticTokensLegend;
}

export interface SemanticTokensLegend {
  token_types: string[];
  token_modifiers: string[];
}

export interface InlineValuesProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface ColorProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface SelectionRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface LinkedEditingRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface Diagnostic {
  uri: string;
  range: Range;
  severity: DiagnosticSeverity;
  code?: string;
  source?: string;
  message: string;
  related_information?: DiagnosticRelatedInformation[];
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Position {
  line: number;
  character: number;
}

export enum DiagnosticSeverity {
  Error = 1,
  Warning = 2,
  Information = 3,
  Hint = 4,
}

export interface DiagnosticRelatedInformation {
  location: Location;
  message: string;
}

export interface Location {
  uri: string;
  range: Range;
}
