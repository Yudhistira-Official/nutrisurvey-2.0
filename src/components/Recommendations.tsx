'use client';

import { useEffect, useRef, useState } from 'react';
import { getRecommendations } from '../lib/commands';
import { classifyUiError } from '../lib/types';
import type { FoodResult, NutrientSummary, RecommendationFilter } from '../lib/types';

const FALLBACK_NUTRIENTS: NutrientSummary[] = [
  { name: 'energi', unit: 'kcal', amount: 0 },
  { name: 'protein', unit: 'g', amount: 0 },
  { name: 'lemak total', unit: 'g', amount: 0 },
  { name: 'karbohidrat total', unit: 'g', amount: 0 },
];

export default function Recommendations({ nutrients, onAdd }: { nutrients: NutrientSummary[]; onAdd: (food: FoodResult) => void }) {
  const loaded = nutrients.length > 0 ? nutrients : FALLBACK_NUTRIENTS;
  const [filters, setFilters] = useState<RecommendationFilter[]>([{ nutrient: loaded[0].name, operator: '>', value: 0 }]); const [results, setResults] = useState<FoodResult[]>([]); const [loading, setLoading] = useState(false); const [error, setError] = useState('');
  const [activeNutrient, setActiveNutrient] = useState<number | null>(null);
  const nutrientRefs = useRef<Array<HTMLDivElement | null>>([]);
  const update = (index: number, patch: Partial<RecommendationFilter>) => setFilters(current => current.map((filter, item) => item === index ? { ...filter, ...patch } : filter));
  useEffect(() => {
    const close = (event: MouseEvent) => { if (!nutrientRefs.current.some(element => element?.contains(event.target as Node))) setActiveNutrient(null); };
    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, []);
  const matchingNutrients = (value: string) => loaded.filter(nutrient => nutrient.name.toLowerCase().includes(value.trim().toLowerCase())).slice(0, 8);
  const search = async () => { setLoading(true); setError(''); try { setResults(await getRecommendations(filters)); } catch (cause) { setError(classifyUiError(cause, 'Search')); } finally { setLoading(false); } };
  return <section><div className="section-header"><div><h2>Rekomendasi Makanan</h2><p>Cari makanan berdasarkan batas nutrisi.</p></div></div><div className="dashboard-grid"><div className="card"><h3>Filter Nutrisi</h3>{filters.map((filter, index) => { const matches = matchingNutrients(filter.nutrient); return <div className="filter-row" key={index}><div className="nutrient-autocomplete" ref={element => { nutrientRefs.current[index] = element; }}><input className="filter-control nutrient-input" value={filter.nutrient} placeholder="Cari nutrisi..." onFocus={() => setActiveNutrient(index)} onChange={event => { update(index, { nutrient: event.target.value }); setActiveNutrient(index); }} onKeyDown={event => { if (event.key === 'Escape') setActiveNutrient(null); if (event.key === 'Enter' && matches[0]) { event.preventDefault(); update(index, { nutrient: matches[0].name }); setActiveNutrient(null); } }} />{activeNutrient === index && matches.length > 0 && <div className="nutrient-dropdown">{matches.map(nutrient => <button type="button" key={nutrient.name} onClick={() => { update(index, { nutrient: nutrient.name }); setActiveNutrient(null); }}>{nutrient.name}<small>{nutrient.unit}</small></button>)}</div>}</div><select className="filter-control filter-operator" value={filter.operator} onChange={event => update(index, { operator: event.target.value as RecommendationFilter['operator'] })}><option value="<">&lt;</option><option value=">">&gt;</option><option value="=">=</option></select><input className="filter-control filter-value" type="number" value={filter.value} onChange={event => update(index, { value: Number(event.target.value) })} /><button className="remove-filter" aria-label="Hapus filter" onClick={() => setFilters(current => current.filter((_, item) => item !== index))}>×</button></div>; })}<button className="add-filter" onClick={() => setFilters(current => [...current, { nutrient: loaded[0].name, operator: '>', value: 0 }])}><span aria-hidden="true">+</span> Filter</button>{error && <p className="error" role="alert">{error}</p>}<button className="primary full" disabled={loading} onClick={search}>{loading ? 'Mencari...' : 'Cari Rekomendasi'}</button></div><div className="card"><h3>Hasil Rekomendasi</h3>{results.length === 0 && !loading && <p className="muted">Belum ada hasil.</p>}{results.slice(0, 20).map(food => <div className="recommendation" key={food.id}><span><b>{food.name}</b><small>{food.category || 'Umum'}</small></span><span className="recommendation-nutrients">{filters.map(filter => <small key={filter.nutrient}>{filter.nutrient}: {food.nutrients[filter.nutrient] ?? '-'}</small>)}</span><button onClick={() => onAdd(food)}>+ Tambah</button></div>)}</div></div></section>;
}
