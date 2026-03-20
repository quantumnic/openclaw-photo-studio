/**
 * Second Monitor Support
 *
 * Detects and manages second display for simplified develop view.
 */

import { invoke } from '@tauri-apps/api/core';

export interface DisplayInfo {
  display_count: number;
  primary: {
    width: number;
    height: number;
  };
  secondary?: {
    width: number;
    height: number;
  };
}

/**
 * Get display information from Tauri backend
 */
export async function getDisplayInfo(): Promise<DisplayInfo | null> {
  try {
    const info = await invoke<DisplayInfo>('get_display_info');
    return info;
  } catch (error) {
    console.error('Failed to get display info:', error);
    return null;
  }
}

/**
 * Check if a second monitor is available
 */
export async function hasSecondMonitor(): Promise<boolean> {
  const info = await getDisplayInfo();
  return info !== null && info.display_count > 1;
}

/**
 * Open a secondary window for the develop view
 *
 * For now, this opens a new browser window at /secondary
 * In production, this would be a proper Tauri window on the second display.
 */
export function openSecondaryWindow(photoId?: string): Window | null {
  const url = photoId ? `/secondary?photo=${photoId}` : '/secondary';

  const secondaryWindow = window.open(
    url,
    'secondary-display',
    'width=1920,height=1080'
  );

  if (secondaryWindow) {
    secondaryWindow.focus();
  }

  return secondaryWindow;
}

/**
 * Close secondary window
 */
export function closeSecondaryWindow(win: Window | null) {
  if (win && !win.closed) {
    win.close();
  }
}
