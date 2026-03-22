/**
 * Authentication and Secrets APIs
 *
 * OAuth authentication and secure secret storage
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';

// Authentication
export interface AuthenticationSession {
  readonly id: string;
  readonly accessToken: string;
  readonly account: AuthenticationSessionAccountInformation;
  readonly scopes: readonly string[];
}

export interface AuthenticationSessionAccountInformation {
  readonly id: string;
  readonly label: string;
}

export interface AuthenticationProvider {
  onDidChangeSessions: Event<AuthenticationProviderAuthenticationSessionsChangeEvent>;
  getSessions(scopes?: string[]): Promise<readonly AuthenticationSession[]>;
  createSession(scopes: string[]): Promise<AuthenticationSession>;
  removeSession(sessionId: string): Promise<void>;
}

export interface AuthenticationProviderAuthenticationSessionsChangeEvent {
  readonly added: readonly AuthenticationSession[] | undefined;
  readonly removed: readonly AuthenticationSession[] | undefined;
  readonly changed: readonly AuthenticationSession[] | undefined;
}

export interface AuthenticationGetSessionOptions {
  createIfNone?: boolean;
  clearSessionPreference?: boolean;
  forceNewSession?: boolean | { detail: string };
}

export class AuthenticationAPI {
  private _onDidChangeSessions = new EventEmitter<any>();
  readonly onDidChangeSessions = this._onDidChangeSessions.event;

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on('authSessionChanged', (data: any) => {
      this._onDidChangeSessions.fire(data);
    });
  }

  async getSession(
    providerId: string,
    scopes: string[],
    options?: AuthenticationGetSessionOptions
  ): Promise<AuthenticationSession | undefined> {
    const result = await this.bridge.request('authGetSession', {
      providerId,
      scopes,
      options
    }) as { session?: AuthenticationSession };
    return result.session;
  }

  registerAuthenticationProvider(
    id: string,
    label: string,
    provider: AuthenticationProvider,
    options?: { supportsMultipleAccounts?: boolean }
  ): Disposable {
    this.bridge.send('registerAuthProvider', {
      id,
      label,
      options
    });

    return {
      dispose: () => {
        this.bridge.send('unregisterAuthProvider', { id });
      }
    };
  }
}

// Secrets Storage
export interface SecretStorage {
  get(key: string): Promise<string | undefined>;
  store(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
  onDidChange: Event<string>;
}

export class SecretStorageImpl implements SecretStorage {
  private _onDidChange = new EventEmitter<string>();
  readonly onDidChange = this._onDidChange.event;

  constructor(
    private bridge: ExtensionHostBridge,
    private extensionId: string
  ) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on('secretChanged', (data: any) => {
      // Only fire if the change is for this extension
      if (data.extensionId === this.extensionId) {
        this._onDidChange.fire(data.key);
      }
    });
  }

  async get(key: string): Promise<string | undefined> {
    const result = await this.bridge.request('secretGet', {
      extensionId: this.extensionId,
      key
    }) as { value?: string };
    return result?.value;
  }

  async store(key: string, value: string): Promise<void> {
    await this.bridge.request('secretStore', {
      extensionId: this.extensionId,
      key,
      value
    });
  }

  async delete(key: string): Promise<void> {
    await this.bridge.request('secretDelete', {
      extensionId: this.extensionId,
      key
    });
  }
}
