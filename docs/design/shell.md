# Design spec: app shell (T-001, part 1)

Covers U-01, U-03, AC-01, AC-04 and the shell parts of V-02, V-05. The ribbon contents are in `ribbon.md`; tokens in
`theme.md`; shortcuts and focus order in `keymap.md`.

Principles: the document is the product, so chrome is quiet and small. **No promotional UI, no pop-ups, no modal
"tips", no banners that ask for anything.** Messages are inline and dismissible, and never steal focus (except dialogs the
user asked for, such as the password prompt).

## 1. Window layout (default, 1280 x 800, dark theme, ribbon expanded)

```
+--------------------------------------------------------------------------------------------+
| [=] report.pdf  x | invoice.pdf * x | manual.pdf  x | +                        [-] [o] [x] | <- Tab bar (36px)
+--------------------------------------------------------------------------------------------+
| Home | Comment | Pages | Forms & Sign | Protect | View                   [v collapse]  [?]   | <- Ribbon tab row (32px)
+--------------------------------------------------------------------------------------------+
| File        | Navigate          | Zoom                     | Find       | Tools            | <- Ribbon commands
| [Open][Save]| [<][3 / 120][>]   | [-] [100% v] [+] [Fit v] | [ Find   ] | [Select][Hand]   |    (~80px)
| [Print]     |                   |                          |            |                  |
+--------------------------------------------------------------------------------------------+
| Find: [ search text            ] [Aa] [W] [.*]   3 of 17   [^] [v]  [Results]        [x]   | <- Find bar (only when open, 40px)
+----------+---------------------------------------------------------------------------------+
| Sidebar  |                                                                                 |
| (Thumbs) |        +------------------------+                                               |
| [T][O]   |        |                        |                                               |
| [A][P]   |        |      Page canvas       |   <- Page canvas: scrollable, virtualised     |
|          |        |      (rendered tiles)  |      pages on --bg-canvas                     |
| +------+ |        |                        |                                               |
| | pg 1 | |        |                        |                                               |
| +------+ |        +------------------------+                                               |
| | pg 2 | |                                                                                 |
| +------+ |                                                                                 |
+----------+---------------------------------------------------------------------------------+
| Page 3 of 120   |   Saved   |   Safe mode   |   [Continuous v]   [-----o-----] 100%   | <- Status bar (24px)
+--------------------------------------------------------------------------------------------+
```

Legend: `*` unsaved dot in the tab; `[T][O][A][P]` = sidebar panel switcher (Thumbnails, Outline, Annotations, Attachments).

### 1.1 Regions and landmarks

| # | Region | Element / role | Accessible name | Notes |
|---|---|---|---|---|
| 1 | Tab bar | `div role="tablist"` inside `header` | "Open documents" | each tab `role="tab"`, `aria-selected` |
| 2 | Ribbon | `div role="toolbar"` per group (see ribbon.md) in `nav`-less `section` | "Ribbon" | tab row is `role="tablist"` "Ribbon tabs" |
| 3 | Find bar | `div role="search"` | "Find in document" | rendered only while open (not in first-paint DOM) |
| 4 | Sidebar | `aside` | "Navigation panel" | panel switcher is a `role="tablist"` (vertical) |
| 5 | Page canvas | `main` with `role="document"`-like scroller | "Document pages" | scroller is focusable (`tabindex="0"`) |
| 6 | Status bar | `footer` (`role="status"` only on the saved/search text) | "Status" | not a live region as a whole |

Every region is reachable with **F6 / Shift+F6** (see keymap.md section 4). A visually-hidden skip link "Skip to document"
is the first Tab stop; it appears only when focused (no persistent clutter).

## 2. Tab bar (V-02)

```
+--------------------------------------------------------------------------------------------+
| [=] | report.pdf   x | invoice.pdf  * x | manual.pdf   x | [+]                   [-] [ ] [x] |
+--------------------------------------------------------------------------------------------+
   ^          ^                  ^    ^
   |          |                  |    close button (also middle-click)
   |          |                  unsaved-changes dot + text "Unsaved" in tooltip and accessible name
   |          file name (ellipsis in the middle, full path in tooltip)
   app menu (File commands overflow, Settings, About, Check for updates)
```

- Height 36 px; tabs 120 to 220 px wide, shrinking to 48 px (icon + close) as the count grows; then a left/right scroll chevron appears (buttons, not auto-hidden scrollbar).
- Active tab uses `--bg-raised` plus a 3 px `--selected-bar` on top; inactive tabs `--bg-app`. Unsaved dot is `--warn`-free, uses `--text` and has the text equivalent in the name.
- Accessible name: `"report.pdf, unsaved changes"`; the close button: `"Close report.pdf"`.
- Drag to reorder; keyboard equivalent: `Ctrl+Shift+PageUp / PageDown` moves the focused tab left/right (announced "report.pdf moved to position 2 of 5").
- Arrow keys move between tabs (roving tabindex), Enter/Space activates, Delete closes. Context menu key: Close, Close others, Close tabs to the right, Copy path, Show in folder.
- The "+" button opens the file dialog (same as Ctrl+O). It is not a new blank tab.
- Close with unsaved changes (R-05): an inline-anchored dialog (modal because data may be lost) with **Save**, **Don't save**, **Cancel**; default focus on Save; Escape = Cancel.

## 3. Ribbon area (U-01)

Expanded: tab row (32 px) plus command row (about 80 px, 3 rows of small commands or 1 row of large ones). Collapsed:
one row (see ribbon.md section 3). The state persists in settings (`ribbon.collapsed`). Full content: `ribbon.md`.

## 4. Sidebar (V-05)

```
+---------------------+
| [T] [O] [A] [P]   x |   <- switcher row 32px; x = hide (F4)
+---------------------+
| Thumbnails          |
| +-----------------+ |
| |   [ page 1 ]    | |
| |       1         | |
| +-----------------+ |
| |   [ page 2 ]    | |
| +-----------------+ |
+---------------------+
```

- Default width 260 px, user-resizable 180 to 480 px by dragging the 6 px splitter, or by keyboard: focus the splitter (`role="separator"`, `aria-orientation="vertical"`, `aria-valuenow/min/max`), Left/Right = 16 px, Home/End = min/max.
- Panels: Thumbnails (V-05), Outline/bookmarks, Annotations list, Attachments. Panels beyond Thumbnails and Outline are lazy-loaded chunks. Each panel's detailed spec is in the T-017 viewer spec.
- Hidden with F4 (toggle). When hidden, the canvas takes the full width and focus returns to the canvas. State per window, persisted.
- Empty panel states are one calm line: "This document has no bookmarks." No illustrations, no suggestions to upgrade.

## 5. Page canvas

- Scrollable region on `--bg-canvas`, pages centered with a 1 px `--page-border` and 12 px gap; no drop shadow required.
- Focusable (`tabindex="0"`) with `aria-label="Document pages, page 3 of 120"` updated on page change (debounced; not announced while scrolling).
- Each rendered page is `role="img"` with `aria-label="Page 3"` until real text exposure (AC-02, P1) replaces it with a text layer in reading order.
- Loading: low-res placeholder instantly (V-07/section 6.1 of requirements), sharp tiles replace it. No spinners over the page.
- Errors (bad file, unsupported XFA, wrong password) show an inline panel centered in the canvas (see section 8).

## 6. Status bar (24 px)

```
| Page [ 3 ] of 120  |  Saved  |  Safe mode  |  Layout: Continuous v  |  [-]--o--[+] 100%  |
```

| Item | Content | Accessible behaviour |
|---|---|---|
| Page box | editable number + "of N" | `aria-label="Current page"`; Enter jumps; Escape restores |
| Save state | "Saved" / "Unsaved changes" / "Saving..." / "Recovered copy" | `role="status"` (polite), updates only on change |
| Safe mode | text "Safe mode" with a shield icon; click opens a read-only explanation popover | static; X-06 is always on in v1 so it is informational |
| Layout | menu button for layout mode | name "Page layout, Continuous" |
| Zoom | slider (10 to 6400, log scale) plus % combobox | `role="slider"` with `aria-valuetext="100 percent"` |

Mouse-wheel over the status bar does nothing. The bar never shows ads, tips or "what's new".

## 7. Start screen (no document open) (V-04)

```
+--------------------------------------------------------------------------------------------+
| Tab bar (empty)  |  Ribbon (Home, View active; document commands disabled)                  |
+--------------------------------------------------------------------------------------------+
|                                                                                            |
|                       Drop a PDF here, or press Ctrl+O to open one.                        |
|                                                                                            |
|   Recent files                                                          [Clear list]       |
|   +--------+ +--------+ +--------+ +--------+ +--------+                                  |
|   | thumb  | | thumb  | | thumb  | | thumb  | | thumb  |   <- list of recent files        |
|   | name   | | name   | | name   | | name   | | name   |      (links; Enter opens)        |
|   | folder | | folder | | folder | | folder | | folder |                                  |
|   +--------+ +--------+ +--------+ +--------+ +--------+                                  |
|                                                                                            |
+--------------------------------------------------------------------------------------------+
```

One sentence of guidance, the recent files, nothing else. Detailed behaviour (arrow-key grid, remove-from-list, missing-file state) is specified in the T-017 viewer spec.

## 8. Empty and error states (shell level)

All are inline panels in the canvas area, `role="alert"` only for failures that block the document, otherwise `role="status"`. They never open modal dialogs.

```
+------------------------------------------------------+
| (!)  This file could not be opened                   |
|      invoice.pdf is damaged or is not a PDF.         |
|      [Close tab]   [Show in folder]                  |
+------------------------------------------------------+
```

| State | Message | Actions |
|---|---|---|
| Bad / malformed file (X-07) | "This file could not be opened. <name> is damaged or is not a PDF." | Close tab, Show in folder. Other tabs unaffected. |
| Unsupported XFA form (F-08) | "This form uses XFA, which is not supported. The page content may be incomplete." | Dismiss |
| File locked (R-06) | "Opened read-only because another program is using it." | Dismiss |
| Missing file in session restore | "invoice.pdf was moved or deleted." | Close tab, Locate... |
| Recovery after crash (R-03) | inline bar on top of canvas: "A recovered copy of X from 14:32 is available." | Restore, Discard |

## 9. Responsive and accessibility behaviour

| Condition | Behaviour |
|---|---|
| Window width < 900 px | ribbon groups collapse right-to-left into a "More" overflow menu; sidebar auto-hides (F4 shows it as overlay) |
| 200 to 300% text scale | ribbon rows wrap to a max of 3 rows, then overflow; tab bar scrolls with chevrons; status bar can wrap to 2 rows; no clipped text |
| Per-monitor DPI change | re-render tiles for the new device pixel ratio; UI uses `rem`, icons are SVG |
| `forced-colors: active` | system colors per `theme.md` section 5 |
| `prefers-reduced-motion` | no transitions or smooth scroll (`theme.md` section 7) |
| Narrator / NVDA | landmarks above; tabs, toolbars and menus use the ARIA patterns in ribbon.md; save-state and search results announced politely |

## 10. First-paint bundle (section 7 of REQUIREMENTS)

In the first-paint bundle: tab bar, ribbon tab row, **Home** and **View** tab content, status bar, canvas and the
Thumbnails/Outline panels, theme CSS. Lazy-loaded: the Comment, Pages, Forms & Sign and Protect tab content, the find bar's
results panel, the annotation/attachment panels, command palette, settings, and all dialogs except the unsaved-changes prompt.
