# Performance results

Budgets: `docs/REQUIREMENTS.md` §6.1. Each section is one measurement session; newest first.

---

## 2026-10-23 · T-010 · Benchmark harness and first baseline

**Verdict: every measured metric is within budget.** This is the first baseline (`bench/baseline.json`).
Six §6.1 budgets now have a metric; four (scroll fps, sharp tile, find, incremental save) cannot be measured
yet because the features do not exist. The first-page numbers are **core only** (PDFium open + render, no IPC
or UI paint). The idle-memory number is the **empty window** (the app cannot open a PDF before T-018).

### Harness

| Part | File | What it measures |
|---|---|---|
| Budgets | `bench/budgets.json` | All 11 §6.1 budgets (limit, unit, `max`/`min`), the result keys that check each one, and what each metric covers today |
| Criterion benches | `bench/benches/first_page.rs` (crate `swiftpdf-bench`) | `open_render_10mb`, `open_render_300mb`: open + render page 1 at 1280 px + close; `rerender_page1_10mb`: render page 1 of an open document again (zoom proxy). One engine, so caches are warm (steady state). 20 samples, 5 s each |
| Fresh-process probe | `bench/src/bin/first-page-probe.rs` | One open + render of page 1 in a new process (PDFium bind, cold PDFium caches), plus the process's peak working set. Run 9 times per file; median reported, first run kept as `first_run_ms` |
| Cold start | `bench/cold-start.ps1` (T-005/T-111) | Warm mode, 20 launches: visible window (wall), content visible (internal), WebView2 built |
| Memory sampler | `bench/memory.ps1` | Private working set summed over the app **and all descendant processes** (6 `msedgewebview2.exe`), re-listed every sample. Launch mode: 5 launches, 5 s settle, 1 sample/s for 10 s. Attach mode (`-AppPid`) for a running app with a document open |
| Payload size | in `bench/run.ps1` | `swiftpdf.exe` + `pdfium.dll`, uncompressed (proxy for installer size) |
| Runner | `bench/run.ps1` | Builds, runs all parts, writes one JSON file (default `D:\dev\swiftpdf-bench\results-<stamp>.json`, raw outputs in `raw-<stamp>\`). `-Affinity 0x55` restricts the run and all child processes to 4 cores. `-Skip core,app,memory,size` |
| Compare | `bench/compare.ps1` | Exit 1 if any metric breaks its budget or is > 10 % worse than `bench/baseline.json`; exit 2 on bad input. Missing metrics warn (fail with `-Strict`) |

Checked by hand: compare against itself → exit 0; a copy with visible window +11.2 %, idle memory +10.1 % and
first page 250 ms → exit 1 with `REGRESSION`, `REGRESSION` and `OVER BUDGET, REGRESSION`; a missing results
file → exit 2. Affinity inheritance checked: a child of a process with mask 0x55 reports 0x55 (85).

### Setup

| Item | Value |
|---|---|
| Commit | `145281c` plus the uncommitted T-010 changes |
| Build | `npm run tauri build` (rebuilt for this session) → `D:\dev\swiftpdf-target\release\swiftpdf.exe` (7.35 MB); benches in cargo's `bench` profile (release, LTO) |
| PDFium | `src-tauri/pdfium/pdfium.dll` (7.24 MB), pinned build from `scripts/fetch-pdfium.ps1` |
| Corpus | Generated `large_10mb.pdf` (10 pages, one 1 MB noise image each) and `large_1000pages_300mb.pdf` (1,000 pages, one 315 KB noise image each) |
| Machine | AMD Ryzen 5 2600 (6C/12T), 16 GB RAM, Windows 10 Pro 22H2 (build 19045) — **not** the 4-core/8 GB reference machine |
| Disks | Exe and corpus on D: = Seagate ST1000DM010 **HDD**; WebView2 runtime and profile on C: = Apacer AS350 SATA **SSD** (recorded by `run.ps1` in `machine.*_disk`) |
| Background | Normal desktop session; other WebView2 hosts running (as in T-005) |

```powershell
. .\scripts\dev-env.ps1
powershell -ExecutionPolicy Bypass -File bench\run.ps1 -Label 'T-010 baseline' -OutJson D:\dev\swiftpdf-bench\t010-baseline.json
powershell -ExecutionPolicy Bypass -File bench\run.ps1 -Affinity 0x55 -Label 'T-010 4 cores (affinity 0x55)' -OutJson D:\dev\swiftpdf-bench\t010-4core.json
Copy-Item D:\dev\swiftpdf-bench\t010-baseline.json bench\baseline.json
powershell -ExecutionPolicy Bypass -File bench\compare.ps1 -Results D:\dev\swiftpdf-bench\t010-4core.json
```

### Results (medians)

"4 cores" = the whole run with CPU affinity 0x55 (one logical CPU on each of 4 physical cores, no SMT), the
closest cheap stand-in for the 4-core reference CPU. RAM was not limited to 8 GB.

| Metric (result key) | Budget | Baseline, 6 cores | 4 cores | Result |
|---|---|---|---|---|
| Cold start → visible window (`cold_start.visible_window_ms`, n = 20) | < 500 ms | **94.5 ms** (p95 110.6) | 94.3 ms (p95 108.1) | **PASS** |
| Cold start → content visible (`cold_start.content_visible_ms`, n = 20) | none (M1 target) | 591.3 ms (p95 618.2) | 602.5 ms (p95 631.1) | info |
| 10 MB first page, fresh process (`core.first_page_10mb_fresh_ms`, n = 9) | < 200 ms | **13.5 ms** (first run 14.8) | 13.8 ms | **PASS** (core only) |
| 10 MB first page, criterion (`core.first_page_10mb_ms`) | < 200 ms | 12.0 ms (CI 11.9–12.3) | 13.0 ms | **PASS** (core only) |
| 300 MB / 1,000 pages first page, fresh process (`core.first_page_300mb_fresh_ms`, n = 9) | < 400 ms | **13.9 ms** (first run 15.2) | 13.9 ms | **PASS** (core only) |
| 300 MB / 1,000 pages first page, criterion (`core.first_page_300mb_ms`) | < 400 ms | 11.6 ms (CI 11.4–11.8) | 12.7 ms | **PASS** (core only) |
| Re-render open page at 1280 px (`core.rerender_page1_10mb_ms`) | < 150 ms (zoom, proxy) | 10.4 ms | 11.2 ms | **PASS** (proxy) |
| PDFium bind (`core.pdfium_bind_ms`) | none | 6.0 ms | 5.9 ms | info |
| Idle memory, empty window, app + WebView2 private WS (`memory.idle_private_ws_mb`, n = 5) | < 120 MB | **66.4 MB** (max 69.5) | 66.7 MB | **PASS** (no PDF open) |
| Idle private bytes (commit) (`memory.idle_private_bytes_mb`) | none | 297.1 MB | 298.1 MB | info |
| 300 MB file, core peak working set (`core.peak_ws_300mb_first_page_mb`) | < 400 MB | 26.3 MB | 26.4 MB | **PASS** (core only, no WebView2 or tile cache) |
| Payload exe + pdfium.dll (`size.payload_mb`) | < 15 MB installer | 14.58 MB | – | **PASS, 0.4 MB margin** (uncompressed) |
| Scroll 60 fps / sharp tile < 100 ms / find first hit < 300 ms / incremental save < 300 ms | see §6.1 | not measurable yet | – | T-020, T-021, find and save tasks |

Idle memory breakdown (one run): WebView2 browser 30.4 MB, GPU 12.5, renderer 11.4, network service 7.5,
`swiftpdf.exe` 3.7, storage service 2.9, crashpad 2.4.

### Notes and risks

- **The first-page numbers flatter the product.** They cover PDFium only; the end-to-end budget also pays IPC,
  pixel transfer to WebView2 and paint (T-018/T-020). And the corpus pages are one image each: real 10 MB
  files with many fonts and vector paths will render slower. A realistic text/vector-heavy large file should be
  added to the corpus before M1.
- **Disk cache.** All numbers are with the PDF in the file cache. The first probe run is the only one that could
  read from the HDD, and it was already cached by earlier work (14.8 ms). A true cold-disk open was not measured:
  eviction pushes other apps out of memory (see T-005).
- **4 cores changed little.** PDFium rendering is single-threaded and WebView2 start-up is not CPU-bound here.
  The criterion numbers were 7–10 % slower under the mask; part of that is run-to-run noise (see next point).
- **Noise against the 10 % rule.** The criterion medians moved by up to ~10 % between sessions on this desktop
  (12.0 vs 13.0 ms), so a hard 10 % gate on ~12 ms steady-state benches will flap on a shared or hosted
  machine. Cold start (±1 ms on 94 ms) and memory (±0.5 %) were stable. T-012 plans warn-only on hosted
  runners; for the self-hosted gate (T-047) consider more criterion samples or an absolute floor.
- **Installer budget is tight**: 14.58 MB uncompressed today, with no installer, no fonts and almost no UI yet.
  The compressed installer will be smaller, but this needs watching once bundling exists.
- Cold-start numbers match T-111 (94.5 vs 95.4 ms visible, 591 vs 607 ms content). The first launch after a
  rebuild was slower (235 ms visible, `main` at 101 ms; Defender scan) in a trial run and is not in the baseline.

---

## 2026-10-23 · T-111 · Themed window frame shown at creation

**Verdict: PASS for "visible window".** The main window is now created visible, painted in the dark theme
background (`#1b1d21`, `--bg-app`) with a dark title bar. Warm **process start → visible window: median 95.4 ms
(p95 109.3 ms)**, budget 500 ms (was 620.9 ms in T-005). **Content visible** (UI first paint, `window_ready`) is
unchanged at **median 607.1 ms (p95 635.9 ms)**: WebView2 startup still costs about 550 ms. Closing that gap is the
M1 work (native first-page preview, ROADMAP risk 1).

### Change

- `src-tauri/tauri.conf.json`: main window `"visible": true`, `"theme": "Dark"`, `"backgroundColor": "#1b1d21"`
  (unchanged). tao shows the frame before wry builds WebView2, and wry gives the WebView2 controller the same
  background colour.
- `src-tauri/src/lib.rs`: `window_ready` no longer shows the window. It only writes the `content_ready` startup
  mark (when `SWIFTPDF_STARTUP_LOG` is set). The 3 s show fallback was removed (nothing to fall back to).
- `bench/cold-start.ps1`: `internal_ms` now reads `content_ready` (falls back to the old `shown_by_*` marks for
  older builds). Output is labelled "visible window" (wall) and "content visible" (internal).

Same machine, runtime and method as T-005 (see below). Build: `npm run tauri build`, release exe
`D:\dev\swiftpdf-target\release\swiftpdf.exe`.

```powershell
powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode warm          -Runs 20 -OutJson D:\dev\swiftpdf-bench\t111-warm.json
powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode fresh-profile -Runs 20 -OutJson D:\dev\swiftpdf-bench\t111-fresh-profile.json
```

### Results

| Metric | Budget | Median | p95 | Min / max | Result |
|---|---|---|---|---|---|
| Warm, **visible window** (wall, n = 20) | < 500 ms | **95.4 ms** | 109.3 ms | 94.0 / 110.2 | **PASS** |
| Warm, **content visible** (internal, n = 20) | M1 target, see ROADMAP risk 1 | 607.1 ms | 635.9 ms | 583.1 / 644.0 | not yet |
| First launch after install (fresh profile), visible window (wall, n = 20) | < 500 ms | **100.6 ms** | 111.9 ms | 94.3 / 134.7 | **PASS** |
| First launch after install (fresh profile), content visible (internal, n = 20) | M1 target | 641.9 ms | 682.3 ms | 618.1 / 1,067.1 | not yet |

Other medians (warm): `main` 9 ms, `webview_built` 561 ms, 6 `msedgewebview2.exe` processes, private working set
of the tree 64.7 MB (fresh profile: 69.7 MB). No timeouts. The `evict` mode was not re-run: it pages out the
user's other applications, and the change cannot make it slower (the frame now appears before WebView2 loads).

### No white flash (pixel check)

A scratch script launched the exe, found the main window by process id and title, and captured **only that
window** with `PrintWindow(hwnd, PW_RENDERFULLCONTENT)` (the window's own DWM content, nothing behind it) as
fast as possible (one frame every 15 to 35 ms) from the moment it became visible until 1 s later. For every frame
it counted pixels brighter than 160/255 (average of R, G, B) in the client area and in the title bar. 3 runs plus
2 earlier runs, about 60 frames each, all reviewed as numbers and spot-checked as images:

| Time after process start | Client area | Bright pixels |
|---|---|---|
| First visible frame (~95 ms) | black `(0,0,0)` for one frame, dark title bar with white caption text | 0.00 % client, 0.22 % title (caption text and icon) |
| ~125 ms → content ready | `--bg-app` `#1b1d21` (sampled 26,28,32) | 0.00 % |
| One frame at WebView2 hand-over (~585 to 630 ms) | part of the client area briefly `#121212`-ish (sampled 17 to 20), rest `#1b1d21` | 0.00 % |
| After content ready | `#1b1d21` | 0.00 % |

No frame had a white or light client area. Two short dark-on-dark changes remain (black first frame, one
`#121212` frame at WebView2 hand-over); both are darker than the theme background, not lighter. The `#121212`
frame is most likely Chromium's dark default canvas (`<meta name="color-scheme" content="dark light">`) before
`app.css` applies; inlining the background colour into `index.html` would remove it (frontend, optional).
A screen-region capture (BitBlt) also showed the DWM open animation fading the dark frame in over the content
behind it, with no white frame. Not checked: the light theme and high-contrast modes (the light theme does not
exist yet; `"theme": "Dark"` forces a dark title bar even when Windows is set to light).

Trade-off kept from T-005: for about 500 ms the user sees an empty dark frame that does not respond to dragging,
because tao's main thread is blocked in WebView2 creation.

---

## 2026-10-23 · T-005 · Cold start of the empty Tauri window

**Verdict: FAIL.** Process start to visible, painted window has a **median of 621 ms (p95 905 ms) warm**,
against a 500 ms budget and the 300 ms stop threshold in T-005. Nearly all of the time is WebView2 creation
(about 540 ms), not our code. The best mitigation is measured below: showing the window frame at creation
brings "visible window" to **107 ms**.

### Setup

| Item | Value |
|---|---|
| Commit | `5ff7d62` plus the uncommitted T-005 changes (startup marks in `src-tauri/src/lib.rs`, `bench/cold-start.ps1`) |
| Build | `npm run tauri build` → `D:\dev\swiftpdf-target\release\swiftpdf.exe` (7.7 MB, release, bundling off) |
| Stack | Tauri 2.12.5, wry 0.57.1, tao 0.37.2, Svelte 5, first-paint JS 10.0 kB gz, CSS 0.18 kB gz |
| WebView2 runtime | Evergreen 156.0.4314.42 (on C:, SATA SSD) |
| Machine | AMD Ryzen 5 2600 (6C/12T), 16 GB RAM, Windows 10 Pro 22H2 (build 19045) |
| Disks | Exe on D: = Seagate ST1000DM010 **HDD**; WebView2 runtime and the app's WebView2 profile (`%LOCALAPPDATA%\org.swiftpdf.viewer\EBWebView`) on C: = Apacer AS350 SATA SSD |
| Background | Normal desktop session. Two other WebView2 hosts were running (uTorrent, Windows Search), so the WebView2 runtime DLLs were partly resident in every mode. |

### Method

`bench/cold-start.ps1` launches the exe N times, one at a time, and records per run:

- **wall_ms**: external clock. A `Stopwatch` starts just before `Process.Start`; the script polls `EnumWindows`
  about every 1 ms until an unowned top-level window titled "SwiftPDF" owned by the process is visible.
  (tao's event-target window reports itself visible at about 85 ms and is ignored.)
- **internal_ms**: kernel process-creation time (`Process.StartTime`) to the moment Rust shows the window after
  the UI's first paint (`window_ready`, called after two `requestAnimationFrame`s). The app writes this mark only
  when `SWIFTPDF_STARTUP_LOG` names a file. It is off by default and costs one env-var lookup.
- **main_ms** (Rust `run()` entered) and **webview_built_ms** (Tauri `setup` hook, which Tauri 2 runs right
  after it has built the config window and its WebView2, see `tauri::app::setup`).
- After a 2 s settle: the number of descendant `msedgewebview2.exe` processes, the sum of working sets (counts
  shared DLL pages more than once), the sum of private bytes (commit) and the sum of **private working sets**
  (Task Manager's "Memory" column, via `GetProcessMemoryInfo`).
- The app is then closed (`CloseMainWindow`, `taskkill /T` fallback). The script waits for all WebView2 children
  to exit, then pauses 1 s, so every run creates a new WebView2 browser process.

Modes:

- **warm**: plain relaunch. Exe, runtime and profile are in the file cache.
- **fresh-profile**: the app's `EBWebView` folder is deleted before each run, as on the first launch after install.
- **evict**: before each run, a helper fills free and standby RAM with touched pages (until 300 MB is
  available), then frees it, so Windows drops its file cache. It does not need admin rights.

What I could and could not control:

- I could not reboot or purge the standby list (`RAMMap -Ec` / `NtSetSystemInformation` need admin; the
  session is not elevated). **No true post-reboot number exists yet.**
- `evict` is a pessimistic upper bound, not a reboot. Filling RAM also trims other processes, which then page
  back in during our launch. You can see this in `main_ms`: 70 to 900 ms (median 189 ms) versus 9 ms warm, for
  a 7.7 MB exe. Results got worse when I waited longer after eviction (500 ms → 5 s settle: median 1,963 →
  2,414 ms), which points to system-wide paging rather than our own file reads.
- Files mapped by running processes stay resident. With other WebView2 apps running, parts of the runtime are
  always warm. That is also typical for real users (Windows Search, Teams, Widgets, Office all host WebView2).
- The first launch right after a build (exe newly written, Defender scan) was 831 ms wall / 796 ms internal
  (`main` at 120 ms). That is one sample, recorded for reference only.
- `wall_ms` stops when `IsWindowVisible` turns true. DWM composes the frame on the next vsync, so the
  on-screen pixel time is up to about 16 ms later.

Commands (outputs are on D:, not committed):

```powershell
powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode warm          -Runs 20 -OutJson D:\dev\swiftpdf-bench\t005-warm.json
powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode fresh-profile -Runs 20 -OutJson D:\dev\swiftpdf-bench\t005-fresh-profile.json
powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode evict         -Runs 20 -OutJson D:\dev\swiftpdf-bench\t005-evict.json
```

To get the missing true cold number, run `bench\cold-start.ps1 -Mode warm -Runs 1` as the first thing after a
reboot and login. Only that first run counts as cold.

### Results (process start → visible painted window)

| Metric | Budget | Median | p95 | Min / max | Result |
|---|---|---|---|---|---|
| Warm, wall (n = 20) | < 500 ms | **620.9 ms** | 904.5 ms | 600.2 / 963.7 | **FAIL** |
| Warm, internal (n = 20) | < 500 ms | 617.0 ms | 898.0 ms | 591.5 / 969.7 | FAIL |
| Warm, earlier session (n = 20, wall) | < 500 ms | 601.3 ms | 657.5 ms | 572.2 / 660.2 | FAIL |
| First launch after install (fresh profile, n = 20, wall) | < 500 ms | 660.8 ms | 677.0 ms | 641.5 / 735.5 | FAIL |
| Evicted file cache, worst case (n = 20, wall, 5 s settle) | < 500 ms | 2,414 ms | 3,440 ms | 1,768 / 6,212 | FAIL |
| Evicted file cache, worst case (n = 20, wall, 0.5 s settle) | < 500 ms | 1,963 ms | 2,383 ms | 1,594 / 2,594 | FAIL |
| Evicted, exe copied to SSD (n = 10, wall, 0.5 s settle) | < 500 ms | 2,194 ms | 2,785 ms | 1,972 / 2,785 | FAIL |

The warm p95 of the main run (905 ms) comes from 5 slow runs out of 20 (778 to 964 ms) during background
activity. In the earlier session on the same build the p95 was 658 ms.

Where the time goes (warm, medians):

| Phase | Time from process start | Share |
|---|---|---|
| Rust `run()` entered (loader, CRT, statics) | 9 ms | 1% |
| Tao event loop + window + **WebView2 environment and controller** built (Tauri builds these synchronously before `setup`) | 544 ms (main → webview_built ≈ 535 ms) | ~88% |
| Navigate, parse 10 kB JS, mount Svelte, 2 × rAF, `window_ready` IPC, `show()` | ~50 to 60 ms | ~9% |
| `show()` → window visible to `EnumWindows` | 4 to 10 ms | ~1% |

Memory and processes (warm, 2 s after the window appeared, medians):

| Metric | Value |
|---|---|
| `msedgewebview2.exe` processes per launch | 6 (browser, GPU, 2 × utility, renderer, crashpad) |
| Private working set, app + WebView2 tree (Task Manager "Memory") | **67.6 MB** (budget for idle with a small PDF: < 120 MB) |
| Sum of working sets (counts shared pages more than once) | 279.5 MB |
| Sum of private bytes / commit | 299.9 MB |
| `swiftpdf.exe` alone, working set | 21 MB |

One-off breakdown (WS / private bytes): browser 99 / 81 MB, renderer 51 / 26 MB, GPU 47 / 31 MB,
utility 33 / 57 MB, utility 19 / 8 MB, crashpad 10 / 48 MB, swiftpdf.exe 21 / 52 MB.

### Mitigation experiments (measured, same build, back to back, 10 warm runs each)

| Variant | Visible window median (p95) | Painted content median | vs baseline |
|---|---|---|---|
| Baseline: hidden until first paint (T-004 design) | 613.7 ms (691.7) | 610.9 ms | — |
| **Window `visible: true` at creation, `backgroundColor #1b1d21`** | **107.3 ms (141.7)** | 617.4 ms | **−506 ms visible**, content unchanged |
| `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--disable-gpu` | 650.4 ms (775.8) | 640.9 ms | none (noise) |
| `--disable-features=msSmartScreenProtection,msWebOOUI,msPdfOOUI,SpareRendererForSitePerProcess --disable-background-networking --disable-component-update` | 628.2 ms (828.7) | 626.6 ms | none (noise) |
| `--disable-gpu-compositing` | 589.1 ms (756.4) | 579.9 ms | ≈ −25 ms, within noise |

The `visible: true` build was a temporary config change for this experiment only. It was reverted, and the
release exe was rebuilt with `visible: false`.

### Mitigation options, ranked by expected gain on "process start → visible window"

1. **Show the native window frame at creation** (`visible: true` plus the theme `backgroundColor`, or create
   the window from Rust and show it before the WebView is attached). **Measured: −506 ms → 107 ms.** It costs
   one config line and passes the budget alone. Trade-offs: it reverses the T-004 rule "hidden until first
   paint". For about 500 ms the user sees an empty dark frame, and it cannot be dragged, because tao's main thread
   is blocked inside WebView2 creation. It needs a visual check that the frame paints in the theme color and
   never flashes white on light/dark/high-contrast themes. Painted content stays at about 615 ms.
2. **Native splash / native first-paint skeleton** inside that frame (GDI or Direct2D: ribbon outline, or a
   PDFium bitmap of the first page rendered in parallel with WebView2 creation). Visible gain is the same as
   option 1 (about 500 ms), and the user sees meaningful content instead of an empty frame. The first page
   could appear before the WebView is ready. Effort: medium to high (native drawing, hand-off without a flicker
   when the WebView takes over).
3. **Prewarm**: a resident background instance (started at login or kept after the last window closes) that
   already holds a WebView2 environment. New launches are forwarded through single instance (T-024) and only
   open a window. Expected visible-with-content time: about 50 to 150 ms (window plus controller on a warm
   environment, not measured yet). Cost: about 68 MB of private working set held all the time, an autostart
   entry, and product/privacy questions (a background process on a "no background stuff" viewer). It needs
   user approval.
4. **Reuse one WebView2 environment for all windows and tabs** (one browser process): no gain on the first
   window. It prevents later windows from paying about 500 ms each. Do it from the start.
5. **Parallelise Rust start-up work with WebView2 creation** (PDFium load, file open, as planned in §7 / T-019):
   0 ms gain today. It keeps future work from adding to the 535 ms critical path.
6. **Trim first-paint JS/CSS**: at most about 30 to 40 ms. The whole navigate → paint → IPC tail is about 55 ms
   with a 10 kB bundle. Keep the bundle small (T-022).
7. **WebView2 browser flags** (SmartScreen off, GPU flags): **measured no gain** (within noise). They are
   still worth doing for privacy in T-019, but not as a speed fix.

**Recommendation:** adopt option 1 now (it meets the budget, and measured warm and fresh-profile numbers
leave large headroom), plan option 2 as the polish for M1, and treat option 3 as a decision for the user. Get
one true post-reboot sample before T-014, because the worst-case `evict` numbers (about 2 s) show that cold
disk state can dominate. With option 1 the frame appears before WebView2 loads, so most of the cold WebView2
cost should move out of "visible window" and into "content painted". This is expected but not yet measured cold.
