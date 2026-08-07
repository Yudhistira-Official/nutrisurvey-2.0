'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import { getRecommendations } from '../lib/commands';
import { classifyUiError } from '../lib/types';
import type { FoodResult, NutrientSummary, RecommendationFilter } from '../lib/types';

const FALLBACK_NUTRIENTS: NutrientSummary[] = [
  { name: 'energi', unit: 'kcal', amount: 0 },
  { name: 'protein', unit: 'g', amount: 0 },
  { name: 'lemak total', unit: 'g', amount: 0 },
  { name: 'karbohidrat total', unit: 'g', amount: 0 },
];

const QUICK_PRESETS: { label: string; filters: RecommendationFilter[] }[] = [
  { label: 'Tinggi Protein', filters: [{ nutrient: 'protein', operator: '>', value: 20 }] },
  { label: 'Rendah Lemak', filters: [{ nutrient: 'lemak total', operator: '<', value: 5 }] },
  { label: 'Tinggi Serat', filters: [{ nutrient: 'serat', operator: '>', value: 3 }] },
  { label: 'Rendah Kalori', filters: [{ nutrient: 'energi', operator: '<', value: 100 }] },
  { label: 'Kaya Kalsium', filters: [{ nutrient: 'kalsium', operator: '>', value: 100 }] },
  { label: 'Tinggi Zat Besi', filters: [{ nutrient: 'zat besi', operator: '>', value: 2 }] },
];

const SORT_OPTIONS = [
  { value: 'energi', label: 'Energi' },
  { value: 'protein', label: 'Protein' },
  { value: 'karbohidrat total', label: 'Karbohidrat' },
  { value: 'lemak total', label: 'Lemak' },
  { value: 'name', label: 'Nama' },
];

function getNutrientValue(food: FoodResult, key: string): number {
  const lower = key.toLowerCase();
  for (const [k, v] of Object.entries(food.nutrients)) {
    if (k.toLowerCase().includes(lower)) return Number(v) || 0;
  }
  return 0;
}

function formatValue(value: number, unit: string): string {
  return unit === 'kcal' ? `${Math.round(value)} kcal` : `${value.toFixed(1)}${unit}`;
}

export default function Recommendations({ nutrients, onAdd }: { nutrients: NutrientSummary[]; onAdd: (food: FoodResult) => void }) {
  const loaded = nutrients.length > 0 ? nutrients : FALLBACK_NUTRIENTS;
  const [filters, setFilters] = useState<RecommendationFilter[]>([{ nutrient: 'protein', operator: '>', value: 10 }]);
  const [results, setResults] = useState<FoodResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [activeNutrient, setActiveNutrient] = useState<number | null>(null);
  const [sortBy, setSortBy] = useState('protein');
  const [sortDesc, setSortDesc] = useState(true);
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const nutrientRefs = useRef<Array<HTMLDivElement | null>>([]);

  const update = (index: number, patch: Partial<RecommendationFilter>) =>
    setFilters(current => current.map((f, i) => i === index ? { ...f, ...patch } : f));

  useEffect(() => {
    const close = (e: MouseEvent) => {
      if (!nutrientRefs.current.some(el => el?.contains(e.target as Node))) setActiveNutrient(null);
    };
    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, []);

  const matchingNutrients = (value: string) =>
    loaded.filter(n => n.name.toLowerCase().includes(value.trim().toLowerCase())).slice(0, 8);

  const applyPreset = (preset: typeof QUICK_PRESETS[number]) => {
    setFilters(preset.filters);
    setActivePreset(preset.label);
  };

  const search = async () => {
    setLoading(true);
    setError('');
    setActivePreset(null);
    try {
      setResults(await getRecommendations(filters));
    } catch (cause) {
      setError(classifyUiError(cause, 'Search'));
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') void search();
  };

  const sorted = useMemo(() => {
    if (!results.length) return results;
    return [...results].sort((a, b) => {
      let av = 0, bv = 0;
      if (sortBy === 'name') {
        return sortDesc ? b.name.localeCompare(a.name) : a.name.localeCompare(b.name);
      }
      av = getNutrientValue(a, sortBy);
      bv = getNutrientValue(b, sortBy);
      return sortDesc ? bv - av : av - bv;
    });
  }, [results, sortBy, sortDesc]);

  // compute max values per nutrient across results for bar scaling
  const maxValues = useMemo(() => {
    const keys = ['energi', 'protein', 'karbohidrat total', 'lemak total'];
    const maxes: Record<string, number> = {};
    for (const key of keys) {
      maxes[key] = Math.max(...sorted.map(f => getNutrientValue(f, key)), 1);
    }
    return maxes;
  }, [sorted]);

  const NUTRIENT_BARS = [
    { key: 'energi', label: 'Energi', unit: 'kcal' },
    { key: 'protein', label: 'Protein', unit: 'g' },
    { key: 'karbohidrat total', label: 'Karbohidrat', unit: 'g' },
    { key: 'lemak total', label: 'Lemak', unit: 'g' },
  ];

  return (
    <section className="rec-page">
      {/* LEFT SIDEBAR */}
      <aside className="rec-sidebar">
        <div className="rec-sidebar-header">
          <span className="eyebrow">Pencarian Nutrisi</span>
          <h2>Cari Makanan</h2>
          <p>Temukan makanan berdasarkan kandungan nutrisi</p>
        </div>

        {/* Quick preset chips */}
        <div className="rec-preset-group">
          {QUICK_PRESETS.map(preset => (
            <button
              key={preset.label}
              className={`rec-preset-chip${activePreset === preset.label ? ' active' : ''}`}
              onClick={() => applyPreset(preset)}
            >
              {preset.label}
            </button>
          ))}
        </div>

        {/* Divider */}
        <div className="rec-divider"><span>Filter Kustom</span></div>

        {/* Active filter chips */}
        {filters.length > 0 && (
          <div className="rec-active-filters">
            {filters.map((f, i) => (
              <span key={`${f.nutrient}-${f.operator}-${f.value}-${i}`} className="rec-filter-chip">
                {f.nutrient} {f.operator} {f.value}
                <button
                  aria-label="Hapus filter"
                  onClick={() => setFilters(current => current.filter((_, idx) => idx !== i))}
                >×</button>
              </span>
            ))}
          </div>
        )}

        {/* Filter builder rows */}
        <div className="rec-filter-list">
          {filters.map((filter, index) => {
            const matches = matchingNutrients(filter.nutrient);
            return (
              <div key={`filter-row-${index}`} className="rec-filter-row">
                <div className="rec-nutrient-autocomplete" ref={el => { nutrientRefs.current[index] = el; }}>
                  <input
                    className="rec-input"
                    value={filter.nutrient}
                    placeholder="Nutrisi..."
                    onFocus={() => setActiveNutrient(index)}
                    onChange={e => { update(index, { nutrient: e.target.value }); setActiveNutrient(index); }}
                    onKeyDown={e => {
                      if (e.key === 'Escape') setActiveNutrient(null);
                      if (e.key === 'Enter' && matches[0]) { e.preventDefault(); update(index, { nutrient: matches[0].name }); setActiveNutrient(null); }
                    }}
                  />
                  {activeNutrient === index && matches.length > 0 && (
                    <div className="rec-dropdown">
                      {matches.map((n, ni) => (
                        <button
                          key={`${n.name}-${ni}`}
                          type="button"
                          onClick={() => { update(index, { nutrient: n.name }); setActiveNutrient(null); }}
                        >
                          {n.name}<small>{n.unit}</small>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
                <select
                  className="rec-select-op"
                  value={filter.operator}
                  onChange={e => update(index, { operator: e.target.value as RecommendationFilter['operator'] })}
                >
                  <option value="<">&lt;</option>
                  <option value=">">&gt;</option>
                  <option value="=">=</option>
                </select>
                <input
                  className="rec-input-num"
                  type="number"
                  value={filter.value}
                  onKeyDown={handleKeyDown}
                  onChange={e => update(index, { value: Number(e.target.value) })}
                />
                <button
                  className="rec-remove-btn"
                  aria-label="Hapus filter"
                  onClick={() => setFilters(current => current.filter((_, i) => i !== index))}
                >×</button>
              </div>
            );
          })}
        </div>

        <button
          className="rec-add-filter-btn"
          onClick={() => setFilters(current => [...current, { nutrient: loaded[0].name, operator: '>', value: 0 }])}
        >
          + Tambah Filter
        </button>

        <button
          className="rec-search-btn primary"
          onClick={() => void search()}
          disabled={loading || filters.length === 0}
        >
          {loading ? 'Mencari...' : 'Cari Makanan'}
        </button>
      </aside>

      {/* RIGHT MAIN */}
      <main className="rec-main">
        {/* Results header */}
        <div className="rec-results-header">
          <div>
            <h3 className="rec-results-count">
              {results.length > 0 ? `${sorted.length} Makanan Ditemukan` : 'Rekomendasi Makanan'}
            </h3>
            {error && <p className="rec-error">{error}</p>}
          </div>
          {results.length > 0 && (
            <div className="rec-sort-row">
              <span>Urutkan:</span>
              <select
                className="rec-sort-select"
                value={sortBy}
                onChange={e => setSortBy(e.target.value)}
              >
                {SORT_OPTIONS.map(opt => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
              <button
                className="rec-sort-dir"
                aria-label="Balik urutan"
                onClick={() => setSortDesc(d => !d)}
                title={sortDesc ? 'Terbesar ke terkecil' : 'Terkecil ke terbesar'}
              >
                {sortDesc ? '↓' : '↑'}
              </button>
            </div>
          )}
        </div>

        {/* Empty state */}
        {results.length === 0 && !loading && (
          <div className="rec-empty">
            <div className="rec-empty-icon">🔍</div>
            <h4>Atur filter dan mulai pencarian</h4>
            <p>Pilih preset cepat atau buat filter kustom, lalu tekan Cari Makanan.</p>
          </div>
        )}

        {/* Loading */}
        {loading && (
          <div className="rec-empty">
            <div className="rec-empty-icon">⏳</div>
            <h4>Mencari makanan...</h4>
          </div>
        )}

        {/* Result cards */}
        {!loading && sorted.length > 0 && (
          <div className="rec-results-list">
            {sorted.map(food => (
              <article key={food.id} className="rec-card">
                <div className="rec-card-header">
                  <div>
                    <h4 className="rec-food-name">{food.name}</h4>
                    <p className="rec-food-serving">{food.servingSize}{food.servingUnit} per sajian</p>
                  </div>
                  <button
                    className="rec-add-btn"
                    onClick={() => onAdd(food)}
                  >
                    + Tambah
                  </button>
                </div>
                <div className="rec-nutrient-bars">
                  {NUTRIENT_BARS.map(({ key, label, unit }) => {
                    const val = getNutrientValue(food, key);
                    const pct = Math.min((val / maxValues[key]) * 100, 100);
                    return (
                      <div key={key} className="rec-bar-row">
                        <span className="rec-bar-label">{label}</span>
                        <div className="rec-bar-track">
                          <div className="rec-bar-fill" style={{ width: `${pct}%` }} />
                        </div>
                        <span className="rec-bar-value">{formatValue(val, unit)}</span>
                      </div>
                    );
                  })}
                </div>
              </article>
            ))}
          </div>
        )}
      </main>
    </section>
  );
}
