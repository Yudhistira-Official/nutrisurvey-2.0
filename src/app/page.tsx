'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import Navigation, { type Section } from '../components/Navigation';
import Dashboard from '../components/Dashboard';
import FoodSearch from '../components/FoodSearch';
import Recommendations from '../components/Recommendations';
import TdeeCalculator from '../components/TdeeCalculator';
import AiMealPlanner from '../components/AiMealPlanner';
import { exportToWord, getFoodStatus, getNutrientList, importCsvFromPath } from '../lib/commands';
import { projectFile } from '../lib/project';
import { defaultMeals, defaultTargets, implementAiRows, moveFoodToMeal, type AiMealRow, type FoodResult, type MealTime, type NutrientSummary, type SessionFood, type Targets } from '../lib/types';

const id = () => `${Date.now()}-${Math.random()}`;
const csvFilter = [{ name: 'CSV database', extensions: ['csv'] }];

export default function Home() {
  const [section, setSection] = useState<Section>('dashboard');
  const [foods, setFoods] = useState<SessionFood[]>([]);
  const [meals, setMeals] = useState<MealTime[]>(defaultMeals);
  const [targets, setTargets] = useState<Targets>(defaultTargets);
  const [nutrients, setNutrients] = useState<NutrientSummary[]>([]);
  const [ready, setReady] = useState(false);
  const [searchMeal, setSearchMeal] = useState<string | null>(null);
  const [message, setMessage] = useState('');
  const [toastVisible, setToastVisible] = useState(false);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const toastExitTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [newMeal, setNewMeal] = useState(false);
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
    if (result.added.length) { setFoods(current => [...current, ...result.added]); setSection('dashboard'); setMessage(`${result.added.length} item AI ditambahkan`); }
    if (result.unmatched.length) setMessage(`Kategori AI tidak cocok: ${result.unmatched.join(', ')}`);
  };
  const reportRequest = useMemo(() => ({ foods: foods.map(food => ({ mealTime: food.mealTime, name: food.name, amount: food.amount, servingSize: food.servingSize, servingUnit: food.servingUnit, nutrients: food.nutrients })), mealTimes: meals, targets }), [foods, meals, targets]);

  const saveProject = async () => {
    try {
      if (!await projectFile.save({ foods, meals, targets })) { setMessage('Penyimpanan dibatalkan'); return; }
      setMessage('Proyek berhasil disimpan');
    } catch (error) { notifyError(error, 'Gagal menyimpan proyek'); }
  };
  const openProject = async () => {
    try {
      const project = await projectFile.open();
      if (!project) { setMessage('Pembukaan proyek dibatalkan'); return; }
      setFoods(project.foods); setMeals(project.meals); setTargets(project.targets); setSection('dashboard'); setMessage('Proyek berhasil diimpor');
    } catch (error) { notifyError(error, 'Gagal membuka proyek'); }
  };
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

  return <div className="app-shell"><Navigation section={section} onSection={setSection} onSave={saveProject} onOpen={openProject} onImportCsv={openCsv} onReport={exportReport} /><main className="main-content"><header className="top-bar"><span>Nutrition workspace · {ready ? 'Database siap' : 'Menyiapkan database'}</span></header>{message && <div className={`toast ${toastVisible ? 'toast-visible' : 'toast-exiting'}`} role="status" onClick={() => setToastVisible(false)}>{message}</div>}{section === 'dashboard' && <Dashboard foods={foods} meals={meals} targets={targets} onAddMeal={() => setNewMeal(true)} onAddFood={setSearchMeal} onMoveFood={(foodId, mealId) => setFoods(current => moveFoodToMeal(current, foodId, mealId))} onRemoveFood={foodId => setFoods(current => current.filter(food => food.id !== foodId))} onAmount={(foodId, amount) => setFoods(current => current.map(food => food.id === foodId ? { ...food, amount } : food))} onRemoveMeal={mealId => { if (meals.length <= 1) return setMessage('Minimal harus ada 1 waktu makan'); const fallback = meals.find(meal => meal.id !== mealId); if (!fallback) return; setMeals(current => current.filter(meal => meal.id !== mealId)); setFoods(current => current.map(food => food.mealTime === mealId ? { ...food, mealTime: fallback.id } : food)); }} />}{section === 'recommendations' && <Recommendations nutrients={nutrients} onAdd={food => { addFood(food, food.servingSize || 100, meals[0].id); setSection('dashboard'); }} />}{section === 'tdee' && <TdeeCalculator onTargets={setTargets} onApplied={setMessage} />}{section === 'ai' && <AiMealPlanner meals={meals} targets={targets} onImplement={implementAi} />}<FoodSearch open={searchMeal !== null} onClose={() => setSearchMeal(null)} onSelect={(food, amount) => addFood(food, amount, searchMeal || meals[0].id)} />{newMeal && <div className="modal-backdrop"><div className="modal card"><h3>Tambah Waktu Makan</h3><input ref={newMealRef} placeholder="Contoh: Snack Sore" /><button className="primary full" onClick={() => { const label = newMealRef.current?.value.trim(); if (label) setMeals(current => [...current, { id: `MEAL_${id()}`, label }]); setNewMeal(false); }}>Tambah</button></div></div>}</main></div>;
}
