import { invoke } from '@tauri-apps/api/core';

import type {
  AiConfig,
  AiMealRow,
  ExportRequest,
  ExportResult,
  FoodResult,
  NutrientSummary,
  RecommendationFilter,
  TdeeRequest,
  TdeeResponse,
} from './types';

export type CommandPayload = Record<string, unknown>;
export type CommandInvoker = <T>(command: string, payload?: CommandPayload) => Promise<T>;

export function createCommandInvoker(invokeFn: typeof invoke = invoke): CommandInvoker {
  return <T>(command: string, payload?: CommandPayload) => invokeFn<T>(command, payload);
}

export const invokeCommand = createCommandInvoker();

export function searchFoodsByName(query: string): Promise<FoodResult[]> {
  return invokeCommand<FoodResult[]>('food_search', { query, limit: 20 });
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
