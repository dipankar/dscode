/**
 * Tests for Secrets API
 */

import { SecretStorageImpl, AuthenticationAPI } from '../api/authentication';
import { ExtensionHostBridge } from '../bridge';

jest.mock('../bridge');

describe('SecretStorageImpl', () => {
  let secrets: SecretStorageImpl;
  let mockBridge: jest.Mocked<ExtensionHostBridge>;

  beforeEach(() => {
    mockBridge = new ExtensionHostBridge() as jest.Mocked<ExtensionHostBridge>;
    mockBridge.request = jest.fn();
    mockBridge.on = jest.fn();
    secrets = new SecretStorageImpl(mockBridge, 'test.extension');
  });

  describe('get', () => {
    it('should retrieve secret from bridge', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ value: 'my-secret' });

      const result = await secrets.get('api-key');

      expect(mockBridge.request).toHaveBeenCalledWith('secretGet', {
        extensionId: 'test.extension',
        key: 'api-key'
      });
      expect(result).toBe('my-secret');
    });

    it('should return undefined for non-existent secret', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ value: undefined });

      const result = await secrets.get('non-existent');

      expect(result).toBeUndefined();
    });

    it('should return undefined for null value', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ value: null });

      const result = await secrets.get('null-key');

      expect(result).toBeNull();
    });
  });

  describe('store', () => {
    it('should store secret via bridge', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ success: true });

      await secrets.store('api-key', 'secret-value');

      expect(mockBridge.request).toHaveBeenCalledWith('secretStore', {
        extensionId: 'test.extension',
        key: 'api-key',
        value: 'secret-value'
      });
    });

    it('should handle storing empty string', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ success: true });

      await secrets.store('empty-key', '');

      expect(mockBridge.request).toHaveBeenCalledWith('secretStore', {
        extensionId: 'test.extension',
        key: 'empty-key',
        value: ''
      });
    });
  });

  describe('delete', () => {
    it('should delete secret via bridge', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ success: true });

      await secrets.delete('api-key');

      expect(mockBridge.request).toHaveBeenCalledWith('secretDelete', {
        extensionId: 'test.extension',
        key: 'api-key'
      });
    });
  });

  describe('onDidChange event', () => {
    it('should provide onDidChange event', () => {
      expect(secrets.onDidChange).toBeDefined();
      expect(typeof secrets.onDidChange).toBe('function');
    });
  });

  describe('extension isolation', () => {
    it('should use extension ID in all requests', async () => {
      const secrets1 = new SecretStorageImpl(mockBridge, 'extension.one');
      const secrets2 = new SecretStorageImpl(mockBridge, 'extension.two');

      mockBridge.request = jest.fn().mockResolvedValue({ value: 'value' });

      await secrets1.get('key');
      expect(mockBridge.request).toHaveBeenCalledWith('secretGet', {
        extensionId: 'extension.one',
        key: 'key'
      });

      await secrets2.get('key');
      expect(mockBridge.request).toHaveBeenCalledWith('secretGet', {
        extensionId: 'extension.two',
        key: 'key'
      });
    });
  });
});

describe('AuthenticationAPI', () => {
  let auth: AuthenticationAPI;
  let mockBridge: jest.Mocked<ExtensionHostBridge>;

  beforeEach(() => {
    mockBridge = new ExtensionHostBridge() as jest.Mocked<ExtensionHostBridge>;
    mockBridge.request = jest.fn();
    mockBridge.send = jest.fn();
    mockBridge.on = jest.fn();
    auth = new AuthenticationAPI(mockBridge);
  });

  describe('getSession', () => {
    it('should request session from bridge', async () => {
      const mockSession = {
        id: 'session-1',
        accessToken: 'token-123',
        account: { id: 'user-1', label: 'User' },
        scopes: ['read', 'write']
      };
      mockBridge.request = jest.fn().mockResolvedValue({ session: mockSession });

      const result = await auth.getSession('github', ['read', 'write']);

      expect(mockBridge.request).toHaveBeenCalledWith('authGetSession', {
        providerId: 'github',
        scopes: ['read', 'write'],
        options: undefined
      });
      expect(result).toEqual(mockSession);
    });

    it('should handle createIfNone option', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ session: null });

      await auth.getSession('github', ['read'], { createIfNone: true });

      expect(mockBridge.request).toHaveBeenCalledWith('authGetSession', {
        providerId: 'github',
        scopes: ['read'],
        options: { createIfNone: true }
      });
    });

    it('should return undefined when no session exists', async () => {
      mockBridge.request = jest.fn().mockResolvedValue({ session: undefined });

      const result = await auth.getSession('github', ['read']);

      expect(result).toBeUndefined();
    });
  });

  describe('registerAuthenticationProvider', () => {
    it('should register provider with bridge', () => {
      const mockProvider = {
        onDidChangeSessions: jest.fn(),
        getSessions: jest.fn(),
        createSession: jest.fn(),
        removeSession: jest.fn()
      };

      const disposable = auth.registerAuthenticationProvider(
        'my-auth',
        'My Auth Provider',
        mockProvider as any
      );

      expect(mockBridge.send).toHaveBeenCalledWith('registerAuthProvider', {
        id: 'my-auth',
        label: 'My Auth Provider',
        options: undefined
      });
      expect(disposable).toBeDefined();
    });

    it('should unregister on dispose', () => {
      const mockProvider = {
        onDidChangeSessions: jest.fn(),
        getSessions: jest.fn(),
        createSession: jest.fn(),
        removeSession: jest.fn()
      };

      const disposable = auth.registerAuthenticationProvider(
        'my-auth',
        'My Auth',
        mockProvider as any
      );

      disposable.dispose();

      expect(mockBridge.send).toHaveBeenCalledWith('unregisterAuthProvider', {
        id: 'my-auth'
      });
    });
  });
});
