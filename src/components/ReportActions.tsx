'use client';

import { exportToWord } from '../lib/commands';
import type { ExportRequest } from '../lib/types';

export default function ReportActions({ request, onMessage }: { request: ExportRequest; onMessage: (message: string) => void }) {
  return <button className="report-button" onClick={async () => { try { const result = await exportToWord(request); onMessage(result.savedPath ? `Laporan tersimpan: ${result.savedPath}` : 'Laporan berhasil diekspor'); } catch { onMessage('Gagal ekspor Word'); } }}>Ekspor Word</button>;
}
