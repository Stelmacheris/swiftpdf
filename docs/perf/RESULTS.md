# Performance results

Budgets: `docs/REQUIREMENTS.md` §6.1. Each section is one measurement session; newest first.

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
