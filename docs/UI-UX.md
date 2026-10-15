# UI/UX specification — Modern Cyberpunk

## Tone

Serious sci-fi workstation, not noisy fake hacking. Core qualities: legible, efficient, keyboard-first, high contrast, low-motion. Theme is decoration over *real search results*.

## Palette (suggested RGB)

| Token | Hex | Purpose |
|---|---|---|
| background | `#080E1B` | Terminal canvas |
| surface | `#101C30` | Panel background |
| border | `#263653` | Separators |
| foreground | `#E3EDFA` | Main text |
| muted | `#91A4BE` | Secondary info |
| cyan | `#39E4ED` | Focus and search |
| violet | `#9C8BFB` | Categories and metadata |
| green | `#52E7AD` | Healthy state |
| amber | `#F4B860` | Warnings |
| red | `#FF778A` | Errors |

Terminal capabilities vary: use indexed/monochrome fallback when truecolor unsupported, maintain contrast, do not rely on color alone for selection.

## Layouts

**Wide (>=120 columns):** fixed 24–30-column sidebar left; search and results center; inspector right if sufficient width; header top; status/help bottom.  
**Medium (80–119):** compact sidebar + results, inspector as lower panel/overlay.  
**Narrow (60–79):** results take primary screen; source selection and inspector are separate views.  
**Very narrow (<60 or <16 rows):** clear compact fallback message with escape/quit instructions; never panic or underflow.

Do not hardcode height assumptions; use Ratatui constraint-based layout and saturating arithmetic. All lists scroll; selected items are visually distinguishable without color.

## Component tree

```text
Root
├─ Header: app version, index status, clock optional
├─ Workspace
│  ├─ Sources: source list, selected source, counts
│  ├─ Search: input, query mode, active filters
│  ├─ Results: row number, title, category, clipped snippet
│  └─ Inspector: ID, title, category, source, content, metadata
├─ Status: progress/errors/record counts
└─ Help overlay / confirmation dialog
```

## Focus states

`SearchInput | SourceList | ResultList | Inspector | HelpDialog`. Cursor shown only during editing. Route keyboard events by focus *before* applying global shortcuts. Inspector must handle long Unicode text and vertical scrolling. Empty/no matches/import pending/error states are first-class views, not blank screens.

## Motion

- Startup: at most a short splash, skippable immediately. Never block database load for dramatic effect.
- While background worker is busy: subtle spinner; keep keyboard and resizing responsive.
- No typewriter animation on actual query/results; content should appear as soon as available.
- Idle rendering should not spin at 60 FPS: use event-driven redraw and a modest tick only for needed animations.

## Rendering checklist

- Distinct focused/unfocused borders and full keyboard accessibility.
- Text clipping/wrapping; safe handling of control characters and ANSI escape sequences in imported content (display as plain, sanitized text).
- Correct cursor placement for multibyte and wide graphemes; use appropriate terminal-aware width calculation.
- Graceful terminal size changes, no panics or out-of-bounds list selection.
- Non-blocking error banners with a path to details/retry.
