'use client';

export type Section = 'dashboard' | 'ai' | 'recommendations' | 'tdee';

export default function Navigation({ section, onSection, onSave, onOpen, onReport }: {
  section: Section;
  onSection: (section: Section) => void;
  onSave: () => void;
  onOpen: () => void;
  onReport: () => void;
}) {
  const items: Array<[Section, string]> = [
    ['dashboard', 'Dashboard'],
    ['ai', 'AI Meal Planner'],
    ['recommendations', 'Rekomendasi'],
    ['tdee', 'Kalkulator TDEE'],
  ];
  return <nav className="sidebar">
    <h1>NutriSurvey <span>Pro</span></h1>
    <div className="nav-links">{items.map(([id, label]) => <button className={section === id ? 'active' : ''} key={id} onClick={() => onSection(id)}>{label}</button>)}</div>
    <div className="nav-actions">
      <button onClick={onSave}>Simpan Proyek</button>
      <button onClick={onOpen}>Buka Proyek</button>
      <button onClick={onReport}>Word Report</button>
    </div>
  </nav>;
}
