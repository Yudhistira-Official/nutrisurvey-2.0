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
export type FoodStatus = { foodCount: number; isReady: boolean };
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
  isManualFactors: boolean;
  bmiStandard: 'asia_pacific' | 'who';
};
export type TdeeResponse = {
  basalMetabolicRate: number;
  totalDailyEnergyExpenditure: number;
  formulaUsed: string;
  bmi: number;
  nutritionClassification: string;
  idealWeight: number;
  adjustedWeight: number;
  referenceWeight: number;
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
  delivery: 'saved';
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

function finite(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value);
}

function validFood(value: unknown): value is SessionFood {
  if (!value || typeof value !== 'object') return false;
  const food = value as Partial<SessionFood>;
  return typeof food.id === 'string' && typeof food.name === 'string' && typeof food.mealTime === 'string' && finite(food.amount) && finite(food.servingSize) && food.servingSize > 0 && typeof food.servingUnit === 'string' && finite(food.servingsPerContainer) && !!food.nutrients && typeof food.nutrients === 'object' && Object.values(food.nutrients).every(finite);
}

function validMeal(value: unknown): value is MealTime {
  if (!value || typeof value !== 'object') return false;
  const meal = value as Partial<MealTime>;
  return typeof meal.id === 'string' && meal.id.length > 0 && typeof meal.label === 'string' && meal.label.trim().length > 0;
}

function validTargets(value: unknown): value is Targets {
  if (!value || typeof value !== 'object') return false;
  const targets = value as Partial<Targets>;
  return finite(targets.kcal) && finite(targets.carbs) && finite(targets.protein) && finite(targets.fat) && targets.kcal >= 0 && targets.carbs >= 0 && targets.protein >= 0 && targets.fat >= 0;
}

export function parseProject(value: string): ProjectFile {
  let data: unknown;
  try { data = JSON.parse(value); } catch { throw new Error('File proyek tidak valid'); }
  if (!data || typeof data !== 'object' || (data as { version?: unknown }).version !== 1) throw new Error('File proyek tidak didukung');
  const project = data as Partial<ProjectFile>;
  if (!Array.isArray(project.foods) || !project.foods.every(validFood) || !Array.isArray(project.meals) || !project.meals.every(validMeal) || !validTargets(project.targets)) throw new Error('File proyek tidak valid');
  const mealIds = new Set(project.meals.map(meal => meal.id));
  if (!project.foods.every(food => mealIds.has(food.mealTime))) throw new Error('File proyek tidak valid');
  return { version: 1, foods: project.foods, meals: project.meals, targets: project.targets };
}

export function classifyUiError(error: unknown, operation: 'Export' | 'Import' | 'Search' | 'Tdee' = 'Export'): string {
  const value = error && typeof error === 'object' ? error as { kind?: unknown; message?: unknown } : {};
  const kind = String(value.kind || '').toLowerCase();
  const message = String(value.message || '').toLowerCase();
  if (kind.includes('validation') || message.includes('valid')) return 'Data tidak valid';
  if (kind.includes('cancel') || message.includes('cancel')) return operation === 'Export' ? 'Ekspor dibatalkan' : 'Operasi dibatalkan';
  if (kind.includes('io') || kind.includes('export') || kind.includes('import') || message.includes('file') || message.includes('write')) return 'Operasi file gagal';
  if (operation === 'Search') return 'Gagal mencari makanan';
  if (operation === 'Tdee') return 'Gagal menghitung TDEE';
  return operation === 'Import' ? 'Gagal impor data' : 'Gagal ekspor Word';
}

export function calculateMacroTargets(response: Pick<TdeeResponse, 'totalDailyEnergyExpenditure'>, percentages: { carbs: number; protein: number; fat: number }): Targets | null {
  if (![percentages.carbs, percentages.protein, percentages.fat].every(finite) || Math.abs(percentages.carbs + percentages.protein + percentages.fat - 100) >= 0.1) return null;
  const kcal = response.totalDailyEnergyExpenditure;
  return { kcal, carbs: kcal * percentages.carbs / 100 / 4, protein: kcal * percentages.protein / 100 / 4, fat: kcal * percentages.fat / 100 / 9 };
}

export function moveFoodToMeal(foods: SessionFood[], foodId: string, mealTime: string): SessionFood[] {
  return foods.map(food => food.id === foodId ? { ...food, mealTime } : food);
}

export function implementAiRows(rows: AiMealRow[], meals: MealTime[]) {
  const added: SessionFood[] = [];
  const unmatched: string[] = [];
  for (const row of rows) {
    const meal = meals.find(item => item.label.trim().toLowerCase() === row.meal_type.trim().toLowerCase());
    if (!meal) { unmatched.push(row.meal_type); continue; }
    const grams = row.suggested_grams || 100;
    added.push({ id: `${Date.now()}-${Math.random()}`, sourceFoodId: row.matched_food_id, name: row.matched_food_name || row.requested_keyword, amount: grams, servingSize: grams, servingUnit: 'g', servingsPerContainer: 1, mealTime: meal.id, nutrients: { ...row.nutrients, energi: row.calories, protein: row.protein, 'lemak total': row.fat, 'karbohidrat total': row.carbohydrate } });
  }
  return { added, unmatched };
}
