# Why This License? — License Choice Rationale

## TL;DR

OpenClaw Photo Studio uses the **PolyForm Noncommercial License 1.0.0** because we want:
- Full source transparency
- Free use for individuals, students, researchers, and non-commercial projects
- Protection against large companies embedding our work into paid products without contributing back

This is **source-available**, not "open source" as defined by the OSI.

---

## Why Not Classic Open Source?

### MIT / Apache 2.0
These licenses allow anyone — including billion-dollar companies — to take the entire codebase, embed it in a commercial product, and sell it without giving anything back. This has happened repeatedly:
- AWS forked Elasticsearch → OpenSearch
- AWS offered Redis as a managed service, competing with Redis Labs
- MongoDB changed to SSPL after AWS offered MongoDB-as-a-Service

We respect MIT/Apache for many projects, but for a product that requires years of engineering effort, they enable a dynamic where the creators bear all the cost and large companies capture the value.

### GPL / AGPL
GPL forces derivative works to also be GPL ("copyleft"). This:
- Scares away commercial contributors and partners
- Makes dual-licensing complicated (CLA becomes more contentious)
- AGPL adds SaaS protection but is even more restrictive
- Legal complexity around "what is a derivative work" creates uncertainty

### SSPL (MongoDB)
Too aggressive. Requires anyone offering the software as a service to open-source their *entire* stack. Not proportionate.

---

## Why PolyForm Noncommercial?

| Property | PolyForm NC |
|----------|-------------|
| Source visible | ✅ |
| Personal use | ✅ Free |
| Academic/Research | ✅ Free |
| Professional photographer using OCPS | ✅ Free |
| Community fork (non-commercial) | ✅ Free |
| Modify and share (non-commercial) | ✅ Free |
| Embed in commercial software | ❌ Needs commercial license |
| Offer as SaaS | ❌ Needs commercial license |
| OEM/White-label | ❌ Needs commercial license |
| Written by lawyers | ✅ PolyForm Project |
| Short and readable | ✅ ~1 page |
| Dual-license compatible | ✅ |

---

## What's Commercially Free, What's Not?

### FREE — No License Needed
- 📸 Photographer editing and exporting photos (even commercially)
- 🎓 Student learning RAW development
- 🔬 Researcher studying image processing algorithms
- 🏠 Hobbyist organizing their photo library
- 🔧 Developer building and testing locally
- 🌐 Non-profit using OCPS for their work
- 🍴 Forking and modifying for non-commercial purposes
- 📦 Building free plugins or presets
- 📝 Writing about or teaching OCPS

### NEEDS COMMERCIAL LICENSE
- 🏭 Camera manufacturer embedding OCPS engine in their software (OEM)
- ☁️ Cloud service offering OCPS as a service (SaaS)
- 🏷️ Company selling a product built on OCPS (White-label)
- 💼 Enterprise deploying to 50+ workstations (Enterprise)
- 📱 Company integrating OCPS into a paid app

### GRAY AREAS (Contact Us)
- 💰 Selling paid support for OCPS (generally OK, contact us)
- 🔌 Selling plugins that deeply integrate with OCPS internals
- 📚 Paid training courses using OCPS (Education license, often free)

---

## Future Option: Business Source License (BSL)

We may migrate to BSL in the future. Under BSL:
- Same restrictions as now for the first 36 months after each release
- After 36 months, each version automatically converts to Apache 2.0
- This gives contributors confidence: "eventually it will be fully free"
- Track record: MariaDB, CockroachDB, Sentry, HashiCorp all use BSL

---

## CLA and Contributor Rights

Because we dual-license, contributors must sign a CLA. We understand this can be a concern. Our commitments:

1. **You retain copyright.** The CLA is a license grant, not a transfer.
2. **Non-exclusive.** You can use your contribution anywhere else.
3. **Fairness clause.** If the project is ever sold to a third party, all CLA contributors automatically receive the right to use their contributions under Apache 2.0.
4. **Transparent.** All license decisions require public RFC and community input.

---

## How We Stay Fair to the Community

1. **No feature gating.** The community version has full functionality.
2. **No "open core" tricks.** We don't cripple the free version.
3. **Community voice.** License changes require 30-day public review + TC supermajority.
4. **Fairness clause in CLA.** Protects contributors from "sell-out" scenarios.
5. **Transparent governance.** All decisions documented publicly.
6. **Commercial revenue funds development.** Not shareholder returns.

---

## Comparison with Similar Projects

| Project | License | Model |
|---------|---------|-------|
| Lightroom | Proprietary | Subscription |
| darktable | GPL-3.0 | Fully open source |
| RawTherapee | GPL-3.0 | Fully open source |
| Capture One | Proprietary | Purchase/Subscription |
| Sentry | BSL → Apache 2.0 | Source-available, time-delayed open |
| GitLab | MIT (CE) + Proprietary (EE) | Open core |
| **OCPS** | **PolyForm NC + Commercial** | **Source-available, dual-license** |

---

*Questions about licensing? Contact licensing@openclaw.photo or open a Discussion.*
