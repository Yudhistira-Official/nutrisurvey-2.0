import { createCommandAdapters } from './commands.ts';
import type { ProjectFile } from './types.ts';

type ProjectCommands = Pick<ReturnType<typeof createCommandAdapters>, 'saveProject' | 'openProject'>;

export function createProjectFileAdapter(commands: ProjectCommands) {
  return {
    save: (project: Omit<ProjectFile, 'version'>) => commands.saveProject({ version: 1, ...project }),
    open: () => commands.openProject(),
  };
}

export const projectFile = createProjectFileAdapter(createCommandAdapters());
