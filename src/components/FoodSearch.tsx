'use client';

import { useEffect, useState } from 'react';
import { searchFoodsByName } from '../lib/commands';
import type { FoodResult } from '../lib/types';

export default function FoodSearch({ open, onClose, onSelect }: { open: boolean; onClose: () => void; onSelect: (food: FoodResult) => void }) {
  const [query, setQuery] = useState(''); const [results, setResults] = useState<FoodResult[]>([]); const [loading, setLoading] = useState(false);
  useEffect(() => { if (!open) { setQuery(''); setResults([]); return; } const timer = setTimeout(async () => { if (query.trim().length < 2) return setResults([]); setLoading(true); try { setResults(await searchFoodsByName(query)); } finally { setLoading(false); } }, 300); return () => clearTimeout(timer); }, [open, query]);
  if (!open) return null;
  return <div className="modal-backdrop" onClick={onClose}><div className="modal card" onClick={event => event.stopPropagation()}><div className="section-header"><h3>Cari Makanan</h3><button onClick={onClose}>×</button></div><input autoFocus value={query} onChange={event => setQuery(event.target.value)} placeholder="Ketik nama makanan..." />{loading && <p>Mencari...</p>}<div className="search-results">{results.map(food => <button className="result" key={food.id} onClick={() => { onSelect(food); onClose(); }}><span><b>{food.name}</b><small>{food.category || 'Umum'}</small></span><strong>{food.nutrients.energi ?? food.nutrients.energy ?? 0} kcal</strong></button>)}</div></div></div>;
}
