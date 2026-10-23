# SwiftPDF Tasks

Source of truth: `docs/REQUIREMENTS.md`. Milestones and risks: `docs/ROADMAP.md`. Owner: project-manager.

**Format:** `- [ ] T-### · requirement IDs · title · owner · depends`, followed by an `Accept:` line.
**Status:** `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked (reason given).
**Rules:** Work top to bottom. Start a task only when every task in `depends` is `[x]`. Only one code-writing agent runs at a time
(rust-core-developer, frontend-developer, performance-engineer, tester). Tasks tagged **parallel-ok** (ui-ux-accessibility,
security-reviewer, docs-writer) can run beside them. IDs are sequential and never reused. Gate findings become new tasks,
appended to the end of the milestone's section.
**Shared paths:** generated large corpus `D:\dev\swiftpdf-corpus` (gitignored) · build output `D:\dev\swiftpdf-target` · PDFium binaries `src-tauri/pdfium/` (gitignored).

---

## M0: Setup

- [ ] T-001 · U-01, U-03, AC-01 · Design spec: app shell and ribbon · owner: ui-ux-accessibility · depends: none · **parallel-ok**
      Accept: `docs/design/shell.md` and `docs/design/ribbon.md` with ASCII wireframes of the window (tab bar, ribbon, sidebar, page canvas, status bar). All six ribbon tabs (Home, Comment, Pages, Forms & Sign, Protect, View) are listed with groups and commands, each mapped to a requirement ID. The collapsed single-row ribbon is shown. Home and View are marked as the first-paint bundle (§7). No promotional or interrupting UI.
- [ ] T-002 · U-03, AC-04, V-09 · Design spec: theme tokens · owner: ui-ux-accessibility · depends: none · **parallel-ok**
      Accept: `docs/design/theme.md` defines CSS custom properties for dark (default) and light themes. Every text/background pair is ≥ 4.5:1 and non-text UI is ≥ 3:1, with the ratios listed. Includes a `forced-colors` mapping to system colors, focus-ring tokens, page-color presets (normal, dark, sepia) for V-09, and a reduced-motion rule.
- [ ] T-003 · AC-01, U-04, V-02, V-07, V-11, S-01, U-08 · Design spec: default keymap and focus order · owner: ui-ux-accessibility · depends: none · **parallel-ok**
      Accept: `docs/design/keymap.md` has a table of stable command IDs → default shortcuts (Acrobat-familiar: Ctrl+O, Ctrl+W, Ctrl+Tab, Ctrl+F, F3, Alt+←/→, Ctrl+plus/minus, Ctrl+0, Ctrl+P, Ctrl+S, Ctrl+Z/Y…), with no conflicts. Lists the WebView2 built-in keys that must be suppressed (F5, Ctrl+R, Ctrl+P browser print, Ctrl+Shift+I, Ctrl+G…). Defines the main-region focus order and F6 region cycling. The command IDs are reusable later by U-04.
- [ ] T-004 · §9-M0, §7, X-06 · Scaffold Tauri 2 + Svelte + TypeScript · owner: rust-core-developer · depends: none (needs the MSVC Build Tools install to finish)
      Accept: Tauri 2 app in `src-tauri/` plus a separate core library crate (e.g. `crates/swiftpdf-core`) in one Cargo workspace. Svelte + TS + Vite 5.x frontend. `npm install` shows no EBADENGINE warning on Node 18.12.1. `npm run tauri dev` opens an empty window titled "SwiftPDF". `cargo build --release` succeeds with output in `CARGO_TARGET_DIR`. The window is created hidden and shown on first paint. Strict CSP (no remote origins). Minimal capabilities (no shell/http/fs-wide plugins). No network on launch. `.gitignore` covers node_modules, target, `src-tauri/pdfium/` and the generated corpus. MIT `LICENSE` at the root.
- [ ] T-005 · §6.1 cold start, §8 risk 1 · Measure cold start of the empty Tauri window · owner: performance-engineer · depends: T-004
      Accept: The release build is launched ≥ 20 times, both after a reboot or file-cache flush where possible and warm. Process start → window visible (first paint) is reported as median and p95, with WebView2 process count and memory. The measurement script is committed under `bench/`. If the median is > 300 ms, the report lists mitigation options (prewarm, WebView2 environment options, native splash) ranked by expected gain.
- [ ] T-006 · §7, U-03 · Frontend tooling and theme tokens · owner: frontend-developer · depends: T-004, T-002
      Accept: `npm run check` (svelte-check/tsc) has 0 errors. `npm test` (Vitest) runs a sample test. `npm run lint` (ESLint + Prettier) is clean. `theme.css` implements the T-002 tokens with dark as default. All tools are compatible with Node 18.12. The first-paint JS/CSS size (gzipped) is printed by the build.
- [ ] T-007 · §7, X-06, §6.3 supply chain · PDFium binary fetch script · owner: rust-core-developer · depends: T-004
      Accept: `scripts/fetch-pdfium.ps1` downloads a pinned bblanchon/pdfium-binaries release (**non-V8, non-XFA** build; win-x64, with a flag for win-arm64) into `src-tauri/pdfium/`. It verifies a SHA-256 hard-coded in one place, fails loudly on a mismatch, is idempotent, keeps the PDFium LICENSE file, and works unattended in CI.
- [ ] T-008 · §6.2, X-07 · Test corpus and generator (no copyrighted files) · owner: tester · depends: T-004
      Accept: `tests/corpus/README.md` records provenance and license for every file. A generator in the repo (MIT/Apache deps only) produces: 1-page text, 100-page mixed text/vector/images, outline + internal/external links, AcroForm (text, checkbox, radio, combo, list), an XFA-flagged form, attachments, rotated pages, mixed page sizes, and a malformed set (truncated, broken xref, garbage bytes, zero-length, deep nesting). A 10 MB file and a 1,000-page/300 MB file are generated on demand into `D:\dev\swiftpdf-corpus`, not committed. Committed corpus is < 5 MB. Encrypted samples are deferred to T-037.
- [ ] T-009 · §7, X-07 · pdfium-render integration and smoke test · owner: rust-core-developer · depends: T-007, T-008
      Accept: The core crate binds PDFium dynamically from the bundled path and returns a clear error (no panic) if it is missing. `cargo test` opens the 1-page and 100-page corpus files, checks the page count, renders page 1 to a bitmap and asserts it is not blank. Every malformed file returns `Err` without a panic. The PDFium threading approach (PDFium is not thread-safe) is documented in code.
- [ ] T-010 · §6.1 · Benchmark harness skeleton · owner: performance-engineer · depends: T-005, T-009
      Accept: `bench/` contains criterion benches (open + render first page of the 10 MB file), the repeatable cold-start command from T-005, and a memory sampler that includes WebView2 child processes. Results are written as JSON. A compare script exits non-zero if any metric regresses > 10% against `bench/baseline.json`. All §6.1 budgets are encoded in one file. A first baseline is recorded on the reference machine.
- [ ] T-011 · License policy (§1, §6.3) · cargo-deny and npm license check · owner: rust-core-developer · depends: T-004
      Accept: `deny.toml` allows only MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, Zlib, ISC and Unicode (Unicode-DFS-2016 / Unicode-3.0). `cargo deny check` passes for licenses, bans, advisories and sources. A script checks npm production dependencies against the same allowlist and passes. Any exception is listed with a reason and flagged to the PM for user approval.
- [ ] T-012 · §6.1, §9-M0 · GitHub Actions CI · owner: rust-core-developer · depends: T-006, T-007, T-009, T-010, T-011
      Accept: `.github/workflows/ci.yml` runs on push and PR on windows-latest: fetch PDFium (cached by checksum), `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo deny check`, `npm ci`, `npm run check`, `npm test`, `npm run lint`. Actions are pinned by SHA and `permissions: contents: read`. A perf job runs the benches and uploads JSON (warn-only on hosted runners until T-047). CI is green on main and takes < 20 min.
- [ ] T-013 · M0 gate · Tester full pass · owner: tester · depends: T-012
      Accept: From a clean clone: fetch-pdfium → `npm ci` → all Rust and frontend tests are green locally and in CI. Results are reported with real output.
- [ ] T-014 · M0 gate, §6.1 · Performance baseline check · owner: performance-engineer · depends: T-012
      Accept: `bench/baseline.json` is committed. The report table covers cold start (empty window) against the 500 ms budget and first-page render of the 10 MB file against 200 ms (core only), with pass/fail and the remaining risks.
- [ ] T-015 · M0 gate · Security audit · owner: security-reviewer · depends: T-012 · **parallel-ok**
      Accept: Reviews the CSP and capabilities, fetch-script checksum pinning and the non-V8 build, deny.toml, CI workflow permissions and pinning, and finds no network code. Findings are ranked, with a verdict.
- [ ] T-016 · M0 gate · Developer docs and README · owner: docs-writer · depends: T-012 · **parallel-ok**
      Accept: `docs/dev/SETUP.md` covers rustup + MSVC Build Tools + Windows SDK, WebView2, Node ≥ 18.12, the RUSTUP_HOME/CARGO_HOME/CARGO_TARGET_DIR variables (described generically), fetch-pdfium, dev/build/test/bench commands, and the OneDrive/disk-space notes. `README.md` covers what the product is, why it is different, build steps, MIT license and "no telemetry", and states the status as pre-alpha.

---

## M1: Fast viewer

- [ ] T-017 · V-02, V-04, V-05, V-09, V-12, S-01, S-02, X-01, U-08 · Design spec: viewer screens · owner: ui-ux-accessibility · depends: T-001 · **parallel-ok**
      Accept: `docs/design/` specs cover the tab bar, sidebar panels (thumbnails, outline, annotations, attachments), find bar and results panel, start screen with recent files, password dialog, print dialog, external-link confirmation, page-color menu, and empty/error states (bad file, unsupported). Each spec defines keyboard behaviour and accessible names.
- [ ] T-018 · V-01, X-07, §6.1 · DocumentManager: open/close and metadata · owner: rust-core-developer · depends: T-009
      Accept: Tauri commands `open_document(path)` → {id, page count, page sizes, title} and `close_document(id)`. Uses lazy loading with no full parse. Errors are typed results, never panics. A failing document does not affect other open documents (integration test). Metadata for the 1,000-page/300 MB file is returned in < 150 ms on the reference machine.
- [ ] T-019 · §6.4, X-06, §6.1 cold start · WebView2 hardening and startup path · owner: rust-core-developer · depends: T-005, T-018
      Accept: WebView2 environment options turn off SmartScreen, autofill, spellcheck downloads and other features that contact Microsoft services. There are no remote origins. PDFium loads in parallel with WebView2 creation (§7). Connection monitoring (netstat/Resource Monitor or ETW) shows 0 outbound connections during 60 s idle after launch. Cold start does not regress against the T-005 numbers.
- [ ] T-020 · V-07, §7 tiles, §6.1 memory · `pdf://` tile protocol and LRU TileCache · owner: rust-core-developer · depends: T-018
      Accept: The custom protocol serves tiles at `pdf://tile/{doc}/{page}/{zoom}/{x}/{y}`, 256 to 512 px at the device pixel ratio. Every parameter is validated: unknown doc → 404, out-of-range values → 400, and no file paths are accepted. The pixel format (raw vs WebP) is chosen with measured costs documented. The cache is memory-bounded with LRU eviction (cap configurable), and a cache hit takes < 5 ms. Unit tests included.
- [ ] T-021 · S-06, X-07, §6.1 scroll · RenderScheduler with priorities and cancellation · owner: rust-core-developer · depends: T-020
      Accept: Priority order is visible > near > thumbnails. Stale requests are cancelled. A low-res pass comes first, then the sharp pass. Each render has a timeout, so a hanging page never blocks other pages or tabs. Tests cover ordering, cancellation and timeout.
- [ ] T-022 · U-01, U-03, AC-04 · App shell: ribbon, tab-bar container, sidebar container, status bar, themes · owner: frontend-developer · depends: T-006, T-001, T-003
      Accept: Built to T-001 and T-002. Home and View tabs are live. Other ribbon tabs are lazy-loaded chunks, not in the first-paint bundle. The ribbon collapses to one row. Dark (default), light and "follow Windows" themes work. `forced-colors` is supported. The first-paint bundle size is reported.
- [ ] T-023 · V-06, V-07, §6.1 scroll/zoom · Virtualized continuous page view with tiles and zoom · owner: frontend-developer · depends: T-021, T-022
      Accept: Only visible and nearby pages exist in the DOM. A low-res placeholder appears immediately, then sharp tiles. Zoom supports fit width, fit page, actual size, 10 to 6400%, Ctrl+wheel and touchpad pinch. Zoom shows an immediately scaled bitmap, then re-renders. Scrolls at 60 fps on the 1,000-page file (measured by T-026).
- [ ] T-024 · V-01 · Open sources: command line, single instance, drag-drop, File > Open · owner: rust-core-developer · depends: T-018
      Accept: `swiftpdf.exe file.pdf` opens the file. A second launch forwards its files to the running window (single instance). Dropped files open. The native File > Open dialog filters for PDF. "Open with" works through the command-line path (file association is in T-099). Non-PDF or missing files show an error without crashing.
- [ ] T-025 · V-02, R-05, V-01 · Document tabs · owner: frontend-developer · depends: T-023, T-024
      Accept: Open 5 PDFs and reorder them by drag. Middle-click and Ctrl+W close a tab. Ctrl+Tab / Ctrl+Shift+Tab cycle tabs. A dirty indicator and close prompt exist (wired to real edits in M2). Each tab keeps its own page, zoom and scroll.
- [ ] T-026 · §6.1 · Mid-M1 performance checkpoint · owner: performance-engineer · depends: T-025
      Accept: Measures cold start, first sharp page of the 10 MB file (< 200 ms), first page of the 1,000-page file (< 400 ms), scroll fps (60), zoom re-render (< 150 ms) and idle memory (< 120 MB). Hot paths are profiled and fixed, or fix tasks are proposed. The report is a before/after table.
- [ ] T-027 · V-08, V-09 · Render options: view rotation and page colors · owner: rust-core-developer · depends: T-021
      Accept: Rotation (0/90/180/270) is per document and not saved. Page colors: normal, dark (inverted text and vectors, images kept natural, via PDFium color-scheme rendering), sepia, and custom foreground/background. Both options are part of the tile cache key. Tests compare pixel samples.
- [ ] T-028 · V-06, V-08, V-09 · Layout modes, rotate view and page-color UI · owner: frontend-developer · depends: T-027, T-025
      Accept: Continuous, single page, two-page and book (separate cover) layouts. Rotate left/right per tab. The page-color menu works per T-017. Settings persist per tab during the session.
- [ ] T-029 · V-05, V-11, V-12 · Outline, links, attachments and annotation list data · owner: rust-core-developer · depends: T-018
      Accept: Commands return the outline tree, per-page link rectangles with destinations (internal page/position or URI), the attachment list (never auto-opened; save-to-disk only), page labels and a read-only list of existing annotations. Tested on corpus files.
- [ ] T-030 · V-05 · Sidebar: thumbnails, outline, annotations, attachments · owner: frontend-developer · depends: T-029, T-025
      Accept: Thumbnails are virtualized and rendered at thumbnail priority. The outline tree is keyboard-navigable and click jumps. The annotations list jumps to the annotation. Attachments offer "Save as…" only. The sidebar is collapsible and resizable.
- [ ] T-031 · V-11, V-12, X-06 · Link navigation and back/forward history · owner: frontend-developer · depends: T-030
      Accept: Internal links jump. Alt+←/→ and the mouse back/forward buttons walk the history (links, outline and page jumps). External links show a confirmation dialog with the full URL. Only http/https/mailto are allowed; other schemes are blocked with a message.
- [ ] T-032 · X-06 · Safe mode enforcement · owner: rust-core-developer · depends: T-029
      Accept: A runtime check asserts the PDFium build has no V8/XFA. Document, page and field JavaScript never runs. Launch, GoToR, SubmitForm and ImportData actions are never executed. URI actions are only passed to the UI for confirmation. Fixtures with each action type are added to the corpus generator, with tests.
- [ ] T-033 · V-13, S-01 · Text extraction API · owner: rust-core-developer · depends: T-018
      Accept: Per-page characters with boxes (lazy, cached). Selection by character range and by rectangle (column). Copied text follows reading order. Tested on the corpus, including rotated pages.
- [ ] T-034 · V-13 · Text selection overlay and copy · owner: frontend-developer · depends: T-033, T-028
      Accept: Drag selects text. Double-click selects a word and triple-click a line. Alt+drag selects a column. Ctrl+C copies. The selection is visible in all page-color modes and in forced-colors.
- [ ] T-035 · S-01, S-02, S-03, S-06 · SearchEngine: background, streaming, options · owner: rust-core-developer · depends: T-033
      Accept: Runs on a background thread and can be cancelled. Hits stream as {page, rects, context snippet}. Options: match case, whole word, regex (regex crate with size limits) and diacritics-insensitive. On the 1,000-page file the first hit arrives in < 300 ms. Scrolling is never blocked (tested alongside render).
- [ ] T-036 · S-01, S-02, S-03 · Find bar and results panel · owner: frontend-developer · depends: T-035, T-031
      Accept: Ctrl+F opens find. Enter/Shift+Enter and F3/Shift+F3 move between hits. All hits are highlighted, and the hit count streams in. The results panel shows context and click jumps. Option toggles. Screen readers get the count through a live region. Escape closes find.
- [ ] T-037 · X-01 · Password-protected open · owner: rust-core-developer · depends: T-018
      Accept: Detects encryption and accepts a password retry. A wrong password gives a typed error. The password is never logged or persisted. Encrypted fixtures (RC4-40, RC4-128, AES-128, AES-256) are generated with a test-only qpdf download with a pinned checksum (never shipped). Tests included.
- [ ] T-038 · X-01 · Password prompt dialog · owner: frontend-developer · depends: T-037, T-025
      Accept: A per-tab modal with a masked field, retry on a wrong password, and Cancel closes the tab. Accessible name and focus trap. Matches the T-017 spec.
- [ ] T-039 · U-06, V-03, V-04 · Settings JSON, session store and recent files · owner: rust-core-developer · depends: T-018
      Accept: Readable `settings.json`, stored next to the exe in portable mode (exe folder writable plus a marker file), otherwise in `%APPDATA%\SwiftPDF` (decision flagged, see ROADMAP risk 15). Schema is versioned. A corrupt file falls back to defaults and keeps a backup copy. The session stores open tabs with page, zoom and scroll. Recent files are kept with cached thumbnails in `%LOCALAPPDATA%`. Tests included.
- [ ] T-040 · V-03, V-04 · Start screen and session restore · owner: frontend-developer · depends: T-039, T-036
      Accept: The start screen shows recent files with thumbnails, is keyboard-accessible and lets the user remove an entry. Restart restores tabs at the same page, zoom and scroll. Only the active tab renders on startup (protects cold start).
- [ ] T-041 · U-08 · Print backend · owner: rust-core-developer · depends: T-027
      Accept: Uses the native Windows print dialog. Renders to the printer DC through PDFium. Supports page range, scaling (fit, actual, shrink to fit), duplex, and printing annotations on/off. Runs off the UI thread, reports progress and can be cancelled. Tested with "Microsoft Print to PDF".
- [ ] T-042 · U-08 · Print UI · owner: frontend-developer · depends: T-041, T-040
      Accept: Ctrl+P is intercepted from WebView2 and opens the app's print flow (T-017 spec) with options and a progress/cancel UI.
- [ ] T-043 · AC-01, AC-04 · Command registry, keymap, focus and DPI · owner: frontend-developer · depends: T-042
      Accept: A data-driven command registry uses the T-003 keymap (ready for U-04 later). WebView2 built-in keys are suppressed. Focus is visible on every control. F6 cycles regions. The UI is correct at 100 to 300% text scaling, per-monitor DPI changes and in forced-colors.
- [ ] T-044 · X-07, §6.3 fuzzing · Fuzz/stress harness on the open path · owner: tester · depends: T-032
      Accept: A cargo-fuzz target on open + render page 1 (if nightly/libFuzzer is unusable on MSVC, a stable mutation harness seeded from the corpus, with the reason reported). A ≥ 15-minute run has 0 panics or crashes. Every crash found is fixed in a new task with a regression test. An integration test shows one failing tab leaves the app and other tabs alive.
- [ ] T-045 · M1 features · Integration and E2E tests · owner: tester · depends: T-043, T-044
      Accept: Rust integration tests cover tiles, scheduler, search, text, outline/links, password and safe mode. Vitest covers stores. tauri-driver E2E (msedgedriver matching WebView2) covers open → find → follow link → back → close, and session restore. All run in CI.
- [ ] T-046 · M1 gate, AC-01, AC-04 · Accessibility and usability audit · owner: ui-ux-accessibility · depends: T-045 · **parallel-ok**
      Accept: Audit of the frontend against T-001 to T-003 and T-017: roles, labels, focus, live regions, forced-colors and 100 to 300% scaling. Findings ranked with file:line and fixes.
- [ ] T-047 · M1 gate, §6.1 · Performance budget gate · owner: performance-engineer · depends: T-045
      Accept: Pass/fail on the reference machine for cold start < 500 ms, 10 MB first page < 200 ms, 1,000-page first page < 400 ms, 60 fps scroll, zoom < 150 ms, find first hit < 300 ms, idle memory < 120 MB and 1,000-page memory < 400 MB. Hosted-CI variance is measured, and the > 10% regression check becomes blocking in CI (or a documented self-hosted/alternative plan).
- [ ] T-048 · M1 gate · Tester full pass · owner: tester · depends: T-047
      Accept: All Rust, frontend, E2E and corpus tests are green (0 crashes on the corpus), including after T-047 changes. The report includes real output and coverage gaps.
- [ ] T-049 · M1 gate · Security audit · owner: security-reviewer · depends: T-048 · **parallel-ok**
      Accept: Covers `pdf://` parameter validation, CSP and capabilities, safe mode (X-06), untrusted-input handling (no unwrap on file data), a grep for network code, WebView2 hardening, settings/session file handling and the password-handling path. Verdict given.
- [ ] T-050 · M1 gate · User docs for viewing · owner: docs-writer · depends: T-048 · **parallel-ok**
      Accept: `docs/user-guide/` pages for opening, tabs, navigation, layouts and page colors, search and printing. `docs/shortcuts.md` is generated from the keymap. First version of `docs/PRIVACY.md` (no network; what is stored where). README status updated.

---

## M2: Annotate

- [ ] T-051 · A-01, A-02, A-03, A-04, A-07, A-08 · Design spec: Comment tab and annotation tools · owner: ui-ux-accessibility · depends: T-017 · **parallel-ok**
      Accept: Spec for the Comment ribbon, color picker, properties panel, note popup with replies, selection handles, eraser, and keyboard ways to create, move and delete annotations. Includes unsaved-changes, recovery and read-only banners.
- [ ] T-052 · A-09, A-07 · AnnotationService core · owner: rust-core-developer · depends: T-029
      Accept: Reads all standard annotations per page (type, rect, color, contents, author, dates) with stable IDs. Create/update/delete API. Writes appearance streams so annotations display in Acrobat, Chrome and Edge. Invalidates tiles for affected pages only. Tests included.
- [ ] T-053 · A-08, R-05 · Edit command stack (undo/redo) · owner: rust-core-developer · depends: T-052
      Accept: Every document mutation goes through one command stack, with unlimited undo/redo per session. Dirty state is emitted as events. Tests cover long mixed undo/redo sequences.
- [ ] T-054 · R-01, R-04 · SaveService: incremental and atomic save · owner: rust-core-developer · depends: T-053
      Accept: Ctrl+S saves incrementally (appends changes) using write to temp → fsync → rename on the same volume. The original is untouched on any failure. A 300 MB file with 1 new annotation saves in < 300 ms; if not reachable with temp + rename, a design proposal goes to the PM before closing (ROADMAP risk 9). Tests included.
- [ ] T-055 · R-02 · Save As (optimized) full rewrite · owner: rust-core-developer · depends: T-054
      Accept: Full rewrite to a new or same path that is smaller than the incremental version on the corpus, and atomic. Re-opens identically (page count, annotations).
- [ ] T-056 · R-06 · Locked and read-only files · owner: rust-core-developer · depends: T-054
      Accept: Files locked by another program or read-only open in read-only mode with a flag sent to the UI. Save then forces Save As. The app does not hold an exclusive lock that blocks other apps from reading. Tests included.
- [ ] T-057 · R-03 · Autosave recovery · owner: rust-core-developer · depends: T-054
      Accept: Every 60 s, if dirty, writes a recovery copy atomically to `%LOCALAPPDATA%\SwiftPDF\recovery` (user-only permissions). After a crash, startup detects leftovers and offers a restore. Recovery files are removed on a clean close. Kill-process test included.
- [ ] T-058 · A-01, A-08, R-01, R-05 · Annotation overlay, text-markup tools and save/undo wiring · owner: frontend-developer · depends: T-055, T-034, T-051
      Accept: The Comment tab is lazy-loaded. Highlight, underline, strikeout and squiggly work on selected text with a color picker. Ctrl+S / Ctrl+Shift+S and Ctrl+Z / Ctrl+Y work. The tab dirty indicator and close prompt are live. Cold start does not regress (> 10%).
- [ ] T-059 · A-02 · Notes with replies and free-text boxes (core) · owner: rust-core-developer · depends: T-053
      Accept: Text annotations with Popup and replies (IRT). Author is taken from the Windows user name. FreeText supports font, size, color and border with appearance. Round-trip tests included.
- [ ] T-060 · A-02 · Sticky note and free-text UI · owner: frontend-developer · depends: T-059, T-058
      Accept: Place a note, open/close it, reply in a thread, and edit free text in place, with properties from T-051.
- [ ] T-061 · A-03, A-04 · Ink and shapes (core) · owner: rust-core-developer · depends: T-053
      Accept: Ink (InkList, with pressure shown as width variation in the appearance). Line, arrow (line endings), rectangle, ellipse, polygon and cloud (border effect). All have appearance streams and round-trip tests.
- [ ] T-062 · A-03 · Ink tool with pen pressure and eraser · owner: frontend-developer · depends: T-061, T-060
      Accept: Pointer Events pressure from a Windows Ink pen with smoothing. Mouse falls back to a fixed width. The eraser removes whole strokes and can be undone.
- [ ] T-063 · A-04 · Shape tools · owner: frontend-developer · depends: T-062
      Accept: Line, arrow, rectangle, ellipse, polygon (click to add points, double-click to finish) and cloud. Shift constrains. Stroke and fill colors and width.
- [ ] T-064 · A-07, V-05 · Select, move, resize, delete and properties · owner: frontend-developer · depends: T-063
      Accept: Any annotation can be selected, moved, resized, deleted and edited in the properties panel, including with the keyboard (arrow keys, Delete). The sidebar annotation list updates live. Every action can be undone.
- [ ] T-065 · R-03, R-05, R-06 · Recovery, read-only banners and close prompt · owner: frontend-developer · depends: T-064, T-056, T-057
      Accept: Non-blocking banners for "recovered changes available" and "read-only". The close prompt offers Save / Don't save / Cancel for each dirty tab and on app exit.
- [ ] T-066 · A-09, R-01 to R-04, R-06 · Round-trip and data-safety tests · owner: tester · depends: T-065
      Accept: Every annotation type: create → save → reopen gives equal results. Saved files pass `qpdf --check` (test-only tool). Killing the process mid-save leaves the original intact. Autosave restore and locked-file tests. Undo/redo E2E. A sample set plus checklist is produced for later Acrobat/Chrome/Edge verification (T-105).
- [ ] T-067 · M2 gate · Accessibility audit of annotation tools · owner: ui-ux-accessibility · depends: T-066 · **parallel-ok**
      Accept: Findings ranked with fixes. Annotation creation and editing are reachable without a mouse.
- [ ] T-068 · M2 gate, §6.1 · Performance budget gate · owner: performance-engineer · depends: T-066
      Accept: Incremental save of 1 annotation on 300 MB < 300 ms. Scroll stays at 60 fps on a page with 500 annotations. Cold start and idle memory have not regressed > 10% (lazy loading works). M1 budgets still pass.
- [ ] T-069 · M2 gate · Tester full pass · owner: tester · depends: T-068
      Accept: All suites green, including after T-068 changes. Report with real output.
- [ ] T-070 · M2 gate · Security audit · owner: security-reviewer · depends: T-069 · **parallel-ok**
      Accept: Covers atomic saves, temp-file cleanup, recovery-folder permissions, annotation content handling (no script or HTML injection from Contents/RC into the WebView) and the no-data-loss paths. Verdict given.
- [ ] T-071 · M2 gate · User docs: annotating and saving · owner: docs-writer · depends: T-069 · **parallel-ok**
      Accept: "How to highlight", notes, ink, shapes, undo, save vs. Save As (optimized), recovery after a crash, read-only files. Shortcuts and PRIVACY (recovery folder) updated.

---

## M3: Pages

- [ ] T-072 · P-01, P-02, P-03, P-04 · Design spec: organizer and Pages tab · owner: ui-ux-accessibility · depends: T-051 · **parallel-ok**
      Accept: Spec for the organizer grid (multi-select by mouse and keyboard, drag indicators), the Pages ribbon, the insert, merge and split/extract dialogs, and their empty and error states.
- [ ] T-073 · P-01 · PageOps: reorder, rotate, delete, duplicate · owner: rust-core-developer · depends: T-053
      Accept: All operations go through the command stack (undoable). Rotation is saved as /Rotate. Tiles and thumbnails are invalidated. Works on the 1,000-page file. Tests check page order and per-page content hashes.
- [ ] T-074 · P-02, P-03 · Insert blank pages, insert from file, merge · owner: rust-core-developer · depends: T-073
      Accept: Blank pages match the neighbour's size or a chosen size. Pages can be imported from another file, including password-protected sources. Merge combines N files into a new document in the chosen order, keeping links (how outlines are handled is documented). Undoable. Tests included.
- [ ] T-075 · P-04 · Split and extract · owner: rust-core-developer · depends: T-074
      Accept: Split by ranges ("1-3,5,7-"), every N pages, or top-level bookmarks. Extract selected pages to a new file. Clear output naming, atomic writes, progress and cancel. Invalid ranges give typed errors. Tests included.
- [ ] T-076 · P-01 · Organizer grid · owner: frontend-developer · depends: T-073, T-072
      Accept: Virtualized thumbnail grid with drag reorder, Ctrl/Shift and keyboard multi-select, rotate/delete/duplicate actions, and undo. The Pages tab is lazy-loaded.
- [ ] T-077 · P-02, P-03 · Insert and merge UI · owner: frontend-developer · depends: T-074, T-076
      Accept: Insert blank / insert from file. Merge by dragging files into the organizer, or a Merge dialog with a reorderable file list.
- [ ] T-078 · P-04 · Split/extract dialog · owner: frontend-developer · depends: T-075, T-077
      Accept: Range input with live validation, a preview of the number of output files and their names, output folder choice, and progress/cancel.
- [ ] T-079 · P-01 to P-04 · Page-ops tests · owner: tester · depends: T-078
      Accept: Correctness of order, rotation and per-page content hashes. Merge/split across corpus files, including an encrypted source. All outputs pass `qpdf --check`. Undo E2E. Samples added to the T-105 compatibility set.
- [ ] T-080 · M3 gate · Accessibility audit of the organizer · owner: ui-ux-accessibility · depends: T-079 · **parallel-ok**
      Accept: Findings ranked with fixes. Keyboard reorder and multi-select work and are announced.
- [ ] T-081 · M3 gate, §6.1 · Performance budget gate · owner: performance-engineer · depends: T-079
      Accept: Organizer on the 1,000-page file stays < 400 MB and scrolls the grid at 60 fps. Merge time for 10 × 100-page files is reported. M1/M2 budgets have not regressed > 10%.
- [ ] T-082 · M3 gate · Tester full pass · owner: tester · depends: T-081
      Accept: All suites green. Report with real output.
- [ ] T-083 · M3 gate · Security audit · owner: security-reviewer · depends: T-082 · **parallel-ok**
      Accept: Covers merging untrusted files, output path handling, temp files and resource limits on huge merges. Verdict given.
- [ ] T-084 · M3 gate · User docs: page tools · owner: docs-writer · depends: T-082 · **parallel-ok**
      Accept: "How to merge", split, extract, reorder, rotate, delete and insert pages. Shortcuts updated.

---

## M4: Forms & sign

- [ ] T-085 · F-01 to F-05, F-08 · Design spec: Forms & Sign · owner: ui-ux-accessibility · depends: T-072 · **parallel-ok**
      Accept: Spec for the Forms & Sign ribbon, fillable-field highlight style, the signature pad (draw / type / import), the saved-signatures manager, flat-form tools, and the non-blocking XFA "unsupported" message.
- [ ] T-086 · F-01, F-02, F-08, X-06 · FormService · owner: rust-core-developer · depends: T-053, T-032
      Accept: Form-fill environment with JavaScript off. Enumerates fields and widgets (type, rect, value, options, flags, tab order). Sets values for text, checkbox, radio, combo and list, and regenerates appearances. Clear-form is undoable. An XFA document returns a typed "unsupported" result and never crashes. Tests on corpus forms.
- [ ] T-087 · F-01, F-02, F-08 · Form field overlay · owner: frontend-developer · depends: T-086, T-085
      Accept: Inputs positioned over the widgets at every zoom level. Tab/Shift+Tab follow the field order. "Highlight fields" toggle and Clear form. Accessible names come from the field tooltips (TU). XFA files show the non-blocking banner. The Forms module is lazy-loaded.
- [ ] T-088 · F-03 · Flat-form tools (core) · owner: rust-core-developer · depends: T-059
      Accept: Click-to-type creates FreeText. Checkmark, cross and dot are standard annotations with appearance streams, visible in other viewers. Round-trip tests included.
- [ ] T-089 · F-03 · Flat-form tool UI · owner: frontend-developer · depends: T-088, T-087
      Accept: Click anywhere to type. Check, cross and dot tools. Quick size adjustment. Undoable.
- [ ] T-090 · F-04, F-05 · Signature placement and DPAPI store · owner: rust-core-developer · depends: T-088
      Accept: A signature is inserted as an image Stamp annotation with an appearance stream. Signatures and initials are stored under `%APPDATA%\SwiftPDF` encrypted with DPAPI (CurrentUser), with list and delete. Nothing is logged. Tests include a DPAPI round trip.
- [ ] T-091 · F-04, F-05 · Signature pad UI · owner: frontend-developer · depends: T-090, T-089
      Accept: Draw (pen pressure), type with fonts already installed with Windows (no bundled fonts, see ROADMAP risk 11), or import PNG/JPG. A saved-signature picker. Place and resize on the page.
- [ ] T-092 · F-01 to F-05, F-08 · Forms and signing tests · owner: tester · depends: T-091
      Accept: Fill each field type → save → reopen and the values persist. The XFA sample shows the message and does not crash. The signature store round-trips. E2E: fill and sign a form. Samples added to the T-105 compatibility set.
- [ ] T-093 · M4 gate · Accessibility audit of forms and signing · owner: ui-ux-accessibility · depends: T-092 · **parallel-ok**
      Accept: Findings ranked with fixes. Forms can be filled with the keyboard only and fields are announced correctly.
- [ ] T-094 · M4 gate, §6.1 · Performance budget gate · owner: performance-engineer · depends: T-092
      Accept: Typing latency in form fields is reported. Opening a form-heavy file meets the first-page budget. No regression > 10% on earlier budgets.
- [ ] T-095 · M4 gate · Tester full pass · owner: tester · depends: T-094
      Accept: All suites green. Report with real output.
- [ ] T-096 · M4 gate · Security audit · owner: security-reviewer · depends: T-095 · **parallel-ok**
      Accept: Covers DPAPI usage, form actions (submit/JS) never executed, the XFA path, and imported-image handling. Verdict given.
- [ ] T-097 · M4 gate · User docs: forms and signing · owner: docs-writer · depends: T-095 · **parallel-ok**
      Accept: "How to fill a form", "How to sign a PDF", flat forms, saved signatures (and where they are stored), XFA limitation. PRIVACY updated.

---

## v1.0 release

- [ ] T-098 · §6.4, §1 license · About dialog, "Check for updates" link, third-party licenses · owner: frontend-developer · depends: T-097
      Accept: Shows the version. "Check for updates" opens the GitHub releases page in the default browser; the app itself makes no request. A viewer lists the third-party licenses (PDFium and all bundled crates/npm packages), generated at build time.
- [ ] T-099 · V-01, §7 installer, §6.1 installer size · MSI and NSIS installers · owner: rust-core-developer · depends: T-098
      Accept: The Tauri bundler produces an MSI (WiX) and an NSIS EXE with the WebView2 bootstrapper and the bundled PDFium DLL. Registers the .pdf file association ("Open with", without taking over the default silently). Per-user install with no unneeded elevation. Uninstall is clean. Installer < 15 MB.
- [ ] T-100 · §6.2 ARM64 · ARM64 build · owner: rust-core-developer · depends: T-099
      Accept: Builds for `aarch64-pc-windows-msvc` with the pinned win-arm64 PDFium checksum. Builds in CI. Documented as untested on ARM64 hardware.
- [ ] T-101 · §6.3 signing, §9 v1.0 · Release workflow · owner: rust-core-developer · depends: T-100
      Accept: On a `v*` tag, builds x64 and ARM64 MSI/EXE with SHA-256 sums and creates a draft GitHub release. Signs when signing secrets exist; otherwise the build is marked unsigned with a warning. **Signing needs a user decision (ROADMAP risk 12); mark `[!]` for the signing part if there is none.**
- [ ] T-102 · §10, X-07 · Corpus at scale and fuzz campaign · owner: tester · depends: T-099
      Accept: A script fetches the pdf.js test suite (Apache-2.0) and PDF Association samples into `D:\dev\swiftpdf-corpus` (not committed). ≥ 1,000 files are opened, rendered and searched with 0 crashes or hangs. A fuzz campaign of ≥ 1 h has 0 crashes. Every failure becomes a fix task with a regression test.
- [ ] T-103 · §10, §6.4 · Network-silence verification · owner: tester · depends: T-102
      Accept: A full E2E + corpus run of the installed build with outbound-connection monitoring (Windows Firewall log or ETW) on the app and its WebView2 processes shows 0 connections. The log is attached to the report.
- [ ] T-104 · §10 · P0 coverage matrix · owner: tester · depends: T-102
      Accept: Every P0 requirement ID in REQUIREMENTS §5 maps to named tests (or a documented manual check). Gaps become new tasks.
- [ ] T-105 · §10, A-09 · Compatibility verification package · owner: tester · depends: T-104
      Accept: Annotated, filled, signed and page-op output files plus a checklist. Edge and Chrome renders are checked automatically where possible (headless screenshots compared with the app's render). Acrobat Reader is a **manual check by the user**, with clear steps.
- [ ] T-106 · §10, §6.1 · Final performance report · owner: performance-engineer · depends: T-103
      Accept: All §6.1 budgets pass on the reference machine using the **installed release build**, plus installer size. The report is a full table.
- [ ] T-107 · §10, AC-01, AC-04, U-01 · Final usability and accessibility audit · owner: ui-ux-accessibility · depends: T-103 · **parallel-ok**
      Accept: Common tasks (open, find, highlight, fill and sign, merge, save) are reachable in ≤ 2 clicks or one shortcut. Full accessibility pass. Blockers become fix tasks.
- [ ] T-108 · §6.3, §10 · Release security audit · owner: security-reviewer · depends: T-106 · **parallel-ok**
      Accept: Covers the installer (elevation, file association), signing state, supply chain (`cargo audit`, `npm audit`, `cargo deny`), network-silence evidence and a re-check of all earlier findings. Release verdict given.
- [ ] T-109 · §9 v1.0 docs · Release documentation · owner: docs-writer · depends: T-106 · **parallel-ok**
      Accept: Final README, complete user guide, PRIVACY.md, shortcuts.md, CHANGELOG, release notes, and known limitations (no EPUB/XPS, no XFA, no certificate signatures, ARM64 untested, signing status, WebView2 install note).
- [ ] T-110 · §9 v1.0, §10 · Cut v1.0.0 release · owner: rust-core-developer · depends: T-101, T-105, T-106, T-107, T-108, T-109
      Accept: Only when all v1.0 gates pass and the security verdict is not "block". Version 1.0.0 is set in Cargo, package.json and tauri.conf. The release workflow runs on the tag and the GitHub release is published with installers and checksums. The §10 definition of done checklist is confirmed in the release notes.
