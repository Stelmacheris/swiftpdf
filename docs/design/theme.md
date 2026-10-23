# Design spec: theme tokens (T-002)

Covers U-03, AC-04, V-09. Dark is the default; light is available; "Follow Windows" picks one from
`prefers-color-scheme`. Implemented by `theme.css` in T-006 / T-022. All ratios below were computed with the
WCAG 2.x relative-luminance formula (sRGB), not estimated.

## 1. Rules

- Text on its background: **>= 4.5:1**. Non-text UI (control borders, icons, focus ring, selection indicators): **>= 3:1**.
- Colored text (`accent`, `link`, `danger`, `success`, `warn`) is allowed **only on `app`, `surface` and `raised`**
  backgrounds. It is not allowed on `hover`, `pressed` or `selected` fills (some pairs there fall under 4.5:1).
  On those states use `--text` / `--text-muted`.
- Color is never the only signal. Status also uses an icon and a word; selection also uses a 3 px accent bar
  (the selected fill alone is only about 1.3:1 against its neighbour and does not count as an indicator).
- Theme is set on `<html data-theme="dark|light">`. "Follow Windows" sets it from `matchMedia('(prefers-color-scheme: dark)')`
  and updates live. Before first paint an inline one-line script (in `index.html`) sets the attribute from settings to avoid a flash.
- No gradients, no translucency on text backgrounds (translucent fills make contrast unverifiable).

## 2. Tokens

```css
:root,
:root[data-theme="dark"] {
  color-scheme: dark;

  /* surfaces */
  --bg-app:      #1b1d21;  /* window, tab bar strip */
  --bg-surface:  #23262b;  /* ribbon, sidebar, status bar, dialogs */
  --bg-raised:   #2c3036;  /* menus, popovers, inputs, active tab */
  --bg-hover:    #343942;
  --bg-pressed:  #3d434d;
  --bg-selected: #1f3a5f;  /* always paired with --selected-bar */
  --bg-canvas:   #2b2e33;  /* area behind the pages */

  /* text */
  --text:        #e8eaed;
  --text-muted:  #a9b0ba;
  --text-disabled: #7b848f;  /* exempt from 4.5:1 (WCAG 1.4.3), kept at 4.00:1 on --bg-surface */
  --link:        #8ab4f8;

  /* accent and status */
  --accent:      #5aa2ff;
  --on-accent:   #0b1220;  /* text on a filled accent button */
  --danger:      #ff8a80;
  --success:     #5fd38d;
  --warn:        #e3b341;
  --selected-bar:#5aa2ff;

  /* lines */
  --border-ui:     #7b848f; /* control and input borders (needs 3:1) */
  --border-subtle: #3a3f47; /* decorative dividers only, carry no meaning */

  /* focus */
  --focus-ring:   #8cc8ff;
  --focus-width:  2px;
  --focus-offset: 2px;

  /* misc */
  --page-border:  var(--border-ui);
  --shadow-menu:  0 4px 16px rgb(0 0 0 / 0.5);
}

:root[data-theme="light"] {
  color-scheme: light;

  --bg-app:      #f3f4f6;
  --bg-surface:  #ffffff;
  --bg-raised:   #eceef1;
  --bg-hover:    #e1e4e9;
  --bg-pressed:  #d4d8df;
  --bg-selected: #d6e6ff;
  --bg-canvas:   #d9dce1;

  --text:        #1c1f24;
  --text-muted:  #4b5563;
  --text-disabled: #6b7280;  /* 4.83:1 on --bg-surface */
  --link:        #0b5cd5;

  --accent:      #0b5cd5;
  --on-accent:   #ffffff;
  --danger:      #b42318;
  --success:     #166534;
  --warn:        #8a5a00;
  --selected-bar:#0b5cd5;

  --border-ui:     #6b7280;
  --border-subtle: #d0d4db;

  --focus-ring:   #0b5cd5;
  --focus-width:  2px;
  --focus-offset: 2px;

  --page-border:  var(--border-ui);
  --shadow-menu:  0 4px 16px rgb(0 0 0 / 0.18);
}

/* shared, theme-independent */
:root {
  --font-ui: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
  --font-size-ui: 13px;          /* scales with Windows text size (use rem; 1rem = 16px => 0.8125rem) */
  --radius: 4px;
  --control-height: 32px;        /* minimum hit target; WCAG 2.5.8 needs 24px */
  --dur-fast: 120ms;
  --dur-base: 200ms;
}

/* every interactive element */
:focus-visible {
  outline: var(--focus-width) solid var(--focus-ring);
  outline-offset: var(--focus-offset);
}
/* never remove an outline without replacing it; never use :focus { outline: none } */
```

Disabled text note: WCAG exempts inactive components from the contrast rule. `--text-disabled` is still kept
at 3:1 or better on `--bg-surface` in both themes so it stays readable (dark 4.00:1, light 4.83:1; same values as `--border-ui`).
Disabled controls also get `aria-disabled="true"` (not removed from the tab order inside toolbars, so screen-reader users can find them).

## 3. Verified contrast ratios

### 3.1 Dark theme: text

| Foreground | on app | on surface | on raised | on hover | on pressed | on selected |
|---|---|---|---|---|---|---|
| `--text` #e8eaed | 14.00 | 12.59 | 11.01 | 9.63 | 8.26 | 9.53 |
| `--text-muted` #a9b0ba | 7.72 | 6.94 | 6.07 | 5.31 | 4.56 | 5.25 |
| `--accent` #5aa2ff | 6.47 | 5.82 | 5.08 | n/a | n/a | n/a |
| `--link` #8ab4f8 | 8.01 | 7.20 | 6.29 | n/a | n/a | n/a |
| `--danger` #ff8a80 | 7.39 | 6.65 | 5.81 | n/a | n/a | n/a |
| `--success` #5fd38d | 8.99 | 8.09 | 7.07 | n/a | n/a | n/a |
| `--warn` #e3b341 | 8.67 | 7.80 | 6.82 | n/a | n/a | n/a |

`--on-accent` #0b1220 on `--accent` #5aa2ff: **7.17**.

### 3.2 Dark theme: non-text

| Element | Against | Ratio |
|---|---|---|
| `--border-ui` #7b848f | app / surface / raised / canvas | 4.45 / 4.00 / 3.50 / 3.59 |
| `--focus-ring` #8cc8ff | app / surface / raised / canvas | 9.50 / 8.54 / 7.47 / 7.67 |
| `--accent` (icons, selected bar) | app / surface / raised / canvas | 6.47 / 5.82 / 5.08 / 5.22 |

### 3.3 Light theme: text

| Foreground | on app | on surface | on raised | on hover | on pressed | on selected |
|---|---|---|---|---|---|---|
| `--text` #1c1f24 | 15.02 | 16.52 | 14.22 | 12.96 | 11.56 | 13.08 |
| `--text-muted` #4b5563 | 6.87 | 7.56 | 6.50 | 5.93 | 5.29 | 5.98 |
| `--accent` #0b5cd5 | 5.44 | 5.99 | 5.15 | n/a | n/a | n/a |
| `--link` #0b5cd5 | 5.44 | 5.99 | 5.15 | n/a | n/a | n/a |
| `--danger` #b42318 | 5.97 | 6.57 | 5.66 | n/a | n/a | n/a |
| `--success` #166534 | 6.48 | 7.13 | 6.13 | n/a | n/a | n/a |
| `--warn` #8a5a00 | 5.39 | 5.93 | 5.10 | n/a | n/a | n/a |

`--on-accent` #ffffff on `--accent` #0b5cd5: **5.99**.

### 3.4 Light theme: non-text

| Element | Against | Ratio |
|---|---|---|
| `--border-ui` #6b7280 | app / surface / raised / canvas | 4.39 / 4.83 / 4.16 / 3.52 |
| `--focus-ring` #0b5cd5 | app / surface / raised / canvas | 5.44 / 5.99 / 5.15 / 4.36 |
| `--accent` (icons, selected bar) | app / surface / raised / canvas | 5.44 / 5.99 / 5.15 / 4.36 |

`--border-subtle` is decorative (separators between groups) and is intentionally below 3:1. It must never be the only
boundary of an interactive control; inputs, buttons with no fill and tabs use `--border-ui`.

## 4. Page-color presets (V-09)

These recolor only the rendered page, never the app chrome. The page sits on `--bg-canvas` with a 1 px `--page-border`
(3.59:1 dark theme, 3.52:1 light theme), which keeps the page edge visible even for the dark preset.

| Preset | Text/vector color | Background | Ratio | Images | PDFium implementation hint (T-027) |
|---|---|---|---|---|---|
| `normal` | document's own colors | paper `#ffffff` | document-defined | unchanged | no color scheme |
| `dark` | `#e0e0e0` | `#1e1e1e` | 12.63 | kept natural (not inverted) | color-scheme render: text/path fill `#e0e0e0`, page fill `#1e1e1e` |
| `sepia` | `#3b2f1e` | `#f4ecd8` | 11.07 | unchanged, with a warm multiply tint | page fill `#f4ecd8`, text/path `#3b2f1e` |
| `custom` | user | user | shown live | unchanged | foreground and background pickers |

```css
:root {
  --page-normal-fg: #1c1f24;  --page-normal-bg: #ffffff;
  --page-dark-fg:   #e0e0e0;  --page-dark-bg:   #1e1e1e;
  --page-sepia-fg:  #3b2f1e;  --page-sepia-bg:  #f4ecd8;
}
```

Custom colors: the picker shows the contrast ratio next to the swatches ("Contrast 7.2:1"). Under 4.5:1 it shows an
inline note, "Low contrast, text may be hard to read." It never blocks and never opens a dialog. The page preset is
independent of the UI theme (dark UI with a normal page is the default; the menu is specified in the T-017 viewer spec).

## 5. forced-colors (Windows High Contrast, AC-04)

Under `forced-colors: active` the browser overrides colors; the tokens are remapped to system colors so that custom
components (drawn with CSS variables) follow the user's contrast theme. Do not rely on `box-shadow` or background-image
for meaning, since they are removed.

```css
@media (forced-colors: active) {
  :root,
  :root[data-theme="dark"],
  :root[data-theme="light"] {
    --bg-app: Canvas;      --bg-surface: Canvas;   --bg-raised: Canvas;
    --bg-hover: Canvas;    --bg-pressed: Canvas;   --bg-canvas: Canvas;
    --bg-selected: Highlight;
    --text: CanvasText;    --text-muted: CanvasText;  --text-disabled: GrayText;
    --link: LinkText;
    --accent: Highlight;   --on-accent: HighlightText; --selected-bar: Highlight;
    --danger: CanvasText;  --success: CanvasText;     --warn: CanvasText;  /* meaning carried by icon + text */
    --border-ui: ButtonBorder;  --border-subtle: ButtonBorder;
    --focus-ring: Highlight;
    --page-border: CanvasText;
    --shadow-menu: none;
  }
  /* hover and pressed need a visible cue because fills collapse to Canvas */
  .btn:hover, .menu-item:hover { outline: 1px solid Highlight; }
  .btn[aria-pressed="true"], .tab[aria-selected="true"], [aria-current="page"] {
    outline: 2px solid Highlight; background: Canvas; color: CanvasText;
  }
  .menu-item[aria-selected="true"] { background: Highlight; color: HighlightText; }
  /* the rendered page keeps its own pixels; only the frame is system-colored */
  .page-canvas { forced-color-adjust: none; border: 1px solid CanvasText; }
  svg.icon { forced-color-adjust: auto; fill: currentColor; stroke: currentColor; } /* icons use currentColor */
}
```

Requirements for T-022 / T-043: all icons draw with `currentColor`; no icon is a raster or has a hard-coded color; focus
ring stays visible (`Highlight`); separators that carry no meaning may vanish, but group boundaries are also expressed by
group labels.

## 6. Focus-ring tokens

| Token | Value | Use |
|---|---|---|
| `--focus-ring` | per theme | outline color |
| `--focus-width` | 2px | outline width (>= 2 px per WCAG 2.4.13 intent) |
| `--focus-offset` | 2px | gap so the ring reads on filled buttons and does not touch text |

Ring contrast is >= 3:1 against every surface it can sit on (tables in 3.2 and 3.4). On a filled accent button the 2 px
offset puts the ring on the surface behind it. In inputs the ring is drawn outside the border. Focus is never hidden by a
sticky region: scroll containers use `scroll-padding` equal to the sticky ribbon/tab height.

## 7. Reduced motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

Beyond the CSS rule: the zoom change does not animate its scale (the scaled bitmap swaps in one frame, V-07); jump-to-page and
search-hit navigation scroll instantly; there is no auto-playing animation; progress for long jobs is a determinate bar or a
static "Working..." text (no spinning or pulsing). Ordinary motion (when allowed) is limited to <= `--dur-base` fades on menus and the ribbon collapse.

## 8. Text scaling and DPI (AC-04)

- All sizes in `rem`/`em`; the root follows the Windows text-size setting (WebView2 passes it through). Layouts are verified at 100%, 150%, 200% and 300% text size. Controls grow in height (`min-height: var(--control-height)`), never clip text, and never use fixed heights on text containers.
- Icons are SVG (crisp at any per-monitor DPI), 16 px at 100% in a 32 px target.
- No horizontal scrolling of the app chrome at 1280 px viewport and 300% text; the ribbon overflows into a "More" menu instead (see ribbon.md).
