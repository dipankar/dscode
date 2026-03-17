import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

let windowCloseHandlerInstalled = false;

/**
 * Enable window persistence - hide window instead of closing
 * This provides instant "reopening" since the process stays alive
 */
export async function enableWindowPersistence() {
  if (windowCloseHandlerInstalled) return;

  const window = getCurrentWindow();

  // Listen for close requested event
  await window.onCloseRequested(async (event) => {
    // Prevent the default close behavior
    event.preventDefault();

    try {
      // Hide the window instead of closing it
      await invoke('minimize_to_tray');
      console.log('[WindowPersistence] Window hidden (process still running)');
    } catch (error) {
      console.error('[WindowPersistence] Failed to minimize to tray:', error);
      // If hiding fails, allow the window to close normally
    }
  });

  windowCloseHandlerInstalled = true;
  console.log('[WindowPersistence] Window persistence enabled - close = hide');
}

/**
 * Check if window is visible
 */
export async function isWindowVisible(): Promise<boolean> {
  try {
    return await invoke('is_window_visible');
  } catch (error) {
    console.error('[WindowPersistence] Failed to check visibility:', error);
    return true;
  }
}

/**
 * Restore window from hidden state
 */
export async function restoreWindow() {
  try {
    await invoke('restore_from_tray');
    console.log('[WindowPersistence] Window restored');
  } catch (error) {
    console.error('[WindowPersistence] Failed to restore window:', error);
  }
}
