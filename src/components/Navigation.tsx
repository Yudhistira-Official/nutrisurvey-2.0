'use client';

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { checkUpdate, installUpdate } from '../lib/commands';
import { APP_VERSION, t } from '../lib/app-config';

export type Section = 'dashboard' | 'ai' | 'recommendations' | 'tdee';

type UpdateStage = 'download' | 'install' | 'restart';

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
  const [stage, setStage] = useState<UpdateStage>('download');
  const [percent, setPercent] = useState<number | null>(null);
  const [updateError, setUpdateError] = useState(false);

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

  // Progres unduhan dikirim backend lewat event updater://progress.
  useEffect(() => {
    if (!installing) return;
    const unlisten = listen<{ downloaded?: number; total?: number | null; installed?: boolean }>(
      'updater://progress',
      event => {
        const payload = event.payload;
        if (payload.installed) {
          setStage('restart');
          setPercent(100);
          return;
        }
        setStage('download');
        if (typeof payload.total === 'number' && payload.total > 0) {
          setPercent(Math.min(99, Math.round(((payload.downloaded ?? 0) / payload.total) * 100)));
        }
      },
    );
    return () => { void unlisten.then(stop => stop()); };
  }, [installing]);

  const doInstall = async () => {
    if (installing) return;
    setInstalling(true);
    setUpdateError(false);
    setStage('download');
    setPercent(0);
    try {
      await installUpdate();
      // Backend menutup & membuka ulang aplikasi; bila tetap kembali, tampilkan status akhir.
      setStage('restart');
      setPercent(100);
    } catch {
      setUpdateError(true);
    }
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
    {installing && <div className="update-overlay" role="alertdialog" aria-label={t('updater.title')}>
      <div className="update-overlay-card card">
        <h3>{t('updater.title')}</h3>
        {updateError
          ? <><p className="error">{t('updater.failed')}</p><button className="primary" onClick={() => { setInstalling(false); setUpdateError(false); }}>{t('export.cancel')}</button></>
          : <>
            <p>{stage === 'download' ? t('updater.downloading') : stage === 'install' ? t('updater.installing') : t('updater.restarting')}</p>
            {stage === 'download' && percent !== null && <div className="progress"><i style={{ width: `${percent}%` }} /></div>}
            {stage === 'download' && percent !== null && <b className="update-percent">{percent}%</b>}
            <p className="muted">{stage === 'restart' ? t('updater.restarting') : t('updater.pleaseWait')}</p>
          </>}
      </div>
    </div>}
  </nav>;
}
