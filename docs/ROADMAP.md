# SwiftPDF Roadmap

Source of truth: `docs/REQUIREMENTS.md` (§9 roadmap, §10 definition of done).
Task breakdown: `docs/TASKS.md`. Owner of this file: project-manager.

_Last updated: 2026-10-23_

## Milestones

| Milestone | Scope (requirement IDs) | Tasks | Status |
|---|---|---|---|
| **M0: Setup** | Toolchain, Tauri 2 + Svelte + TS scaffold, PDFium fetch + smoke test, test corpus, perf harness, CI, license policy, early design specs | T-001 to T-016 | In progress (toolchain partly installed, see below) |
| **M1: Fast viewer** | V-01 to V-09, V-11 to V-13, S-01 to S-03, S-06, U-01, U-03, U-06, U-08, R-05, X-01, X-06, X-07, plus AC-01 and AC-04 (P0, not placed in §9 but required by §10) | T-017 to T-050 | Not started |
| **M2: Annotate** | A-01 to A-04, A-07 to A-09, R-01 to R-04, plus R-06 (P0, not placed in §9 but required by §10) | T-051 to T-071 | Not started |
| **M3: Pages** | P-01 to P-04 (organizer, merge, split, extract) | T-072 to T-084 | Not started |
| **M4: Forms & sign** | F-01 to F-05, F-08 | T-085 to T-097 | Not started |
| **v1.0 release** | MSI/EXE installer, signing, ARM64 build, docs, GitHub release, §10 definition of done | T-098 to T-110 | Not started |
| v1.1 | Compress, convert, stamps/images, protect (X-02/03), redaction (X-04), search across tabs/folder | not broken down | Not planned yet |
| v1.2 | OCR add-on, read aloud, reflow, screen reader, command palette, keymap editor, CBZ/images | not broken down | Not planned yet |
| v1.3+ | DOCX via Word, cloud AI, headers/watermarks, digital signatures, EPUB investigation | not broken down | Not planned yet |

### Environment already in place (not re-planned)
- Reference machine for §6.1 budgets: Windows 10 Pro, Ryzen 5 2600 (6 cores), 16 GB RAM.
- Rust 1.99.1 stable (MSVC) at `D:\dev\rust`; `CARGO_TARGET_DIR=D:\dev\swiftpdf-target`.
- MSVC C++ Build Tools + Windows SDK: **installing now**. T-004 cannot finish until this is done.
- WebView2 runtime installed. Node 18.12.1 / npm 9.6, so frontend tools must support Node 18.12 (Vite 5.x).
- Git repo on `main`, auto-pushed to https://github.com/Stelmacheris/swiftpdf after every agent.

## Milestone exit gates (every milestone)
1. `ui-ux-accessibility` audit (design and accessibility; blockers become fix tasks).
2. `performance-engineer` budget check against §6.1 (pass/fail table; no regression over 10%).
3. `tester` full pass (Rust, frontend, E2E, corpus), run after any perf changes.
4. `security-reviewer` audit with a verdict: pass / pass with fixes / block.
5. `docs-writer` update for the features shipped.

A milestone is **done** only when all five gates pass. Gate findings become new tasks, appended to the end of
that milestone's section in TASKS.md with the next free IDs. They are dispatched before the next milestone starts.

## Dispatch rules
- Execute TASKS.md top to bottom. A task can start once all tasks in its `depends` list are `[x]`.
- One code-writing agent at a time: rust-core-developer, frontend-developer, performance-engineer, tester.
- Tasks tagged `parallel-ok` (owned by ui-ux-accessibility, security-reviewer, docs-writer) may run beside a code agent.
- If a task is blocked, mark it `[!]` with the reason and continue with the next task whose dependencies are met.

## Risks and watch items

| # | Risk | Mitigation / owner | Where |
|---|---|---|---|
| 1 | **WebView2 cold start eats the 500 ms budget** (top risk, §8) | Measure an empty window in M0 (T-005), right after the scaffold. Prewarm, minimal first-paint bundle, show window on first paint. Fallback: native splash. If the empty window median is > 300 ms, PM escalates before M1 view work | T-005, T-019, T-026, T-047 |
| 2 | MSVC Build Tools still installing | T-004 waits for it. Design tasks T-001 to T-003 can run meanwhile | T-004 |
| 3 | Project folder is inside OneDrive and C: has ~15 GB free | Build output already on D:. Large generated corpus goes to `D:\dev\swiftpdf-corpus` (gitignored). `node_modules` (~200 MB) will sit in the synced folder: the user may want to exclude it from OneDrive sync or pause sync during installs | T-004, T-008 |
| 4 | Perf checks in CI on GitHub hosted runners are noisy, but §6.1 requires CI to fail on regressions over 10% | Hosted CI is warn-only in M0. The performance-engineer measures variance in M1 and makes the check blocking once stable (T-047). Budgets are judged on the reference machine | T-010, T-012, T-047 |
| 5 | Reference machine (6 cores / 16 GB) is stronger than §6.1's 4-core / 8 GB | Proposal (not a task): performance-engineer also reports results limited to 4 cores via CPU affinity at gates | T-047, T-106 |
| 6 | PDFium build with V8/XFA would allow PDF JavaScript | Pin the **non-V8** bblanchon build only; T-032 asserts it | T-007, T-032 |
| 7 | WebView2 itself may contact Microsoft services (SmartScreen, etc.), which breaks "zero network" (§6.4, §10) | Harden WebView2 options in M1 (T-019); verify with connection monitoring (T-103) | T-019, T-103 |
| 8 | cargo-fuzz needs a nightly toolchain and libFuzzer, which is weak on Windows MSVC | Tester tries cargo-fuzz first. Fallback: a stable mutation harness seeded from the corpus. Report which one is used | T-044, T-102 |
| 9 | Incremental save < 300 ms on a 300 MB file conflicts with atomic temp-copy-rename (R-01 vs R-04) | rust-core-developer measures; if over budget, a design proposal goes to the user before closing T-054 | T-054, T-067 |
| 10 | Encrypted test files need a writer (qpdf is Apache-2.0 but not installed) | Test-only download of qpdf with a pinned checksum, never shipped in M1 to M4 | T-037, T-066 |
| 11 | Typed signatures need script fonts; bundled OFL fonts are outside the MIT/Apache/BSD allowlist | Use fonts already installed with Windows (Segoe Script, Ink Free). Bundling needs user approval | T-091 |
| 12 | **Code-signing certificate** (§6.3) needs a purchase or an OSS signing programme (e.g. SignPath Foundation): **user action** | Release workflow signs when secrets exist; otherwise ships unsigned with a warning. Blocks a "signed" v1.0 until the user decides | T-101 |
| 13 | §10 needs checks in Acrobat Reader, Chrome and Edge | Chrome/Edge headless screenshots automated where possible; Acrobat check is **manual by the user** from a prepared package | T-105 |
| 14 | ARM64 (§6.2) cannot be tested on the reference machine | Build-only in CI, documented as untested on hardware | T-100 |
| 15 | U-06 "settings next to the app" is not writable under Program Files | Portable mode (next to exe when writable plus a marker file), otherwise `%APPDATA%\SwiftPDF`. Flagged as a decision in T-039 | T-039 |
| 16 | WebView2 installer bootstrapper downloads the runtime at install time | Allowed (installer, not app); offline installer variant noted in release docs | T-099, T-109 |

## Decisions log (project management)
- 2026-10-23: AC-01, AC-04 placed in M1 and R-06 in M2. They are P0 requirements that §9 did not assign to a milestone, and §10 requires every P0.
- 2026-10-23: The PDFium build is pinned to the non-V8, non-XFA variant (supports X-06 safe mode).
- 2026-10-23: The core is planned as a separate Rust library crate, so it can be tested without a WebView.

## Proposals awaiting user approval (not tasks)
- 4-core CPU-affinity runs at perf gates (risk 5).
- Bundling script fonts for typed signatures (risk 11).
- Code-signing route: buy a certificate or apply to SignPath Foundation (risk 12).
