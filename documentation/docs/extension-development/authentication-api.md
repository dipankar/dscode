# Authentication API

DSCode implements the `vscode.authentication` namespace, enabling extensions to authenticate with external services (GitHub, Azure, etc.) and securely store credentials. The authentication API integrates with DSCode's `SecretStorage` for token persistence using the OS keychain.

**Source:** `extension-host/src/api/authentication.ts`

---

## Overview

The Authentication API provides two primary capabilities:

| Capability | Description |
|-----------|-------------|
| **Consuming authentication** | Extensions request sessions from registered providers |
| **Providing authentication** | Extensions register as authentication providers (e.g., GitHub, Azure) |

---

## Requesting an Authentication Session

Extensions that need to authenticate with a service can request a session:

```typescript
import * as vscode from 'vscode';

async function getGitHubToken() {
    // Request a GitHub session with specific scopes
    const session = await vscode.authentication.getSession(
        'github',           // Provider ID
        ['repo', 'user'],   // Required scopes
        { createIfNone: true }  // Prompt user to sign in if needed
    );

    if (session) {
        console.log(`Authenticated as: ${session.account.label}`);
        console.log(`Token: ${session.accessToken}`);
    }
}
```

### Session Options

| Option | Type | Description |
|--------|------|-------------|
| `createIfNone` | `boolean` | Prompt the user to sign in if no session exists |
| `clearSessionPreference` | `boolean` | Clear any saved session preference |
| `forceNewSession` | `boolean` or `{ detail }` | Force creation of a new session |

---

## Authentication Session

A session contains the authenticated user's information and access token:

```typescript
interface AuthenticationSession {
    readonly id: string;          // Unique session identifier
    readonly accessToken: string; // The OAuth access token
    readonly account: {
        readonly id: string;      // Account identifier
        readonly label: string;   // Display name
    };
    readonly scopes: readonly string[];  // Granted scopes
}
```

---

## Registering an Authentication Provider

Extensions can register as authentication providers to handle sign-in flows:

```typescript
import * as vscode from 'vscode';

class GitHubAuthProvider implements vscode.AuthenticationProvider {
    private _onDidChangeSessions = new vscode.EventEmitter<
        vscode.AuthenticationProviderAuthenticationSessionsChangeEvent
    >();
    readonly onDidChangeSessions = this._onDidChangeSessions.event;

    async getSessions(scopes?: string[]): Promise<vscode.AuthenticationSession[]> {
        // Return existing sessions from storage
        const token = await this.secretStorage.get('github-token');
        if (!token) return [];

        return [{
            id: 'github-session-1',
            accessToken: token,
            account: { id: 'user123', label: 'octocat' },
            scopes: scopes ?? ['repo']
        }];
    }

    async createSession(scopes: string[]): Promise<vscode.AuthenticationSession> {
        // Implement OAuth flow (open browser, handle callback)
        const token = await this.performOAuthFlow(scopes);

        // Store token securely
        await this.secretStorage.store('github-token', token);

        const session: vscode.AuthenticationSession = {
            id: `session-${Date.now()}`,
            accessToken: token,
            account: { id: 'user123', label: 'octocat' },
            scopes
        };

        this._onDidChangeSessions.fire({
            added: [session],
            removed: undefined,
            changed: undefined
        });

        return session;
    }

    async removeSession(sessionId: string): Promise<void> {
        await this.secretStorage.delete('github-token');

        this._onDidChangeSessions.fire({
            added: undefined,
            removed: [/* removed session */],
            changed: undefined
        });
    }

    // ... OAuth implementation details
}
```

### Registration

```typescript
export function activate(context: vscode.ExtensionContext) {
    const provider = new GitHubAuthProvider(context.secrets);

    context.subscriptions.push(
        vscode.authentication.registerAuthenticationProvider(
            'github',                    // Provider ID
            'GitHub',                    // Display label
            provider,
            { supportsMultipleAccounts: false }
        )
    );
}
```

---

## SecretStorage Integration

DSCode's authentication system integrates with `SecretStorage` for secure token persistence. The `SecretStorage` uses the OS-native keychain:

| Platform | Backend |
|----------|---------|
| **macOS** | Keychain Services |
| **Linux** | Secret Service (GNOME Keyring / KWallet) |
| **Windows** | Credential Manager |

### Using SecretStorage

```typescript
export function activate(context: vscode.ExtensionContext) {
    // context.secrets is a SecretStorage instance scoped to your extension
    const secrets = context.secrets;

    // Store a secret
    await secrets.store('api-key', 'sk-abc123...');

    // Retrieve a secret
    const key = await secrets.get('api-key');

    // Delete a secret
    await secrets.delete('api-key');

    // Listen for changes
    secrets.onDidChange(key => {
        console.log(`Secret changed: ${key}`);
    });
}
```

Secrets are namespaced by extension ID, so extensions cannot read each other's secrets.

---

## Listening for Session Changes

Extensions can react to authentication changes:

```typescript
vscode.authentication.onDidChangeSessions(event => {
    if (event.provider.id === 'github') {
        // Re-fetch data with new credentials
        refreshGitHubData();
    }
});
```

---

## Example: GitHub OAuth Integration

```typescript
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    // Register a command that requires authentication
    context.subscriptions.push(
        vscode.commands.registerCommand('myext.fetchRepos', async () => {
            // Get or create a GitHub session
            const session = await vscode.authentication.getSession(
                'github',
                ['repo'],
                { createIfNone: true }
            );

            if (!session) {
                vscode.window.showErrorMessage('GitHub authentication required.');
                return;
            }

            // Use the token to call GitHub API
            const response = await fetch('https://api.github.com/user/repos', {
                headers: {
                    'Authorization': `Bearer ${session.accessToken}`,
                    'Accept': 'application/vnd.github.v3+json'
                }
            });

            const repos = await response.json();
            vscode.window.showInformationMessage(
                `Found ${repos.length} repositories`
            );
        })
    );
}
```

---

## See Also

- [Security Architecture](../architecture/security.md) -- secret storage and permissions
- [VS Code API Reference](vscode-api-reference.md) -- full API documentation
- [VS Code Compatibility Matrix](../reference/vscode-compatibility-matrix.md) -- Authentication API coverage
