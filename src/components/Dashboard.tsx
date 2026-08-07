'use client';

import { useState } from 'react';
import type { MealTime, SessionFood, Targets } from '../lib/types';
import { calculateTotals, getMacroValue, toNumber } from '../lib/types';

export default function Dashboard({ foods, meals, targets, onAddMeal, onAddFood, onRemoveMeal, onRemoveFood, onAmount, onMoveFood, onReorderMeals, onReorderFoods }: {
  foods: SessionFood[]; meals: MealTime[]; targets: Targets;
  onAddMeal: () => void; onAddFood: (mealId: string) => void; onRemoveMeal: (mealId: string) => void;
  onRemoveFood: (id: string) => void; onAmount: (id: string, amount: number) => void; onMoveFood: (foodId: string, mealId: string) => void;
  onReorderMeals: (draggedId: string, targetId: string) => void; onReorderFoods: (draggedId: string, targetId: string) => void;
}) {
  const totals = calculateTotals(foods, targets);
  const [dragging, setDragging] = useState<{ type: 'meal' | 'food'; id: string } | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null);
  const progress = (value: number, target: number) => target > 0 ? `${Math.min(value / target * 100, 100)}%` : '0%';
  const progressColor = (value: number, target: number) => {
    if (!target) return '#3b82f6';
    const diff = value - target;
    if (diff < 0) return '#3b82f6';
    if (diff <= 5) return '#4ade80';
    if (diff <= 10) return '#facc15';
    return '#ef4444';
  };
  const startDrag = (event: React.DragEvent, type: 'meal' | 'food', id: string) => {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('application/x-nutrisurvey-type', type);
    event.dataTransfer.setData('text/plain', id);
    setDragging({ type, id });
  };
  const finishDrag = () => { setDragging(null); setDropTarget(null); };
  const readDragPayload = (event: React.DragEvent) => {
    const id = event.dataTransfer.getData('text/plain').trim();
    const type = event.dataTransfer.getData('application/x-nutrisurvey-type');
    return id && (type === 'meal' || type === 'food') ? { id, type } : null;
  };
  const dropOnMeal = (event: React.DragEvent, targetMealId: string) => {
    event.preventDefault();
    const payload = readDragPayload(event);
    if (payload?.type === 'meal') onReorderMeals(payload.id, targetMealId);
    if (payload?.type === 'food') onMoveFood(payload.id, targetMealId);
    finishDrag();
  };
  const dropOnFood = (event: React.DragEvent, targetFoodId: string) => {
    event.preventDefault();
    event.stopPropagation();
    const payload = readDragPayload(event);
    if (payload?.type === 'food') onReorderFoods(payload.id, targetFoodId);
    finishDrag();
  };
  return <section>
    <div className="section-header"><div><h2>Manajemen Menu</h2><p>Atur konsumsi harian dan pantau makronutrien.</p></div><button className="primary" onClick={onAddMeal}>+ Waktu Makan</button></div>
    <div className="dashboard-grid">
      <div className="card table-card"><div className="dashboard-table-heading"><div><h3>Daftar Konsumsi</h3><p className="muted">Tarik seluruh waktu makan untuk mengubah urutan. Tarik makanan untuk memindahkan atau mengurutkan isinya.</p></div></div><table><thead><tr><th>Nama Makanan</th><th>Gram</th><th>kcal</th><th>Carb</th><th>Prot</th><th>Fat</th><th /></tr></thead>
        {meals.map(meal => <tbody key={meal.id} className={`meal-drop-zone ${dropTarget === `meal:${meal.id}` ? 'meal-drop-active' : ''} ${dragging?.id === meal.id ? 'meal-dragging' : ''}`} onDragOver={event => { event.preventDefault(); if (dragging) setDropTarget(`meal:${meal.id}`); }} onDrop={event => dropOnMeal(event, meal.id)}><tr className="meal-header" draggable onDragStart={event => startDrag(event, 'meal', meal.id)} onDragEnd={finishDrag}><td colSpan={7}><strong>{meal.label}</strong><span><button aria-label={`Tambah makanan ke ${meal.label}`} onClick={() => onAddFood(meal.id)}>+</button><button aria-label={`Hapus waktu makan ${meal.label}`} onClick={() => onRemoveMeal(meal.id)}>×</button></span></td></tr>
          {(() => {
              const mealFoods = foods.filter(food => food.mealTime === meal.id);
              const maxKcalId = mealFoods.reduce<string | null>((best, food) => {
                const ratio = toNumber(food.amount) / (toNumber(food.servingSize) || 100);
                const kcal = getMacroValue(food.nutrients, 'energy') * ratio;
                if (best === null) return food.id;
                const bestFood = mealFoods.find(f => f.id === best)!;
                const bestRatio = toNumber(bestFood.amount) / (toNumber(bestFood.servingSize) || 100);
                return kcal > getMacroValue(bestFood.nutrients, 'energy') * bestRatio ? food.id : best;
              }, null);
              return mealFoods.map(food => {
                const ratio = toNumber(food.amount) / (toNumber(food.servingSize) || 100);
                const carbs = getMacroValue(food.nutrients, 'carbs') * ratio;
                const protein = getMacroValue(food.nutrients, 'protein') * ratio;
                const fat = getMacroValue(food.nutrients, 'fat') * ratio;
                const macros = [carbs, protein, fat];
                const maxMacro = macros.indexOf(Math.max(...macros));
                const isMaxKcal = food.id === maxKcalId;
                return <tr key={food.id} draggable className={`${dragging?.id === food.id ? 'food-dragging' : ''} ${dropTarget === `food:${food.id}` ? 'food-drop-active' : ''}`} onDragStart={event => startDrag(event, 'food', food.id)} onDragOver={event => { event.preventDefault(); event.stopPropagation(); if (dragging?.type === 'food') setDropTarget(`food:${food.id}`); }} onDrop={event => dropOnFood(event, food.id)}>
                  <td><span className="food-drag-handle" aria-hidden="true">⋮⋮</span>{food.name}</td>
                  <td><input type="number" value={food.amount} onChange={event => onAmount(food.id, toNumber(event.target.value))} /> {food.servingUnit}</td>
                  <td>{isMaxKcal ? <span className="macro-highlight">{(getMacroValue(food.nutrients, 'energy') * ratio).toFixed(0)}</span> : (getMacroValue(food.nutrients, 'energy') * ratio).toFixed(0)}</td>
                  <td>{maxMacro === 0 ? <span className="macro-highlight">{carbs.toFixed(1)}</span> : carbs.toFixed(1)}</td>
                  <td>{maxMacro === 1 ? <span className="macro-highlight">{protein.toFixed(1)}</span> : protein.toFixed(1)}</td>
                  <td>{maxMacro === 2 ? <span className="macro-highlight">{fat.toFixed(1)}</span> : fat.toFixed(1)}</td>
                  <td><button className="danger" aria-label={`Hapus ${food.name}`} onClick={() => onRemoveFood(food.id)}>×</button></td>
                </tr>;
              });
            })()}
          {dropTarget === `meal:${meal.id}` && <tr className="drop-hint"><td colSpan={7}>Lepaskan di sini untuk memindahkan ke {meal.label}</td></tr>}
        </tbody>)}
      </table></div>
      <aside className="analysis-panel"><div className="card"><h3>Makronutrien</h3>{[['Energi', totals.energy, targets.kcal, 'kcal'], ['Karbohidrat', totals.carbs, targets.carbs, 'g'], ['Protein', totals.protein, targets.protein, 'g'], ['Lemak', totals.fat, targets.fat, 'g']].map(([label, value, target, unit]) => <div className="metric" key={label as string}><div><span>{label}</span><b>{(value as number).toFixed(unit === 'kcal' ? 0 : 1)}{target ? ` / ${(target as number).toFixed(1)}` : ''} {unit}</b></div><div className="progress"><i style={{ width: progress(value as number, target as number), background: progressColor(value as number, target as number) }} /></div></div>)}</div><div className="card"><h3>Mikronutrien</h3><p className="muted">Data mikronutrien ditampilkan berdasarkan makanan yang dipilih.</p></div></aside>
    </div>
  </section>;
}
