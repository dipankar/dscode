import * as monaco from 'monaco-editor';
import { executeProviderCommand } from '../transport';

export class BaseProviderClient {
  protected executeProviderCommand<T = unknown>(
    providerId: string,
    method: string,
    args: unknown[] = [],
  ): Promise<T> {
    return executeProviderCommand<T>(providerId, method, args);
  }
}

export function toProviderPosition(position: monaco.Position) {
  return {
    line: position.lineNumber - 1,
    character: position.column - 1,
  };
}

export function toProviderRange(range: monaco.IRange) {
  return {
    start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
    end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
  };
}
