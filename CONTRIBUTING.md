# Contributing to OpenClaw Photo Studio

Welcome! We're glad you're interested in contributing.

---

## Before You Start

1. **Read [GOVERNANCE.md](GOVERNANCE.md)** — understand how decisions are made
2. **Read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** — our community standards
3. **Sign the [CLA](CLA.md)** — required for all code contributions (automated via CLA bot on your first PR)

## Why a CLA?

OCPS uses dual licensing (PolyForm Noncommercial + Commercial). The CLA grants the project the right to offer your contributions under both licenses. **You retain full copyright of your work.** The CLA includes a fairness clause: if the project is ever sold, all CLA contributors receive the right to use their contributions under Apache 2.0.

---

## Types of Contributions

| Type | How |
|------|-----|
| 🐛 Bug Reports | Open an issue with reproduction steps |
| 💡 Feature Requests | Open an issue with use case description |
| 📝 RFCs | For larger changes → `rfcs/` directory |
| 🔧 Code | PRs against `develop` branch |
| 📖 Documentation | Always welcome, no CLA needed for docs-only |
| 🌍 Translations | i18n contributions |
| 🎨 Design | UI/UX proposals with mockups |
| 🧪 Tests | Unit, integration, visual regression |
| 📷 Camera Profiles | New camera/lens support |
| ⌨️ Shortcut Sets | Custom keymap profiles |
| 🎛️ Presets | Community presets |
| 🔌 Plugins | Independent or in-repo |

---

## Getting Started Locally

### Prerequisites
- Rust 1.78+ (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js 20+: via nvm or package manager
- pnpm 9+: `npm install -g pnpm`
- System dependencies:
  - macOS: Xcode Command Line Tools
  - Linux: `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev`
  - Windows: Visual Studio Build Tools 2022

### Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/photo-studio.git
cd photo-studio

# Install JS dependencies
pnpm install

# Build Rust crates
cargo build

# Run tests
cargo test
pnpm test

# Start development server
cargo tauri dev
```

### Project Structure

```
photo-studio/
├── crates/              # Rust crates
│   ├── ocps-core/       # Core engine (RAW, pipeline, GPU)
│   ├── ocps-catalog/    # SQLite catalog
│   ├── ocps-xmp/        # XMP/IPTC/EXIF
│   ├── ocps-export/     # Export engine
│   ├── ocps-plugin-host/# Plugin system
│   └── ocps-cli/        # CLI tool
├── app/                 # Tauri desktop app
│   ├── src-tauri/       # Tauri Rust backend
│   └── src/             # SolidJS frontend
├── plugins/             # Official plugins
├── presets/             # Built-in presets
├── tests/               # Integration & golden image tests
├── rfcs/                # RFC documents
└── docs/                # Documentation
```

---

## Workflow

### 1. Find Something to Work On

- Check issues labeled `good first issue` or `help wanted`
- Check the [Roadmap](ROADMAP.md) for planned features
- Ask in Discussions if you're unsure

### 2. Create a Branch

```bash
git checkout develop
git pull origin develop
git checkout -b feat/my-feature   # or fix/my-bugfix
```

### Branch Naming Convention:
- `feat/description` — new features
- `fix/description` — bug fixes
- `docs/description` — documentation
- `refactor/description` — code refactoring
- `test/description` — test additions
- `perf/description` — performance improvements

### 3. Develop

- Write code following our standards (below)
- Write tests for new functionality
- Run `cargo test` and `cargo clippy` before committing
- Run `pnpm lint` for frontend code

### 4. Commit

We use **Conventional Commits**:

```
feat: add tone curve point editing
fix: correct XMP exposure value parsing
docs: update plugin development guide
refactor: extract shader compilation into module
test: add golden image tests for HSL
perf: optimize thumbnail generation pipeline
chore: update dependencies
```

Keep commits atomic. One logical change per commit.

### 5. Submit PR

- PR against `develop` branch
- Fill out the PR template
- Link related issues
- Ensure CI passes
- Wait for review from module maintainer

### 6. Code Review

- At least 1 approval from a module maintainer
- All CI checks must pass
- Golden image tests must pass (if pipeline changes)
- No unresolved conversations

---

## Code Standards

### Rust
- **Formatting:** `rustfmt` (run `cargo fmt`)
- **Linting:** `clippy` (run `cargo clippy -- -W clippy::all`)
- **Tests:** `#[test]` for unit tests, `tests/` for integration
- **Documentation:** Doc comments for all public APIs
- **Error handling:** `thiserror` for library errors, `anyhow` for application
- **Unsafe:** Only when absolutely necessary, always documented

### TypeScript (UI)
- **Formatting:** `prettier`
- **Linting:** `eslint` with our config
- **Components:** SolidJS functional components
- **Styling:** TailwindCSS utility classes
- **State:** SolidJS stores, no external state library
- **Types:** Strict TypeScript, no `any`

### Shaders (WGSL)
- One file per pipeline stage
- Comments explaining the algorithm
- Performance annotations (workgroup size, memory access pattern)

---

## Specific Contribution Guides

### Adding Camera Profile Support

1. Obtain a test RAW file from the camera (CC0 or your own)
2. Add format parser in `crates/ocps-core/src/raw/formats/`
3. Add color matrix from DNG spec or camera documentation
4. Add demosaic handling (Bayer or X-Trans)
5. Create golden image test
6. Add to supported cameras list in docs
7. Submit PR with test file (via Git LFS)

### Adding Shortcut Sets

1. Create a JSON keymap file: `app/src/keymaps/my-profile.json`
2. Follow the keymap schema (see `app/src/lib/shortcuts.ts`)
3. Run shortcut conflict tests: `pnpm test:shortcuts`
4. Document the profile in `docs/shortcuts.md`
5. Submit PR

### Contributing Presets

1. Create a JSON preset: `presets/community/my-preset.json`
2. Follow the preset schema (see `docs/preset-format.md`)
3. Include a description and tags
4. No copyrighted names (no "Portra 400" — use "Warm Film" etc.)
5. Submit PR

### Adding XMP Compatibility Tests

1. Create a test XMP file (from Lightroom or manually)
2. Place in `tests/xmp-compat/fixtures/`
3. Add test case in `tests/xmp-compat/`
4. Document what the test validates
5. Submit PR

### Submitting Benchmarks

1. Run benchmark suite: `cargo bench`
2. Include hardware info (CPU, GPU, RAM)
3. Include OS and driver versions
4. Submit results as a GitHub Discussion or PR to `docs/benchmarks/`

---

## Reporting Bugs

Use the bug report template. Include:

1. **OCPS version** (from About dialog or `ocps --version`)
2. **OS and version**
3. **GPU and driver version**
4. **Steps to reproduce** (numbered, specific)
5. **Expected behavior**
6. **Actual behavior**
7. **Screenshots/logs** if applicable
8. **Test RAW file** if format-specific (strip personal EXIF first)

---

## RFC Process

For larger changes (new features, architecture changes, breaking changes):

1. Copy `rfcs/0000-template.md` to `rfcs/XXXX-feature-name.md`
2. Fill out all sections
3. Submit PR to `rfcs/` directory
4. 14-day community comment period
5. TC review and vote
6. Implementation after approval

---

## Community Roles

| Role | Requirements | Responsibilities |
|------|-------------|-----------------|
| **Contributor** | Signed CLA, ≥1 merged PR | Submit PRs, file issues |
| **Reviewer** | ≥10 merged PRs, invited by maintainer | Review PRs in their area |
| **Module Maintainer** | ≥20 merged PRs, TC nomination | Approve/merge PRs for their module |
| **Core Maintainer** | TC member | Architecture decisions, releases |
| **Plugin Author** | Published plugin | Plugin ecosystem contributions |
| **Compatibility Tester** | Submitted XMP/RAW test data | Compatibility validation |
| **UX Contributor** | Design proposals accepted | UI/UX improvements |
| **Documentation Contributor** | ≥5 doc PRs | Documentation maintenance |

### Advancement Path

```
Contributor → Reviewer → Module Maintainer → Core Maintainer
                                          ↗
             Plugin Author ──────────────/
             Compatibility Tester ──────/
```

---

## Security

Found a vulnerability? **Do not open a public issue.** See [SECURITY.md](SECURITY.md).

---

## Thank You

Every contribution matters — whether it's a typo fix, a camera profile, a bug report, or a core feature. Thank you for helping build the photo workflow tool the community deserves. 🌊
