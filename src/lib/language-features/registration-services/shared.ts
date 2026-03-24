import type { LanguageFeatureMonacoRegistrar } from '../monaco-registrar';

export interface LanguageFeatureRegistrationContext {
  monacoRegistrar: LanguageFeatureMonacoRegistrar;
}

export function logProviderRegistered(kind: string, id: string) {
  console.log(`[LanguageFeatures] Registered ${kind} provider:`, id);
}
