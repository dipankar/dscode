import * as monaco from 'monaco-editor';
import { LanguageFeatureProviderClient } from '../provider-client';
import type { DocumentSelector } from '../types';

export interface MonacoRegistrationContext {
  disposables: monaco.IDisposable[];
  providerClient: LanguageFeatureProviderClient;
}

export function registerForSelectorLanguages(
  context: MonacoRegistrationContext,
  selector: DocumentSelector,
  register: (language: string) => monaco.IDisposable,
) {
  for (const filter of selector.filters) {
    if (!filter.language) {
      continue;
    }

    context.disposables.push(register(filter.language));
  }
}
