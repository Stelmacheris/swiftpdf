# PDF Viewer & Editor: Requirements Specification

**Status:** Draft v1.0 · 2026-10-23
**Working name:** *SwiftPDF* (placeholder)
**License:** MIT (all dependencies must be MIT / Apache-2.0 / BSD compatible)

---

## 1. Vision

A **free, open-source, Windows PDF viewer with light editing** that opens files as fast as SumatraPDF
and covers the everyday editing that people currently pay for or put up with Adobe for:
annotate, fill forms, sign, merge/split, protect, redact.

**Rules the product follows:**
1. **Speed first.** Features that are not in use cost nothing at startup.
2. **No nags.** No upsells, no pop-ups, no accounts, no ads.
3. **Private by default.** Zero network traffic unless the user explicitly runs a cloud AI action.
4. **Never lose work.** Incremental saves, autosave backup, crash recovery. The original file is never corrupted.

---

## 2. Market research summary

### 2.1 Popular PDF apps

| App | Strengths | Weaknesses users report |
|---|---|---|
| **Adobe Acrobat Reader** | Most compatible; forms, signatures, accessibility | Installer >500 MB, slow to open, constant Pro upsell, AI-summary pop-ups on every file, usage-data collection, disliked UI redesigns, hard-to-cancel subscriptions |
| **Foxit PDF Reader** | Feature-rich, free, AI assistant | Growing upsell prompts, heavier than it used to be |
| **PDF-XChange Editor** | Strong annotation and OCR | Watermarks on "Pro" features in the free version, cluttered UI |
| **PDFgear** | Free editing | Closed source, less proven |
| **SumatraPDF** | ~5 MB, opens instantly, portable, open source, EPUB/XPS/CBZ | No form editing, no annotations beyond basics, it is a viewer only |
| **Okular** | Open source, annotations, text-to-speech | Weak editing (e.g. no image insertion), feels foreign on Windows |
| **Browser viewers (Chrome/Edge)** | Already installed | Basic tools, tied to the browser |

### 2.2 The gap in the market
Nobody combines **Sumatra-level speed and size** with **Foxit-level everyday editing** and
**no nags or telemetry**. That gap is this product.

### 2.3 Rendering engine choice
- **PDFium** (BSD-3/Apache-2.0): Chrome's engine. Fast, high fidelity, supports annotations, forms and page editing. **Chosen.**
- **MuPDF**: Fastest and supports EPUB/XPS, but **AGPL**, which is incompatible with the MIT goal. Rejected.
- **pdf.js**: Easy in browsers but slower on large or image-heavy files. Rejected for rendering.

Sources: [denser.ai](https://denser.ai/blog/best-pdf-reader/), [online-tech-tips](https://www.online-tech-tips.com/free-pdf-viewer-and-reader/),
[AlternativeTo: Adobe Reader](https://alternativeto.net/software/adobe-reader/), [Adobe community thread](https://community.adobe.com/t5/acrobat-reader-discussions/hate-adobe-acrobat-reader/m-p/11798203),
[Substack: "Screw Adobe Acrobat"](https://majeris.substack.com/p/screw-adobe-acrobat-tip-rant), [Syncfusion engine comparison](https://www.syncfusion.com/blogs/post/pdf-rendering-engines-comparison),
[MuPDF.js FAQ](https://mupdfjs.readthedocs.io/en/latest/faq/index.html).

---

## 3. Decisions log (from the 24-question interview)

| # | Topic | Decision |
|---|---|---|
| 1 | Platforms | Windows desktop only (v1) |
| 2 | Stack | Tauri 2 + Rust + PDFium |
| 3 | License | MIT / Apache, open source |
| 4 | Core focus | Fast viewer + light editing |
| 5 | Viewing | Tabs + session restore, thumbnails + outline, layout modes, dark/night page mode |
| 6 | Annotation | Highlight/underline/strike, notes + text boxes, ink + shapes, stamps + images |
| 7 | Page tools | Merge/split/extract, reorder/rotate/delete, compress, convert (images ↔ PDF, PDF → text) |
| 8 | Forms/sign | AcroForm fill, flat-form typing, simple signature (no certificate signing in v1) |
| 9 | Speed target | Sumatra-class (see §6) |
| 10 | OCR | Optional offline add-on, downloaded on demand |
| 11 | Search | In-doc with all hits, across tabs, folder search, regex/whole-word/case |
| 12 | Privacy | 100% offline, zero telemetry (see AI exception, #22) |
| 13 | UI | Ribbon (Office-like): Home / Comment / Pages / Forms / Protect / View |
| 14 | Theme | Dark by default, light available |
| 15 | Shortcuts | Fully customizable (standard defaults, all remappable) |
| 16 | Accessibility | Read aloud (Windows TTS), screen-reader support, high contrast + DPI scaling, text reflow |
| 17 | Saving | Incremental save + autosave backup |
| 18 | Formats | PDF + images (PNG/JPG/TIFF) + CBZ; DOCX → PDF via Word |
| 19 | Security | Open protected PDFs, add/remove password (AES-256), true redaction, safe mode |
| 20 | Distribution | Installer (MSI/EXE) |
| 21 | EPUB/XPS | Postponed (no MIT-compatible engine); CBZ done in-house |
| 22 | DOCX | Uses Microsoft Word if installed; button hidden otherwise |
| 23 | AI | Optional cloud AI with the user's own API key, **off by default** |
| 24 | Deliverable | Requirements doc first; build later |

> **Conflict resolved (#12 vs #23):** The app makes **no network calls** by default. Cloud AI is a
> disabled-by-default feature. When the user enables it and enters their own key, the app only
> connects when they click an AI action, and it shows exactly what will be sent before the first send.

---

## 4. Users

| Persona | Needs |
|---|---|
| **Everyday user** | Opens bills/tickets/manuals; wants it instant and simple, can fill and sign a form |
| **Student / researcher** | Long papers and books; highlights, notes, search across many files, dark mode |
| **Office worker** | Merge, split, reorder, compress for email, sign, password-protect, redact |
| **Accessibility user** | Screen reader, read aloud, high contrast, reflow |
| **Privacy-conscious user** | Offline, no accounts, no telemetry, open source |

---

## 5. Functional requirements

Priority: **P0** = v1.0 must-have · **P1** = v1.x · **P2** = later

### 5.1 Opening & viewing
| ID | Requirement | Pri |
|---|---|---|
| V-01 | Open PDF via double-click, drag-drop, File > Open, command line, "Open with" | P0 |
| V-02 | Tabs; drag to reorder; middle-click close; Ctrl+Tab cycling | P0 |
| V-03 | Session restore: reopen tabs with page, zoom and scroll position | P0 |
| V-04 | Recent files list with thumbnails on the start screen | P0 |
| V-05 | Sidebar: page thumbnails, outline/bookmarks, annotations list, attachments | P0 |
| V-06 | Layouts: continuous, single page, two-page, book (cover separate) | P0 |
| V-07 | Zoom: fit width, fit page, actual size, 10–6400%, Ctrl+wheel, pinch on touchpads | P0 |
| V-08 | Rotate view (per document, not saved unless chosen) | P0 |
| V-09 | Page colors: normal, dark (inverted, keeps images natural), sepia, custom | P0 |
| V-10 | Full screen and presentation mode | P1 |
| V-11 | Back/forward navigation history (Alt+←/→) for links and jumps | P0 |
| V-12 | Internal and external links; external links ask for confirmation | P0 |
| V-13 | Text selection and copy; select a column with Alt+drag | P0 |
| V-14 | Auto-reload when the file changes on disk (for LaTeX users) | P1 |
| V-15 | Split view: two places of the same document side by side | P2 |
| V-16 | Open images (PNG/JPG/TIFF multi-page) and CBZ comics as documents | P1 |

### 5.2 Search
| ID | Requirement | Pri |
|---|---|---|
| S-01 | Ctrl+F find: highlight all hits, hit count, next/previous | P0 |
| S-02 | Results panel with surrounding text, click to jump | P0 |
| S-03 | Options: match case, whole word, regex, diacritics-insensitive | P0 |
| S-04 | Search across all open tabs | P1 |
| S-05 | Folder search: local full-text index (incremental, background, cancellable) | P1 |
| S-06 | Searches run in the background and never block scrolling | P0 |

### 5.3 Annotation
| ID | Requirement | Pri |
|---|---|---|
| A-01 | Highlight, underline, strikeout, squiggly on text, with a color picker | P0 |
| A-02 | Sticky notes with replies; free-text boxes (font, size, color, border) | P0 |
| A-03 | Freehand ink with pen pressure (Windows Ink); eraser | P0 |
| A-04 | Shapes: line, arrow, rectangle, ellipse, polygon, cloud | P0 |
| A-05 | Stamps: built-in (Approved, Draft, Confidential…), custom image stamps | P1 |
| A-06 | Insert image onto a page | P1 |
| A-07 | Select, move, resize, delete, change properties of any annotation | P0 |
| A-08 | Undo/redo (unlimited per session) | P0 |
| A-09 | Annotations stored as **standard PDF annotations** (visible in Acrobat, browsers) | P0 |
| A-10 | Export annotations/highlights to Markdown/text summary | P1 |
| A-11 | Flatten annotations into page content | P1 |

### 5.4 Page & document tools
| ID | Requirement | Pri |
|---|---|---|
| P-01 | Page organizer grid: drag to reorder, multi-select, rotate, delete, duplicate | P0 |
| P-02 | Insert blank pages or pages from another file | P0 |
| P-03 | Merge multiple PDFs (drag files into the organizer) | P0 |
| P-04 | Split by range, every N pages, by bookmarks; extract selected pages | P0 |
| P-05 | Compress: presets (Screen / eBook / Print) using image downsampling, object streams, deduplication | P1 |
| P-06 | Images → PDF (one or many, page size options) | P1 |
| P-07 | PDF → images (PNG/JPG, chosen DPI) | P1 |
| P-08 | PDF → plain text / Markdown | P1 |
| P-09 | DOCX/DOC/RTF → PDF via installed Microsoft Word (COM automation); hidden if Word absent | P2 |
| P-10 | Edit document properties (title, author, keywords) | P1 |
| P-11 | Edit outline/bookmarks | P2 |
| P-12 | Headers/footers, page numbers, watermark | P2 |

### 5.5 Forms & signatures
| ID | Requirement | Pri |
|---|---|---|
| F-01 | Fill AcroForm fields: text, checkbox, radio, combo, list; Tab moves between fields | P0 |
| F-02 | Highlight fillable fields; "clear form" | P0 |
| F-03 | Flat forms: click anywhere to type (free text), checkmark/cross/dot tools | P0 |
| F-04 | Signature: draw (mouse/pen), type (script fonts), or import image | P0 |
| F-05 | Save signatures and initials locally for reuse (stored encrypted with Windows DPAPI) | P0 |
| F-06 | Date stamp tool | P1 |
| F-07 | Option to flatten the form after filling | P1 |
| F-08 | XFA forms: show a clear "unsupported" message, never crash | P0 |
| F-09 | Digital certificate signatures (PKCS#12) | P2 |

### 5.6 Security & protection
| ID | Requirement | Pri |
|---|---|---|
| X-01 | Open password-protected PDFs (user password prompt) | P0 |
| X-02 | Add password protection with AES-256; set permissions (print/copy/edit) | P1 |
| X-03 | Remove password (when the owner password is known) | P1 |
| X-04 | **True redaction:** mark areas/text → apply → underlying text, images and vector content are removed, not just covered; metadata scrub option | P1 |
| X-05 | Redaction search: "redact all matches of …" (e.g. phone numbers via regex) | P2 |
| X-06 | **Safe mode by default:** never execute PDF JavaScript; no launch actions; external links require confirmation | P0 |
| X-07 | Fuzz-tested parser path; malformed files never crash the app (one tab fails, the app survives) | P0 |

### 5.7 OCR (optional add-on)
| ID | Requirement | Pri |
|---|---|---|
| O-01 | Detect scanned pages (no text layer) and offer "Make searchable" | P1 |
| O-02 | OCR engine downloaded on demand: Tesseract (Apache-2.0), with the user picking language packs | P1 |
| O-03 | Alternative zero-download engine: built-in **Windows.Media.Ocr** (Windows 10+) | P1 |
| O-04 | Adds an invisible text layer; the original look is unchanged | P1 |
| O-05 | Runs in the background with progress and cancel | P1 |

> Exception to "zero network": the OCR download (O-02) only happens when the user clicks it, and the user is shown the URL and size first.

### 5.8 Accessibility
| ID | Requirement | Pri |
|---|---|---|
| AC-01 | All UI keyboard-reachable; visible focus | P0 |
| AC-02 | Screen readers (Narrator, NVDA, JAWS): labelled controls; page text exposed in reading order | P1 |
| AC-03 | Read aloud with offline Windows voices: page / from cursor / selection; speed and voice controls; highlight the word being read | P1 |
| AC-04 | Respect Windows high-contrast mode and text-scaling; per-monitor DPI | P0 |
| AC-05 | Reflow mode: extracted text re-wrapped in a readable column with adjustable font | P1 |
| AC-06 | Tagged-PDF structure used for reading order when present | P2 |

### 5.9 UI & customization
| ID | Requirement | Pri |
|---|---|---|
| U-01 | Ribbon tabs: **Home** (open, save, print, zoom, find), **Comment**, **Pages**, **Forms & Sign**, **Protect**, **View**; collapsible to a single row | P0 |
| U-02 | Command palette (Ctrl+K / Ctrl+Shift+P) to reach every action | P1 |
| U-03 | Dark theme by default; light theme; "follow Windows" option | P0 |
| U-04 | Every shortcut remappable; import/export keymap; conflict warning | P1 |
| U-05 | Optional Vim-style preset (j/k, gg/G, /) | P2 |
| U-06 | Settings stored as a readable JSON file next to the app (portable-friendly) | P0 |
| U-07 | Localization-ready (all strings in resource files); English first | P1 |
| U-08 | Print: page range, scaling, duplex, print annotations on/off | P0 |

### 5.10 Saving & reliability
| ID | Requirement | Pri |
|---|---|---|
| R-01 | Ctrl+S = **incremental save** (append changes, fast even for 500 MB files) | P0 |
| R-02 | "Save As (optimized)" = full rewrite, smaller file | P0 |
| R-03 | Autosave recovery copy every 60 s to `%LOCALAPPDATA%`; offer restore after a crash | P0 |
| R-04 | Saves are atomic: write to a temp file → fsync → rename | P0 |
| R-05 | Unsaved-changes indicator on the tab; prompt on close | P0 |
| R-06 | Handle files that are locked or opened by another program (read-only fallback) | P0 |

### 5.11 Optional cloud AI (off by default)
| ID | Requirement | Pri |
|---|---|---|
| AI-01 | Disabled until the user enables it in Settings and enters their own API key | P2 |
| AI-02 | Provider-agnostic (Anthropic, OpenAI-compatible endpoints); key stored in Windows Credential Manager | P2 |
| AI-03 | Actions: summarize document/selection, ask a question about the document, explain selection | P2 |
| AI-04 | A preview shows exactly what text will be sent before the first request; never automatic, **never pop-ups** | P2 |
| AI-05 | Removing the key or disabling the feature returns the app to zero network | P2 |

---

## 6. Non-functional requirements

### 6.1 Performance budgets (reference machine: 4-core CPU, 8 GB RAM, SSD, Windows 10)
| Metric | Target |
|---|---|
| Cold start to visible window | **< 500 ms** |
| Open 10 MB PDF → first page sharp | **< 200 ms** |
| Open 1,000-page / 300 MB PDF → first page | < 400 ms (no full parse) |
| Scrolling | 60 fps; low-res placeholder shown immediately, sharp tile < 100 ms |
| Zoom | Immediate scaled bitmap, re-render sharp tiles < 150 ms |
| Find over 1,000 pages | First hit < 300 ms, full count streams in the background |
| Memory: idle with 1 small PDF | < 120 MB (includes WebView2) |
| Memory: 1,000-page PDF | < 400 MB (bounded tile cache with LRU eviction) |
| Installer size | < 15 MB (excluding the WebView2 runtime) |
| Incremental save, 1 annotation on 300 MB file | < 300 ms |

Performance tests run in CI on a fixed corpus; any regression over 10% fails the build.

### 6.2 Compatibility
- Windows 10 (1809+) and Windows 11, x64 and ARM64.
- PDF 1.0–2.0, linearized, encrypted (RC4/AES), broken xref (repair on open).
- Files saved by the app open correctly in Acrobat, Chrome and Edge (verified in tests).
- Test corpus: PDF.js test suite, PDF Association samples, real-world forms, scanned docs, huge files, malformed/fuzzed files.

### 6.3 Reliability & security
- Rendering of each document runs on a worker thread; a PDFium crash in one document must not take down others (v1.x: separate render process).
- Continuous fuzzing (cargo-fuzz) on the file-open path.
- No PDF JavaScript, ever. No automatic external connections.
- Signed installer and binaries (code-signing certificate).

### 6.4 Privacy
- No telemetry, analytics, crash uploads or update pings.
- Updates: the user checks GitHub releases manually ("Check for updates" opens the releases page in the browser).
- The only network features (OCR download, cloud AI) are explicit, user-initiated and documented.

---

## 7. Architecture

```
┌──────────────────────── Tauri 2 window (WebView2) ────────────────────────┐
│  UI: TypeScript + Svelte (small, compiled, no virtual DOM)                │
│  Ribbon · Tabs · Sidebar · Canvas view (virtualized pages, tile layer)    │
│  Annotation overlay (SVG/canvas) · Command palette · Settings             │
└───────────────▲──────────────────────────────────────────▲────────────────┘
                │ Tauri commands (JSON, small)             │ custom protocol
                │                                          │ pdf://tile/{doc}/{page}/{zoom}/{x}/{y}
┌───────────────┴──────────────── Rust core ───────────────┴────────────────┐
│  DocumentManager  · RenderScheduler (priority: visible > near > thumbs)    │
│  TileCache (LRU, memory-bounded) · SearchEngine (regex crate, tantivy)     │
│  AnnotationService · FormService · PageOps · SaveService (incremental)     │
│  Security (qpdf for AES-256 + compression; redaction module)               │
│  Platform: Windows TTS, Windows OCR, DPAPI, Credential Manager, Word COM   │
└───────────────▲────────────────────────────────────────────────────────────┘
                │ pdfium-render crate (FFI)
          PDFium binary (bblanchon/pdfium-binaries, BSD/Apache)
```

**Key technical choices**

| Concern | Choice | Why |
|---|---|---|
| Rendering | PDFium via `pdfium-render` | Chrome-grade fidelity and speed; permissive license |
| Pixel transfer | Tauri **custom URI protocol** that serves raw/WebP tiles | Avoids slow JSON/base64 IPC for bitmaps |
| Tiling | 256–512 px tiles, rendered at device pixel ratio, progressive low→high res | Smooth zoom and scroll on huge pages |
| Encryption & compression | `qpdf` (Apache-2.0), linked or bundled | PDFium cannot write encryption; qpdf is the standard |
| Redaction | Remove intersecting page objects; split text objects at glyph level; rasterize-and-replace images only inside the box | True content removal |
| Folder search | `tantivy` (MIT) index in `%LOCALAPPDATA%` | Fast full-text, incremental |
| OCR | Windows.Media.Ocr (built-in) + optional Tesseract download | Zero size cost by default |
| TTS | WinRT `SpeechSynthesizer` | Offline, built into Windows |
| Installer | Tauri bundler → **MSI (WiX)** + NSIS EXE; WebView2 bootstrapper | Standard Windows install, file associations |

**Startup strategy (to hit < 500 ms):** a minimal first-paint bundle (viewer + Home ribbon only), with
editing modules lazy-loaded. The window is shown on first paint, PDFium is loaded in parallel with
WebView2 creation, and the first page renders at low resolution before the sharp pass.

---

## 8. Risks

| Risk | Impact | Mitigation |
|---|---|---|
| WebView2 cold start (~200–400 ms) eats the 500 ms budget | Missed speed goal | Prewarm, minimal bundle, measure early in M1; fallback idea: native first-page splash |
| True redaction of text is hard to get right | Leaked data, legal risk | Dedicated test suite that verifies text extraction returns nothing in redacted areas; "verify redaction" report |
| PDFium form handling edge cases | Broken forms | Large real-world forms corpus; XFA explicitly unsupported |
| Rust not installed on the dev machine | Setup delay | M0 installs rustup + MSVC build tools |
| EPUB/XPS missing compared with Sumatra | Some users disappointed | Clearly documented; revisit an MIT engine later |
| Cloud AI conflicts with the privacy promise | Trust | Off by default, explicit preview, zero network when disabled |

---

## 9. Roadmap

| Milestone | Scope |
|---|---|
| **M0: Setup** | Install Rust + MSVC tools; scaffold Tauri 2 + Svelte; PDFium binaries; CI with perf harness |
| **M1: Fast viewer** | V-01…V-09, V-11–V-13, S-01–S-03, S-06, U-01, U-03, U-06, U-08, R-05, X-01, X-06/07; perf budgets met |
| **M2: Annotate** | A-01–A-04, A-07–A-09, R-01–R-04 (saving, undo, autosave) |
| **M3: Pages** | P-01–P-04, merge/split organizer |
| **M4: Forms & sign** | F-01–F-05, F-08 |
| **v1.0 release** | MSI installer, signed, docs, GitHub release |
| **v1.1** | Compress, convert, stamps/images, protect (X-02/03), redaction (X-04), search across tabs/folder |
| **v1.2** | OCR add-on, read aloud, reflow, screen reader, command palette, keymap editor, CBZ/images |
| **v1.3+** | DOCX via Word, cloud AI, headers/watermarks, digital signatures, EPUB investigation |

---

## 10. Definition of done for v1.0
- All P0 requirements implemented and covered by tests.
- Every performance budget in §6.1 met on the reference machine.
- Corpus test: 0 crashes on 1,000+ real-world and fuzzed PDFs.
- Annotated/filled/signed files verified to display correctly in Acrobat Reader, Chrome and Edge.
- No network connections observed during a full test run (verified with a firewall log).
