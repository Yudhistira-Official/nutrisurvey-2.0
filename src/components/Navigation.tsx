'use client';

export type Section = 'dashboard' | 'ai' | 'recommendations' | 'tdee';

export default function Navigation({ section, onSection, onSave, onOpen, onImportCsv, onReport }: {
  section: Section;
  onSection: (section: Section) => void;
  onSave: () => void;
  onOpen: () => void;
  onImportCsv: () => void;
  onReport: () => void;
}) {
  const items: Array<[Section, string, string]> = [
    ['dashboard', 'Dashboard', '⌂'],
    ['ai', 'AI Meal Planner', '✦'],
    ['recommendations', 'Rekomendasi', '◇'],
    ['tdee', 'Kalkulator TDEE', '◌'],
  ];
  return <nav className="sidebar">
    <div className="sidebar-brand"><div className="brand-mark">N</div><div><h1>NutriSurvey <span>Pro</span></h1><p>Nutrition workspace</p></div></div>
    <div className="database-status"><i /> Database siap</div>
    <div className="nav-group"><span className="nav-group-label">Menu Utama</span><div className="nav-links">{items.map(([id, label, icon]) => <button className={section === id ? 'active' : ''} key={id} onClick={() => onSection(id)}><span className="nav-icon" aria-hidden="true">{icon}</span><span>{label}</span></button>)}</div></div>
    <div className="nav-group nav-actions"><span className="nav-group-label">Workspace</span>
      <button onClick={onSave}><span className="nav-icon" aria-hidden="true">↓</span><span>Simpan Proyek</span></button>
      <button onClick={onOpen}><span className="nav-icon" aria-hidden="true">↑</span><span>Buka Proyek</span></button>
      <button onClick={onImportCsv}><span className="nav-icon" aria-hidden="true">＋</span><span>Impor CSV</span></button>
      <button onClick={onReport}><span className="nav-icon" aria-hidden="true">▤</span><span>Word Report</span></button>
    </div>
    <div className="sidebar-footer"><strong>NutriSurvey Pro</strong><small>Versi 0.1.0</small></div>
  </nav>;
}
