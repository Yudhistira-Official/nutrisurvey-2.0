import { open, save } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { parseProject, serializeProject, type ProjectFile } from './types.ts';

type Dialog = {
  open: typeof open;
  save: typeof save;
};

type FileSystem = {
  readFile: typeof readFile;
  writeFile: typeof writeFile;
};

export function createProjectFileAdapter(
  dialog: Dialog = { open, save },
  fileSystem: FileSystem = { readFile, writeFile },
) {
  return {
    async save(project: Omit<ProjectFile, 'version'>) {
      const path = await dialog.save({ defaultPath: 'Nutri.nutri', filters: [{ name: 'Nutri project', extensions: ['nutri'] }] });
      if (!path) return false;
      await fileSystem.writeFile(path, new TextEncoder().encode(serializeProject(project)));
      return true;
    },
    async open() {
      const path = await dialog.open({ multiple: false, filters: [{ name: 'Nutri project', extensions: ['nutri'] }] });
      if (!path || Array.isArray(path)) return null;
      return parseProject(new TextDecoder().decode(await fileSystem.readFile(path)));
    },
  };
}

export const projectFile = createProjectFileAdapter();
