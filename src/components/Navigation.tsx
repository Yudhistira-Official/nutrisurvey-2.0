'use client';

import { useEffect, useState } from 'react';
import { checkUpdate, installUpdate } from '../lib/commands';

export type Section = 'dashboard' | 'ai' | 'recommendations' | 'tdee';

export default function Navigation({ section, onSection, onSave, onOpen, onImportCsv, onReport, onSettings }: {
  section: Section;
  onSection: (section: Section) => void;
  onSave: () => void;
  onOpen: () => void;
  onImportCsv: () => void;
  onReport: () => void;
  onSettings: () => void;
}) {
  const items: Array<[Section, string, string]> = [
    ['dashboard', 'Dashboard', '⌂'],
    ['ai', 'AI Meal Planner', '✦'],
    ['recommendations', 'Rekomendasi', '◇'],
    ['tdee', 'Kalkulator TDEE', '◌'],
  ];
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    const check = async () => {
      try {
        const version = await checkUpdate();
        if (version) setUpdateVersion(version);
      } catch { }
    };
    const delay = window.setTimeout(check, 3000);
    return () => window.clearTimeout(delay);
  }, []);

  const doInstall = async () => {
    if (installing) return;
    setInstalling(true);
    try { await installUpdate(); } catch { setInstalling(false); }
  };

  return <nav className="sidebar">
    <div className="sidebar-brand"><div className="brand-mark">N</div><div><h1>NutriSurvey <span>Pro</span></h1><p>Nutrition workspace</p></div></div>
    {updateVersion && <div className="update-banner"><div><b>Update tersedia</b><small>v{updateVersion}</small></div><button onClick={() => void doInstall()} disabled={installing}>{installing ? 'Menginstal...' : 'Instal Sekarang'}</button></div>}
    <div className="nav-group"><span className="nav-group-label">Menu Utama</span><div className="nav-links">{items.map(([id, label, icon]) => <button className={section === id ? 'active' : ''} key={id} onClick={() => onSection(id)}><span className="nav-icon" aria-hidden="true">{icon}</span><span>{label}</span></button>)}</div></div>
    <div className="nav-group nav-actions"><span className="nav-group-label">Workspace</span>
      <button onClick={onSave}><span className="nav-icon" aria-hidden="true">↓</span><span>Simpan Proyek</span></button>
      <button onClick={onOpen}><span className="nav-icon" aria-hidden="true">↑</span><span>Buka Proyek</span></button>
      <button onClick={onImportCsv}><span className="nav-icon" aria-hidden="true">＋</span><span>Impor CSV</span></button>
      <button onClick={onReport}><span className="nav-icon" aria-hidden="true">▤</span><span>Word Report</span></button>
    </div>
    <div className="sidebar-footer"><strong>NutriSurvey Pro</strong><small>Versi 2.0.0</small></div><button className="settings-trigger" aria-label="Pengaturan" onClick={onSettings}>⚙ <span>Pengaturan</span></button>
  </nav>;
}
