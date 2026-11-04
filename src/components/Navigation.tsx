'use client';

import { useEffect, useState } from 'react';
import { checkUpdate, installUpdate } from '../lib/commands';
import { APP_VERSION, t } from '../lib/app-config';

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
    ['dashboard', t('nav.dashboard'), '⌂'],
    ['ai', t('nav.ai'), '✦'],
    ['recommendations', t('nav.recommendations'), '◇'],
    ['tdee', t('nav.tdee'), '◌'],
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
    <div className="sidebar-brand"><div className="brand-mark">N</div><div><h1>{t('app.name')} <span>{t('app.suffix')}</span></h1><p>{t('app.subtitle')}</p></div></div>
    {updateVersion && <div className="update-banner"><div><b>{t('update.available')}</b><small>v{updateVersion}</small></div><button onClick={() => void doInstall()} disabled={installing}>{installing ? t('update.installing') : t('update.install')}</button></div>}
    <div className="nav-group"><span className="nav-group-label">{t('menu.label')}</span><div className="nav-links">{items.map(([id, label, icon]) => <button className={section === id ? 'active' : ''} key={id} onClick={() => onSection(id)}><span className="nav-icon" aria-hidden="true">{icon}</span><span>{label}</span></button>)}</div></div>
    <div className="nav-group nav-actions"><span className="nav-group-label">{t('workspace.label')}</span>
      <button onClick={onSave}><span className="nav-icon" aria-hidden="true">↓</span><span>{t('action.save')}</span></button>
      <button onClick={onOpen}><span className="nav-icon" aria-hidden="true">↑</span><span>{t('action.open')}</span></button>
      <button onClick={onImportCsv}><span className="nav-icon" aria-hidden="true">＋</span><span>{t('action.importCsv')}</span></button>
      <button onClick={onReport}><span className="nav-icon" aria-hidden="true">▤</span><span>{t('action.export')}</span></button>
    </div>
    <div className="sidebar-footer"><strong>{t('app.name')} {t('app.suffix')}</strong><small>{t('version.label')} {APP_VERSION}</small></div><button className="settings-trigger" aria-label={t('action.settings')} onClick={onSettings}>⚙ <span>{t('action.settings')}</span></button>
  </nav>;
}
