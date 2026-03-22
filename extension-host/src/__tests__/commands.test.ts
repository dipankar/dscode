/**
 * Tests for Commands API
 */

import { CommandsAPI } from '../api/commands';
import { ExtensionHostBridge } from '../bridge';

jest.mock('../bridge');

describe('CommandsAPI', () => {
  let commands: CommandsAPI;
  let mockBridge: jest.Mocked<ExtensionHostBridge>;

  beforeEach(() => {
    mockBridge = new ExtensionHostBridge() as jest.Mocked<ExtensionHostBridge>;
    mockBridge.send = jest.fn().mockResolvedValue(undefined);
    mockBridge.request = jest.fn().mockResolvedValue({ result: null });
    commands = new CommandsAPI(mockBridge);
  });

  describe('registerCommand', () => {
    it('should register a command and return disposable', () => {
      const handler = jest.fn();
      const disposable = commands.registerCommand('test.command', handler);

      expect(disposable).toBeDefined();
      expect(typeof disposable.dispose).toBe('function');
      expect(mockBridge.send).toHaveBeenCalledWith('command-registered', expect.objectContaining({
        command: 'test.command'
      }));
    });

    it('should call handler when command is executed', async () => {
      const handler = jest.fn().mockReturnValue('result');
      commands.registerCommand('test.command', handler);

      const result = await commands.executeCommand('test.command', 'arg1', 'arg2');

      expect(handler).toHaveBeenCalledWith('arg1', 'arg2');
      expect(result).toBe('result');
    });

    it('should unregister command on dispose', () => {
      const handler = jest.fn();
      const disposable = commands.registerCommand('test.command', handler);

      disposable.dispose();

      expect(mockBridge.send).toHaveBeenCalledWith('command-unregistered', expect.objectContaining({
        command: 'test.command'
      }));
    });

    it('should handle async command handlers', async () => {
      const handler = jest.fn().mockResolvedValue('async-result');
      commands.registerCommand('test.asyncCommand', handler);

      const result = await commands.executeCommand('test.asyncCommand');

      expect(result).toBe('async-result');
    });
  });

  describe('registerTextEditorCommand', () => {
    it('should register a text editor command', () => {
      const handler = jest.fn();
      const disposable = commands.registerTextEditorCommand('test.editorCommand', handler);

      expect(disposable).toBeDefined();
      expect(mockBridge.send).toHaveBeenCalledWith('command-registered', expect.objectContaining({
        command: 'test.editorCommand'
      }));
    });
  });

  describe('executeCommand', () => {
    it('should execute local commands first', async () => {
      const handler = jest.fn().mockReturnValue('local-result');
      commands.registerCommand('local.command', handler);

      const result = await commands.executeCommand('local.command');

      expect(result).toBe('local-result');
      // Should not call bridge for local command
    });

    it('should delegate to bridge for unknown commands', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ success: true, result: 'remote-result' });

      const result = await commands.executeCommand('remote.command', 'arg1');

      expect(mockBridge.request).toHaveBeenCalledWith('executeCommandRequest', {
        command: 'remote.command',
        args: ['arg1']
      });
      expect(result).toBe('remote-result');
    });

    it('should pass arguments to command handler', async () => {
      const handler = jest.fn().mockReturnValue('result');
      commands.registerCommand('test.command', handler);

      await commands.executeCommand('test.command', 1, 'two', { three: 3 });

      expect(handler).toHaveBeenCalledWith(1, 'two', { three: 3 });
    });
  });

  describe('getCommands', () => {
    it('should return registered commands', async () => {
      commands.registerCommand('test.command1', jest.fn());
      commands.registerCommand('test.command2', jest.fn());

      mockBridge.request = jest.fn().mockResolvedValue({ commands: ['remote.command'] });

      const result = await commands.getCommands();

      expect(result).toContain('test.command1');
      expect(result).toContain('test.command2');
    });

    it('should return all registered commands including internal', async () => {
      commands.registerCommand('test.command', jest.fn());
      commands.registerCommand('_internal.command', jest.fn());

      mockBridge.request = jest.fn().mockResolvedValue({ commands: [] });

      const result = await commands.getCommands();

      expect(result).toContain('test.command');
      expect(result).toContain('_internal.command');
    });
  });

  describe('command error handling', () => {
    it('should propagate errors from command handlers', async () => {
      const error = new Error('Command failed');
      const handler = jest.fn().mockRejectedValue(error);
      commands.registerCommand('test.failingCommand', handler);

      await expect(commands.executeCommand('test.failingCommand'))
        .rejects.toThrow('Command failed');
    });

    it('should handle synchronous errors', async () => {
      const handler = jest.fn().mockImplementation(() => {
        throw new Error('Sync error');
      });
      commands.registerCommand('test.syncError', handler);

      await expect(commands.executeCommand('test.syncError'))
        .rejects.toThrow('Sync error');
    });
  });
});
