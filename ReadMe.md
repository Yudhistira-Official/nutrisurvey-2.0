# NutriSurvey 2.0

NutriSurvey 2.0 adalah aplikasi analisis nutrisi modern berbasis web yang dirancang untuk menggantikan aplikasi legacy NutriSurvey.de. Aplikasi ini memungkinkan pengguna untuk melacak asupan makanan, menghitung target TDEE (Total Daily Energy Expenditure), menganalisis lebih dari 50 jenis mikronutrien, dan membuat rencana makan berbantuan AI yang divalidasi terhadap database makanan lokal.

## Struktur Proyek

- **`/Backend`**: Web API menggunakan ASP.NET Core 8.0.
  - **Controllers**: Endpoint untuk pencarian makanan, rekomendasi, kalkulasi nutrisi, dan AI Meal Planner.
  - **Models**: Struktur data untuk Food, Nutrient, FoodNutrient, dan DTO AI.
  - **Services**: Logika import CSV, kalkulasi TDEE, komunikasi AI, dan pemetaan hasil AI ke database makanan.
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
5.  **AI Meal Planner**: Membuat rencana makan berbasis target TDEE, makro, dan kategori waktu makan dari Dashboard.
6.  **Validasi Database Lokal**: Output AI dipetakan kembali ke database SQLite agar makanan yang tampil berasal dari data lokal, bukan halusinasi model.
7.  **Auto-Scaling Nutrisi**: Porsi hasil AI dinormalisasi otomatis agar total kalori mendekati target TDEE dengan toleransi 5%, dan porsi di bawah 25 g dibuang agar menu tetap realistis.
8.  **Implementasi ke Dashboard**: Rencana AI yang sudah tervalidasi dapat dimasukkan ke Manajemen Menu untuk diedit manual, dihapus, atau ditambah makanan lain.
9.  **Ekspor Laporan**: Mengekspor hasil analisis harian ke format Microsoft Word.

## AI Meal Planner

Fitur AI Meal Planner menggunakan konsep BYOK (Bring Your Own Key). API key hanya dikirim ke backend saat request berjalan dan tidak disimpan di aplikasi.

Provider yang didukung:

- OpenRouter: format OpenAI-compatible `/chat/completions`.
- OpenAI: format OpenAI-compatible `/chat/completions`.
- Google Project: Google Gemini native `:generateContent`.
- Anthropic: Claude native `/messages`.
- Custom Router: router OpenAI-compatible dengan Base URL manual.

Alur kerja AI:

1. Hitung TDEE terlebih dahulu di Kalkulator TDEE.
2. Atur kategori waktu makan di Dashboard, misalnya `Makan Pagi`, `Makan Siang`, dan `Makan Malam`.
3. Buka AI Meal Planner, pilih provider, masukkan model dan API key.
4. Aplikasi mengirim target TDEE, target makro, dan kategori waktu makan saat ini ke AI.
5. Backend memetakan makanan hasil AI ke database SQLite lokal.
6. Backend menormalisasi gramasi agar total kalori mendekati target TDEE.
7. User dapat meninjau hasil, lalu menekan `Implementasikan Plan ke Dashboard` untuk mengedit menu secara manual.

Catatan keamanan:

- Jangan commit API key ke repository.
- Gunakan API key sementara atau terbatas jika memungkinkan.
- Provider custom harus kompatibel dengan schema OpenAI chat completions.

## Cara Menjalankan

1.  **Setup (sekali saja)**:
    Unduh `Setup.exe` dari GitHub Release, lalu jalankan sebagai Administrator.
    - Installer akan mengunduh project terbaru dari GitHub dan memasangnya ke `C:\NutriSurvey2.0`.
    - Jika folder `C:\NutriSurvey2.0` tidak dapat ditulis, installer memakai fallback `%LocalAppData%\NutriSurvey2.0`.
    - Installer akan membuat shortcut `NutriSurvey 2.0.lnk` di Desktop dan folder install.
    - Installer akan cek `.NET SDK 8.0.420` beserta runtime `Microsoft.AspNetCore.App 8.0.26`, `Microsoft.NETCore.App 8.0.26`, dan `Microsoft.WindowsDesktop.App 8.0.26`.
    - Jika dependency belum ada, installer akan mengunduh dan menginstall .NET dengan tampilan progress.
    - Setelah selesai, installer menampilkan popup sukses.
2.  **Run Application**:
    Jalankan `NutriSurvey 2.0.lnk` yang dibuat oleh setup, atau `Assets/NutriSurvey.vbs`.
    - Frontend akan berjalan di: `http://localhost:8080`
    - API Backend akan berjalan di: `http://localhost:5000`
    - Progress launcher berjalan per task: 25% (cek komponen), 50% (backend), 75% (frontend), 100% (membuka browser).

## Database
Database SQLite (`nutrition.db`) dibuat otomatis saat backend pertama kali dijalankan. Saat tabel `Foods` masih kosong, backend akan memindai semua file CSV di folder `DatabaseMakanan` lalu mengimpor datanya di background agar API tetap bisa mulai berjalan. Pada pemakaian pertama, pencarian makanan dapat kosong sementara sampai proses import selesai.

## Catatan Repository
- File hasil build (`bin/`, `obj/`), database lokal (`*.db`), file shortcut (`*.lnk`), dan data upload runtime (`Backend/Data/MasterDatabases/`) tidak disarankan untuk dipublikasikan ke repository.
