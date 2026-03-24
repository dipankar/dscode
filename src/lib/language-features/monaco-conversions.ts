import * as monaco from 'monaco-editor';
import {
  DiagnosticSeverity,
  type Diagnostic,
  type Location,
  type Range,
} from './types';

export function convertRangeToMonaco(range: Range): monaco.IRange {
  return {
    startLineNumber: range.start.line + 1,
    startColumn: range.start.character + 1,
    endLineNumber: range.end.line + 1,
    endColumn: range.end.character + 1,
  };
}

export function convertLocationToMonaco(location: Location): monaco.languages.Location {
  return {
    uri: monaco.Uri.parse(location.uri),
    range: convertRangeToMonaco(location.range),
  };
}

export function convertSelectionRange(range: any): monaco.languages.SelectionRange {
  return {
    range: convertRangeToMonaco(range.range),
  };
}

export function convertDocumentSymbol(symbol: any): monaco.languages.DocumentSymbol {
  return {
    name: symbol.name,
    detail: symbol.detail || '',
    kind: symbol.kind || monaco.languages.SymbolKind.Variable,
    tags: symbol.tags || [],
    range: convertRangeToMonaco(symbol.range),
    selectionRange: convertRangeToMonaco(symbol.selectionRange || symbol.range),
    children: symbol.children
      ? symbol.children.map((child: any) => convertDocumentSymbol(child))
      : [],
  };
}

export function convertSeverityToMonaco(
  severity: DiagnosticSeverity,
): monaco.MarkerSeverity {
  switch (severity) {
    case DiagnosticSeverity.Error:
      return monaco.MarkerSeverity.Error;
    case DiagnosticSeverity.Warning:
      return monaco.MarkerSeverity.Warning;
    case DiagnosticSeverity.Information:
      return monaco.MarkerSeverity.Info;
    case DiagnosticSeverity.Hint:
      return monaco.MarkerSeverity.Hint;
    default:
      return monaco.MarkerSeverity.Info;
  }
}

export function updateMonacoDiagnostics(uri: string, diagnostics: Diagnostic[]) {
  const model = monaco.editor.getModels().find((candidate) => candidate.uri.toString() === uri);
  if (!model) {
    return;
  }

  const markers: monaco.editor.IMarkerData[] = diagnostics.map((diag) => ({
    severity: convertSeverityToMonaco(diag.severity),
    startLineNumber: diag.range.start.line + 1,
    startColumn: diag.range.start.character + 1,
    endLineNumber: diag.range.end.line + 1,
    endColumn: diag.range.end.character + 1,
    message: diag.message,
    source: diag.source,
    code: diag.code,
  }));

  monaco.editor.setModelMarkers(model, 'extension', markers);
}
