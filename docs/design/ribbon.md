# Design spec: ribbon (T-001, part 2)

Covers U-01 (six tabs, collapsible to one row), plus the requirement IDs each command implements. Window context is in
`shell.md`; command IDs match `keymap.md` where a shortcut exists. Release column: **M1** = v1.0 viewer milestone (live in
Home/View first-paint bundle), **v1.0** = later v1.0 milestone, **v1.x** = after v1.0 (P1), **later** = P2.

Rules: a command that is not yet implemented is **not shown** (no greyed-out teasers, no "coming soon"). Commands that
need an open document are disabled when none is open. No promotional buttons, upgrade links or badges.

## 1. Structure

```
 Tab row:   Home | Comment | Pages | Forms & Sign | Protect | View            [collapse v] [Help]
            ^^^^                                            ^^^^
            first-paint bundle (live at start)       first-paint bundle
            Comment, Pages, Forms & Sign, Protect: lazy chunks, loaded on first activation or on hover/focus idle
 Command row:  | Group A       | Group B      | Group C   |   <- group label under each group (11px, --text-muted)
               | [big][big]    | [sm][sm]     | [sm][sm]  |
```

- Command sizes: large (icon above label, 56 px tall) for the 2 to 4 most-used commands per tab, small (icon + label inline, 28 px; target 32 px) for the rest. All commands have a visible text label in the expanded ribbon.
- Hit target 32 px minimum; icons 16 px (small) / 24 px (large).
- A dropdown command is a split button when the primary action is common (Save, Print), otherwise a menu button.
- Tooltip (hover and keyboard focus after 600 ms, `Escape` dismisses): "Open (Ctrl+O)". The shortcut shown is the user's current binding (U-04).

## 2. Keyboard and ARIA

- Ribbon tab row: `role="tablist"` "Ribbon tabs"; Left/Right moves, Home/End to first/last, Enter or Space or Down moves into the command row. `Ctrl+F1` toggles collapse.
- Each group: `role="toolbar"` with `aria-label="<group name>"`, roving tabindex (one Tab stop per group), Left/Right between commands, Home/End. Tab leaves the group to the next group; Shift+Tab reverse.
- Menu buttons: `aria-haspopup="menu"`, `aria-expanded`; menu is `role="menu"` with `menuitem`, `menuitemradio`, `menuitemcheckbox`; Up/Down, Enter/Space activates, Escape closes and returns focus to the button.
- Toggle commands use `aria-pressed`. Disabled commands use `aria-disabled="true"` and stay focusable.
- Tab content panel: `role="tabpanel"`, `aria-labelledby` the tab.
- Lazy tabs: while a chunk loads, the tab's panel shows "Loading..." in a polite live region; the load is expected under 100 ms from disk.

## 3. Collapsed single-row ribbon

```
Expanded (2 rows):
+-------------------------------------------------------------------------------+
| Home | Comment | Pages | Forms & Sign | Protect | View            [v]  [?]      |
+-------------------------------------------------------------------------------+
| File | Navigate | Zoom | Find | Tools ...                                      |
+-------------------------------------------------------------------------------+

Collapsed (1 row, 36 px):
+-------------------------------------------------------------------------------+
| Home | Comment | Pages | Forms & Sign | Protect | View     [Open][Save][Print][Find][-][100%][+]  [^]  |
+-------------------------------------------------------------------------------+
                                                      ^ quick strip, fixed 7 items
```

- Collapse by the `[v]` button, `Ctrl+F1`, or double-click on a ribbon tab.
- When collapsed, activating a tab (click, Enter, or Down arrow) opens that tab's commands as a **temporary panel** overlaying the top of the canvas. It closes after running a command, on `Escape`, on click outside, or when focus leaves. Focus returns to the tab.
- The quick strip (Open, Save, Print, Find, Zoom out, Zoom level, Zoom in) is always visible when collapsed. These are the same command IDs as in Home.
- `aria-expanded` on the collapse button; state announced "Ribbon collapsed" / "Ribbon expanded" (polite).
- Overflow: when the window is narrow, the quick strip loses items right-to-left into a "More" menu.

## 4. Tabs, groups and commands

Req = requirement implemented. "ID" is the stable command ID (keymap.md).

### 4.1 Home (first-paint bundle) (M1)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| File | Open (large) | `file.open` | V-01 | M1 |
| File | Open recent (menu) | `file.openRecent` | V-04 | M1 |
| File | Save (split: Save / Save As / Save optimized copy) | `file.save`, `file.saveAs`, `file.saveOptimized` | R-01, R-02 | v1.0 (M2) |
| File | Print (large) | `file.print` | U-08 | M1 |
| Navigate | Back | `nav.back` | V-11 | M1 |
| Navigate | Forward | `nav.forward` | V-11 | M1 |
| Navigate | First, Previous, Page box, Next, Last | `nav.firstPage`, `nav.prevPage`, `nav.goToPage`, `nav.nextPage`, `nav.lastPage` | V-01, V-11 | M1 |
| Zoom | Zoom out, Zoom level combobox (10 to 6400%), Zoom in | `view.zoomOut`, `view.zoomSet`, `view.zoomIn` | V-07 | M1 |
| Zoom | Fit width, Fit page, Actual size | `view.fitWidth`, `view.fitPage`, `view.actualSize` | V-07 | M1 |
| Find | Find (opens find bar) | `search.open` | S-01, S-03 | M1 |
| Find | Find next / previous | `search.next`, `search.prev` | S-01 | M1 |
| Tools | Select text | `tool.select` | V-13 | M1 |
| Tools | Hand (pan) | `tool.hand` | V-07 | M1 |
| Tools | Copy | `edit.copy` | V-13 | M1 |

### 4.2 Comment (lazy chunk) (v1.0 / M2)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| Markup | Highlight (large, with color menu), Underline, Strikeout, Squiggly | `ann.highlight`, `ann.underline`, `ann.strikeout`, `ann.squiggly` | A-01 | M2 |
| Notes | Sticky note, Text box | `ann.note`, `ann.textBox` | A-02 | M2 |
| Draw | Pen (pressure), Eraser | `ann.ink`, `ann.eraser` | A-03 | M2 |
| Draw | Line, Arrow, Rectangle, Ellipse, Polygon, Cloud (shapes menu) | `ann.line`, `ann.arrow`, `ann.rect`, `ann.ellipse`, `ann.polygon`, `ann.cloud` | A-04 | M2 |
| Insert | Stamp (menu: built-in, custom) | `ann.stamp` | A-05 | v1.1 |
| Insert | Image | `ann.image` | A-06 | v1.1 |
| Edit | Select annotation / properties pane | `ann.select`, `ann.properties` | A-07 | M2 |
| Edit | Undo, Redo | `edit.undo`, `edit.redo` | A-08 | M2 |
| Manage | Show annotations list (opens sidebar panel) | `sidebar.annotations` | A-07, V-05 | M2 |
| Manage | Export summary (Markdown/text) | `ann.exportSummary` | A-10 | v1.1 |
| Manage | Flatten annotations | `ann.flatten` | A-11 | v1.1 |

### 4.3 Pages (lazy chunk) (v1.0 / M3)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| Organize | Page organizer (large; switches canvas to the grid) | `pages.organizer` | P-01 | M3 |
| Edit | Rotate left, Rotate right, Delete, Duplicate | `pages.rotateLeft`, `pages.rotateRight`, `pages.delete`, `pages.duplicate` | P-01 | M3 |
| Insert | Blank page, Pages from file | `pages.insertBlank`, `pages.insertFromFile` | P-02 | M3 |
| Combine | Merge files | `pages.merge` | P-03 | M3 |
| Combine | Split (by range / every N / bookmarks), Extract pages | `pages.split`, `pages.extract` | P-04 | M3 |
| Convert | Images to PDF, PDF to images, PDF to text/Markdown | `convert.imagesToPdf`, `convert.pdfToImages`, `convert.pdfToText` | P-06, P-07, P-08 | v1.1 |
| Convert | Word to PDF (**hidden unless Word is installed**) | `convert.wordToPdf` | P-09 | later |
| Optimize | Compress (Screen / eBook / Print) | `pages.compress` | P-05 | v1.1 |
| Document | Properties (title, author, keywords) | `doc.properties` | P-10 | v1.1 |
| Document | Edit bookmarks | `doc.bookmarks` | P-11 | later |
| Document | Header/footer, page numbers, watermark | `pages.headerFooter` | P-12 | later |
| Text recognition | Make searchable (OCR), OCR language packs | `ocr.run`, `ocr.languages` | O-01..O-05 | v1.2 |

### 4.4 Forms & Sign (lazy chunk) (v1.0 / M4)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| Fill | Highlight fillable fields (toggle) | `form.highlightFields` | F-02 | M4 |
| Fill | Clear form | `form.clear` | F-02 | M4 |
| Fill | Add text (flat form), Checkmark, Cross, Dot | `form.addText`, `form.checkmark`, `form.cross`, `form.dot` | F-03 | M4 |
| Sign | Sign (large; draw / type / image), Initials | `sign.signature`, `sign.initials` | F-04, F-05 | M4 |
| Sign | Manage saved signatures | `sign.manage` | F-05 | M4 |
| Sign | Date stamp | `sign.date` | F-06 | v1.1 |
| Finish | Flatten form | `form.flatten` | F-07 | v1.1 |
| Certificate | Digital signature (PKCS#12) | `sign.certificate` | F-09 | later |

XFA documents (F-08): this tab shows an inline notice in the canvas, not in the ribbon.

### 4.5 Protect (lazy chunk) (v1.1)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| Password | Add password / permissions | `protect.addPassword` | X-02 | v1.1 |
| Password | Remove password | `protect.removePassword` | X-03 | v1.1 |
| Redact | Mark for redaction (area / text), Apply redactions | `redact.mark`, `redact.apply` | X-04 | v1.1 |
| Redact | Redact all matches (regex) | `redact.findAll` | X-05 | later |
| Redact | Scrub metadata | `redact.scrubMetadata` | X-04 | v1.1 |
| Status | Safe mode (read-only status button; explains what is blocked) | `protect.safeModeInfo` | X-06 | M1 |

The six tabs are always shown so the tab row never changes between releases. A tab shows only the commands that exist in the current build.

### 4.6 View (first-paint bundle) (M1)

| Group | Command | Command ID | Req | Release |
|---|---|---|---|---|
| Layout | Continuous, Single page, Two-page, Book (cover separate) | `view.layoutContinuous`, `view.layoutSingle`, `view.layoutTwo`, `view.layoutBook` | V-06 | M1 |
| Zoom | Zoom level, Fit width, Fit page, Actual size | `view.zoomSet`, `view.fitWidth`, `view.fitPage`, `view.actualSize` | V-07 | M1 |
| Rotate | Rotate view left, Rotate view right (not saved) | `view.rotateLeft`, `view.rotateRight` | V-08 | M1 |
| Page color | Normal, Dark, Sepia, Custom... | `view.pageColorNormal`, `view.pageColorDark`, `view.pageColorSepia`, `view.pageColorCustom` | V-09 | M1 |
| Show | Sidebar (toggle), panel: Thumbnails / Outline / Annotations / Attachments | `view.toggleSidebar`, `sidebar.thumbnails`, `sidebar.outline`, `sidebar.annotations`, `sidebar.attachments` | V-05 | M1 |
| Show | Status bar (toggle), Ribbon collapse | `view.toggleStatusBar`, `view.toggleRibbon` | U-01 | M1 |
| App theme | Dark, Light, Follow Windows | `theme.dark`, `theme.light`, `theme.system` | U-03 | M1 |
| Window | Full screen | `view.fullscreen` | V-10 | v1.0 |
| Window | Presentation mode | `view.presentation` | V-10 | v1.x |
| Reading | Read aloud (page / from cursor / selection), Voice and speed | `tts.page`, `tts.fromCursor`, `tts.selection`, `tts.settings` | AC-03 | v1.2 |
| Reading | Reflow | `view.reflow` | AC-05 | v1.2 |
| Reading | Auto-reload on change (toggle) | `view.autoReload` | V-14 | v1.x |

### 4.7 Cross-cutting items (not ribbon tabs)

| Item | Where | Req |
|---|---|---|
| Command palette | `Ctrl+K`, `Ctrl+Shift+P`; icon `[?]` area has a "Commands" button | U-02 (v1.2) |
| Settings, keymap editor | App menu `[=]` in the tab bar and palette | U-04, U-06 |
| Check for updates | App menu; opens the GitHub releases page in the system browser (6.4) | n/a |
| Cloud AI | Appears **only after the user enables it in Settings** (AI-01): an extra "AI" tab at the end, with Summarize, Ask, Explain. No hint of it exists in the UI while disabled. | AI-01..05 (later) |

## 5. Rules checked against this spec

- Every common task is within 2 clicks or one shortcut: open (`Ctrl+O`, 1 click), find (`Ctrl+F`), highlight (Comment, 1 click after selecting + `ann.highlight`), fill and sign (Forms & Sign > Sign, 2 clicks), merge (Pages > Merge, 2 clicks), save (`Ctrl+S`, 1 click).
- Home and View are the only tabs in the first-paint bundle; the others are lazy (section 7 of REQUIREMENTS, startup strategy).
- No command in this spec contains upsell, "Pro", "Upgrade", "Try", "Learn more" or AI pop-up UI.
