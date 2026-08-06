import test from 'node:test';
import assert from 'node:assert/strict';
import { createCommandInvoker, createCommandAdapters } from '../src/lib/commands.ts';

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

test('project adapters use native Rust commands with selected paths', async () => {
  const calls = [];
  const adapters = createCommandAdapters(async (command, payload) => {
    calls.push({ command, payload });
    return command === 'project_save' ? true : { version: 1 };
  });
  await adapters.saveProject({ version: 1, foods: [], meals: [], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } });
  await adapters.openProject();
  assert.deepEqual(calls, [
    { command: 'project_save', payload: { project: { version: 1, foods: [], meals: [], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } } } },
    { command: 'project_open', payload: undefined },
  ]);
});

test('project adapters open a saved project by exact history path', async () => {
  const calls = [];
  const adapters = createCommandAdapters(async (command, payload) => {
    calls.push({ command, payload });
    return { version: 1, foods: [], meals: [], targets: { kcal: 0, carbs: 0, protein: 0, fat: 0 } };
  });
  await adapters.openProjectPath('/tmp/saved.nutri');
  assert.deepEqual(calls, [{ command: 'project_open_path', payload: { path: '/tmp/saved.nutri' } }]);
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
