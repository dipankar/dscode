/**
 * Tests for Workspace API
 */

import { WorkspaceAPI } from '../api/workspace';
import { ExtensionHostBridge } from '../bridge';

// Mock the bridge
jest.mock('../bridge');

describe('WorkspaceAPI', () => {
  let workspace: WorkspaceAPI;
  let mockBridge: jest.Mocked<ExtensionHostBridge>;

  beforeEach(() => {
    mockBridge = new ExtensionHostBridge() as jest.Mocked<ExtensionHostBridge>;
    workspace = new WorkspaceAPI(mockBridge);
  });

  describe('loadWorkspaceFolders', () => {
    it('should handle array response correctly', async () => {
      // Mock successful array response
      mockBridge.request = jest.fn().mockResolvedValue(['/path/to/folder1', '/path/to/folder2']);

      // Access private method via any cast for testing
      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.folders).toBeDefined();
      expect(workspace.folders).toHaveLength(2);
      expect(workspace.folders?.[0].uri.fsPath).toBe('/path/to/folder1');
      expect(workspace.folders?.[1].uri.fsPath).toBe('/path/to/folder2');
    });

    it('should handle empty array response', async () => {
      mockBridge.request = jest.fn().mockResolvedValue([]);

      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.folders).toBeUndefined();
    });

    it('should handle non-array response gracefully', async () => {
      // Mock malformed response (this was the bug we fixed)
      mockBridge.request = jest.fn().mockResolvedValue({ status: 'ok' });

      await (workspace as any).loadWorkspaceFolders();

      // Should not throw, should default to empty
      expect(workspace.folders).toBeUndefined();
    });

    it('should handle null response', async () => {
      mockBridge.request = jest.fn().mockResolvedValue(null);

      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.folders).toBeUndefined();
    });

    it('should handle undefined response', async () => {
      mockBridge.request = jest.fn().mockResolvedValue(undefined);

      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.folders).toBeUndefined();
    });

    it('should handle request error gracefully', async () => {
      mockBridge.request = jest.fn().mockRejectedValue(new Error('Connection failed'));

      await (workspace as any).loadWorkspaceFolders();

      // Should not throw, should default to empty
      expect(workspace.folders).toBeUndefined();
    });

    it('should correctly map folder paths to WorkspaceFolder objects', async () => {
      mockBridge.request = jest.fn().mockResolvedValue(['/home/user/project']);

      await (workspace as any).loadWorkspaceFolders();

      const folder = workspace.folders?.[0];
      expect(folder).toBeDefined();
      expect(folder?.uri.fsPath).toBe('/home/user/project');
      expect(folder?.uri.scheme).toBe('file');
      expect(folder?.name).toBe('project');
      expect(folder?.index).toBe(0);
    });
  });

  describe('findFiles', () => {
    it('should return files from bridge request', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({
        files: ['/path/to/file1.ts', '/path/to/file2.ts'],
      });

      const result = await workspace.findFiles('**/*.ts');

      expect(result).toHaveLength(2);
      expect(result[0].fsPath).toBe('/path/to/file1.ts');
      expect(result[0].scheme).toBe('file');
    });

    it('should handle empty result', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ files: [] });

      const result = await workspace.findFiles('**/*.nonexistent');

      expect(result).toHaveLength(0);
    });
  });

  describe('getConfiguration', () => {
    it('should return default values for unknown keys', () => {
      const config = workspace.getConfiguration('editor');

      expect(config.get('fontSize', 14)).toBe(14);
      expect(config.get('unknownKey', 'default')).toBe('default');
    });

    it('should report has() as false for all keys', () => {
      const config = workspace.getConfiguration('editor');

      expect(config.has('anyKey')).toBe(false);
    });
  });

  describe('workspace properties', () => {
    it('should return undefined name when no folders', async () => {
      mockBridge.request = jest.fn().mockResolvedValue([]);
      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.name).toBeUndefined();
    });

    it('should return first folder name when folders exist', async () => {
      mockBridge.request = jest.fn().mockResolvedValue(['/home/user/my-project']);
      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.name).toBe('my-project');
    });

    it('should return undefined rootPath when no folders', async () => {
      mockBridge.request = jest.fn().mockResolvedValue([]);
      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.rootPath).toBeUndefined();
    });

    it('should return first folder path as rootPath', async () => {
      mockBridge.request = jest.fn().mockResolvedValue(['/home/user/project']);
      await (workspace as any).loadWorkspaceFolders();

      expect(workspace.rootPath).toBe('/home/user/project');
    });
  });
});
