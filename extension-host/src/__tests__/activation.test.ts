/**
 * Tests for Activation Events System
 */

import { ActivationEventManager, ParsedActivationEvent } from '../extensions/activation';

describe('ActivationEventManager', () => {
  let manager: ActivationEventManager;

  beforeEach(() => {
    manager = new ActivationEventManager();
  });

  describe('registerExtension', () => {
    it('should register extension with activation events', () => {
      manager.registerExtension('my-extension', ['onLanguage:javascript', 'onCommand:myext.doSomething']);

      // Check immediate activations doesn't include this extension
      expect(manager.getImmediateActivations()).not.toContain('my-extension');
    });

    it('should add immediate activation extensions to immediate list', () => {
      manager.registerExtension('always-ext', ['*']);

      expect(manager.getImmediateActivations()).toContain('always-ext');
    });

    it('should handle empty activation events', () => {
      manager.registerExtension('empty-ext', []);

      expect(manager.getImmediateActivations()).not.toContain('empty-ext');
    });
  });

  describe('unregisterExtension', () => {
    it('should remove extension from registry', () => {
      manager.registerExtension('ext', ['*']);
      expect(manager.getImmediateActivations()).toContain('ext');

      manager.unregisterExtension('ext');

      expect(manager.getImmediateActivations()).not.toContain('ext');
    });

    it('should handle unregistering non-existent extension', () => {
      expect(() => manager.unregisterExtension('non-existent')).not.toThrow();
    });
  });

  describe('parseActivationEvent', () => {
    it('should parse onLanguage events', () => {
      const result = manager.parseActivationEvent('onLanguage:javascript');

      expect(result).toEqual({
        type: 'onLanguage',
        argument: 'javascript'
      });
    });

    it('should parse onCommand events', () => {
      const result = manager.parseActivationEvent('onCommand:myext.doSomething');

      expect(result).toEqual({
        type: 'onCommand',
        argument: 'myext.doSomething'
      });
    });

    it('should parse onView events', () => {
      const result = manager.parseActivationEvent('onView:myExtensionView');

      expect(result).toEqual({
        type: 'onView',
        argument: 'myExtensionView'
      });
    });

    it('should parse onDebug events', () => {
      const result = manager.parseActivationEvent('onDebug:node');

      expect(result).toEqual({
        type: 'onDebug',
        argument: 'node'
      });
    });

    it('should parse onDebugInitialConfigurations', () => {
      const result = manager.parseActivationEvent('onDebugInitialConfigurations');

      expect(result).toEqual({
        type: 'onDebugInitialConfigurations'
      });
    });

    it('should parse onDebugDynamicConfigurations with value', () => {
      const result = manager.parseActivationEvent('onDebugDynamicConfigurations:node');

      expect(result).toEqual({
        type: 'onDebugDynamicConfigurations',
        argument: 'node'
      });
    });

    it('should parse onUri event', () => {
      const result = manager.parseActivationEvent('onUri');

      expect(result).toEqual({
        type: 'onUri'
      });
    });

    it('should parse onFileSystem events', () => {
      const result = manager.parseActivationEvent('onFileSystem:sftp');

      expect(result).toEqual({
        type: 'onFileSystem',
        argument: 'sftp'
      });
    });

    it('should parse onWebviewPanel events', () => {
      const result = manager.parseActivationEvent('onWebviewPanel:myWebview');

      expect(result).toEqual({
        type: 'onWebviewPanel',
        argument: 'myWebview'
      });
    });

    it('should parse onCustomEditor events', () => {
      const result = manager.parseActivationEvent('onCustomEditor:myCustomEditor');

      expect(result).toEqual({
        type: 'onCustomEditor',
        argument: 'myCustomEditor'
      });
    });

    it('should parse onNotebook events', () => {
      const result = manager.parseActivationEvent('onNotebook:jupyter-notebook');

      expect(result).toEqual({
        type: 'onNotebook',
        argument: 'jupyter-notebook'
      });
    });

    it('should parse onAuthenticationRequest events', () => {
      const result = manager.parseActivationEvent('onAuthenticationRequest:github');

      expect(result).toEqual({
        type: 'onAuthenticationRequest',
        argument: 'github'
      });
    });

    it('should parse onTerminalProfile events', () => {
      const result = manager.parseActivationEvent('onTerminalProfile:myShell');

      expect(result).toEqual({
        type: 'onTerminalProfile',
        argument: 'myShell'
      });
    });

    it('should parse workspaceContains events', () => {
      const result = manager.parseActivationEvent('workspaceContains:**/package.json');

      expect(result).toEqual({
        type: 'workspaceContains',
        argument: '**/package.json'
      });
    });

    it('should parse onStartupFinished', () => {
      const result = manager.parseActivationEvent('onStartupFinished');

      expect(result).toEqual({
        type: 'onStartupFinished'
      });
    });

    it('should parse wildcard activation (*)', () => {
      const result = manager.parseActivationEvent('*');

      expect(result).toEqual({
        type: '*'
      });
    });
  });

  describe('activation triggers', () => {
    it('should call callback when language event triggers', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      manager.setActivationCallback(callback);
      manager.registerExtension('js-ext', ['onLanguage:javascript']);

      await manager.triggerOnLanguage('javascript');

      expect(callback).toHaveBeenCalled();
      const [extId, event] = callback.mock.calls[0];
      expect(extId).toBe('js-ext');
      expect(event.type).toBe('onLanguage');
      expect(event.argument).toBe('javascript');
    });

    it('should call callback when command event triggers', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      manager.setActivationCallback(callback);
      manager.registerExtension('cmd-ext', ['onCommand:myext.action']);

      await manager.triggerOnCommand('myext.action');

      expect(callback).toHaveBeenCalled();
      const [extId, event] = callback.mock.calls[0];
      expect(extId).toBe('cmd-ext');
      expect(event.type).toBe('onCommand');
    });

    it('should not trigger activation for non-matching events', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      manager.setActivationCallback(callback);
      manager.registerExtension('js-ext', ['onLanguage:javascript']);

      await manager.triggerOnLanguage('typescript');

      expect(callback).not.toHaveBeenCalled();
    });
  });

  describe('isActivated and markActivated', () => {
    it('should mark extension as activated', () => {
      expect(manager.isActivated('ext')).toBe(false);

      manager.markActivated('ext');

      expect(manager.isActivated('ext')).toBe(true);
    });

    it('should not re-trigger activation for activated extensions', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      manager.setActivationCallback(callback);
      manager.registerExtension('ext', ['onLanguage:javascript']);

      // First trigger should activate
      await manager.triggerOnLanguage('javascript');
      expect(callback).toHaveBeenCalledTimes(1);

      // Second trigger should not activate again
      await manager.triggerOnLanguage('javascript');
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });

  describe('triggerStartupFinished', () => {
    it('should activate extensions registered for startup finished', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      manager.setActivationCallback(callback);
      manager.registerExtension('startup-ext', ['onStartupFinished']);

      await manager.triggerStartupFinished();

      expect(callback).toHaveBeenCalled();
      const [extId, event] = callback.mock.calls[0];
      expect(extId).toBe('startup-ext');
      expect(event.type).toBe('onStartupFinished');
    });
  });

  describe('getPendingActivations', () => {
    it('should return pending activations', () => {
      manager.registerExtension('ext1', ['onLanguage:javascript']);
      manager.registerExtension('ext2', ['onLanguage:typescript']);

      const pending = manager.getPendingActivations();

      expect(pending.get('onLanguage:javascript')).toContain('ext1');
      expect(pending.get('onLanguage:typescript')).toContain('ext2');
    });
  });

  describe('getImmediateActivations', () => {
    it('should return extensions with * activation', () => {
      manager.registerExtension('always-ext', ['*']);
      manager.registerExtension('lazy-ext', ['onLanguage:python']);

      const immediate = manager.getImmediateActivations();

      expect(immediate).toContain('always-ext');
      expect(immediate).not.toContain('lazy-ext');
    });
  });

  describe('events', () => {
    it('should fire onWillActivate before activation', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      const willActivateListener = jest.fn();

      manager.setActivationCallback(callback);
      manager.onWillActivate(willActivateListener);
      manager.registerExtension('ext', ['onLanguage:javascript']);

      await manager.triggerOnLanguage('javascript');

      expect(willActivateListener).toHaveBeenCalled();
      expect(willActivateListener.mock.calls[0][0].extensionId).toBe('ext');
    });

    it('should fire onDidActivate after activation', async () => {
      const callback = jest.fn().mockResolvedValue(undefined);
      const didActivateListener = jest.fn();

      manager.setActivationCallback(callback);
      manager.onDidActivate(didActivateListener);
      manager.registerExtension('ext', ['onLanguage:javascript']);

      await manager.triggerOnLanguage('javascript');

      expect(didActivateListener).toHaveBeenCalled();
      expect(didActivateListener.mock.calls[0][0].extensionId).toBe('ext');
    });
  });
});
