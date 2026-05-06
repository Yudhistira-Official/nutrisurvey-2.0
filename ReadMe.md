# NutriSurvey 2.0

NutriSurvey 2.0 adalah aplikasi analisis nutrisi modern berbasis web yang dirancang untuk menggantikan aplikasi legacy NutriSurvey.de. Aplikasi ini memungkinkan pengguna untuk melacak asupan makanan, menghitung target TDEE (Total Daily Energy Expenditure), dan menganalisis lebih dari 50 jenis mikronutrien secara akurat.

## Struktur Proyek

- **`/Backend`**: Web API menggunakan ASP.NET Core 8.0.
  - **Controllers**: Endpoint untuk pencarian makanan, rekomendasi, dan kalkulasi nutrisi.
  - **Models**: Struktur data untuk Food, Nutrient, dan FoodNutrient.
  - **Services**: Logika import CSV dan kalkulasi TDEE.
  - **Data**: Konteks database SQLite (Entity Framework Core).
- **`/Frontend`**: Single-Page Application (SPA).
  - **`index.html`**: Antarmuka pengguna utama.
  - **`css/style.css`**: Desain responsif dengan Vanilla CSS.
  - **`js/app.js`**: Logika state management dan kalkulasi di sisi klien.
  - **`js/api.js`**: Client library untuk berkomunikasi dengan Backend.
- **`/DatabaseMakanan`**: Folder penyimpanan data CSV nutrisi.
  - `DatabaseNilaiGiziCom.csv`: Data makanan referensi dengan format delimiter `;`.
  - `DatabaseFatSecret.csv`: Data makanan tambahan dengan format delimiter `,`.
- **`/Assets`**: Launcher, installer, dan aset pendukung aplikasi.
  - `NutriSurvey.vbs`: Launcher utama (popup progress, tanpa terminal).
  - `RUN.bat`: Runner backend/frontend.
  - `NutriSurvey.ico` / `logo.png`: Ikon launcher.
  - `template.rtf`: Template laporan Word.

## Fitur Utama

1.  **Pencarian Makanan Cepat**: Mencari dari ribuan database makanan lokal dan internasional.
2.  **Kalkulasi Berbasis Sajian**: Menghitung nutrisi secara dinamis berdasarkan jumlah yang diinput pengguna (mendukung satuan g dan ml).
3.  **Rekomendasi Pintar**: Mencari makanan berdasarkan filter nutrisi tertentu (misal: "makanan dengan protein > 20g").
4.  **Target Nutrisi Kustom**: Menghitung TDEE berdasarkan profil fisik dan aktivitas pengguna.
5.  **Ekspor Laporan**: (In Progress) Mengekspor hasil analisis harian ke format Microsoft Word.

## Cara Menjalankan

1.  **Setup (sekali saja)**:
    Jalankan file `Setup.bat`.
    - Script akan membuat shortcut `NutriSurvey 2.0.lnk` (Desktop + folder project).
    - Script akan cek dependency `.NET 8`; jika belum ada, script install per-user tanpa admin.
    - Script akan menghapus dirinya sendiri setelah setup sukses.
2.  **Run Application**:
    Jalankan `NutriSurvey 2.0.lnk` yang dibuat oleh setup, atau `Assets/NutriSurvey.vbs`.
    - Frontend akan berjalan di: `http://localhost:8080`
    - API Backend akan berjalan di: `http://localhost:5000`
    - Progress launcher berjalan per task: 25% (cek komponen), 50% (backend), 75% (frontend), 100% (membuka browser).

## Database
Database SQLite (`nutrition.db`) dibuat otomatis saat backend pertama kali dijalankan. Saat tabel `Foods` masih kosong, backend akan memindai semua file CSV di folder `DatabaseMakanan` lalu mengimpor datanya.

## Catatan Repository
- File hasil build (`bin/`, `obj/`), database lokal (`*.db`), file shortcut (`*.lnk`), dan data upload runtime (`Backend/Data/MasterDatabases/`) tidak disarankan untuk dipublikasikan ke repository.
