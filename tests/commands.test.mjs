import test from 'node:test';
import assert from 'node:assert/strict';
import { createCommandInvoker } from '../src/lib/commands.ts';

test('command invoker forwards named payload to Tauri invoke', async () => {
  const calls = [];
  const invoke = async (command, payload) => {
    calls.push({ command, payload });
    return 'pong';
  };
  const invokeCommand = createCommandInvoker(invoke);

  const result = await invokeCommand('calculate', { grams: 125, unit: 'g' });

  assert.equal(result, 'pong');
  assert.deepEqual(calls, [
    { command: 'calculate', payload: { grams: 125, unit: 'g' } },
  ]);
});

test('command invoker supports commands without arguments', async () => {
  const calls = [];
  const invoke = async (command, payload) => {
    calls.push({ command, payload });
    return 'pong';
  };
  const invokeCommand = createCommandInvoker(invoke);

  const result = await invokeCommand('ping');

  assert.equal(result, 'pong');
  assert.deepEqual(calls, [{ command: 'ping', payload: undefined }]);
});
