# User Guide — OpenClaw Photo Studio

Complete guide for using OCPS to manage and edit your RAW photos.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Library Workflow](#library-workflow)
3. [Develop Workflow](#develop-workflow)
4. [Keyboard Shortcuts Reference](#keyboard-shortcuts-reference)
5. [Tips & Best Practices](#tips--best-practices)

---

## Getting Started

### First Launch

When you first launch OpenClaw Photo Studio, you'll see the **Library Module** with an empty catalog.

### Import Your First Folder

1. **Click "Import Folder"** in the toolbar (or press `Cmd/Ctrl+Shift+I`)
2. **Select a folder** containing your RAW photos
3. **Wait for import** to complete (progress shown in toolbar)
4. **View results** in the grid

**Import Options:**
- **Copy**: Copies files to a managed location
- **Move**: Moves files to managed location
- **Add**: Adds files in place (references original location)

**Note:** During import, OCPS will:
- Calculate SHA-256 hash for each file (deduplication)
- Extract EXIF metadata (camera, lens, settings)
- Generate thumbnails for fast browsing
- Skip duplicate files automatically

**Supported Formats:**
- **RAW**: ARW, NEF, RAF, DNG, CR2, CR3, ORF, RW2
- **Standard**: JPEG, TIFF, PNG

---

## Library Workflow

The Library module is where you organize, rate, flag, and filter your photos.

### View Modes

Switch between view modes using keyboard shortcuts:

- **Grid View** (`G`): Thumbnail grid for browsing
- **Loupe View** (`E`): Full-screen single photo view
- **Compare View** (`C`): Side-by-side comparison of 2 photos
- **Survey View** (`N`): View multiple selected photos at once

### Rating Photos

Quickly rate photos using number keys:

- `0` — Clear rating
- `1` — ★ (1 star)
- `2` — ★★ (2 stars)
- `3` — ★★★ (3 stars)
- `4` — ★★★★ (4 stars)
- `5` — ★★★★★ (5 stars)

**Tip:** After rating, the selection automatically advances to the next photo for fast culling.

### Flagging Photos

Flag photos as picks or rejects:

- `P` — Flag as **Pick** (blue flag)
- `X` — Flag as **Reject** (red flag)
- `U` — **Unflag** (clear flag)

**Workflow:** Use flags during your first pass, then filter to show only picks.

### Color Labels

Assign color labels for custom organization:

- `6` — Red label
- `7` — Yellow label
- `8` — Green label
- `9` — Blue label

**Use Cases:**
- Red = Needs editing
- Yellow = Client favorites
- Green = Export ready
- Blue = Portfolio pieces

### Navigation

Navigate through photos efficiently:

- `←` / `→` — Previous / Next photo
- `↑` / `↓` — Move up/down in grid
- `Home` / `End` — First / Last photo
- `Page Up` / `Page Down` — Scroll grid by page
- `Space` — Next unrated/unflagged photo

### Filtering & Search

Use the **Filter Bar** to narrow down photos:

1. **Rating Filter**: Show only photos rated 3+ stars
2. **Flag Filter**: Show picks, rejects, or all
3. **Color Label Filter**: Filter by color label
4. **Search**: Full-text search across filenames, EXIF, keywords

**Example:** To show all 5-star picks:
- Set Rating to "★★★★★"
- Set Flag to "Pick"

### Collections

Organize photos into collections:

- **Manual Collections**: Drag and drop photos into collections
- **Smart Collections**: Auto-update based on rules (e.g., "All 5-star photos from 2024")
- **Quick Collection** (`B`): Temporary collection for ad-hoc selection

**Creating a Collection:**
1. Press `Ctrl+N` (or click "+" in sidebar)
2. Name your collection
3. Drag photos into it

**Creating a Smart Collection:**
1. Press `Ctrl+Shift+N`
2. Define rules: "Rating >= 4 AND Camera = Sony A7IV"
3. Collection updates automatically

### Keywords

Add hierarchical keywords for searchability:

1. Click a photo
2. Focus the Keywords field (`Cmd+K`)
3. Type keywords (comma-separated): `nature, birds, eagles`

**Hierarchical Keywords:**
- Nature
  - Birds
    - Eagles
    - Hawks
  - Mammals

**Tip:** Create keyword sets (e.g., "Wildlife", "Portraits") for quick tagging.

---

## Develop Workflow

The Develop module is where you non-destructively edit your RAW photos.

### Opening a Photo in Develop

1. Select a photo in Library
2. Press `D` or click "Develop" in the top bar

### Basic Adjustments

Located in the **Basic Panel** (right sidebar):

**White Balance:**
- **Temperature**: Cool (blue) ↔ Warm (yellow)
- **Tint**: Green ↔ Magenta
- **Tip:** Click the eyedropper (`W`) and click a neutral gray area

**Exposure & Tone:**
- **Exposure**: Overall brightness (-5 to +5 EV)
- **Contrast**: Overall contrast
- **Highlights**: Recover blown highlights
- **Shadows**: Lift shadow detail
- **Whites**: Set white point
- **Blacks**: Set black point

**Presence:**
- **Clarity**: Local contrast (midtones)
- **Vibrance**: Intelligent saturation boost
- **Saturation**: Global saturation

**Sharpening:**
- **Amount**: Sharpening strength
- **Radius**: Sharpening width

### Copy/Paste Edits

OCPS's killer feature — copy settings to multiple photos in 2 keystrokes:

1. **Select a photo** with edits you like
2. **Copy settings** (`Cmd/Ctrl+C`)
3. **Select other photos** (one or many)
4. **Paste settings** (`Cmd/Ctrl+V`)

**Advanced:**
- `Cmd+Shift+C` — Copy **selected settings** (opens dialog)
- `Cmd+Shift+V` — Paste **selected settings**

**Sync Settings:**
- Select multiple photos
- Press `Cmd+Shift+S` to sync settings across all selected

### Before/After Toggle

Quickly compare your edits:

- `\` — Toggle before/after (press again to return)
- `Shift+\` — Side-by-side before/after
- `Alt+\` — Split view (vertical divider)

### Zoom & Pan

Inspect your photo at 1:1 magnification:

- `Cmd/Ctrl+=` — Zoom in
- `Cmd/Ctrl+-` — Zoom out
- `Cmd/Ctrl+0` — Fit to window
- `Cmd/Ctrl+Alt+0` — 100% (1:1 view)
- `Z` — Toggle between fit and 100%

**Pan:** When zoomed, click and drag to pan (or use space+drag)

### History & Snapshots

Every edit is non-destructive and reversible:

- **History Panel**: View all edit steps (bottom of right sidebar)
- **Undo/Redo**: `Cmd/Ctrl+Z` / `Cmd/Ctrl+Shift+Z`
- **Create Snapshot** (`Cmd/Ctrl+N`): Save current state with a name
- **Reset All** (`Cmd/Ctrl+Shift+R`): Return to unedited state

### Presets

Save and apply editing presets:

1. **Create a preset:**
   - Edit a photo to your liking
   - Press `Cmd+Shift+N` → "Save as New Preset"
   - Name it (e.g., "Moody B&W", "Golden Hour")

2. **Apply a preset:**
   - Select a photo
   - Click a preset in the Presets panel (left sidebar)
   - Or use Command Palette: `Cmd+K` → type preset name

**Import Lightroom Presets:**
- File → Import Presets → Select `.xmp` or `.lrtemplate` files

### Crop Tool

Crop and straighten your photos:

1. Press `R` to activate Crop Tool
2. Drag corners to resize
3. Press `A` to activate Straighten Tool (drag to set horizon)
4. Press `X` to swap aspect ratio (landscape ↔ portrait)
5. Press `O` to cycle crop overlay guides (rule of thirds, golden ratio, etc.)
6. Press `Enter` to apply, `Escape` to cancel

**Aspect Ratios:**
- Original
- 1:1 (square)
- 4:5 (Instagram portrait)
- 3:2 (standard photo)
- 16:9 (widescreen)
- Custom

### Virtual Copies

Create multiple variations without duplicating the file:

1. Select a photo
2. Press `Cmd+'` to create a virtual copy
3. Edit each copy independently

**Use Case:** Create both color and B&W versions of the same photo.

---

## Keyboard Shortcuts Reference

### Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+K` | Open Command Palette |
| `Cmd/Ctrl+,` | Preferences |
| `Cmd/Ctrl+Z` | Undo |
| `Cmd/Ctrl+Shift+Z` | Redo |
| `Tab` | Toggle side panels |
| `Shift+Tab` | Toggle all panels |
| `F` | Toggle filmstrip |
| `?` | Show keyboard shortcuts overlay |

### Module Navigation

| Shortcut | Action |
|----------|--------|
| `G` | Grid View (Library) |
| `E` | Loupe View |
| `D` | Develop Module |
| `C` | Compare View |
| `N` | Survey View |
| `M` | Map Module |

### Rating & Flagging

| Shortcut | Action |
|----------|--------|
| `0`-`5` | Set rating (0-5 stars) |
| `P` | Flag as Pick |
| `X` | Flag as Reject |
| `U` | Unflag |
| `6`-`9` | Color labels (Red, Yellow, Green, Blue) |

### Navigation

| Shortcut | Action |
|----------|--------|
| `←` `→` | Previous / Next photo |
| `↑` `↓` | Move in grid |
| `Home` / `End` | First / Last photo |
| `Space` | Next unrated photo |

### Develop Module

| Shortcut | Action |
|----------|--------|
| `\` | Before/After toggle |
| `R` | Crop tool |
| `W` | White balance picker |
| `Cmd/Ctrl+C` | Copy settings |
| `Cmd/Ctrl+V` | Paste settings |
| `Cmd/Ctrl+Shift+R` | Reset all edits |
| `Cmd/Ctrl+N` | New snapshot |
| `Z` | Toggle fit ↔ 1:1 zoom |

### Export

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+Shift+E` | Open Export dialog |
| `Cmd/Ctrl+J` | Quick export as JPEG |

---

## Tips & Best Practices

### Workflow Best Practices

1. **First Pass — Flag & Rate:**
   - Use `P` to flag keepers, `X` to flag rejects
   - Delete rejects later (saves time during shoot review)
   - Rate your picks: 3 stars = good, 4 stars = great, 5 stars = portfolio

2. **Second Pass — Develop:**
   - Filter to show only 3+ star picks
   - Edit one photo from a series
   - Copy/paste settings to similar shots (`Cmd+C` → select others → `Cmd+V`)

3. **Third Pass — Export:**
   - Select all edited photos
   - Press `Cmd+Shift+E` to export
   - Choose format, quality, and output folder

### Performance Tips

- **Use Smart Previews** (coming in Phase 2): Generates compressed proxies for faster editing
- **Limit grid size**: Filter photos to reduce grid to <1000 items for smooth scrolling
- **Close unused panels**: Press `Tab` to hide side panels and gain screen space

### Organization Tips

- **Date-based folders**: Import by date (2024/03/March) for easy navigation
- **Collections for projects**: Create a collection per client/project
- **Smart Collections for workflows**:
  - "Needs Editing" = Rating 3+ AND has_edits = false
  - "Export Ready" = Color Label = Green

### Editing Tips

- **Start with White Balance**: Always correct WB before exposure adjustments
- **Exposure before tone**: Adjust exposure first, then fine-tune with highlights/shadows
- **Clarity sparingly**: Too much clarity creates halos (use 0-30 range)
- **Use Virtual Copies**: Experiment with multiple edits without fear

### Backup Tips

- **XMP Sidecars**: OCPS auto-saves edits to `.xmp` files next to your RAWs
- **Backup catalog**: Export catalog (`File → Export Catalog`) regularly
- **Cloud backup**: Use Backblaze/Dropbox for both RAWs and XMP files

---

## FAQ

### How do I batch rename photos?
*Coming in Phase 4* — Use export templates for now: `{date}_{seq}_{original}`

### Can I edit JPEG files?
Yes! OCPS supports JPEG, TIFF, and PNG editing (though RAW gives best results)

### How do I import Lightroom catalogs?
File → Import → Lightroom Catalog (.lrcat) → Select file → Import

### Are edits reversible?
Yes! All edits are non-destructive. Original files are never modified.

### How do I share my settings with others?
Export as preset: Right-click edited photo → "Export Settings as Preset" → Share `.xmp` file

### Can I use OCPS offline?
Yes! OCPS is fully local — no cloud or internet required.

---

## Next Steps

- Explore [Keyboard Shortcuts](./shortcuts.md) for full reference
- Read [Architecture](./architecture.md) to understand how OCPS works
- Check out [Plugin Guide](./plugin-guide.md) to extend OCPS

---

**Last Updated:** 2026-03-21
**Version:** 0.9.0
