'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { searchFoodsByName } from '../lib/commands';
import { classifyUiError } from '../lib/types';
import type { FoodResult } from '../lib/types';

export default function FoodSearch({ open, onClose, onSelect }: { open: boolean; onClose: () => void; onSelect: (food: FoodResult, amount: number) => void }) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<FoodResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [selectedFoodId, setSelectedFoodId] = useState<number | null>(null);
  const [grams, setGrams] = useState(100);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const requestRef = useRef(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) { setQuery(''); setResults([]); setError(''); setSelectedFoodId(null); setGrams(100); }
  }, [open]);

  const doSearch = useCallback(async (term: string) => {
    const normalizedTerm = term.trim();
    const requestId = ++requestRef.current;
    if (normalizedTerm.length < 1) { setResults([]); setError(''); setLoading(false); return; }
    setLoading(true); setError('');
    try {
      const data = await searchFoodsByName(normalizedTerm);
      if (requestId === requestRef.current) setResults(data);
    } catch (cause) {
      if (requestId === requestRef.current) { setResults([]); setError(classifyUiError(cause, 'Search')); }
    } finally {
      if (requestId === requestRef.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => { doSearch(query); }, 200);
    return () => { clearTimeout(debounceRef.current); };
  }, [open, query, doSearch]);

  if (!open) return null;

  const selectedFood = results.find(food => food.id === selectedFoodId) ?? null;
  const add = (food: FoodResult) => {
    onSelect(food, grams);
    setSelectedFoodId(null);
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && results.length > 0) {
      event.preventDefault();
      setSelectedFoodId(results[0].id);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal card" onClick={event => event.stopPropagation()}>
        <div className="section-header">
          <h3>Cari Makanan</h3>
          <button onClick={onClose}>×</button>
        </div>
        <div className="food-search-inputs"><label className="food-name-field"><span>Nama makanan</span><input ref={inputRef} autoFocus value={query} onChange={event => setQuery(event.target.value)} onKeyDown={handleKeyDown} placeholder="Ketik nama makanan..." /></label><label className="food-gram-field"><span>Berat</span><div><input type="number" className="gram-input" value={grams} min={1} onChange={event => setGrams(Number(event.target.value) || 100)} /><small>gram</small></div></label></div>
        {loading && <p aria-live="polite">Mencari...</p>}
        {!loading && !error && query.trim().length < 1 && <p>Masukkan minimal 1 karakter untuk mencari.</p>}
        {!loading && !error && query.trim().length >= 1 && results.length === 0 && <p>Tidak ditemukan. Coba kata kunci lain.</p>}
        {error && <p className="error" role="alert">{error}</p>}
        <div className="search-results">
          {results.map(food => (
            <div className={`search-result-row${selectedFoodId === food.id ? ' selected' : ''}`} key={food.id} role="button" tabIndex={0} onClick={() => setSelectedFoodId(food.id)} onKeyDown={event => { if (event.key === 'Enter' || event.key === ' ') setSelectedFoodId(food.id); }} aria-pressed={selectedFoodId === food.id}>
              <span className="result"><span><b>{food.name}</b><small>{food.category || 'Umum'}</small></span><strong>{food.nutrients.energi ?? food.nutrients.energy ?? 0} kcal</strong></span>
            </div>
          ))}
        </div>
        <button className="primary full" disabled={!selectedFood} onClick={() => { if (selectedFood) add(selectedFood); }}>+ Tambah</button>
        <button className="full" onClick={onClose}>Tutup</button>
      </div>
    </div>
  );
}
