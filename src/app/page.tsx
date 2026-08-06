'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import Navigation, { type Section } from '../components/Navigation';
import Dashboard from '../components/Dashboard';
import FoodSearch from '../components/FoodSearch';
import Recommendations from '../components/Recommendations';
import TdeeCalculator from '../components/TdeeCalculator';
import AiMealPlanner from '../components/AiMealPlanner';
import SettingsPanel from '../components/SettingsPanel';
import { exportToWord, getFoodStatus, getNutrientList, importCsvFromPath, openProjectPath } from '../lib/commands';
import { projectFile } from '../lib/project';
import { defaultMeals, defaultTargets, implementAiRows, moveFoodToMeal, moveFoodToMealAtPosition, reorderFoods, reorderItems, type AiMealRow, type FoodResult, type MealTime, type NutrientSummary, type SessionFood, type Targets, type TdeeClinicalContext } from '../lib/types';

const id = () => `${Date.now()}-${Math.random()}`;
const csvFilter = [{ name: 'CSV database', extensions: ['csv'] }];

export default function Home() {
  const [section, setSection] = useState<Section>('dashboard');
  const [foods, setFoods] = useState<SessionFood[]>([]);
  const [meals, setMeals] = useState<MealTime[]>(defaultMeals);
  const [targets, setTargets] = useState<Targets>(defaultTargets);
  const [tdeeContext, setTdeeContext] = useState<TdeeClinicalContext | null>(null);
  const [nutrients, setNutrients] = useState<NutrientSummary[]>([]);
  const [ready, setReady] = useState(false);
  const [searchMeal, setSearchMeal] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [toastVisible, setToastVisible] = useState(false);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const toastExitTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [newMeal, setNewMeal] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const newMealRef = useRef<HTMLInputElement>(null);

  useEffect(() => { getFoodStatus().then(status => setReady(status.isReady)).catch(() => setReady(false)); getNutrientList().then(setNutrients).catch(() => setNutrients([])); }, []);
  useEffect(() => {
    if (!message) return;
    setToastVisible(true);
    if (toastTimer.current) clearTimeout(toastTimer.current);
    if (toastExitTimer.current) clearTimeout(toastExitTimer.current);
    toastTimer.current = setTimeout(() => {
      setToastVisible(false);
      toastExitTimer.current = setTimeout(() => setMessage(''), 250);
    }, 3000);
    return () => { if (toastTimer.current) clearTimeout(toastTimer.current); if (toastExitTimer.current) clearTimeout(toastExitTimer.current); };
  }, [message]);

  const notifyError = (error: unknown, fallback: string) => setMessage(error instanceof Error ? error.message : fallback);
  const addFood = (food: FoodResult, amount: number, mealTime: string) => setFoods(current => [...current, { ...food, id: id(), amount, mealTime }]);
  const implementAi = (rows: AiMealRow[]) => {
    const result = implementAiRows(rows, meals);
    setFoods(result.added);
    setSection('dashboard');
    if (result.unmatched.length) {
      setMessage(`${result.added.length} item AI diterapkan. Kategori tidak cocok: ${result.unmatched.join(', ')}`);
    } else {
      setMessage(`${result.added.length} item AI diterapkan ke Dashboard`);
    }
  };
  const reportRequest = useMemo(() => ({ foods: foods.map(food => ({ mealTime: food.mealTime, name: food.name, amount: food.amount, servingSize: food.servingSize, servingUnit: food.servingUnit, nutrients: food.nutrients })), mealTimes: meals, targets }), [foods, meals, targets]);

  const saveProject = async () => {
    try {
      if (!await projectFile.save({ foods, meals, targets })) { setMessage('Penyimpanan dibatalkan'); return; }
      setMessage('Proyek berhasil disimpan');
    } catch (error) { notifyError(error, 'Gagal menyimpan proyek'); }
  };
  const applyProject = (project: NonNullable<Awaited<ReturnType<typeof projectFile.open>>>) => {
    setFoods(project.foods); setMeals(project.meals); setTargets(project.targets); setTdeeContext(null); setSection('dashboard'); setSettingsOpen(false); setMessage('Proyek berhasil dibuka');
  };
  const requestOpenProject = (loader: () => Promise<NonNullable<Awaited<ReturnType<typeof projectFile.open>>> | null>) => {
    if (!window.confirm('Buka project ini? Data Dashboard saat ini akan diganti seluruhnya.')) return;
    loader().then(project => { if (project) applyProject(project); else setMessage('Pembukaan proyek dibatalkan'); }).catch(error => notifyError(error, 'Gagal membuka proyek'));
  };
  const openProject = () => requestOpenProject(() => projectFile.open());
  const openSavedProject = (path: string) => requestOpenProject(() => openProjectPath(path));
  const openCsv = async () => {
    try {
      const path = await open({ multiple: false, filters: csvFilter });
      if (!path || Array.isArray(path)) { setMessage('Impor CSV dibatalkan'); return; }
      const count = await importCsvFromPath(path);
      setMessage(`Database berhasil diimpor: ${count} baris`);
      setNutrients(await getNutrientList());
    } catch (error) { notifyError(error, 'Gagal impor database'); }
  };
  const exportReport = async () => {
    try { const result = await exportToWord(reportRequest); setMessage(result.savedPath ? `Laporan tersimpan: ${result.savedPath}` : 'Laporan berhasil diekspor'); }
    catch (error) { notifyError(error, 'Gagal ekspor Word'); }
  };

  return <div className="app-shell"><Navigation section={section} onSection={setSection} onSave={saveProject} onOpen={openProject} onImportCsv={openCsv} onReport={exportReport} onSettings={() => setSettingsOpen(true)} /><main className="main-content"><header className="top-bar"><span>Nutrition workspace · {ready ? 'Database siap' : 'Menyiapkan database'}</span></header>{message && <div className={`toast ${toastVisible ? 'toast-visible' : 'toast-exiting'}`} role="status" onClick={() => setToastVisible(false)}>{message}</div>}{section === 'dashboard' && <Dashboard foods={foods} meals={meals} targets={targets} onAddMeal={() => setNewMeal(true)} onAddFood={setSearchMeal} onMoveFood={(foodId, mealId) => setFoods(current => moveFoodToMeal(current, foodId, mealId))} onReorderMeals={(draggedId, targetId) => setMeals(current => reorderItems(current, draggedId, targetId))} onReorderFoods={(draggedId, targetId) => setFoods(current => { const dragged = current.find(food => food.id === draggedId); const target = current.find(food => food.id === targetId); if (!dragged || !target) return current; return dragged.mealTime === target.mealTime ? reorderFoods(current, draggedId, targetId) : moveFoodToMealAtPosition(current, draggedId, target.mealTime, targetId); })} onRemoveFood={foodId => setFoods(current => current.filter(food => food.id !== foodId))} onAmount={(foodId, amount) => setFoods(current => current.map(food => food.id === foodId ? { ...food, amount } : food))} onRemoveMeal={mealId => { if (meals.length <= 1) return setMessage('Minimal harus ada 1 waktu makan'); const fallback = meals.find(meal => meal.id !== mealId); if (!fallback) return; setMeals(current => current.filter(meal => meal.id !== mealId)); setFoods(current => current.map(food => food.mealTime === mealId ? { ...food, mealTime: fallback.id } : food)); }} />}{section === 'recommendations' && <Recommendations nutrients={nutrients} onAdd={food => { addFood(food, food.servingSize || 100, meals[0].id); setSection('dashboard'); }} />}{section === 'tdee' && <TdeeCalculator onTargets={setTargets} onClinicalContext={setTdeeContext} onApplied={setMessage} />}{section === 'ai' && <AiMealPlanner meals={meals} targets={targets} clinicalContext={tdeeContext} onImplement={implementAi} onNotify={setMessage} />}<FoodSearch open={searchMeal !== null} onClose={() => setSearchMeal(null)} onSelect={(food, amount) => addFood(food, amount, searchMeal || meals[0].id)} />{newMeal && <div className="modal-backdrop"><div className="modal card"><h3>Tambah Waktu Makan</h3><input ref={newMealRef} placeholder="Contoh: Snack Sore" /><button className="primary full" onClick={() => { const label = newMealRef.current?.value.trim(); if (label) setMeals(current => [...current, { id: `MEAL_${id()}`, label }]); setNewMeal(false); }}>Tambah</button></div></div>}<SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} onOpenProjectPath={openSavedProject} /></main></div>;
}
