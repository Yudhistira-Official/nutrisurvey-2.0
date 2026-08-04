export type Nutrients = Record<string, number>;

export type FoodResult = {
  id: number;
  name: string;
  brand?: string | null;
  category?: string | null;
  servingSize: number;
  servingUnit: string;
  servingsPerContainer: number;
  nutrients: Nutrients;
};

export type NutrientSummary = { name: string; unit: string; amount: number };
export type RecommendationFilter = { nutrient: string; operator: '<' | '>' | '='; value: number };
export type MealTime = { id: string; label: string };
export type Targets = { kcal: number; carbs: number; protein: number; fat: number };
export type SessionFood = Omit<FoodResult, 'id'> & {
  id: string;
  sourceFoodId?: number;
  amount: number;
  mealTime: string;
};
export type TdeeRequest = {
  gender: string;
  weightKg: number;
  heightCm: number;
  age: number;
  activityFactor: number;
  injuryFactor: number;
};
export type TdeeResponse = {
  basalMetabolicRate: number;
  totalDailyEnergyExpenditure: number;
  formulaUsed: string;
};
export type AiConfig = { provider: string; model: string; apiKey: string; baseUrl: string };
export type AiMealRow = {
  meal_type: string;
  requested_keyword: string;
  matched_food_id: number;
  matched_food_name: string;
  suggested_grams: number;
  reference_grams: number;
  calories: number;
  protein: number;
  fat: number;
  carbohydrate: number;
  nutrients: Nutrients;
  reasoning: string;
};
export type ExportRequest = {
  foods: Array<{
    mealTime: string;
    name: string;
    amount: number;
    servingSize: number;
    servingUnit: string;
    nutrients: Nutrients;
  }>;
  mealTimes: MealTime[];
  targets: Targets;
};
export type ExportResult = {
  filename: string;
  contentType: string;
  bytes: number[];
  delivery: 'share' | 'saved';
  savedPath?: string | null;
};
export type ProjectFile = {
  version: 1;
  foods: SessionFood[];
  meals: MealTime[];
  targets: Targets;
};

export const defaultMeals: MealTime[] = [
  { id: 'BREAKFAST', label: 'Makan Pagi' },
  { id: 'LUNCH', label: 'Makan Siang' },
  { id: 'DINNER', label: 'Makan Malam' },
];

export const defaultTargets: Targets = { kcal: 0, carbs: 0, protein: 0, fat: 0 };

export function toNumber(value: unknown): number {
  if (typeof value === 'number') return Number.isFinite(value) ? value : 0;
  const number = Number.parseFloat(String(value ?? '').trim().replace(',', '.'));
  return Number.isFinite(number) ? number : 0;
}

const macroMap: Record<string, string[]> = {
  energy: ['energi', 'energy', 'energy (kcal)', 'energi (kkal)', 'energi total', 'total energy'],
  carbs: ['karbohidrat total', 'karbohidrat', 'carbohydrate', 'carbohydrates', 'total carbohydrate', 'karbo'],
  protein: ['protein', 'total protein'],
  fat: ['lemak total', 'lemak', 'fat', 'total fat', 'fats'],
};

export function getMacroValue(nutrients: Nutrients | undefined, type: keyof typeof macroMap): number {
  for (const key of macroMap[type]) {
    const value = nutrients?.[key] ?? nutrients?.[key.charAt(0).toUpperCase() + key.slice(1)];
    if (value !== undefined) return toNumber(value);
  }
  return 0;
}

export function calculateTotals(foods: SessionFood[], targets: Targets) {
  const totals = { energy: 0, carbs: 0, protein: 0, fat: 0 };
  for (const food of foods) {
    const ratio = toNumber(food.amount) / (toNumber(food.servingSize) || 100);
    totals.energy += getMacroValue(food.nutrients, 'energy') * ratio;
    totals.carbs += getMacroValue(food.nutrients, 'carbs') * ratio;
    totals.protein += getMacroValue(food.nutrients, 'protein') * ratio;
    totals.fat += getMacroValue(food.nutrients, 'fat') * ratio;
  }
  return { ...totals, targets };
}

export function serializeProject(project: Omit<ProjectFile, 'version'>): string {
  return JSON.stringify({ version: 1, ...project });
}

export function parseProject(value: string): ProjectFile {
  const data: unknown = JSON.parse(value);
  if (!data || typeof data !== 'object' || (data as { version?: unknown }).version !== 1) {
    throw new Error('File proyek tidak didukung');
  }
  const project = data as Partial<ProjectFile>;
  if (!Array.isArray(project.foods) || !Array.isArray(project.meals) || !project.targets) {
    throw new Error('File proyek tidak valid');
  }
  return { version: 1, foods: project.foods, meals: project.meals, targets: project.targets };
}
