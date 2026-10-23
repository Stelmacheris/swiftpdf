# Design spec: default keymap and focus order (T-003)

Covers AC-01, U-04, V-02, V-07, V-11, S-01, U-08. Command IDs here are **stable**: the command registry (T-043), the
ribbon (`ribbon.md`), the command palette (U-02) and the future keymap editor (U-04) all use them. Never rename an ID;
add a new one and keep the old as an alias.

## 1. Conventions

- Chord notation: `Ctrl+Shift+S`. Modifier order is always Ctrl, Alt, Shift. Arrow keys: `Left`, `Right`, `Up`, `Down`.
- **Scope** decides where a chord is active:
  - `G` global: anywhere in the window, including inside text inputs unless the input needs the key (see section 3).
  - `C` canvas: only while the page canvas has focus (not while typing in an input).
  - `F` find bar: only while focus is inside the find bar.
  - `L` list: only inside tab strip, sidebar lists and ribbon (roving-tabindex widgets).
- **Conflict rule:** two commands conflict if they use the same chord and their scopes overlap. `G` overlaps everything;
  `C`, `F`, `L` do not overlap with each other. The table below has **no conflicts**; the registry test (T-043) must assert this
  and the keymap editor (U-04) must warn on any new conflict.
- A command can have more than one binding (written `A` or `B`). Bindings are matched on `event.key` for letters and
  symbols and on `event.code` for physical keys (`Tab`, arrows, `F1..F12`, `Digit`, `Numpad`), so non-US layouts work.
  `+` is matched as `key === "+"` or `key === "="` with Ctrl held (same physical key, with or without Shift).
- Shortcuts that need Alt+Ctrl are avoided (AltGr on many European layouts = Ctrl+Alt).
- Single-letter keys without a modifier are used only in scope `C` and only for tools (H, V), and are off in text inputs.

## 2. Default keymap

### 2.1 File and application

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `file.open` | `Ctrl+O` | G | V-01 |
| `file.save` | `Ctrl+S` | G | R-01 |
| `file.saveAs` | `Ctrl+Shift+S` | G | R-02 |
| `file.print` | `Ctrl+P` | G | U-08 |
| `file.close` (close tab) | `Ctrl+W` or `Ctrl+F4` | G | V-02 |
| `file.closeAll` | `Ctrl+Shift+W` | G | V-02 |
| `file.reopenClosed` | `Ctrl+Shift+T` | G | V-03 |
| `file.openRecent` | (none; palette and Home) | n/a | V-04 |
| `doc.properties` | `Alt+Enter` | G | P-10 |
| `app.settings` | `Ctrl+,` | G | U-06 |
| `app.quit` | `Alt+F4` (Windows) and `Ctrl+Q` | G | n/a |
| `app.commandPalette` | `Ctrl+K` or `Ctrl+Shift+P` | G | U-02 |
| `app.help` (keyboard shortcut list, local page) | `F1` | G | AC-01 |

### 2.2 Tabs

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `tab.next` | `Ctrl+Tab` or `Ctrl+PageDown` | G | V-02 |
| `tab.prev` | `Ctrl+Shift+Tab` or `Ctrl+PageUp` | G | V-02 |
| `tab.goto1` to `tab.goto8` | `Alt+1` to `Alt+8` | G | V-02 |
| `tab.gotoLast` | `Alt+9` | G | V-02 |
| `tab.moveLeft` | `Ctrl+Shift+PageUp` | G | V-02 |
| `tab.moveRight` | `Ctrl+Shift+PageDown` | G | V-02 |
| `tab.closeFocused` | `Delete` | L (tab strip) | V-02 |
| `tab.contextMenu` | `Shift+F10` or `Menu` | L | AC-01 |

Middle-click on a tab = `file.close` for that tab (mouse only).

### 2.3 Navigation

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `nav.back` | `Alt+Left` | G | V-11 |
| `nav.forward` | `Alt+Right` | G | V-11 |
| `nav.goToPage` (focus page box) | `Ctrl+Shift+N` | G | V-01 |
| `nav.nextPage` | `Ctrl+Down` | C | V-06 |
| `nav.prevPage` | `Ctrl+Up` | C | V-06 |
| `nav.firstPage` | `Home` | C | V-06 |
| `nav.lastPage` | `End` | C | V-06 |
| `scroll.lineDown` | `Down` | C | V-06 |
| `scroll.lineUp` | `Up` | C | V-06 |
| `scroll.lineLeft` | `Left` | C | V-06 |
| `scroll.lineRight` | `Right` | C | V-06 |
| `scroll.pageDown` | `PageDown` or `Space` | C | V-06 |
| `scroll.pageUp` | `PageUp` or `Shift+Space` | C | V-06 |

### 2.4 Zoom, layout, view

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `view.zoomIn` | `Ctrl+=` or `Ctrl++` or `Ctrl+NumpadAdd` | G | V-07 |
| `view.zoomOut` | `Ctrl+-` or `Ctrl+NumpadSubtract` | G | V-07 |
| `view.fitPage` | `Ctrl+0` | G | V-07 |
| `view.actualSize` (100%) | `Ctrl+1` | G | V-07 |
| `view.fitWidth` | `Ctrl+2` | G | V-07 |
| `view.zoomSet` (set a percentage via the zoom box) | (none; ribbon, status bar, palette) | n/a | V-07 |
| `view.rotateRight` | `Ctrl+Shift+Right` | C | V-08 |
| `view.rotateLeft` | `Ctrl+Shift+Left` | C | V-08 |
| `view.layoutContinuous` / `Single` / `Two` / `Book` | (none; palette and View tab) | n/a | V-06 |
| `view.pageColorDark` (toggle dark page) | `Ctrl+Shift+D` | G | V-09 |
| `view.toggleSidebar` | `F4` | G | V-05 |
| `view.toggleRibbon` | `Ctrl+F1` | G | U-01 |
| `view.fullscreen` | `F11` or `Ctrl+L` | G | V-10 |

Mouse: `Ctrl+Wheel` = zoom at the pointer; touchpad pinch = zoom (browser zoom is disabled, the app handles it); `Alt+drag` = column select (V-13).

### 2.5 Search (find bar)

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `search.open` | `Ctrl+F` | G | S-01 |
| `search.next` | `F3` or `Enter` (in find bar) | G (`F3`), F (`Enter`) | S-01 |
| `search.prev` | `Shift+F3` or `Shift+Enter` (in find bar) | G (`Shift+F3`), F (`Shift+Enter`) | S-01 |
| `search.close` | `Esc` (returns focus to canvas) | F | S-01 |
| `search.toggleResults` | `Ctrl+Shift+F` | G | S-02 |
| `search.toggleCase` | `Alt+C` | F | S-03 |
| `search.toggleWholeWord` | `Alt+W` | F | S-03 |
| `search.toggleRegex` | `Alt+R` | F | S-03 |

`Ctrl+F` when the find bar is already open moves focus to its input and selects its text.

### 2.6 Edit, selection, tools

| Command ID | Default | Scope | Req |
|---|---|---|---|
| `edit.undo` | `Ctrl+Z` | G | A-08 |
| `edit.redo` | `Ctrl+Y` or `Ctrl+Shift+Z` | G | A-08 |
| `edit.copy` | `Ctrl+C` | G | V-13 |
| `edit.selectAll` | `Ctrl+A` | G | V-13 |
| `edit.delete` | `Delete` | C | A-07 |
| `edit.cancel` (back out one level: tool, selection, then full screen) | `Esc` | C | AC-01 |
| `tool.select` | `V` | C | V-13 |
| `tool.hand` | `H` | C | V-07 |

In text inputs, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+C`, `Ctrl+A` keep their native text-editing behaviour (the app handler stands down when `event.target` is an input, textarea or contenteditable).

### 2.7 Reserved IDs for later milestones (no default yet, assign in the same table when built)

`ann.highlight`, `ann.underline`, `ann.strikeout`, `ann.squiggly`, `ann.note`, `ann.textBox`, `ann.ink`, `ann.line`, `ann.arrow`,
`ann.rect`, `ann.ellipse`, `ann.polygon`, `ann.cloud`, `pages.organizer`, `pages.rotateLeft`, `pages.rotateRight`, `pages.delete`,
`pages.duplicate`, `pages.merge`, `pages.split`, `pages.extract`, `form.highlightFields`, `form.clear`, `sign.signature`,
`redact.mark`, `redact.apply`, `tts.page`, `tts.fromCursor`, `tts.selection`, `view.reflow`. Rule for adding: pick a chord not in
sections 2.1 to 2.6 and not in the suppression list (section 3); prefer `Ctrl+Shift+<letter>`.

## 3. WebView2 / browser keys that must be suppressed

Suppress in two layers: (1) native, `ICoreWebView2Settings`: `AreBrowserAcceleratorKeysEnabled = false`,
`AreDefaultContextMenusEnabled = false` (the app draws its own), `AreDevToolsEnabled = false` in release builds
(true in debug), `IsZoomControlEnabled = false`, `IsPinchZoomEnabled = false`, `IsSwipeNavigationEnabled = false`,
`IsStatusBarEnabled = false`, `IsGeneralAutofillEnabled = false`, `IsPasswordAutosaveEnabled = false`.
(2) a capture-phase `keydown` handler on `window` that calls `preventDefault()` for the chords below unless the app maps
them to a command, as a fallback when layer 1 is not available. Implementers: owner is `frontend-developer` (layer 2) and
`rust-core-developer` (layer 1, T-019).

| Chord(s) | Browser meaning | App behaviour |
|---|---|---|
| `F5`, `Ctrl+F5`, `Ctrl+R`, `Ctrl+Shift+R` | reload | suppressed (a reload would drop in-memory state) |
| `Ctrl+P` | browser print | replaced by `file.print` (our dialog) |
| `Ctrl+S`, `Ctrl+Shift+S` | save page | replaced by `file.save`, `file.saveAs` |
| `Ctrl+O` | open file in browser | replaced by `file.open` |
| `Ctrl+F`, `F3`, `Shift+F3`, `Ctrl+G`, `Ctrl+Shift+G` | browser find / find next / previous | `Ctrl+F`, `F3`, `Shift+F3` replaced by our search; `Ctrl+G` and `Ctrl+Shift+G` suppressed and unbound |
| `Ctrl+Shift+I`, `F12`, `Ctrl+Shift+J`, `Ctrl+Shift+C` | DevTools | suppressed in release; allowed in debug builds only |
| `Ctrl+U` | view source | suppressed |
| `Ctrl+W`, `Ctrl+F4`, `Ctrl+Shift+W` | close tab / window | replaced by `file.close`, `file.closeAll` |
| `Ctrl+T`, `Ctrl+Shift+T` | new tab / reopen | `Ctrl+T` suppressed; `Ctrl+Shift+T` replaced by `file.reopenClosed` |
| `Ctrl+N`, `Ctrl+Shift+N` | new window / incognito | `Ctrl+N` suppressed; `Ctrl+Shift+N` replaced by `nav.goToPage` |
| `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Ctrl+PageUp/Down`, `Ctrl+1..9` | browser tab switch | `Ctrl+Tab` family replaced by `tab.next/prev`; `Ctrl+1` and `Ctrl+2` replaced by zoom; `Ctrl+3` to `Ctrl+9` suppressed (tabs use `Alt+1..9`) |
| `Alt+Left`, `Alt+Right`, `Backspace` (outside inputs) | history back / forward | `Alt+Left/Right` replaced by `nav.back/forward`; `Backspace` suppressed |
| `Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`, `Ctrl+Wheel`, pinch | browser zoom | replaced by `view.zoomIn/zoomOut/fitPage` and our zoom |
| `F6` | browser pane focus | replaced by region cycling (section 4) |
| `F7` | caret browsing prompt | suppressed |
| `F11` | browser full screen | replaced by `view.fullscreen` |
| `Ctrl+D`, `Ctrl+Shift+D` | bookmark / bookmark all | `Ctrl+D` suppressed; `Ctrl+Shift+D` replaced by `view.pageColorDark` |
| `Ctrl+H`, `Ctrl+J`, `Ctrl+Shift+O`, `Ctrl+Shift+B`, `Ctrl+B` | history, downloads, clear data, bookmarks | suppressed |
| `Ctrl+L`, `Ctrl+E`, `Alt+D` | address bar / search focus | `Ctrl+L` replaced by `view.fullscreen`; `Ctrl+E` and `Alt+D` suppressed |
| `Ctrl+K` | (browser search box) | replaced by `app.commandPalette` |
| `Ctrl+Shift+M`, `Ctrl+Shift+A`, `Ctrl+Shift+Delete` | profile / tab search / clear data | suppressed |
| `Shift+F10`, `Menu` | native context menu | replaced by our context menu (`tab.contextMenu`, canvas menu) |
| Drag a link/file onto the page | navigate the WebView | suppressed; the drop opens the file in a tab (V-01) |

Never suppress: `Alt+F4` (window close), `Win` key chords, `Alt+Tab`, `Ctrl+Alt+Del`, screen-reader keys (Narrator `Caps/Insert`+...,
NVDA/JAWS modifiers). Do not capture `Alt` alone (it activates the Windows menu key tip), and do not capture bare `Tab`.

## 4. Focus order and F6 region cycling

### 4.1 Region order (visual order, top to bottom, then left to right)

```
 1 Tab bar  ->  2 Ribbon  ->  3 Find bar (only if open)  ->  4 Sidebar (only if visible)  ->  5 Page canvas  ->  6 Status bar
   ^                                                                                                               |
   +--------------------------------- F6 wraps to region 1 ----------------------------------------------------+
```

- `F6` moves to the next region, `Shift+F6` to the previous. Hidden or collapsed regions (closed find bar, hidden sidebar) are skipped. A short screen-reader announcement names the region: "Navigation panel", "Document pages", etc.
- On arriving by F6, focus lands on the region's **last-focused** element, or its first element if none: tab bar = active tab; ribbon = the active ribbon tab; find bar = input; sidebar = selected item in the current panel; canvas = the scroller; status bar = page box.
- Opening the find bar (`Ctrl+F`) moves focus into its input; closing it (`Esc`) returns focus to the canvas. Opening a dialog traps focus (Tab cycles inside); closing returns to the opener. Menus return to the invoking button.
- **Skip link** "Skip to document" is the first Tab stop on a fresh window; Enter moves focus to the canvas.

### 4.2 Tab order inside each region (`Tab` / `Shift+Tab`)

| Region | Tab stops (in order) | Arrow keys inside |
|---|---|---|
| 1 Tab bar | app menu button, tab strip (one stop = active tab), scroll chevrons if shown, window controls are native | Left/Right between tabs, Home/End |
| 2 Ribbon | collapse button, ribbon tab row (one stop), then **one stop per group** in order, help button | Left/Right/Home/End within the row or group; Down opens a menu button |
| 3 Find bar | input, Match case, Whole word, Regex, Previous, Next, Results, Close | Left/Right across the buttons |
| 4 Sidebar | panel switcher (one stop), splitter, then the panel content (one stop for the list; items roving) | Up/Down items, Home/End, Enter activates, Left/Right expand/collapse in the outline tree |
| 5 Page canvas | scroller (one stop). Links and annotations on the page are reachable with `Tab` once the canvas is focused when "tabbed content" applies: inside the scroller, `Tab` moves through links/form fields on the visible page in reading order; `Esc` returns focus to the scroller itself | scroll keys (section 2.3) |
| 6 Status bar | page box, save state (not a stop), safe-mode button, layout menu, zoom slider, zoom combobox | Left/Right on the slider |

Rules: no positive `tabindex`; every Tab stop shows the focus ring (`theme.md` section 6); focus is never trapped except
in modal dialogs; `Esc` always backs out one level (menu, then popover, then find bar, then selection).

### 4.3 Dialog behaviour

Dialogs the user opened (password, print, external-link confirmation, unsaved changes): focus moves to the first
input or, for confirmations, to the safe default button; `Tab` cycles inside; `Esc` cancels; `Enter` activates the default
button. Titles are `aria-labelledby`, descriptions `aria-describedby`. These are never shown unprompted by the app.

## 5. Customisation hooks (U-04, v1.x)

- Settings stores overrides as `{"keymap": {"file.open": ["Ctrl+O"], "tool.hand": []}}` (empty array = unbound; missing = default).
- Export/import is this same JSON. Conflict warning compares chord + overlapping scopes (section 1).
- The Vim preset (U-05) is a separate map with ids unchanged (`scroll.lineDown` = `j`, `scroll.lineUp` = `k`, `nav.firstPage` = `gg`, `nav.lastPage` = `G`, `search.open` = `/`), active in scope `C` only.
