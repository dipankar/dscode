/**
 * TextDocument API
 *
 * Represents a text document, such as a source file
 */

import { ExtensionHostBridge } from '../bridge';

export interface Position {
  line: number;
  character: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface TextLine {
  lineNumber: number;
  text: string;
  range: Range;
  rangeIncludingLineBreak: Range;
  firstNonWhitespaceCharacterIndex: number;
  isEmptyOrWhitespace: boolean;
}

export class TextDocument {
  private bridge: ExtensionHostBridge;

  constructor(
    bridge: ExtensionHostBridge,
    public readonly uri: { path: string; fsPath: string; scheme: string },
    public readonly fileName: string,
    public readonly languageId: string,
    public readonly version: number,
    private _text: string,
    public readonly isDirty: boolean = false,
    public readonly isClosed: boolean = false,
    public readonly eol: number = 1 // 1 = LF, 2 = CRLF
  ) {
    this.bridge = bridge;
  }

  get lineCount(): number {
    return this._text.split('\n').length;
  }

  lineAt(lineOrPosition: number | Position): TextLine {
    const lineNumber = typeof lineOrPosition === 'number'
      ? lineOrPosition
      : lineOrPosition.line;

    const lines = this._text.split('\n');
    if (lineNumber < 0 || lineNumber >= lines.length) {
      throw new Error(`Line number ${lineNumber} is out of range`);
    }

    const text = lines[lineNumber];
    const trimmedText = text.trimStart();
    const firstNonWhitespaceCharacterIndex = text.length - trimmedText.length;
    const isEmptyOrWhitespace = trimmedText.length === 0;

    return {
      lineNumber,
      text,
      range: {
        start: { line: lineNumber, character: 0 },
        end: { line: lineNumber, character: text.length }
      },
      rangeIncludingLineBreak: {
        start: { line: lineNumber, character: 0 },
        end: { line: lineNumber + 1, character: 0 }
      },
      firstNonWhitespaceCharacterIndex,
      isEmptyOrWhitespace
    };
  }

  offsetAt(position: Position): number {
    const lines = this._text.split('\n');
    let offset = 0;

    for (let i = 0; i < position.line && i < lines.length; i++) {
      offset += lines[i].length + 1; // +1 for newline
    }

    offset += Math.min(position.character, lines[position.line]?.length || 0);
    return offset;
  }

  positionAt(offset: number): Position {
    const lines = this._text.split('\n');
    let currentOffset = 0;

    for (let line = 0; line < lines.length; line++) {
      const lineLength = lines[line].length + 1; // +1 for newline

      if (currentOffset + lineLength > offset) {
        return {
          line,
          character: offset - currentOffset
        };
      }

      currentOffset += lineLength;
    }

    // If offset is beyond the document, return the end position
    const lastLine = lines.length - 1;
    return {
      line: lastLine,
      character: lines[lastLine]?.length || 0
    };
  }

  getText(range?: Range): string {
    if (!range) {
      return this._text;
    }

    const startOffset = this.offsetAt(range.start);
    const endOffset = this.offsetAt(range.end);
    return this._text.substring(startOffset, endOffset);
  }

  getWordRangeAtPosition(position: Position, regex?: RegExp): Range | undefined {
    const line = this.lineAt(position);
    const text = line.text;

    // Default word regex if not provided
    const wordRegex = regex || /\w+/g;
    let match: RegExpExecArray | null;

    while ((match = wordRegex.exec(text)) !== null) {
      const startChar = match.index;
      const endChar = match.index + match[0].length;

      if (startChar <= position.character && position.character <= endChar) {
        return {
          start: { line: position.line, character: startChar },
          end: { line: position.line, character: endChar }
        };
      }
    }

    return undefined;
  }

  validateRange(range: Range): Range {
    const lines = this._text.split('\n');

    const startLine = Math.max(0, Math.min(range.start.line, lines.length - 1));
    const endLine = Math.max(0, Math.min(range.end.line, lines.length - 1));

    const startChar = Math.max(0, Math.min(range.start.character, lines[startLine]?.length || 0));
    const endChar = Math.max(0, Math.min(range.end.character, lines[endLine]?.length || 0));

    return {
      start: { line: startLine, character: startChar },
      end: { line: endLine, character: endChar }
    };
  }

  validatePosition(position: Position): Position {
    const lines = this._text.split('\n');
    const line = Math.max(0, Math.min(position.line, lines.length - 1));
    const character = Math.max(0, Math.min(position.character, lines[line]?.length || 0));

    return { line, character };
  }

  async save(): Promise<boolean> {
    try {
      await this.bridge.request('saveDocument', { uri: this.uri.fsPath });
      return true;
    } catch (error) {
      console.error('[TextDocument] Save failed:', error);
      return false;
    }
  }
}

/**
 * TextDocumentAPI manages text documents
 */
export class TextDocumentAPI {
  private documents = new Map<string, TextDocument>();

  constructor(private bridge: ExtensionHostBridge) {}

  async openTextDocument(uriOrPath: string | { path: string }): Promise<TextDocument> {
    const path = typeof uriOrPath === 'string' ? uriOrPath : uriOrPath.path;

    // Check if already open
    if (this.documents.has(path)) {
      return this.documents.get(path)!;
    }

    // Request document from main process
    const docData = await this.bridge.request('openTextDocument', { path });

    const uri = {
      path: docData.uri || path,
      fsPath: path,
      scheme: 'file'
    };

    const document = new TextDocument(
      this.bridge,
      uri,
      path,
      docData.languageId || 'plaintext',
      docData.version || 1,
      docData.text || '',
      docData.isDirty || false,
      false,
      docData.eol || 1
    );

    this.documents.set(path, document);
    return document;
  }

  getDocument(uri: string): TextDocument | undefined {
    return this.documents.get(uri);
  }

  getAllDocuments(): TextDocument[] {
    return Array.from(this.documents.values());
  }

  closeDocument(uri: string): void {
    this.documents.delete(uri);
  }
}
