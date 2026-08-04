import { invoke } from '@tauri-apps/api/core';

import type {
  AiConfig,
  AiMealRow,
  ExportRequest,
  ExportResult,
  FoodResult,
  FoodStatus,
  NutrientSummary,
  RecommendationFilter,
  TdeeRequest,
  TdeeResponse,
  ProjectFile,
} from './types';

export type CommandPayload = Record<string, unknown>;
export type CommandInvoker = <T>(command: string, payload?: CommandPayload) => Promise<T>;

export function createCommandInvoker(invokeFn: typeof invoke = invoke): CommandInvoker {
  return <T>(command: string, payload?: CommandPayload) => invokeFn<T>(command, payload);
}

export const invokeCommand = createCommandInvoker();

export function createCommandAdapters(invokeFn: CommandInvoker = invokeCommand) {
  return {
    getFoodStatus: () => invokeFn<FoodStatus>('food_status'),
    searchFoodsByName: (query: string) => invokeFn<FoodResult[]>('food_search', { query, limit: 20 }),
    getNutrientList: () => invokeFn<NutrientSummary[]>('nutrient_list'),
    getRecommendations: (filters: RecommendationFilter[]) => invokeFn<FoodResult[]>('food_recommendations', { filters }),
    calculateTdee: (request: TdeeRequest) => invokeFn<TdeeResponse>('calculate_tdee', { request }),
    exportToWord: (request: ExportRequest) => invokeFn<ExportResult>('export_word', { request }),
    saveProject: (project: ProjectFile) => invokeFn<boolean>('project_save', { project }),
    openProject: () => invokeFn<ProjectFile | null>('project_open'),
  };
}

const adapters = createCommandAdapters();

export function getFoodStatus(): Promise<FoodStatus> {
  return adapters.getFoodStatus();
}

export function searchFoodsByName(query: string): Promise<FoodResult[]> {
  return adapters.searchFoodsByName(query);
}

export function getNutrientList(): Promise<NutrientSummary[]> {
  return invokeCommand<NutrientSummary[]>('nutrient_list');
}

export function getRecommendations(filters: RecommendationFilter[]): Promise<FoodResult[]> {
  return invokeCommand<FoodResult[]>('food_recommendations', { filters });
}

export function calculateTdee(request: TdeeRequest): Promise<TdeeResponse> {
  return invokeCommand<TdeeResponse>('calculate_tdee', { request });
}

export function generateAiMenu(request: {
  targetTDEE: number;
  targetCarbs: number;
  targetProtein: number;
  targetFat: number;
  availableMealTypes: string[];
  aiConfig: AiConfig;
}): Promise<AiMealRow[]> {
  return invokeCommand<AiMealRow[]>('generate_ai_menu', {
    request: {
      targetTdee: request.targetTDEE,
      targetCarbs: request.targetCarbs,
      targetProtein: request.targetProtein,
      targetFat: request.targetFat,
      availableMealTypes: request.availableMealTypes,
      provider: request.aiConfig.provider,
      model: request.aiConfig.model,
      apiKey: request.aiConfig.apiKey,
      baseUrl: request.aiConfig.baseUrl,
    },
  });
}

export function exportToWord(request: ExportRequest): Promise<ExportResult> {
  return invokeCommand<ExportResult>('export_word', { request });
}

export function importCsv(bytes: number[], sourceName: string): Promise<number> {
  return invokeCommand<number>('import_food_csv', { bytes, sourceName });
}

export function importCsvFromPath(path: string): Promise<number> {
  return invokeCommand<number>('import_csv_from_path', { path });
}
