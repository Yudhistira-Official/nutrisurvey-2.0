'use client';

import type { MealTime, SessionFood, Targets } from '../lib/types';
import { calculateTotals, getMacroValue, toNumber } from '../lib/types';

export default function Dashboard({ foods, meals, targets, onAddMeal, onAddFood, onRemoveMeal, onRemoveFood, onAmount }: {
  foods: SessionFood[]; meals: MealTime[]; targets: Targets;
  onAddMeal: () => void; onAddFood: (mealId: string) => void; onRemoveMeal: (mealId: string) => void;
  onRemoveFood: (id: string) => void; onAmount: (id: string, amount: number) => void;
}) {
  const totals = calculateTotals(foods, targets);
  const progress = (value: number, target: number) => target > 0 ? `${Math.min(value / target * 100, 100)}%` : '0%';
  return <section>
    <div className="section-header"><div><h2>Manajemen Menu</h2><p>Atur konsumsi harian dan pantau makronutrien.</p></div><button className="primary" onClick={onAddMeal}>+ Waktu Makan</button></div>
    <div className="dashboard-grid">
      <div className="card table-card"><h3>Daftar Konsumsi</h3><table><thead><tr><th>Nama Makanan</th><th>Gram</th><th>kcal</th><th>Carb</th><th>Prot</th><th>Fat</th><th /></tr></thead>
        {meals.map(meal => <tbody key={meal.id}><tr className="meal-header"><td colSpan={7}><strong>{meal.label}</strong><span><button onClick={() => onAddFood(meal.id)}>+</button><button onClick={() => onRemoveMeal(meal.id)}>×</button></span></td></tr>
          {foods.filter(food => food.mealTime === meal.id).map(food => { const ratio = toNumber(food.amount) / (toNumber(food.servingSize) || 100); return <tr key={food.id}><td>{food.name}</td><td><input type="number" value={food.amount} onChange={event => onAmount(food.id, toNumber(event.target.value))} /> {food.servingUnit}</td><td>{(getMacroValue(food.nutrients, 'energy') * ratio).toFixed(0)}</td><td>{(getMacroValue(food.nutrients, 'carbs') * ratio).toFixed(1)}</td><td>{(getMacroValue(food.nutrients, 'protein') * ratio).toFixed(1)}</td><td>{(getMacroValue(food.nutrients, 'fat') * ratio).toFixed(1)}</td><td><button className="danger" onClick={() => onRemoveFood(food.id)}>×</button></td></tr>; })}</tbody>)}
      </table></div>
      <aside className="analysis-panel"><div className="card"><h3>Makronutrien</h3>{[['Energi', totals.energy, targets.kcal, 'kcal'], ['Karbohidrat', totals.carbs, targets.carbs, 'g'], ['Protein', totals.protein, targets.protein, 'g'], ['Lemak', totals.fat, targets.fat, 'g']].map(([label, value, target, unit]) => <div className="metric" key={label as string}><div><span>{label}</span><b>{(value as number).toFixed(unit === 'kcal' ? 0 : 1)}{target ? ` / ${(target as number).toFixed(1)}` : ''} {unit}</b></div><div className="progress"><i style={{ width: progress(value as number, target as number) }} /></div></div>)}</div><div className="card"><h3>Mikronutrien</h3><p className="muted">Data mikronutrien ditampilkan berdasarkan makanan yang dipilih.</p></div></aside>
    </div>
  </section>;
}
