'use client';

import { useEffect, useRef, useState } from 'react';
import { deleteAiKey, getAiDefaultInfo, loadAiKey, openExistingFile, saveAiKey, listFileHistory } from '../lib/commands';
import type { AiConfig, FileHistoryItem } from '../lib/types';

export default function SettingsPanel({ open, onClose, onOpenProjectPath }: { open: boolean; onClose: () => void; onOpenProjectPath: (path: string) => void }) {
  const [history, setHistory] = useState<FileHistoryItem[]>([]);
  const [config, setConfig] = useState<AiConfig>({ provider: 'openrouter', model: '', apiKey: '', baseUrl: '' });
  const [defaultAvailable, setDefaultAvailable] = useState(false);
  const [message, setMessage] = useState('');
  const messageTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => () => {
    if (messageTimer.current) clearTimeout(messageTimer.current);
  }, []);

  useEffect(() => {
    if (!open) return;
    listFileHistory().then(setHistory).catch(() => setHistory([]));
    try {
      const saved = localStorage.getItem('nutrisurvey.ai-preferences');
      if (saved) setConfig(current => ({ ...current, ...JSON.parse(saved) }));
    } catch { }
    loadAiKey().then(value => { if (value) setConfig(current => ({ ...current, apiKey: value })); }).catch(() => undefined);
    getAiDefaultInfo().then(info => { setDefaultAvailable(info.available); if (info.available && !localStorage.getItem('nutrisurvey.ai-preferences')) setConfig(current => ({ ...current, provider: 'builtin_default', model: info.model, baseUrl: info.baseUrl })); }).catch(() => undefined);
  }, [open]);

  if (!open) return null;
  const patch = (value: Partial<AiConfig>) => setConfig(current => ({ ...current, ...value }));
  const save = async () => {
    const savedConfig = { provider: config.provider, model: config.model, baseUrl: config.baseUrl };
    localStorage.setItem('nutrisurvey.ai-preferences', JSON.stringify(savedConfig));
    if (config.provider !== 'builtin_default') {
      if (config.apiKey.trim()) await saveAiKey(config.apiKey);
      else await deleteAiKey();
    }
    window.dispatchEvent(new CustomEvent('nutrisurvey.ai-config-updated', { detail: { ...config, apiKey: config.provider === 'builtin_default' ? '' : config.apiKey } }));
    setMessage('Konfigurasi Tersimpan');
    if (messageTimer.current) clearTimeout(messageTimer.current);
    messageTimer.current = setTimeout(() => setMessage(''), 5000);
  };
  const projects = history.filter(item => item.kind === 'project');
  const reports = history.filter(item => item.kind === 'report');
  const fileName = (path: string) => path.split(/[\\/]/).pop() || path;
  const openFile = async (item: FileHistoryItem) => { try { if (item.kind === 'project') { onOpenProjectPath(item.path); return; } await openExistingFile(item.path); } catch { setMessage('File tidak dapat dibuka'); } };
  const list = (items: FileHistoryItem[]) => items.length ? items.map(item => <button className="history-item" key={`${item.kind}:${item.path}`} onClick={() => openFile(item)}><span><b>{fileName(item.path)}</b><small>{item.path}</small></span><strong>Buka</strong></button>) : <p className="muted">Belum ada file tersimpan.</p>;

  return <section className="settings-page"><div className="settings-page-header"><div><span className="eyebrow">Workspace</span><h2>Pengaturan</h2><p>Kelola konfigurasi AI dan akses file tersimpan.</p></div><button className="settings-back" onClick={onClose}>← Kembali</button></div><div className="settings-page-grid"><div className="card settings-modal"><div className="section-header"><h3>Konfigurasi AI</h3></div><div className="settings-section settings-section-first"><label>Provider<select value={config.provider} onChange={event => patch({ provider: event.target.value })}>{defaultAvailable && <option value="builtin_default">AI Default</option>}<option value="openrouter">OpenRouter</option><option value="openai">OpenAI</option><option value="google">Google</option><option value="anthropic">Anthropic</option><option value="custom">Custom Router</option></select></label>{config.provider === 'builtin_default' ? null : <>{config.provider === 'custom' && <label>Base URL<input type="url" value={config.baseUrl} onChange={event => patch({ baseUrl: event.target.value })} /></label>}<label>Model<input value={config.model} onChange={event => patch({ model: event.target.value })} /></label><label>API Key<input type="password" value={config.apiKey} onChange={event => patch({ apiKey: event.target.value })} /></label></>}<div className="settings-actions"><button className="primary" onClick={() => save().catch(() => setMessage('Gagal menyimpan konfigurasi'))}>Simpan Konfigurasi</button></div>{message && <p className="success" role="status">{message}</p>}</div></div><div className="card settings-modal"><div className="section-header"><h3>Riwayat Proyek</h3></div><div className="settings-section settings-section-first">{list(projects)}</div><div className="settings-section"><h4>Word Report</h4>{list(reports)}</div></div></div></section>;
}
