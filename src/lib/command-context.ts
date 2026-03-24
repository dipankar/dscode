import { get, writable } from 'svelte/store';
import { activeActivity } from './activity-store';
import { editorStore } from '../stores/editor';
import { workspaceStore } from '../stores/workspace';

type FocusArea = 'editor' | 'sidebar' | 'panel' | 'other';

export interface CommandContext {
  activeEditor: boolean;
  editorFocus: boolean;
  editorTextFocus: boolean;
  editorHasSelection: boolean;
  explorerViewletFocus: boolean;
  searchViewletFocus: boolean;
  scmViewletFocus: boolean;
  sideBarFocus: boolean;
  resourceExtname?: string;
  activeEditorLangId?: string;
  custom?: Record<string, unknown>;
}

const focusedArea = writable<FocusArea>('other');
let trackingInitialized = false;

function resolveFocusArea(target: EventTarget | null): FocusArea {
  if (!(target instanceof HTMLElement)) {
    return 'other';
  }

  const focusRoot = target.closest<HTMLElement>('[data-command-focus]');
  const area = focusRoot?.dataset.commandFocus;

  if (area === 'editor' || area === 'sidebar' || area === 'panel') {
    return area;
  }

  return 'other';
}

export function initializeCommandContextTracking() {
  if (trackingInitialized || typeof document === 'undefined') {
    return;
  }

  trackingInitialized = true;

  document.addEventListener(
    'focusin',
    (event) => {
      focusedArea.set(resolveFocusArea(event.target));
    },
    true,
  );

  document.addEventListener(
    'focusout',
    () => {
      window.setTimeout(() => {
        focusedArea.set(resolveFocusArea(document.activeElement));
      }, 0);
    },
    true,
  );
}

export function getCommandContext(): CommandContext {
  const editorState = get(editorStore);
  const workspaceState = get(workspaceStore);
  const activity = get(activeActivity);
  const area = get(focusedArea);
  const editor = editorState.monacoInstance;
  const selection = editor?.getSelection();
  const activeTab = editorState.tabs.find((tab) => tab.id === editorState.activeTabId);
  const extSource = activeTab?.path || workspaceState.selectedFile || undefined;
  const resourceExtname = extSource?.includes('.')
    ? `.${extSource.split('.').pop()}`
    : undefined;
  const activeFile = activeTab ? editorState.openFiles.get(activeTab.path) : undefined;
  const editorTextFocus = !!editor?.hasTextFocus?.() || area === 'editor';

  return {
    activeEditor: !!activeTab,
    editorFocus: area === 'editor',
    editorTextFocus,
    editorHasSelection: !!selection && !selection.isEmpty(),
    explorerViewletFocus: area === 'sidebar' && activity === 'explorer',
    searchViewletFocus: area === 'sidebar' && activity === 'search',
    scmViewletFocus: area === 'sidebar' && activity === 'scm',
    sideBarFocus: area === 'sidebar',
    resourceExtname,
    activeEditorLangId: activeFile?.language,
  };
}
