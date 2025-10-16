# ASM Lab Server

Backend berbasis [Axum](https://github.com/tokio-rs/axum) untuk mengelola kelas, akun, dan proxy eksekusi kode ke layanan Judge0 pada platform ASM Lab.

## Persyaratan
- [Rust](https://www.rust-lang.org/tools/install) (disarankan versi stable terbaru)
- Cargo (terpasang bersama Rust)
- SQLite (opsional jika menggunakan basis data lain yang didukung SQLx)
- Akses ke instance [Judge0](https://judge0.com/) jika ingin mengeksekusi kode sungguhan

## Konfigurasi Environment
1. Salin file contoh environment:
   ```bash
   cp .env.example .env
   ```
2. Sesuaikan nilai variabel di dalam `.env` dengan kebutuhan Anda:
   - `DATABASE_URL`: string koneksi ke basis data (default menggunakan SQLite lokal).
   - `JUDGE0_BASE_URL`: URL basis instance Judge0.
   - `SERVER_ADDR`: alamat dan port tempat server akan dijalankan.
   - `RUST_LOG`: (opsional) level log untuk [tracing-subscriber](https://docs.rs/tracing-subscriber).

## Menjalankan Server
```bash
cargo run
```
Secara default server akan berjalan pada `http://0.0.0.0:3000`.

## Endpoint API & Dokumentasi
- **Swagger UI** dapat diakses setelah server berjalan pada: `http://localhost:3000/docs`
- **OpenAPI JSON** tersedia pada: `http://localhost:3000/api-doc/openapi.json`

Router API utama tersedia pada prefix `/api`. Silakan merujuk ke dokumentasi Swagger untuk detail setiap endpoint (pengelolaan kelas, akun, autentikasi, dan proxy eksekusi kode).

## Pengembangan
- Jalankan format kode (opsional) dengan `cargo fmt`
- Jalankan pengujian dengan `cargo test`

## Lisensi
Proyek ini mengikuti lisensi yang tercantum dalam repositori asli.
