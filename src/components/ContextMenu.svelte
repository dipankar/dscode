<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getCommandContext, type CommandContext } from '../lib/command-context';
  import { executeCommand as dispatchCommand } from '../lib/command-dispatcher';
  import { registryCommands } from '../lib/contracts/commands';
  import { evaluateWhenClause } from '../lib/when-clause';

  export let visible: boolean = false;
  export let x: number = 0;
  export let y: number = 0;
  export let location: string = 'editor/context';
  export let context: MenuContext = {};
  export let onClose: () => void;

  interface MenuItem {
    command: string;
    location?: string;
    when?: string;
    group?: string;
    title?: string;
    icon?: string;
    owner: string;
    alt?: string;
  }

  interface MenuContext {
    has_selection?: boolean;
    editor_focused?: boolean;
    explorer_focused?: boolean;
    resource_extension?: string;
    resource_path?: string;
    language_id?: string;
    in_debug_mode?: boolean;
    custom?: Record<string, any>;
  }

  let menuItems: MenuItem[] = [];
  let menuElement: HTMLDivElement;
  let commandContext: CommandContext = getCommandContext();
  let wasVisible = false;
  let adjustedX = x;
  let adjustedY = y;
  let focusedItemIndex = -1;
  let previouslyFocused: HTMLElement | null = null;
  const explorerResourceCommands = new Set([
    'explorer.newFile',
    'explorer.newFolder',
    'file.rename',
    'file.delete',
    'copyFilePath',
    'copyRelativeFilePath',
  ]);

  function toCommandContext(location: string, menuContext: MenuContext): CommandContext {
    const base = getCommandContext();
    const editorFocus = menuContext.editor_focused ?? base.editorFocus;
    const explorerFocus = menuContext.explorer_focused ?? location === 'explorer/context';
    const sideBarFocus =
      menuContext.explorer_focused !== undefined
        ? menuContext.explorer_focused
        : editorFocus
          ? false
          : base.sideBarFocus;
    const resourceExtname = menuContext.resource_extension
      ? menuContext.resource_extension.startsWith('.')
        ? menuContext.resource_extension
        : `.${menuContext.resource_extension}`
      : base.resourceExtname;

    return {
      ...base,
      activeEditor: base.activeEditor || editorFocus || !!menuContext.language_id,
      editorFocus,
      editorTextFocus: menuContext.editor_focused ?? base.editorTextFocus,
      editorHasSelection: menuContext.has_selection ?? base.editorHasSelection,
      explorerViewletFocus: explorerFocus,
      sideBarFocus,
      resourceExtname,
      activeEditorLangId: menuContext.language_id ?? base.activeEditorLangId,
      custom: menuContext.custom ?? base.custom,
    };
  }

  async function loadMenuItems() {
    try {
      const items = await registryCommands.getMenuItems<MenuItem>(location);
      menuItems = items.filter((item) => evaluateWhenClause(item.when, commandContext));
    } catch (error) {
      console.error('[ContextMenu] Failed to load menu items:', error);
      menuItems = [];
    }
  }

  async function executeMenuItem(item: MenuItem) {
    try {
      const args: unknown[] = context.resource_path ? [context.resource_path] : [];
      if (explorerResourceCommands.has(item.command)) {
        args.push({
          resourceType: context.custom?.explorerResourceIsDirectory ? 'directory' : 'file',
        });
      }

      await dispatchCommand(item.command, args);
      onClose();
    } catch (error) {
      console.error(`[ContextMenu] Failed to execute command ${item.command}:`, error);
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (menuElement && !menuElement.contains(event.target as Node)) {
      onClose();
    }
  }

  function handleEscape(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }

  function getMenuItemElements(): HTMLButtonElement[] {
    if (!menuElement) return [];
    return Array.from(menuElement.querySelectorAll<HTMLButtonElement>('.context-menu-item'));
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    const items = getMenuItemElements();
    if (items.length === 0) return;

    switch (event.key) {
      case 'ArrowDown': {
        event.preventDefault();
        focusedItemIndex = (focusedItemIndex + 1) % items.length;
        items[focusedItemIndex]?.focus();
        break;
      }
      case 'ArrowUp': {
        event.preventDefault();
        focusedItemIndex = (focusedItemIndex - 1 + items.length) % items.length;
        items[focusedItemIndex]?.focus();
        break;
      }
      case 'Home': {
        event.preventDefault();
        focusedItemIndex = 0;
        items[focusedItemIndex]?.focus();
        break;
      }
      case 'End': {
        event.preventDefault();
        focusedItemIndex = items.length - 1;
        items[focusedItemIndex]?.focus();
        break;
      }
    }
  }

  // Group menu items by group
  function groupMenuItems(items: MenuItem[]): Map<string, MenuItem[]> {
    const groups = new Map<string, MenuItem[]>();

    for (const item of items) {
      const group = item.group || 'z_other';
      if (!groups.has(group)) {
        groups.set(group, []);
      }
      groups.get(group)!.push(item);
    }

    // Sort groups
    const sortedGroups = new Map(
      Array.from(groups.entries()).sort((a, b) => a[0].localeCompare(b[0]))
    );

    return sortedGroups;
  }

  $: if (visible && !wasVisible) {
    commandContext = toCommandContext(location, context);
    wasVisible = true;
  }

  $: if (visible) {
    loadMenuItems();
    previouslyFocused = document.activeElement as HTMLElement;
    focusedItemIndex = -1;

    // Position menu within viewport
    setTimeout(() => {
      if (menuElement) {
        const rect = menuElement.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        if (x + rect.width > viewportWidth) {
          adjustedX = viewportWidth - rect.width - 10;
        } else {
          adjustedX = x;
        }

        if (y + rect.height > viewportHeight) {
          adjustedY = viewportHeight - rect.height - 10;
        } else {
          adjustedY = y;
        }

        // Focus the first menu item
        const firstItem = menuElement.querySelector<HTMLButtonElement>('.context-menu-item');
        if (firstItem) {
          focusedItemIndex = 0;
          firstItem.focus();
        }
      }
    }, 0);
  }

  $: groupedItems = groupMenuItems(menuItems);

  $: if (!visible && wasVisible) {
    wasVisible = false;
    if (previouslyFocused) {
      previouslyFocused.focus();
      previouslyFocused = null;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleEscape);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
    document.removeEventListener('keydown', handleEscape);
  });
</script>

{#if visible}
  <div
    bind:this={menuElement}
    class="context-menu"
    role="menu"
    aria-label="Context menu"
    tabindex="-1"
    style="left: {adjustedX}px; top: {adjustedY}px;"
    on:keydown={handleMenuKeydown}
  >
    {#if menuItems.length === 0}
      <div class="context-menu-empty" role="menuitem">No actions available</div>
    {:else}
      {#each Array.from(groupedItems.entries()) as [_groupName, items], groupIndex}
        {#if groupIndex > 0}
          <div class="context-menu-separator" role="separator"></div>
        {/if}
        {#each items as item}
          <button class="context-menu-item" role="menuitem" on:click={() => executeMenuItem(item)}>
            {#if item.icon}
              <span class="context-menu-icon">{item.icon}</span>
            {/if}
            <span class="context-menu-label">
              {item.title || item.command}
            </span>
          </button>
        {/each}
      {/each}
    {/if}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    background-color: var(--modal-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: var(--shadow-md);
    min-width: 200px;
    max-width: 400px;
    padding: 4px 0;
    z-index: 10000;
    font-size: 13px;
  }

  .context-menu-item {
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    transition: background-color 0.1s;
  }

  .context-menu-item:hover {
    background-color: var(--editor-selection);
  }

  .context-menu-icon {
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .context-menu-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .context-menu-separator {
    height: 1px;
    background-color: var(--color-border);
    margin: 4px 0;
  }

  .context-menu-empty {
    padding: 12px;
    color: var(--color-text-secondary);
    text-align: center;
    font-size: 12px;
  }
</style>
