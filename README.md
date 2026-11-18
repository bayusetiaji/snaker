# SnakeD — Neon Edition (port Rust)

Penulisan ulang **SnakeD** (`D:\exps\cpp\SnakeGames\SnakeD`, C++ + SFML 2.5.1) ke dalam
**Rust + [macroquad](https://github.com/not-fl3/macroquad) 0.4.16**.

Seluruh gameplay, tema neon, HUD, layar menu/pause/game over, partikel, screen shake,
level kecepatan, dan persistensi rekor dipertahankan satu per satu dari versi C++.

## Struktur

```text
snake/
├── Cargo.toml
├── assets/                 # aset disematkan via include_bytes!()
│   ├── Exo-DemiBold.otf
│   ├── pick.wav
│   └── hit.wav
└── src/
    ├── main.rs             # window conf + ikon + loop (init → update → render)
    ├── game.rs             # state machine, input, partikel, audio, hiscore, render
    ├── snake.rs            # gerak, interpolasi, tabrakan, rendering kepala/mata
    ├── segment.rs          # satu ruas tubuh + interpolasi posisi
    ├── fruit.rs            # apel: animasi, masa hidup 9 detik, rendering
    ├── entity.rs           # posisi grid (padanan Entity di C++)
    ├── ui.rs               # rounded rect, panel, glow, gradient, teks, keycap
    └── global.rs           # layout, palet warna, Direction, utilitas, RNG
```

## Build & jalankan

```powershell
cd D:\exps\rust\snake
cargo build --release
.\target\release\snaked_neon.exe
```

Atau:

```powershell
cargo run --release
```

Tidak perlu menyalin DLL/font/WAV ke folder executable — semuanya sudah tertanam di dalam
binary. Satu-satunya file yang ditulis saat runtime adalah `hiscore.dat` di direktori kerja.

## Kontrol

| Tombol | Menu | Playing | Paused | Game Over |
| --- | --- | --- | --- | --- |
| Panah / `WASD` | Mulai + arah | Gerak | — | — |
| `Enter` / `Spasi` | Mulai | — | Resume | Main lagi |
| `P` / `Esc` | Keluar | Pause | Resume | Menu |
| `R` | — | Restart | Restart | Main lagi |
| `M` | — | — | Main menu | Menu |

## Aturan main

- Arena 30 × 30 grid (sel 20 px), ular awal 5 segmen di tengah.
- Makan apel: +1 skor, +1 segmen. Apel berpindah sendiri setelah 9 detik.
- Kecepatan naik tiap 4 skor, maksimal level 8 (190 ms → 99 ms per langkah).
- Tabrakan dinding atau tubuh sendiri → Game Over.
- Rekor disimpan otomatis ke `hiscore.dat`.

## Perbedaan dari versi C++/SFML

| Aspek | SnakeD (C++) | Port Rust |
| --- | --- | --- |
| Library | SFML 2.5.1 (static) | macroquad 0.4.16 (via Cargo) |
| Aset | dibaca dari direktori kerja | `include_bytes!()` di dalam binary |
| Audio | pitch SFX naik sesuai level | pitch tetap (macroquad tidak mendukung pitch) |
| Auto-pause saat kehilangan fokus | ya | tidak ada (macroquad tidak mengekspos event fokus) |
| Grid | `sf::VertexArray` | `draw_line` per garis |
| Subsystem | Console + jendela SFML | jendela saja |

Perilaku lain — termasuk input buffering dua belokan, orientasi ulang tubuh pada input
pertama, pelarangan belokan 180°, dan urutan pelepasan resource SFML yang kini tidak
relevant lagi di Rust — tetap mengikuti implementasi asli.
