import { invoke } from '@tauri-apps/api/core';
import * as monaco from 'monaco-editor';

export interface GitDecoration {
  line: number;
  type: 'added' | 'modified' | 'deleted';
}

export class GitDecorationService {
  private decorations: Map<string, string[]> = new Map();
  private editor: monaco.editor.IStandaloneCodeEditor | null = null;
  private currentFilePath: string | null = null;

  setEditor(editor: monaco.editor.IStandaloneCodeEditor) {
    this.editor = editor;
  }

  setCurrentFile(filePath: string) {
    this.currentFilePath = filePath;
    this.updateDecorations();
  }

  async updateDecorations() {
    if (!this.editor || !this.currentFilePath) return;

    try {
      // Get the workspace root
      const workspaceRoot = await this.getWorkspaceRoot();
      if (!workspaceRoot) return;

      // Get git diff for the current file
      const diff = await invoke('git_get_diff', {
        repoPath: workspaceRoot,
        filePath: this.currentFilePath,
        staged: false
      });

      const decorations = this.parseDiffToDecorations(diff as any);
      this.applyDecorations(decorations);
    } catch {
      // File might not be in a git repo or has no changes
      this.clearDecorations();
    }
  }

  private parseDiffToDecorations(diff: any): monaco.editor.IModelDeltaDecoration[] {
    if (!diff || !diff.diff_text) return [];

    const decorations: monaco.editor.IModelDeltaDecoration[] = [];
    const lines = diff.diff_text.split('\n');
    let currentLine = 0;

    for (const line of lines) {
      if (line.startsWith('@@')) {
        // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
        const match = line.match(/@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
        if (match) {
          currentLine = parseInt(match[1], 10);
        }
      } else if (line.startsWith('+') && !line.startsWith('+++')) {
        // Added line
        decorations.push({
          range: new monaco.Range(currentLine, 1, currentLine, 1),
          options: {
            isWholeLine: true,
            linesDecorationsClassName: 'git-line-added',
            overviewRuler: {
              color: '#89d185',
              position: monaco.editor.OverviewRulerLane.Left
            },
            minimap: {
              color: '#89d185',
              position: monaco.editor.MinimapPosition.Inline
            }
          }
        });
        currentLine++;
      } else if (line.startsWith('-') && !line.startsWith('---')) {
        // Deleted line (shown at the previous line)
        decorations.push({
          range: new monaco.Range(currentLine, 1, currentLine, 1),
          options: {
            isWholeLine: false,
            linesDecorationsClassName: 'git-line-deleted',
            overviewRuler: {
              color: '#f48771',
              position: monaco.editor.OverviewRulerLane.Left
            }
          }
        });
      } else if (line.startsWith(' ')) {
        // Context line (unchanged)
        currentLine++;
      }
    }

    return decorations;
  }

  private applyDecorations(decorations: monaco.editor.IModelDeltaDecoration[]) {
    if (!this.editor || !this.currentFilePath) return;

    const oldDecorations = this.decorations.get(this.currentFilePath) || [];
    const newDecorations = this.editor.deltaDecorations(oldDecorations, decorations);
    this.decorations.set(this.currentFilePath, newDecorations);
  }

  private clearDecorations() {
    if (!this.editor || !this.currentFilePath) return;

    const oldDecorations = this.decorations.get(this.currentFilePath) || [];
    this.editor.deltaDecorations(oldDecorations, []);
    this.decorations.delete(this.currentFilePath);
  }

  private async getWorkspaceRoot(): Promise<string | null> {
    try {
      const result = await invoke('get_workspace_folders');
      if (Array.isArray(result) && result.length > 0) {
        return result[0];
      }
      return null;
    } catch {
      return null;
    }
  }

  dispose() {
    if (this.editor && this.currentFilePath) {
      this.clearDecorations();
    }
    this.editor = null;
    this.currentFilePath = null;
  }
}

// Global instance
export const gitDecorationService = new GitDecorationService();
