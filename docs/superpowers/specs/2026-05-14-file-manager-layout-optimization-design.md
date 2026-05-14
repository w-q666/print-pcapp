# File Management Layout Optimization Design

**Date:** 2026-05-14
**Status:** Approved

## Goal

Merge the two separate file management implementations (HomePage.vue and FileManager.vue) into a single, functionally complete page at `/files`, with improved visual hierarchy, compact upload zone, and optimized right-sidebar card layout.

## Current State

- `HomePage.vue` (`/files` route): dual-column dashboard — upload zone + file list with per-file print buttons + 3 stacked sidebar cards (QR, system status, printer status). Missing: preview, sort controls, file count.
- `FileManager.vue` (not in router): standalone file manager — upload zone + sort controls + file count + preview modal with 4 previewers. Missing: print entry, sidebar cards.
- Right sidebar is 280px fixed, with QR/SystemStatus/PrinterStatus stacked vertically — wastes vertical space.

## Design

### Layout Structure

Two-column grid: `1fr 260px` (was `1fr 280px`), gap 14px.

**Left column (flexible width):**
1. Compact upload zone — horizontal single-row layout (icon + text + format hints), height reduced ~60%
2. Toolbar row — left: "共 N 个文件" file count; right: sort toggle (按名称 / 按类型)
3. File list — bordered container without Card wrapper; each row is a FileListItem with integrated actions

**Right column (260px fixed):**
1. QR code card — full width, top
2. System status + Printer status — side by side in a 2-column row below the QR card

### Component Changes

#### FileUploadZone.vue
- Change from vertical layout (large icon + two lines of text) to horizontal single-row
- Reduce padding: `16px 0` → `12px 18px`; reduce icon: `32px` → `24px`
- Keep drag-and-drop and click-to-upload behavior unchanged

#### FileListItem.vue
- Upgrade from single-row (icon + filename | actions) to two-row (icon + filename + metadata | actions)
- Add file size and modified time display below filename
- Integrate print button into the component's action bar (prop/emit)
- Action bar: preview | delete | print (print as the primary/rightmost action, outlined style)

#### SystemStatusCard.vue
- Compact redesign: queue count and today-completed count stacked vertically in a single card (was: two separate Statistics boxes)
- Keep the purple gradient background
- Reduce padding to fit the half-width sidebar slot

#### PrinterStatusCard.vue
- Compact redesign: printer list with colored status dots in a smaller card
- Reduce padding to fit the half-width sidebar slot
- Keep green/gray dot + printer name layout

#### HomePage.vue
- Merge FileManager features: sort controls, file count, preview modal
- Remove Card wrapper from file list section
- Remove per-file print button from outside FileListItem (now inside)
- Change sidebar grid from 280px single column to 260px mixed layout
- Add preview Modal with PdfPreview / ImagePreview / TextPreview / HtmlPreview
- FileManager.vue can be deleted after merge

### Responsive Breakpoints

| Width | Behavior |
|-------|----------|
| >= 900px | Default: left (1fr) + right (260px), upload horizontal, full action text |
| 680–900px | Right sidebar shrinks to 220px; action buttons icon-only |
| < 680px | Single column; 4 cards become horizontal row at bottom (QR/queue/today/printer equal-width) |

### Files Touched

| File | Action |
|------|--------|
| `src/views/home/HomePage.vue` | Major rewrite — integrate sort, preview, compact layout |
| `src/components/FileUploadZone.vue` | Modify — horizontal compact layout |
| `src/components/FileListItem.vue` | Modify — two-row layout, add print action |
| `src/components/SystemStatusCard.vue` | Modify — compact card for half-width slot |
| `src/components/PrinterStatusCard.vue` | Modify — compact card for half-width slot |
| `src/views/file-manager/FileManager.vue` | Delete (functionality merged into HomePage) |
