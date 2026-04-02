/**
 * bridge.ts — replaces window.Electron.ipcRenderer.*
 *
 * Provides the same API shape that App.tsx used to call on the Electron bridge,
 * implemented with Tauri's invoke() / listen() instead.
 */
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { IPermuteState, GetStateCallback } from './types';

// ─── State ───────────────────────────────────────────────────────────────────

export const getState = (): Promise<IPermuteState> =>
  invoke<IPermuteState>('get_state');

// ─── Processing ──────────────────────────────────────────────────────────────

/**
 * Starts the permutation run. Progress events arrive as 'permute-update';
 * the completion event arrives as 'permute-ended'.
 */
export function runProcessor(
  updateFn: GetStateCallback,
  completeFn: GetStateCallback
): void {
  let unlistenUpdate: (() => void) | undefined;
  let unlistenEnd: (() => void) | undefined;

  Promise.all([
    listen<IPermuteState>('permute-update', (e) => {
      updateFn(e.payload);
    }).then((u) => {
      unlistenUpdate = u;
    }),
    listen<IPermuteState>('permute-ended', (e) => {
      unlistenUpdate?.();
      unlistenEnd?.();
      completeFn(e.payload);
    }).then((u) => {
      unlistenEnd = u;
    }),
  ]).then(() => {
    invoke('run_processor').catch(console.error);
  });
}

export function reverseFile(
  updateFn: GetStateCallback,
  completeFn: GetStateCallback,
  file: string
): void {
  let unlistenEnd: (() => void) | undefined;
  listen<IPermuteState>('permute-ended', (e) => {
    unlistenEnd?.();
    completeFn(e.payload);
  }).then((u) => {
    unlistenEnd = u;
    invoke('reverse_file', { file }).catch(console.error);
  });
}

export function trimFile(
  updateFn: GetStateCallback,
  completeFn: GetStateCallback,
  file: string
): void {
  let unlistenEnd: (() => void) | undefined;
  listen<IPermuteState>('permute-ended', (e) => {
    unlistenEnd?.();
    completeFn(e.payload);
  }).then((u) => {
    unlistenEnd = u;
    invoke('trim_file', { file }).catch(console.error);
  });
}

export const cancel = (): void => { invoke('cancel').catch(console.error); };

// ─── File management ─────────────────────────────────────────────────────────

export const addFile = (file: string): Promise<void> =>
  invoke('add_file', { file });

export const removeFile = (file: string): Promise<void> =>
  invoke('remove_file', { file });

export const clearAllFiles = (): Promise<void> =>
  invoke('clear_all_files');

export const deleteOutputFile = (file: string): Promise<void> =>
  invoke('delete_output_file', { file });

export const deleteAllOutputFiles = (): Promise<void> =>
  invoke('delete_all_output_files');

export const showFile = (file: string): Promise<void> =>
  invoke('show_in_folder', { file });

// ─── Processor pool ──────────────────────────────────────────────────────────

export const addProcessor = (name: string): Promise<void> =>
  invoke('add_processor', { name });

export const removeProcessor = (name: string): Promise<void> =>
  invoke('remove_processor', { name });

export const selectAllProcessors = (): Promise<void> =>
  invoke('select_all_processors');

export const deselectAllProcessors = (): Promise<void> =>
  invoke('deselect_all_processors');

// ─── Configuration ───────────────────────────────────────────────────────────

export const setOutput = (output: string): Promise<void> =>
  invoke('set_output', { output });

export const setDepth = (depth: number): Promise<void> =>
  invoke('set_depth', { depth });

export const setPermutations = (permutations: number): Promise<void> =>
  invoke('set_permutations', { permutations });

export const setNormalised = (normalised: boolean): Promise<void> =>
  invoke('set_normalised', { normalised });

export const setTrimAll = (trimAll: boolean): Promise<void> =>
  invoke('set_trim_all', { trimAll });

export const setInputTrail = (trail: number): Promise<void> =>
  invoke('set_input_trail', { trail });

export const setOutputTrail = (trail: number): Promise<void> =>
  invoke('set_output_trail', { trail });

export const setMaxStretch = (maxStretch: number): Promise<void> =>
  invoke('set_max_stretch', { maxStretch });

export const setCreateSubdirectories = (create: boolean): Promise<void> =>
  invoke('set_create_subdirectories', { create });

export const setViewedWelcome = (viewed: boolean): Promise<void> =>
  invoke('set_viewed_welcome', { viewed });

// ─── Dialogs ─────────────────────────────────────────────────────────────────

/**
 * Opens a folder picker, sets the output path in the backend, and returns the
 * chosen path (or null if cancelled).
 */
export const openOutputDialog = (): Promise<string | null> =>
  invoke<string | null>('open_output_dialog');

export const saveScene = (): Promise<string | null> =>
  invoke<string | null>('save_scene');

export const loadScene = (): Promise<{ success: boolean; error?: string }> =>
  invoke('load_scene');
