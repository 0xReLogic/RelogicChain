# Implementasi Blockchain Sederhana di Rust

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

Proyek ini adalah implementasi blockchain dasar yang ditulis dalam bahasa pemrograman Rust. Proyek ini mencakup fitur-fitur inti dari teknologi blockchain, termasuk struktur data fundamental, mekanisme konsensus Proof of Work (PoW), dan model ekonomi sederhana.

## Fitur Utama

- **Struktur Data Inti**: Implementasi struct `Block` dan `Transaction` yang menjadi dasar dari setiap blockchain.
- **Hashing Kriptografis**: Menggunakan SHA-256 untuk menjamin integritas dan keamanan data blok dan transaksi.
- **Merkle Tree**: Setiap blok berisi `merkle_root` yang dihitung dari semua transaksi di dalamnya. Ini memungkinkan verifikasi transaksi yang efisien dan aman.
- **Proof of Work (PoW)**: Sistem mining yang robust untuk mencapai konsensus dan menambahkan blok baru ke dalam rantai.
- **Penyesuaian Kesulitan Dinamis**: Kesulitan (difficulty) mining disesuaikan secara otomatis setiap 10 blok untuk mempertahankan waktu pembuatan blok target sekitar 10 detik.
- **Ekonomi & Hadiah Mining**: 
    - Penambang diberi hadiah untuk setiap blok yang berhasil ditambang melalui **transaksi coinbase**.
    - Implementasi **Reward Halving** yang mengurangi hadiah mining sebesar 50% setiap 20 blok (interval dipercepat untuk tujuan demonstrasi).
- **Mining Paralel**: Proses mining dioptimalkan menggunakan threading paralel dengan crate `rayon` untuk mempercepat pencarian nonce secara signifikan.
- **Antarmuka CLI**: Dilengkapi dengan antarmuka baris perintah (CLI) sederhana untuk menjalankan simulasi mining dan melihat status blockchain secara real-time.
- **Umpan Balik Visual**: Menggunakan crate `indicatif` untuk menampilkan progress bar dan statistik hashing (H/s) selama proses mining.
- **Penanganan Interupsi**: Proses mining dapat dihentikan dengan aman kapan saja menggunakan `Ctrl+C`.

## Prasyarat

Untuk dapat mengompilasi dan menjalankan proyek ini, Anda memerlukan:

- **Rust**: Pastikan Anda telah menginstal Rust dan Cargo (manajer paketnya). Anda dapat menginstalnya melalui [rustup](https://rustup.rs/).

## Cara Menjalankan

1.  **Clone atau Unduh Proyek**: Dapatkan semua berkas proyek ke komputer lokal Anda.
2.  **Buka Terminal**: Navigasikan ke direktori utama proyek (`blockchain`).
3.  **Jalankan Proyek**: Gunakan Cargo untuk mengompilasi dan menjalankan simulasi mining.

    ```bash
    cargo run --release
    ```

    *Disarankan menggunakan flag `--release` untuk mendapatkan performa mining yang jauh lebih cepat.*

4.  **Amati Simulasi**: Program akan mulai menambang blok baru secara terus-menerus. Anda akan melihat informasi tentang setiap blok yang ditambang, kesulitan saat ini, dan total pasokan koin.

5.  **Hentikan Simulasi**: Tekan `Ctrl+C` di terminal untuk menghentikan proses mining dengan aman.

## Kontak

Jika Anda memiliki pertanyaan atau ingin berdiskusi lebih lanjut, jangan ragu untuk menghubungi saya.

- **Email**: hi@0xrelogic.my.id
- **Telegram**: @relogic
- **WhatsApp**: +65 9095 7469
