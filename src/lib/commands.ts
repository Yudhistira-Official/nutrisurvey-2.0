import { invoke } from '@tauri-apps/api/core';

export type CommandPayload = Record<string, unknown>;
export type CommandInvoker = <T>(command: string, payload?: CommandPayload) => Promise<T>;

export function createCommandInvoker(invokeFn: typeof invoke = invoke): CommandInvoker {
  return <T>(command: string, payload?: CommandPayload) => invokeFn<T>(command, payload);
}

export const invokeCommand = createCommandInvoker();
