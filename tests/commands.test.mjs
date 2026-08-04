import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const commands = await readFile(new URL('../src/lib/commands.ts', import.meta.url), 'utf8');

 test('frontend command boundary delegates to Tauri invoke', () => {
  assert.match(commands, /export function invokeCommand<T>/);
  assert.match(commands, /invoke<T>\(command/);
});
