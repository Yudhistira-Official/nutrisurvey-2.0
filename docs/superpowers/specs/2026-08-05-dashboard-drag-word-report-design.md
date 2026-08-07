# Perbaikan drag makanan dan Word Report

## Tujuan
Memungkinkan makanan dipindahkan antar waktu makan secara konsisten dari Dashboard dan membuat bagian Hasil Perhitungan pada laporan RTF tersusun pada kolom yang benar tanpa tanda slash pada judul diet.

## Desain
- Dashboard memvalidasi tipe dan ID payload drag sebelum memproses drop.
- Drop makanan pada area waktu makan mengubah `mealTime` dan menempatkan makanan di urutan terakhir makanan pada waktu makan tujuan.
- Drop makanan pada baris makanan tetap mengurutkan makanan dalam waktu makan yang sama; drop lintas waktu makan pada baris tujuan memindahkan makanan dan menyisipkannya di posisi baris tersebut.
- Target visual tidak dibersihkan oleh perpindahan pointer antar-elemen anak; target dibersihkan saat drop/end drag.
- Renderer RTF memakai posisi tab header dan baris hasil perhitungan yang identik. Setiap baris mengeluarkan tepat empat kolom: zat gizi, hasil analisis, rekomendasi, dan persentase pemenuhan.
- Judul pertama laporan ditulis `HASIL PERHITUNGAN DIET` tanpa karakter `/`; judul bagian kedua tetap `HASIL PERHITUNGAN`.

## Data flow dan kompatibilitas
State React tetap menjadi sumber kebenaran. Pemindahan makanan hanya mengubah `mealTime` dan urutan array, tanpa mengubah jumlah maupun nilai gizi. Format request export dan marker template tetap kompatibel dengan alur Tauri yang ada.

## Error handling
Drop dengan payload kosong, tipe tidak dikenal, atau ID yang tidak ada tidak mengubah state. Renderer tetap menolak template RTF invalid dan melakukan escaping nilai dinamis seperti sebelumnya.

## Verifikasi
- Unit test memeriksa pemindahan makanan ke waktu makan lain dan penyisipan posisi tujuan.
- Unit test export memastikan judul tidak mengandung `DIET/`, tab stop kolom konsisten, dan rekomendasi/persentase berada di kolom terpisah.
- Jalankan test frontend, typecheck, lint, cargo test export, dan cargo fmt check.
