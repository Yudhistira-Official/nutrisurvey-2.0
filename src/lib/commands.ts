import { invoke } from '@tauri-apps/api/core';

export function invokeCommand<T>(command: string, payload?: unknown): Promise<T> {
  return invoke<T>(command, payload === undefined ? undefined : { payload });
}
