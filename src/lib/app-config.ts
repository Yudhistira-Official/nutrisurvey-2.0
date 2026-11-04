// ============================================================================
// KONFIGURASI APLIKASI — SATU SUMBER
// ----------------------------------------------------------------------------
// Semua variabel global aplikasi (versi, bahasa, teks UI) didefinisikan DI SINI.
// - Ganti APP_VERSION di package.json lalu jalankan `npm run sync:version`
//   (menyinkronkan ke src-tauri/tauri.conf.json dan src-tauri/Cargo.toml).
// - Ganti bahasa cukup ubah LANGUAGE di bawah; semua label yang memakai t()
//   ikut berubah otomatis.
// ============================================================================

import pkg from '../../package.json';

/// Versi aplikasi — diambil dari satu sumber: `version` di package.json.
export const APP_VERSION = pkg.version;

/// Bahasa UI aktif. Pilihan: 'id' (Indonesia) atau 'en' (English).
export const LANGUAGE: Language = 'id';

export type Language = 'id' | 'en';

type StringKey =
  | 'app.name'
  | 'app.suffix'
  | 'app.subtitle'
  | 'menu.label'
  | 'workspace.label'
  | 'nav.dashboard'
  | 'nav.ai'
  | 'nav.recommendations'
  | 'nav.tdee'
  | 'action.save'
  | 'action.open'
  | 'action.importCsv'
  | 'action.export'
  | 'action.settings'
  | 'update.available'
  | 'update.install'
  | 'update.installing'
  | 'export.title'
  | 'export.subtitle'
  | 'export.pdf'
  | 'export.docx'
  | 'export.rtf'
  | 'export.cancel'
  | 'db.ready'
  | 'db.preparing'
  | 'version.label'
  | 'updater.title'
  | 'updater.downloading'
  | 'updater.installing'
  | 'updater.restarting'
  | 'updater.pleaseWait'
  | 'updater.failed';

const STRINGS: Record<StringKey, Record<Language, string>> = {
  'app.name': { id: 'NutriSurvey', en: 'NutriSurvey' },
  'app.suffix': { id: 'Pro', en: 'Pro' },
  'app.subtitle': { id: 'Nutrition workspace', en: 'Nutrition workspace' },
  'menu.label': { id: 'Menu Utama', en: 'Main Menu' },
  'workspace.label': { id: 'Workspace', en: 'Workspace' },
  'nav.dashboard': { id: 'Dashboard', en: 'Dashboard' },
  'nav.ai': { id: 'AI Meal Planner', en: 'AI Meal Planner' },
  'nav.recommendations': { id: 'Rekomendasi', en: 'Recommendations' },
  'nav.tdee': { id: 'Kalkulator TDEE', en: 'TDEE Calculator' },
  'action.save': { id: 'Simpan Proyek', en: 'Save Project' },
  'action.open': { id: 'Buka Proyek', en: 'Open Project' },
  'action.importCsv': { id: 'Impor CSV', en: 'Import CSV' },
  'action.export': { id: 'Ekspor Laporan', en: 'Export Report' },
  'action.settings': { id: 'Pengaturan', en: 'Settings' },
  'update.available': { id: 'Update tersedia', en: 'Update available' },
  'update.install': { id: 'Instal Sekarang', en: 'Install Now' },
  'update.installing': { id: 'Menginstal...', en: 'Installing...' },
  'export.title': { id: 'Ekspor Laporan', en: 'Export Report' },
  'export.subtitle': { id: 'Pilih format berkas laporan.', en: 'Choose the report file format.' },
  'export.pdf': { id: 'Ekspor PDF', en: 'Export PDF' },
  'export.docx': { id: 'Ekspor DOCX', en: 'Export DOCX' },
  'export.rtf': { id: 'Ekspor RTF', en: 'Export RTF' },
  'export.cancel': { id: 'Batal', en: 'Cancel' },
  'db.ready': { id: 'Database siap', en: 'Database ready' },
  'db.preparing': { id: 'Menyiapkan database', en: 'Preparing database' },
  'version.label': { id: 'Versi', en: 'Version' },
  'updater.title': { id: 'Memperbarui aplikasi', en: 'Updating app' },
  'updater.downloading': { id: 'Mengunduh pembaruan...', en: 'Downloading update...' },
  'updater.installing': { id: 'Memasang pembaruan...', en: 'Installing update...' },
  'updater.restarting': { id: 'Aplikasi akan menutup dan terbuka kembali otomatis.', en: 'The app will close and reopen automatically.' },
  'updater.pleaseWait': { id: 'Mohon tunggu, jangan tutup aplikasi.', en: 'Please wait, do not close the app.' },
  'updater.failed': { id: 'Pembaruan gagal. Jalankan ulang aplikasi lalu coba lagi.', en: 'Update failed. Restart the app and try again.' },
};

/// Terjemahkan kunci UI sesuai `LANGUAGE` di atas.
export function t(key: StringKey): string {
  return STRINGS[key][LANGUAGE];
}

/// Nama berkas laporan per format (dipakai label tombol & teks sukses).
export const EXPORT_LABELS: Record<'pdf' | 'docx' | 'rtf', StringKey> = {
  pdf: 'export.pdf',
  docx: 'export.docx',
  rtf: 'export.rtf',
};
