'use client';

import { useState } from 'react';
import { exportToWord } from '../lib/commands';
import { classifyUiError } from '../lib/types';
import type { ExportRequest } from '../lib/types';

export default function ReportActions({ request, onMessage }: { request: ExportRequest; onMessage: (message: string) => void }) {
  const [loading, setLoading] = useState(false);
  const run = async () => { setLoading(true); try { const result = await exportToWord(request); onMessage(result.savedPath ? `Laporan tersimpan: ${result.savedPath}` : 'Laporan selesai'); } catch (cause) { onMessage(classifyUiError(cause, 'Export')); } finally { setLoading(false); } };
  return <button className="report-button" disabled={loading} onClick={run}>{loading ? 'Mengekspor...' : 'Ekspor Word'}</button>;
}
