# OpenClaw Photo Studio — Vollständiges Produktkonzept

> Version: 0.1.0-draft
> Autor: Ocean 🌊 (Lead Product Architect)
> Datum: 2026-03-19
> Status: Konzeptphase

---

# Inhaltsverzeichnis

1. [Produktvision](#1-produktvision)
2. [Lizenz- und Geschäftsmodell](#2-lizenz--und-geschäftsmodell)
3. [Governance & Community](#3-governance--community)
4. [Technische Architektur](#4-technische-architektur)
5. [Feature-Spezifikation](#5-feature-spezifikation)
6. [UX & Bedienkonzept](#6-ux--bedienkonzept)
7. [Lightroom-Kompatibilität](#7-lightroom-kompatibilität)
8. [RAW-Processing-Pipeline](#8-raw-processing-pipeline)
9. [Plugin- & Preset-System](#9-plugin---preset-system)
10. [Implementierungsplan & Roadmap](#10-implementierungsplan--roadmap)
11. [Referenzdokumente](#11-referenzdokumente)

---

# 1. PRODUKTVISION

## 1.1 Das Problem

Adobe Lightroom dominiert den Markt für Foto-Workflow-Software seit über 15 Jahren. Fotografen weltweit haben ihre gesamte Arbeitsweise auf Lightroom aufgebaut — Kataloge, Presets, Metadaten-Workflows, Tastenkürzel, RAW-Entwicklungsroutinen. Doch Lightroom hat fundamentale Probleme:

**Für den Endnutzer:**
- **Abo-Zwang:** Seit 2017 gibt es Lightroom nur noch als Abo (Creative Cloud). Wer nicht zahlt, verliert den Zugang zu seinen eigenen Bearbeitungen.
- **Performance-Probleme:** Lightroom Classic ist berüchtigt für Trägheit bei grossen Katalogen (50'000+ Fotos). Die GPU-Nutzung ist unzureichend.
- **Cloud-Abhängigkeit:** Lightroom CC (die "moderne" Version) zwingt Fotos in die Adobe-Cloud. Datenschutz? Datenhoheit? Fehlanzeige.
- **Vendor Lock-in:** Lightroom-Kataloge (.lrcat) sind SQLite-Datenbanken mit proprietärem Schema. Export ist möglich, aber nicht verlustfrei.
- **Feature-Stagnation:** Grundlegende Features wie Ebenen, bessere Maskierung oder moderne UI-Patterns kamen erst nach Jahren oder gar nicht.
- **Keine echte Offline-Fähigkeit** bei Lightroom CC.

**Für die Industrie:**
- Keine professionelle, source-available Alternative mit Lightroom-kompatiblem Workflow.
- darktable und RawTherapee sind technisch stark, aber UX-schwach und Lightroom-fremd in der Bedienung.
- Capture One ist proprietär und noch teurer als Lightroom.
- Es gibt keinen "VS Code der Fotobearbeitung" — eine tool-agnostische, erweiterbare, performante Plattform.

## 1.2 Zielgruppe

### Primär
- **Professionelle Fotografen** (Hochzeit, Portrait, Landschaft, Architektur), die 500–5'000 Fotos pro Shooting verarbeiten
- **Ambitionierte Hobby-Fotografen**, die RAW-Entwicklung betreiben und Lightroom-Workflows gewohnt sind
- **Fotografen, die Adobe verlassen wollen**, aber keinen Workflow-Bruch akzeptieren

### Sekundär
- **Fotoagenturen und Redaktionen**, die einen lokalen, kontrollierbaren Workflow brauchen
- **Foto-Ausbilder und Schulen**, die eine kostenlose, lehrbare Software benötigen
- **Entwickler und Plugin-Autoren**, die eine erweiterbare Plattform suchen

### Tertiär
- **OEMs und Kamerahersteller**, die eine einbettbare Foto-Workflow-Engine suchen (→ kommerzielle Lizenz)
- **SaaS-Anbieter**, die cloudbasierte Fotobearbeitung anbieten wollen

## 1.3 Warum würden Fotografen wechseln?

| Grund | Erklärung |
|-------|-----------|
| **Kein Abo** | Einmal herunterladen, für immer nutzen. Kein monatlicher Tribut. |
| **Vertrauter Workflow** | Fühlt sich an wie Lightroom. Gleiche Logik, ähnliche Shortcuts, kompatible Presets. |
| **Lokale Datenhoheit** | Fotos bleiben auf der eigenen Festplatte. Kein Cloud-Zwang. |
| **Performance** | GPU-native Rendering, moderner Tech-Stack, reaktive UI auch bei 100'000+ Fotos. |
| **Erweiterbar** | Plugin-System, Community-Presets, Custom-Scripts. |
| **Zukunftssicher** | Source-available. Kein Vendor kann die Software "abstellen". |
| **Tastatur-First** | Jede Aktion per Shortcut. Vim-artige Effizienz für Power-User. |
| **Batch-Workflows** | Copy/Paste von Edits über 1'000 Fotos in Sekunden. |

## 1.4 Kernversprechen

1. **Lightroom-ähnlicher Workflow:** Import → Sichten → Bewerten → Entwickeln → Exportieren. Gleiche Denkweise, vertraute Logik.
2. **Starke Performance:** GPU-beschleunigtes RAW-Rendering. Flüssige Navigation bei grossen Bibliotheken. Sub-100ms-Vorschau.
3. **Volle Tastaturbedienung:** Jede Aktion hat einen Shortcut. Konfigurierbar. Vim-inspirierte Modes für Power-User.
4. **Nicht-destruktive Bearbeitung:** Originaldateien werden nie verändert. Alle Edits in Sidecar-Dateien (XMP-kompatibel).
5. **Lightroom-nahe Metadaten:** Import und Export von XMP-Sidecars, IPTC, EXIF. Maximal kompatibel.
6. **Professionelle RAW-Entwicklung:** Vergleichbare Qualität zu Lightroom/Capture One durch moderne demosaicing-Algorithmen.
7. **Batch-Workflows:** Copy/Paste von Edits, Batch-Export, Batch-Metadaten-Editing.
8. **Lokale Datenhoheit:** Alles lokal. Optional Sync, aber nie Zwang.
9. **Plugin- und Preset-Architektur:** Offen, dokumentiert, community-freundlich.

## 1.5 Mission

> **Die professionellste source-available Foto-Workflow-Software der Welt bauen — so vertraut wie Lightroom, so schnell wie nativ, so offen wie möglich.**

## 1.6 Vision

> **In 3 Jahren ist OpenClaw Photo Studio die erste Wahl für Fotografen, die Lightroom-Qualität ohne Lightroom-Abhängigkeit wollen. In 5 Jahren ist es der Standard für erweiterbare Foto-Workflows.**

## 1.7 Product Principles

1. **Familiarity over Novelty** — Lightroom-Nutzer sollen sich sofort zurechtfinden. Wir innovieren subtil, nicht disruptiv.
2. **Speed is a Feature** — Jede Interaktion unter 100ms. Kein Spinner, kein "Bitte warten". Performance ist nicht verhandelbar.
3. **Keyboard-First, Mouse-Friendly** — Alles per Tastatur möglich. Maus/Trackpad bleibt erstklassig, aber Shortcuts sind der Primärweg.
4. **Non-Destructive Always** — Originaldateien sind heilig. Kein Workflow darf sie verändern.
5. **Local-First, Cloud-Optional** — Fotos und Kataloge leben lokal. Cloud-Sync ist ein Feature, kein Requirement.
6. **Extensibility over Completeness** — Lieber ein starkes Plugin-System als jeden Nischen-Use-Case in Core packen.
7. **Standards over Proprietary** — XMP, IPTC, EXIF, ICC, DNG. Offene Standards zuerst.
8. **Honest UX** — Keine Dark Patterns. Keine versteckten Uploads. Keine "Upgrade to Pro"-Nags.

## 1.8 Non-Goals

- **Kein Photoshop-Ersatz:** Wir bauen keine Pixel-Editing-Software mit Ebenen, Compositing, Text-Overlay etc. Lightroom ≠ Photoshop.
- **Kein DAM-System:** Wir sind eine Foto-Workflow-Software, kein Digital-Asset-Management für Agenturen mit 10 Millionen Files.
- **Kein Video-Editor:** Video-Unterstützung maximal als Vorschau/Metadaten, nicht als Schnitt-Tool.
- **Kein Cloud-Service:** Wir bauen Software, keinen SaaS. Cloud-Sync ist ein optionaler Layer.
- **Kein Mobile-First:** Desktop ist Priorität 1. Mobile kommt eventuell als Companion, nicht als Primärprodukt.
- **Keine KI-Zwangsintegration:** KI-Features (Auto-Masking, Subject Detection) als optionale Plugins, nicht als Core-Abhängigkeit.

## 1.9 Differenzierung

### vs. Adobe Lightroom Classic
| Aspekt | Lightroom Classic | OpenClaw Photo Studio |
|--------|-------------------|-----------------------|
| Preis | ~12€/Monat Abo | Kostenlos (Community) |
| Quellcode | Geschlossen | Source-available |
| Performance | Oft träge | GPU-native, reaktiv |
| Datenhoheit | Adobe-Cloud optional | 100% lokal |
| Erweiterbarkeit | Lua-Plugins (eingeschränkt) | Volles Plugin-System |
| Tastatur | Gut, aber nicht konfigurierbar | Keyboard-First, voll konfigurierbar |
| Lock-in | Proprietärer Katalog | Offene Formate (SQLite + XMP) |

### vs. darktable
| Aspekt | darktable | OpenClaw Photo Studio |
|--------|----------|-----------------------|
| UX | Technisch, steil | Lightroom-vertraut |
| Workflow | Eigen, komplex | Lightroom-kompatibel |
| Presets | Eigenes Format | XMP-kompatibel + eigene |
| Lernkurve | Hoch | Niedrig für Lightroom-User |
| Shortcut-System | Basisch | Vim-artig, konfigurierbar |
| Copy/Paste Edits | Umständlich | Ein Tastendruck |

### vs. RawTherapee
| Aspekt | RawTherapee | OpenClaw Photo Studio |
|--------|------------|-----------------------|
| Bibliothek/Katalog | Keine | Vollständig |
| Workflow | Einzelbild-fokussiert | Shooting-basiert |
| Batch-Verarbeitung | Begrenzt | Erstklassig |
| UI | Veraltet | Modern, reaktiv |

### vs. Capture One
| Aspekt | Capture One | OpenClaw Photo Studio |
|--------|------------|-----------------------|
| Preis | ~350€ oder Abo | Kostenlos (Community) |
| Quellcode | Geschlossen | Source-available |
| Lock-in | Proprietär | Offene Formate |
| Plugin-System | Eingeschränkt | Voll erweiterbar |

---

# 2. LIZENZ- UND GESCHÄFTSMODELL

## 2.1 Lizenzphilosophie

OpenClaw Photo Studio ist **source-available**, nicht "Open Source" im Sinne der OSI-Definition. Der Quellcode ist frei einsehbar, frei nutzbar für nicht-kommerzielle Zwecke, und frei modifizierbar für die Community. Kommerzielle Einbettung erfordert eine separate Lizenz.

**Warum nicht klassisch Open Source (MIT/Apache/GPL)?**

- MIT/Apache: Erlaubt jedem, die Software in kommerzielle Produkte einzubetten, ohne zurückzugeben. Amazon-Effekt (siehe Elasticsearch/OpenSearch, Redis, MongoDB).
- GPL/AGPL: Erzwingt Copyleft, aber schreckt kommerzielle Partner und Contributors ab. Rechtlich komplex bei SaaS.
- Ziel: Die Vorteile von Open Source (Transparenz, Community, Vertrauen) OHNE den Nachteil, dass grosse Firmen den Code gratis in ihre Produkte packen.

## 2.2 Lizenzstrategie (Drei-Säulen-Modell)

### Säule A: PolyForm Noncommercial License 1.0.0

**Anwendung:** Standard-Lizenz für den gesamten Quellcode im Repository.

**Was sie erlaubt:**
- Einsehen, Herunterladen, Kompilieren des Quellcodes
- Private Nutzung (Fotos bearbeiten, lernen, experimentieren)
- Akademische Nutzung und Forschung
- Community-Weiterentwicklung (Patches, Plugins, Forks für nicht-kommerzielle Zwecke)
- Nutzung durch Non-Profits und NGOs
- Nutzung durch Einzelpersonen und Kleinunternehmen für interne Fotobearbeitung (kein Weiterverkauf)

**Was sie verbietet:**
- Einbettung in kommerzielle Software (OEM)
- Anbieten als SaaS oder Managed Service
- Verkauf von modifizierten Versionen
- Nutzung in bezahlten Produkten oder Dienstleistungen, die die Software als Kernbestandteil enthalten

**Warum PolyForm Noncommercial?**
- Von Anwälten geschrieben, rechtlich sauber
- Kurz, verständlich, keine Interpretationslücken
- Explizit für source-available-Projekte designed
- Kompatibel mit dual-licensing

### Säule B: Kommerzielle Lizenz (OpenClaw Photo Studio Commercial License)

**Anwendung:** Für jede Nutzung, die über PolyForm Noncommercial hinausgeht.

**Lizenztypen:**

| Typ | Zielgruppe | Preis (indikativ) |
|-----|-----------|-------------------|
| **OEM Embed** | Kamerahersteller, die die Engine einbetten | Verhandlungsbasis, ab 50'000€/Jahr |
| **SaaS** | Cloud-Foto-Dienste | Umsatzbeteiligung (3-5%) oder Flat Fee |
| **Enterprise** | Grosse Firmen mit >50 Nutzern für internen Gebrauch | Ab 5'000€/Jahr |
| **Indie Commercial** | Einzelentwickler/Startups, die Plugins/Add-ons verkaufen | Ab 500€/Jahr |
| **Education Commercial** | Schulen/Unis, die das Tool in bezahlten Kursen verwenden | Kostenlos bis 100 Seats, danach 10€/Seat/Jahr |

**Vertragsbasis:**
- Standard-Vertrag als Template (COMMERCIAL-LICENSE-AGREEMENT.md)
- Custom-Verhandlungen für OEM und Enterprise
- Kontakt: licensing@openclaw.photo (oder äquivalent)

### Säule C: Business Source License (BSL) — Zukunftsoption

**Das Modell (à la MariaDB/CockroachDB/Sentry):**
- Code wird unter BSL veröffentlicht
- Nach einem **License Change Date** (z.B. 36 oder 48 Monate nach Release einer Version) wird der Code automatisch unter einer permissiven Lizenz (Apache 2.0 oder PolyForm Shield) verfügbar
- Bis dahin gelten die Einschränkungen der Säule A

**Warum als Option vorhalten?**
- BSL hat Track-Record (HashiCorp, Sentry, CockroachDB, MariaDB)
- Es gibt Contributors Vertrauen: "Irgendwann wird es frei"
- Es schützt die ersten Jahre der Kommerzialisierung
- Es ist ein Mittelweg zwischen "ganz offen" und "ganz geschlossen"

**Empfehlung:** Start mit PolyForm Noncommercial (Säule A). Wenn das Projekt Traktion gewinnt und kommerzielle Partner auftreten, Wechsel auf BSL mit 36-Monats-Conversion.

## 2.3 Geschäftsmodell

### Revenue Streams

```
┌─────────────────────────────────────────────────────────┐
│                    REVENUE MODEL                         │
├──────────────────┬──────────────────────────────────────┤
│ Stream           │ Beschreibung                         │
├──────────────────┼──────────────────────────────────────┤
│ Kommerzielle     │ OEM, SaaS, Enterprise Lizenzen       │
│ Lizenzen         │ (Säule B)                            │
├──────────────────┼──────────────────────────────────────┤
│ Support &        │ Priority Support, SLAs,              │
│ Consulting       │ Custom-Entwicklung                   │
├──────────────────┼──────────────────────────────────────┤
│ Pro Features     │ Optional: Cloud-Sync, Team-Features, │
│ (optional)       │ erweiterte KI-Tools                  │
├──────────────────┼──────────────────────────────────────┤
│ Marketplace      │ Provision auf Premium-Plugins und    │
│ Commission       │ Preset-Pakete (15-30%)               │
├──────────────────┼──────────────────────────────────────┤
│ Training &       │ Zertifizierungsprogramm für          │
│ Certification    │ Fotografen und Entwickler             │
├──────────────────┼──────────────────────────────────────┤
│ Sponsoring       │ GitHub Sponsors, Open Collective     │
│                  │ für Community-Entwicklung             │
└──────────────────┴──────────────────────────────────────┘
```

### Pricing Philosophy

- **Community-Version:** 100% kostenlos, voller Funktionsumfang für RAW-Entwicklung und Katalog
- **Keine Feature-Gating-Tricks:** Die kostenlose Version ist nicht kastriert
- **Kommerzielle Lizenz:** Nur relevant, wenn jemand die Software *weiterverkauft* oder *einbettet*
- **Faire Preise:** Ein Indie-Entwickler zahlt nicht dasselbe wie Canon oder Samsung

## 2.4 LICENSE-Datei (Draft)

```
OpenClaw Photo Studio
Copyright (c) 2026 OpenClaw Photo Studio Contributors

This software is licensed under the PolyForm Noncommercial License 1.0.0.
See LICENSE-POLYFORM.md for the full license text.

For commercial licensing options, see COMMERCIAL.md
or contact licensing@openclaw.photo.

This is source-available software, not "open source" as defined by the
Open Source Initiative (OSI). Commercial use requires a separate license.
```

---

# 3. GOVERNANCE & COMMUNITY

## 3.1 Governance-Modell: Benevolent Steward + Technical Council

### Struktur

```
┌──────────────────────────────────────────────────┐
│              PROJECT GOVERNANCE                    │
├──────────────────────────────────────────────────┤
│                                                    │
│  ┌─────────────────────────────┐                  │
│  │    Project Steward           │                  │
│  │    (Benevolent Dictator)     │                  │
│  │    → Endgültige Entscheide   │                  │
│  │    → Lizenzfragen            │                  │
│  │    → Trademark               │                  │
│  └──────────┬──────────────────┘                  │
│             │                                      │
│  ┌──────────▼──────────────────┐                  │
│  │    Technical Council (TC)    │                  │
│  │    3-7 gewählte Maintainer   │                  │
│  │    → Architektur-Entscheide  │                  │
│  │    → Release-Freigabe        │                  │
│  │    → RFC-Review              │                  │
│  └──────────┬──────────────────┘                  │
│             │                                      │
│  ┌──────────▼──────────────────┐                  │
│  │    Module Maintainers        │                  │
│  │    Pro Modul 1-3 Personen    │                  │
│  │    → Code-Review             │                  │
│  │    → Modul-spezifische       │                  │
│  │      Entscheide              │                  │
│  └──────────┬──────────────────┘                  │
│             │                                      │
│  ┌──────────▼──────────────────┐                  │
│  │    Contributors              │                  │
│  │    Jeder mit signiertem CLA  │                  │
│  │    → PRs, Issues, RFCs       │                  │
│  │    → Plugin-Entwicklung      │                  │
│  │    → Dokumentation           │                  │
│  └─────────────────────────────┘                  │
│                                                    │
└──────────────────────────────────────────────────┘
```

### Rollen

**Project Steward (1 Person, initial: Gründer)**
- Finales Veto bei Lizenzänderungen
- Trademark-Entscheide
- Steward kann Rolle übertragen, aber nur an vom TC bestätigte Person
- Kann vom TC überstimmt werden bei rein technischen Fragen (4/5 Mehrheit)

**Technical Council (TC, 3-7 Personen)**
- Gewählt von aktiven Contributors (≥5 merged PRs in 12 Monaten)
- 2-Jahres-Terme, gestaffelt
- Entscheidet über: Architektur-RFCs, Breaking Changes, Release-Readiness
- Mindestens 1 Seat für "Community-at-large" (Nicht-Kern-Entwickler)

**Module Maintainers**
- Verantwortlich für spezifische Module (RAW-Engine, UI, Katalog, Plugin-System etc.)
- Nominiert vom TC, bestätigt durch Community-Vote
- Review-Pflicht für PRs in ihrem Modul
- Können Junior Maintainers ernennen

**Contributors**
- Jeder, der Code, Docs, Translations, Tests oder Designs beiträgt
- Muss CLA/DCO unterzeichnet haben
- Erhält nach 5 merged PRs Stimmrecht für TC-Wahlen

## 3.2 Decision-Making: RFC-Prozess

Für alle nicht-trivialen Änderungen (neue Features, Architektur, API-Änderungen, Breaking Changes):

```
1. RFC erstellen      → rfcs/0000-feature-name.md
2. Community Review   → 14 Tage Kommentarphase
3. TC Discussion      → Technische Bewertung
4. TC Vote            → Einfache Mehrheit
5. Implementation     → Zugeordnet an Maintainer/Contributor
6. Merge              → Nach Code-Review durch Modul-Maintainer
```

**RFC Template:**
```markdown
# RFC-XXXX: [Titel]

## Zusammenfassung
## Motivation
## Detailliertes Design
## Alternativen
## Migrationsstrategie (falls Breaking Change)
## Ungelöste Fragen
```

## 3.3 CLA-Strategie (Contributor License Agreement)

### Empfehlung: CLA (nicht DCO)

**Begründung:**
- Bei einem source-available-Projekt mit kommerzieller Lizenz MUSS der Projektbetreiber das Recht haben, Code unter verschiedenen Lizenzen anzubieten
- Ein DCO (Developer Certificate of Origin) bestätigt nur die Herkunft, nicht die Lizenzübertragung
- Ein CLA überträgt dem Projekt die notwendigen Rechte, um den Code auch unter der kommerziellen Lizenz anzubieten

**CLA-Inhalt (vereinfacht):**

> Ich gewähre dem OpenClaw Photo Studio Projekt eine unbefristete, weltweite, nicht-exklusive, kostenlose, unwiderrufliche Lizenz, meinen Beitrag unter jeder Lizenz zu verwenden, zu kopieren, zu modifizieren und zu verbreiten, die das Projekt für angemessen hält — einschliesslich der PolyForm Noncommercial License, der kommerziellen Lizenz und zukünftiger Lizenzen.
>
> Ich behalte alle Rechte an meinem Beitrag. Der Beitrag wird nicht exklusiv übertragen.
>
> Ich bestätige, dass ich das Recht habe, diesen Beitrag zu machen, und dass er nicht die Rechte Dritter verletzt.

**Signing-Prozess:**
- CLA-Bot auf GitHub (z.B. CLA Assistant von SAP)
- Einmalig per GitHub-Account
- Automatische PR-Checks

**Fairness-Klausel:**
- Wenn das Projekt jemals vollständig an eine andere Entität verkauft wird, erhalten alle CLA-Contributor das Recht, ihre Beiträge unter Apache 2.0 zu nutzen
- Verhindert "Sell-out to Oracle"-Szenarien

## 3.4 CONTRIBUTING.md (Draft)

```markdown
# Contributing to OpenClaw Photo Studio

Willkommen! Wir freuen uns über jeden Beitrag.

## Bevor du anfängst

1. **Lies die [GOVERNANCE.md](GOVERNANCE.md)** — verstehe, wie Entscheidungen getroffen werden
2. **Unterzeichne den CLA** — wird automatisch beim ersten PR angefragt
3. **Lies den [Code of Conduct](CODE_OF_CONDUCT.md)**

## Arten von Beiträgen

- 🐛 **Bug Reports:** Issues mit Reproduktionsschritten
- 💡 **Feature Requests:** Issues mit Use-Case-Beschreibung
- 📝 **RFCs:** Für grössere Änderungen → rfcs/ Verzeichnis
- 🔧 **Code:** PRs gegen `develop` Branch
- 📖 **Dokumentation:** Immer willkommen
- 🌍 **Übersetzungen:** i18n-Beiträge
- 🎨 **Design:** UI/UX-Vorschläge mit Mockups
- 🧪 **Tests:** Unit, Integration, Visual Regression

## Workflow

1. Fork das Repository
2. Erstelle einen Feature-Branch: `git checkout -b feat/my-feature`
3. Entwickle mit Tests
4. Stelle sicher, dass `cargo test` und `cargo clippy` durchlaufen
5. Erstelle einen PR gegen `develop`
6. Warte auf Review durch Modul-Maintainer

## Code-Standards

- Sprache: Rust (Core), TypeScript (UI)
- Formatting: `rustfmt` / `prettier`
- Linting: `clippy` / `eslint`
- Tests: Mindestens Unit-Tests für neue Funktionalität
- Commits: Conventional Commits (feat:, fix:, docs:, refactor:, test:)
- PRs: Ein Feature pro PR, aussagekräftige Beschreibung

## Lizenzhinweis

Alle Beiträge werden unter der PolyForm Noncommercial License 1.0.0
UND der kommerziellen Lizenz des Projekts veröffentlicht
(siehe CLA für Details).
```

## 3.5 COMMERCIAL.md (Draft)

```markdown
# Kommerzielle Nutzung von OpenClaw Photo Studio

## Wann brauche ich eine kommerzielle Lizenz?

Du brauchst eine kommerzielle Lizenz, wenn du OpenClaw Photo Studio:

- In ein kommerzielles Produkt einbettest (OEM)
- Als SaaS oder Managed Service anbietest
- In bezahlter Software verwendest, die OpenClaw Photo Studio als
  Kernbestandteil enthält
- In einem Unternehmen mit >50 Nutzern einsetzt

## Wann brauche ich KEINE kommerzielle Lizenz?

- Private Nutzung (egal ob Hobby oder Beruf — solange du die Software
  selbst zum Fotos bearbeiten verwendest)
- Akademische Nutzung und Forschung
- Non-Profit-Organisationen
- Unternehmen mit ≤50 Nutzern für interne Fotobearbeitung
- Entwicklung von Plugins, die separat vertrieben werden
  (Plugin-Lizenz, nicht OEM)

## Lizenztypen

| Typ | Ab-Preis | Kontakt |
|-----|----------|---------|
| Indie Commercial | 500€/Jahr | licensing@openclaw.photo |
| Enterprise (>50 Seats) | 5'000€/Jahr | licensing@openclaw.photo |
| OEM Embed | Verhandlungsbasis | licensing@openclaw.photo |
| SaaS | Umsatzbeteiligung | licensing@openclaw.photo |

## FAQ

**Bin ich Fotograf und nutze die Software beruflich — brauche ich eine Lizenz?**
Nein. Du nutzt die Software als Endnutzer. Die kommerzielle Lizenz gilt
nur für Weiterverkauf und Einbettung.

**Ich verkaufe Presets — brauche ich eine Lizenz?**
Nein, Presets sind eigenständige Werke. Du brauchst keine Lizenz von uns.

**Ich baue einen Online-Fotoeditor auf Basis der Engine — brauche ich eine Lizenz?**
Ja. Das ist SaaS-Nutzung. Kontaktiere uns.

**Kann ich die Software in meinem Fotokurs verwenden?**
Ja, bis 100 Seats kostenlos. Darüber hinaus Education-Lizenz.
```

## 3.6 Trademark-Strategie

### Geschützte Marken
- **"OpenClaw Photo Studio"** — Wortmarke (Registrierung empfohlen)
- **Logo/Icon** — Bildmarke
- **"OCPS"** — Kurzform

### Trademark Policy

**Erlaubt:**
- Nutzung des Namens in Reviews, Artikeln, Tutorials
- "Compatible with OpenClaw Photo Studio" für Plugins
- "Built on OpenClaw Photo Studio" mit kommerzieller Lizenz

**Verboten:**
- Nutzung des Namens für Forks, die den Eindruck erwecken, sie seien das Originalprojekt
- Nutzung des Logos in kommerziellen Produkten ohne Genehmigung
- Nutzung in Domainnamen, die Verwechslungsgefahr erzeugen

**Sonderregel für Community-Forks:**
- Forks müssen umbenannt werden, wenn sie >20% des Codes ändern oder eigene Features hinzufügen, die nicht upstream gehen
- "OpenClaw Photo Studio Community Fork" ist erlaubt, wenn der Fork aktiv upstream merged

---

# 4. TECHNISCHE ARCHITEKTUR

## 4.1 Designprinzipien

1. **Modular Monolith, nicht Microservices** — Ein Prozess, klar getrennte Module, keine RPC-Overhead.
2. **Rust Core + Web-UI** — Performance-kritische Teile in Rust, UI in TypeScript/WebView.
3. **GPU-First Rendering** — Alle Bildoperationen über GPU (wgpu/Vulkan/Metal).
4. **Local-First Data** — SQLite für Katalog, Dateisystem für Sidecars, kein Server nötig.
5. **Plugin-Sandbox** — Plugins laufen isoliert, können die Hauptanwendung nicht crashen.
6. **Cross-Platform** — macOS, Windows, Linux. Gleicher Code, native Performance.

## 4.2 Architektur-Übersicht

```
┌──────────────────────────────────────────────────────────────────────┐
│                        APPLICATION SHELL                              │
│                     (Tauri v2 / Native WebView)                       │
├──────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │                      UI LAYER (TypeScript)                      │  │
│  │                                                                  │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │  │
│  │  │ Library  │  │ Develop  │  │  Map     │  │  Print   │      │  │
│  │  │ Module   │  │ Module   │  │ Module   │  │ Module   │      │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘      │  │
│  │       │              │              │              │            │  │
│  │  ┌────▼──────────────▼──────────────▼──────────────▼────────┐  │  │
│  │  │              UI Framework (SolidJS + TailwindCSS)          │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │  │  │
│  │  │  │ Shortcut │  │ Command  │  │ Theme    │               │  │  │
│  │  │  │ Engine   │  │ Palette  │  │ Engine   │               │  │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘               │  │  │
│  │  └───────────────────────┬──────────────────────────────────┘  │  │
│  └──────────────────────────┼────────────────────────────────────┘  │
│                              │ IPC (Tauri Commands)                   │
│  ┌──────────────────────────▼────────────────────────────────────┐  │
│  │                      CORE ENGINE (Rust)                         │  │
│  │                                                                  │  │
│  │  ┌──────────────────────────────────────────────────────────┐  │  │
│  │  │                   Module Registry                          │  │  │
│  │  └──────────────────────────────────────────────────────────┘  │  │
│  │                                                                  │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐               │  │
│  │  │ RAW Engine │  │  Catalog   │  │  Export    │               │  │
│  │  │            │  │  Engine    │  │  Engine    │               │  │
│  │  │ • Demosaic │  │            │  │            │               │  │
│  │  │ • WB       │  │ • SQLite   │  │ • JPEG     │               │  │
│  │  │ • Tone Map │  │ • Index    │  │ • TIFF     │               │  │
│  │  │ • HSL      │  │ • Search   │  │ • PNG      │               │  │
│  │  │ • Curves   │  │ • Filter   │  │ • DNG      │               │  │
│  │  │ • Sharpn.  │  │ • Smart    │  │ • WebP     │               │  │
│  │  │ • NR       │  │   Collect. │  │ • Batch    │               │  │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘               │  │
│  │         │               │               │                       │  │
│  │  ┌──────▼───────────────▼───────────────▼───────────────────┐  │  │
│  │  │                   Shared Services                          │  │  │
│  │  │                                                            │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│  │  │
│  │  │  │ GPU      │  │ XMP/IPTC │  │ ICC      │  │ Plugin   ││  │  │
│  │  │  │ Pipeline │  │ Engine   │  │ Color    │  │ Host     ││  │  │
│  │  │  │ (wgpu)   │  │          │  │ Mgmt     │  │ (WASM)   ││  │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘│  │  │
│  │  │                                                            │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐               │  │  │
│  │  │  │ File     │  │ Cache    │  │ Undo     │               │  │  │
│  │  │  │ System   │  │ Manager  │  │ History  │               │  │  │
│  │  │  │ Watcher  │  │ (LRU)   │  │ Stack    │               │  │  │
│  │  │  └──────────┘  └──────────┘  └──────────┘               │  │  │
│  │  └──────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      STORAGE LAYER                                │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │  │
│  │  │ SQLite   │  │ XMP      │  │ Preview  │  │ Config   │        │  │
│  │  │ Catalog  │  │ Sidecars │  │ Cache    │  │ Files    │        │  │
│  │  │ (.ocps)  │  │ (.xmp)   │  │ (.cache) │  │ (.toml)  │        │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
└──────────────────────────────────────────────────────────────────────┘
```

## 4.3 Tech-Stack

### Core Engine (Rust)
| Komponente | Technologie | Begründung |
|-----------|-------------|------------|
| Sprache | **Rust** | Memory-Safety, Performance, kein GC, WebAssembly-Kompatibilität |
| RAW-Decoding | **rawloader** + custom | Rust-native RAW-Dekodierung, erweiterbar pro Kameramodell |
| Demosaicing | Custom (AMaZE, RCD) | Portierung bewährter Algorithmen aus dcraw/LibRaw |
| GPU Pipeline | **wgpu** | Cross-Platform GPU (Vulkan/Metal/DX12/WebGPU) |
| Image Processing | Custom Shader (WGSL) | Maximale GPU-Auslastung, keine CPU-Bottlenecks |
| Color Management | **lcms2** (FFI) oder **littlecms-rs** | ICC-Profil-Handling, Standard in der Branche |
| Katalog DB | **rusqlite** (SQLite) | Bewährt, schnell, embedded, keine Infrastruktur nötig |
| XMP/IPTC | **xmp-toolkit-rs** + custom | Lesen/Schreiben von Adobe-kompatiblen XMP-Sidecars |
| EXIF | **kamadak-exif** oder **rexiv2** | Schnelles EXIF-Parsing |
| File Watching | **notify** | Cross-Platform Dateisystem-Events |
| Async Runtime | **tokio** | Async I/O für File-Operationen und Plugin-Kommunikation |
| Serialization | **serde** | JSON, TOML, MessagePack für Config und IPC |
| Plugin Host | **wasmtime** | WASM-basierte Plugin-Isolation |
| CLI | **clap** | Für headless Batch-Verarbeitung |

### UI Layer (TypeScript)
| Komponente | Technologie | Begründung |
|-----------|-------------|------------|
| Framework | **SolidJS** | Reaktiv wie React, aber schneller (kein VDOM), feingranulare Updates |
| Styling | **TailwindCSS** | Utility-first, konsistentes Design, Dark/Light Mode |
| State Management | **SolidJS Stores** | Reactive Stores, kein Redux-Overhead |
| Canvas Rendering | **Canvas 2D / WebGL** | Für Histogramm, Kurven, Bildvorschau |
| Shortcut Engine | **Custom (tinykeys-inspiriert)** | Vim-artige Modes, voll konfigurierbar |
| Command Palette | **Custom (cmdk-inspiriert)** | Fuzzy-Search über alle Aktionen |
| Icons | **Lucide** | Clean, konsistent, open-source |
| Drag & Drop | **@thisbeyond/solid-dnd** | Für Filmstrip, Sammlungen, Sortierung |
| Virtualization | **Custom Virtual List** | Für 100'000+ Thumbnails ohne DOM-Explosion |
| i18n | **@solid-primitives/i18n** | Mehrsprachigkeit von Anfang an |

### Application Shell
| Komponente | Technologie | Begründung |
|-----------|-------------|------------|
| Desktop Runtime | **Tauri v2** | Rust-native, klein (<10MB), kein Electron-Bloat |
| IPC | **Tauri Commands** | Type-safe Rust↔TypeScript Kommunikation |
| Auto-Update | **Tauri Updater** | Built-in Update-System |
| File Dialogs | **Tauri Dialog** | Native OS-Dialoge |
| System Tray | **Tauri Tray** | Für Hintergrund-Exports |

### Warum nicht Electron?
- Electron-Apps: ~150-300MB, hoher RAM-Verbrauch
- Tauri-Apps: ~5-15MB, nativer RAM-Verbrauch
- Für eine Foto-App, die neben 50GB RAW-Dateien arbeitet, ist jedes MB RAM wertvoll

### Warum nicht native GUI (GTK/Qt/SwiftUI)?
- Cross-Platform-Konsistenz: Web-Technologie sieht überall gleich aus
- Schnellere UI-Entwicklung: CSS/HTML ist produktiver als native UI-Code
- Plugin-UI: Plugins können HTML/CSS für ihre Oberfläche verwenden
- Tauri-WebView nutzt native WebView (Safari/Chromium), kein gebundener Browser

## 4.4 Datenmodell

### Katalog (SQLite Schema — vereinfacht)

```sql
-- Kern-Tabellen

CREATE TABLE photos (
    id              TEXT PRIMARY KEY,     -- UUID
    file_path       TEXT NOT NULL,        -- Absoluter oder relativer Pfad
    file_name       TEXT NOT NULL,
    file_size       INTEGER,
    file_hash       TEXT,                 -- SHA-256 für Duplikat-Erkennung
    mime_type       TEXT,
    width           INTEGER,
    height          INTEGER,
    orientation     INTEGER,
    date_taken      TEXT,                 -- ISO 8601
    date_imported   TEXT NOT NULL,
    camera_make     TEXT,
    camera_model    TEXT,
    lens            TEXT,
    focal_length    REAL,
    aperture        REAL,
    shutter_speed   TEXT,
    iso             INTEGER,
    gps_lat         REAL,
    gps_lon         REAL,
    gps_alt         REAL,
    rating          INTEGER DEFAULT 0,    -- 0-5 Sterne
    color_label     TEXT,                 -- red, yellow, green, blue, purple
    flag            TEXT DEFAULT 'none',  -- none, pick, reject
    has_edits       BOOLEAN DEFAULT 0,
    edit_version    INTEGER DEFAULT 0,
    virtual_copy_of TEXT REFERENCES photos(id),
    UNIQUE(file_path)
);

CREATE TABLE collections (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    type            TEXT DEFAULT 'manual', -- manual, smart, quick
    parent_id       TEXT REFERENCES collections(id),
    smart_rules     TEXT,                  -- JSON für Smart Collections
    sort_order      INTEGER,
    created_at      TEXT,
    updated_at      TEXT
);

CREATE TABLE collection_photos (
    collection_id   TEXT REFERENCES collections(id) ON DELETE CASCADE,
    photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
    sort_order      INTEGER,
    PRIMARY KEY (collection_id, photo_id)
);

CREATE TABLE folders (
    id              TEXT PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    parent_id       TEXT REFERENCES folders(id),
    is_watched      BOOLEAN DEFAULT 0,
    last_scanned    TEXT
);

CREATE TABLE keywords (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    parent_id       TEXT REFERENCES keywords(id),
    synonyms        TEXT                   -- JSON Array
);

CREATE TABLE photo_keywords (
    photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
    keyword_id      TEXT REFERENCES keywords(id) ON DELETE CASCADE,
    PRIMARY KEY (photo_id, keyword_id)
);

CREATE TABLE edits (
    id              TEXT PRIMARY KEY,
    photo_id        TEXT REFERENCES photos(id) ON DELETE CASCADE,
    version         INTEGER NOT NULL,
    edit_data       TEXT NOT NULL,          -- JSON: alle Entwicklungseinstellungen
    created_at      TEXT,
    snapshot_name   TEXT,                   -- Benannte Snapshots
    is_current      BOOLEAN DEFAULT 1
);

CREATE TABLE presets (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    group_name      TEXT,
    edit_data       TEXT NOT NULL,          -- JSON: Preset-Daten
    source          TEXT DEFAULT 'user',    -- user, builtin, imported
    is_favorite     BOOLEAN DEFAULT 0,
    created_at      TEXT
);

CREATE TABLE export_presets (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    settings        TEXT NOT NULL           -- JSON: Export-Einstellungen
);

-- Indizes für Performance
CREATE INDEX idx_photos_date ON photos(date_taken);
CREATE INDEX idx_photos_rating ON photos(rating);
CREATE INDEX idx_photos_flag ON photos(flag);
CREATE INDEX idx_photos_camera ON photos(camera_make, camera_model);
CREATE INDEX idx_photos_path ON photos(file_path);
CREATE INDEX idx_photos_hash ON photos(file_hash);
CREATE INDEX idx_keywords_name ON keywords(name);

-- Full-Text-Search
CREATE VIRTUAL TABLE photos_fts USING fts5(
    file_name, camera_make, camera_model, lens,
    content='photos', content_rowid='rowid'
);
```

### Edit-Daten (JSON-Struktur)

```json
{
  "version": 1,
  "process_version": "ocps-1.0",
  "basic": {
    "white_balance": {
      "mode": "custom",
      "temperature": 5500,
      "tint": 10
    },
    "exposure": 0.0,
    "contrast": 0,
    "highlights": 0,
    "shadows": 0,
    "whites": 0,
    "blacks": 0,
    "clarity": 0,
    "dehaze": 0,
    "vibrance": 0,
    "saturation": 0
  },
  "tone_curve": {
    "mode": "parametric",
    "parametric": {
      "highlights": 0,
      "lights": 0,
      "darks": 0,
      "shadows": 0
    },
    "point_curve": {
      "rgb": [[0, 0], [255, 255]],
      "red": null,
      "green": null,
      "blue": null
    }
  },
  "hsl": {
    "hue": { "red": 0, "orange": 0, "yellow": 0, "green": 0, "aqua": 0, "blue": 0, "purple": 0, "magenta": 0 },
    "saturation": { "red": 0, "orange": 0, "yellow": 0, "green": 0, "aqua": 0, "blue": 0, "purple": 0, "magenta": 0 },
    "luminance": { "red": 0, "orange": 0, "yellow": 0, "green": 0, "aqua": 0, "blue": 0, "purple": 0, "magenta": 0 }
  },
  "color_grading": {
    "shadows": { "hue": 0, "saturation": 0, "luminance": 0 },
    "midtones": { "hue": 0, "saturation": 0, "luminance": 0 },
    "highlights": { "hue": 0, "saturation": 0, "luminance": 0 },
    "global": { "hue": 0, "saturation": 0, "luminance": 0 },
    "blending": 50,
    "balance": 0
  },
  "detail": {
    "sharpening": {
      "amount": 40,
      "radius": 1.0,
      "detail": 25,
      "masking": 0
    },
    "noise_reduction": {
      "luminance": 0,
      "detail": 50,
      "contrast": 50,
      "color": 25,
      "color_detail": 50,
      "color_smoothness": 50
    }
  },
  "lens_corrections": {
    "profile_enabled": false,
    "profile": null,
    "chromatic_aberration": false,
    "vignetting": { "amount": 0, "midpoint": 50 },
    "distortion": 0
  },
  "transform": {
    "vertical": 0,
    "horizontal": 0,
    "rotate": 0,
    "aspect": 0,
    "scale": 100,
    "offset_x": 0,
    "offset_y": 0,
    "auto_upright": "off"
  },
  "effects": {
    "post_vignette": { "amount": 0, "midpoint": 50, "roundness": 0, "feather": 50 },
    "grain": { "amount": 0, "size": 25, "roughness": 50 }
  },
  "crop": {
    "enabled": false,
    "top": 0.0,
    "left": 0.0,
    "bottom": 1.0,
    "right": 1.0,
    "angle": 0.0,
    "aspect_ratio": null,
    "constrain": false
  },
  "local_adjustments": [],
  "calibration": {
    "shadows_tint": 0,
    "red_hue": 0,
    "red_saturation": 0,
    "green_hue": 0,
    "green_saturation": 0,
    "blue_hue": 0,
    "blue_saturation": 0
  }
}
```

## 4.5 Performance-Strategie

### GPU-Pipeline

```
RAW-Datei
    │
    ▼
┌─────────────────┐
│ CPU: RAW Decode  │  ← Bayer-Daten entpacken (CPU, da dateiformat-spezifisch)
│ (Rust, parallel) │
└────────┬────────┘
         │ Upload (Linear Buffer)
         ▼
┌─────────────────┐
│ GPU: Demosaic    │  ← Bayer → RGB (Compute Shader)
│ (WGSL Shader)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GPU: Processing  │  ← WB, Exposure, Curves, HSL, NR, Sharpening
│ Pipeline         │     (Chain von Compute Shaders)
│ (WGSL Shaders)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GPU: Color Space │  ← Linear → sRGB/Display-Profil (ICC)
│ Transform        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GPU: Output      │  ← Render to Screen oder Download für Export
│ Compositing      │
└─────────────────┘
```

### Cache-Strategie

```
┌─────────────────────────────────────────────────┐
│                 CACHE HIERARCHY                   │
├─────────────────────────────────────────────────┤
│                                                   │
│  L1: GPU Texture Cache                            │
│      → Aktuelle + letzte 5 Fotos                 │
│      → Sofortiger Zugriff                        │
│                                                   │
│  L2: RAM Preview Cache                            │
│      → Mittlere Auflösung (2048px)               │
│      → LRU, konfigurierbare Grösse (1-16 GB)    │
│      → Smart Prefetch (nächste/vorherige Fotos)  │
│                                                   │
│  L3: Disk Preview Cache                           │
│      → Volle Vorschau als JPEG (1:1)             │
│      → Thumbnails (Grid-Ansicht)                 │
│      → Persistent, wird beim Import generiert    │
│                                                   │
│  L4: RAW-Datei (Quellmedium)                     │
│      → Original, wird nie verändert              │
│                                                   │
└─────────────────────────────────────────────────┘
```

### Prefetch-Algorithmus
- Beim Navigieren durch Fotos: ±3 Fotos vorausladen
- Bei Filmstrip-Scroll: Sichtbare + 2× Viewport vorausladen
- Bei Grid-Ansicht: Sichtbare Thumbnails + 2 Reihen vorausladen
- Prefetch-Priority: Navigation Direction > Random Access

### Target-Performance

| Operation | Ziel | Methode |
|-----------|------|---------|
| App-Start | <2s | Lazy Loading, precompiled Shaders |
| Foto wechseln (Develop) | <100ms | L1/L2 Cache + Prefetch |
| Slider-Änderung → Preview | <16ms (60fps) | GPU-Pipeline, kein CPU-Roundtrip |
| 1:1 Zoom | <200ms | Tile-basiertes Rendering |
| Import 1'000 RAW | <30s | Parallel EXIF-Extraction, Background Thumbnails |
| Export 100 JPEG | <60s | GPU-Encode (wenn verfügbar) + parallel I/O |
| Katalog öffnen (100k Fotos) | <3s | Indexed SQLite, lazy Thumbnail Loading |
| Suche (100k Katalog) | <100ms | FTS5, vorbereitete Queries |

---

# 5. FEATURE-SPEZIFIKATION

## 5.1 Module (Lightroom-analoges Layout)

### Library Module (Bibliothek)
**Zweck:** Import, Sichten, Bewerten, Organisieren, Suchen

**Features:**
- **Import:**
  - Von Ordner, Karte, Kamera (tethered optional via Plugin)
  - Optionen: Kopieren, Verschieben, An Ort belassen
  - Duplikat-Erkennung (Hash-basiert)
  - Rename-Templates: `{date}_{camera}_{sequence}` etc.
  - Preset-Import (Import-Einstellungen speicherbar)
  - DNG-Konvertierung beim Import (optional)
  - Keyword-Zuweisung beim Import
  - Metadaten-Template beim Import (Copyright, Kontakt)

- **Grid View (Rasteransicht):**
  - Konfigurierbare Thumbnail-Grösse
  - Overlay: Rating, Flag, Color Label, Dateiname, EXIF-Kurzinfo
  - Multi-Select (Shift+Click, Cmd/Ctrl+Click, Lasso)
  - Sort: Datum, Name, Rating, File Size, Focal Length, Edit Status
  - Filter: Rating, Flag, Color, Keyword, Camera, Lens, Date Range
  - Quick Collection (temporäre Sammlung, wie Lightroom)

- **Loupe View (Lupenansicht):**
  - Einzelbild, volle Vorschau
  - Zoom: Fit, Fill, 1:1, 2:1, frei
  - Info-Overlay: EXIF, Histogramm, Filename
  - Schnelles Navigieren mit Pfeiltasten

- **Compare View (Vergleich):**
  - 2 Bilder nebeneinander (Before/After oder 2 verschiedene)
  - Sync-Zoom und Sync-Pan
  - Survey View: N Bilder nebeneinander

- **Filmstrip:**
  - Am unteren Rand, in allen Views
  - Filtert mit aktuellem Filter
  - Drag & Drop für Sortierung in Sammlungen

- **Ordner-Panel:**
  - Ordnerstruktur der importierten Pfade
  - Drag & Drop zwischen Ordnern
  - Ordner-Watch (automatischer Import bei neuen Dateien)
  - Anzeige: Foto-Anzahl pro Ordner

- **Sammlungen (Collections):**
  - Manuelle Sammlungen (wie Playlists)
  - Smart Collections (regelbasiert: Rating ≥ 4 AND Camera = "Sony A7IV")
  - Collection Sets (verschachtelte Ordner für Sammlungen)
  - Quick Collection (Shortcut B)

- **Keywording:**
  - Hierarchische Keywords (Tier > Säugetier > Katze)
  - Keyword-Suggestions (basierend auf Häufigkeit)
  - Keyword-Sets (Sport, Hochzeit, Landschaft etc.)
  - Bulk-Keywording (auf Selektion anwenden)
  - Keyword-Import/Export (Tab-getrennte Liste)

- **Metadaten-Panel:**
  - EXIF (schreibgeschützt)
  - IPTC (editierbar): Titel, Beschreibung, Copyright, Stadt, Land etc.
  - Custom Metadata Fields (via Plugin-System)

- **Suche:**
  - Freitext-Suche (Filename, Keywords, IPTC)
  - Erweiterte Suche mit Filtern
  - Gespeicherte Suchen (→ Smart Collections)

### Develop Module (Entwicklung)
**Zweck:** RAW-Entwicklung, nicht-destruktive Bearbeitung

**Features:**

- **Basis-Adjustments:**
  - Weissabgleich: Auto, Presets (Daylight, Cloudy, Flash etc.), Custom (Temp + Tint)
  - Belichtung (-5 bis +5 EV)
  - Kontrast
  - Highlights / Shadows / Whites / Blacks
  - Clarity (Midtone Contrast)
  - Dehaze
  - Vibrance / Saturation
  - Profile: Camera Matching, Adobe Color/Landscape/Portrait etc. (als Presets nachbildbar)

- **Tone Curve:**
  - Parametric Mode (Regions)
  - Point Curve (RGB, R, G, B einzeln)
  - Preset-Kurven (Linear, Medium Contrast, Strong Contrast)

- **HSL/Color:**
  - Hue, Saturation, Luminance pro Farbe (8 Kanäle)
  - Color Grading (Shadows, Midtones, Highlights, Global)
  - Split Toning (Legacy-Kompatibilität)

- **Detail:**
  - Sharpening: Amount, Radius, Detail, Masking (Alt+Slider für Preview)
  - Noise Reduction: Luminance (Amount, Detail, Contrast), Color (Amount, Detail, Smoothness)

- **Lens Corrections:**
  - Profil-basiert (LensFun-Datenbank)
  - Chromatic Aberration Removal
  - Vignetting-Korrektur
  - Distortion-Korrektur
  - Manuell: Upright/Transform (Vertical, Horizontal, Rotate, Aspect, Scale)

- **Effects:**
  - Post-Crop Vignetting
  - Grain (Amount, Size, Roughness)

- **Crop & Rotate:**
  - Freies Cropping
  - Aspect Ratios: Original, 1:1, 4:3, 3:2, 16:9, 5:4, Custom
  - Straighten (Linie ziehen)
  - Auto-Crop nach Transform
  - Flip Horizontal/Vertical

- **Local Adjustments (Lokale Korrekturen):**
  - Radial Filter
  - Graduated Filter (Verlaufsfilter)
  - Adjustment Brush
  - Range Mask: Luminanz, Farbe (wie Lightroom CC)
  - Jede lokale Korrektur hat: Exposure, Contrast, Highlights, Shadows, Clarity, Dehaze, Saturation, Sharpness, Noise, Moiré, Defringe, Color

- **Calibration:**
  - Shadow Tint
  - RGB Primary Hue/Saturation Shifts

- **History:**
  - Voller Undo-History-Stack
  - Benannte Snapshots
  - Before/After Toggle (Shortcut: \)

- **Copy/Paste Edits:**
  - **Cmd+C / Cmd+V** → Alle Edits kopieren/einfügen
  - **Cmd+Shift+C** → Auswahl-Dialog: Welche Settings kopieren?
  - **Cmd+Shift+V** → Paste mit Auswahl
  - Sync Settings: Mehrere Fotos selektieren → Sync-Button → Settings des aktiven Fotos auf alle anwenden
  - Auto-Sync-Mode: Jede Änderung wird automatisch auf alle selektierten Fotos angewendet
  - **Match Total Exposure:** Automatisches Angleichen der Belichtung über eine Serie
  - Paste-Verlauf: Die letzten 10 Paste-Operationen sind abrufbar

### Map Module (Karte)
**Zweck:** Geo-Lokalisierung, GPS-basierte Navigation

**Features:**
- Kartenansicht mit Foto-Pins (OpenStreetMap / Mapbox)
- Fotos per Drag & Drop auf Karte platzieren
- GPS-Track-Import (.gpx) → Fotos zeitbasiert zuordnen
- Reverse Geocoding: GPS → Ort, Land
- Karten-Filter: Nur Fotos im sichtbaren Bereich

### Print Module (Druck) — Phase 2
**Zweck:** Druckvorlagen, Kontaktbögen

**Features:**
- Layout-Templates (Einzelbild, Grid, Triptych)
- Custom Layouts
- Kontaktbogen-Generator
- Print Sharpening
- Soft Proofing (ICC-Profile)
- PDF-Export

### Export
**Zweck:** Fotos in verschiedene Formate ausgeben

**Features:**
- **Formate:** JPEG, TIFF (8/16 bit), PNG, WebP, AVIF, DNG, HEIF
- **Resize:** Long Edge, Short Edge, Width, Height, Megapixels, Percentage
- **Output Sharpening:** Screen, Matte Paper, Glossy Paper (Low/Standard/High)
- **Metadaten-Optionen:** Alle, Alle ausser Kamera-Info, Copyright only, Keine
- **Watermark:** Text oder Bild, konfigurierbar
- **Naming:** Template-basiert `{original}_{date}_{size}` etc.
- **Post-Export Actions:** Plugin-Hook (z.B. Upload zu Smugmug, Cloud-Sync)
- **Export-Presets:** Speicherbare Konfigurationen
- **Batch-Export:** Hintergrund-Verarbeitung mit Fortschrittsanzeige

## 5.2 Erweiterte Features

### Virtual Copies
- Mehrere Bearbeitungsversionen eines Fotos
- Jeweils eigener Edit-Stack
- Teilen denselben Speicherplatz (kein Datei-Duplikat)

### Stacking
- Fotos gruppieren (z.B. Belichtungsreihe, Burst)
- Stack ein-/ausklappen
- Auto-Stack by Capture Time (konfigurierbar: 0.5s, 1s, 5s etc.)

### Face Detection / Recognition (Phase 3, Plugin)
- Gesichtserkennung für Personen-Keywording
- Lokal, kein Cloud-Service
- Optional: ONNX-basiertes ML-Modell

### HDR Merge (Phase 2, Plugin)
- Belichtungsreihe → 32-bit HDR
- Tone Mapping im Develop Module
- Auto-Align, Ghost Reduction

### Panorama Merge (Phase 2, Plugin)
- Mehrere Bilder → Panorama
- Cylindrical, Spherical, Perspective Projection
- Boundary Warp

---

# 6. UX & BEDIENKONZEPT

## 6.1 Design-Philosophie

```
"Lightroom-Nutzer sollen sich in 5 Minuten zurechtfinden.
 Power-User sollen in 5 Tagen 2× so schnell sein wie in Lightroom."
```

### Layout-Prinzip
- **Gleiche Grundstruktur wie Lightroom:**
  - Links: Navigation (Ordner, Sammlungen, Presets)
  - Mitte: Hauptansicht (Grid, Loupe, Develop)
  - Rechts: Properties/Adjustments
  - Unten: Filmstrip
  - Oben: Toolbar + Module-Tabs
- **Panels sind collapsible, resizable und persistent** (Zustand wird gespeichert)
- **Solo-Mode** für Panels: Nur ein Panel auf einmal offen (wie Lightroom)

### Dark UI (Default)
- Neutralgrau-Palette (kein Blaustich), damit Fotos neutral wirken
- Konfigurierbare UI-Helligkeit (4 Stufen wie Lightroom)
- Akzentfarbe: Dezentes Blau (konfigurierbar)
- Monospace-Font für EXIF-Daten, Sans-Serif für alles andere

## 6.2 Keyboard-First-Konzept

### Grundprinzip
- **Jede Aktion ist per Tastatur erreichbar**
- **Kein Shortcut braucht mehr als 3 Tasten**
- **Lightroom-Shortcuts als Default-Preset**
- **Vim-inspirierter Expert-Mode (optional)**

### Default-Shortcuts (Lightroom-kompatibel)

**Navigation:**
| Shortcut | Aktion |
|----------|--------|
| G | Grid View |
| E | Loupe View |
| D | Develop Module |
| C | Compare View |
| N | Survey View |
| ← → | Vorheriges / Nächstes Foto |
| Home / End | Erstes / Letztes Foto |
| Space | Nächstes unbewertetes Foto |

**Rating & Flagging:**
| Shortcut | Aktion |
|----------|--------|
| 0-5 | Rating setzen |
| ] | Rating +1 |
| [ | Rating -1 |
| P | Flag: Pick |
| X | Flag: Reject |
| U | Flag: Unflagged |
| 6-9 | Color Labels (Rot, Gelb, Grün, Blau) |

**Develop:**
| Shortcut | Aktion |
|----------|--------|
| \ | Before/After Toggle |
| Cmd+C | Copy Settings |
| Cmd+V | Paste Settings |
| Cmd+Shift+C | Copy Settings (mit Auswahl) |
| Cmd+Shift+V | Paste Settings (mit Auswahl) |
| Cmd+Z | Undo |
| Cmd+Shift+Z | Redo |
| R | Crop Tool |
| K | Adjustment Brush |
| M | Graduated Filter |
| Shift+M | Radial Filter |
| . / , | Nächstes / Vorheriges Preset (Preview) |
| Cmd+' | Create Virtual Copy |

**Library:**
| Shortcut | Aktion |
|----------|--------|
| Cmd+Shift+I | Import |
| Cmd+Shift+E | Export |
| B | Add to Quick Collection |
| Cmd+A | Select All |
| / | Filter Bar Toggle |
| Cmd+F | Find / Search |
| Cmd+G | Group into Stack |

**System:**
| Shortcut | Aktion |
|----------|--------|
| Cmd+K | Command Palette |
| Cmd+, | Preferences |
| Tab | Panels ein/aus |
| Shift+Tab | Alle Panels ein/aus |
| L | Lights Out (1 Press = Dim, 2 = Black, 3 = Normal) |
| F | Fullscreen |
| T | Toolbar ein/aus |
| I | Info Overlay cycle |

### Vim-Mode (Expert, Optional)

Aktivierbar in Preferences. Fügt modale Bedienung hinzu:

```
NORMAL MODE (Default):
  h/j/k/l    → Navigate photos (left/down/up/right)
  /          → Search
  :          → Command Mode
  v          → Visual Select (Range)
  dd         → Reject current photo
  yy         → Copy settings
  pp         → Paste settings
  5*         → Set 5 stars
  .          → Repeat last action
  u          → Undo
  Ctrl+R     → Redo
  gg         → Go to first
  G          → Go to last
  zz         → Center current photo
  za         → Toggle panel

COMMAND MODE (:):
  :export    → Open export dialog
  :import    → Open import dialog
  :sync      → Sync settings
  :preset <name> → Apply preset by name
  :rate <n>  → Set rating
  :flag pick/reject/none
  :sort <field>
  :filter <expression>
  :q         → Quit (mit Bestätigung)
```

### Command Palette (Cmd+K)

Fuzzy-Search über:
- Alle Menü-Aktionen
- Alle Shortcuts
- Alle Presets
- Alle Sammlungen
- Kürzlich geöffnete Kataloge
- Plugin-Aktionen

Inspiriert von VS Code Command Palette, aber für Foto-Workflows optimiert.

## 6.3 Slider-Interaktion

Slider sind das Herzstück des Develop-Moduls. Sie müssen perfekt sein:

- **Drag:** Standard-Interaktion
- **Click:** Springt zum Wert
- **Double-Click:** Reset auf Default
- **Scroll-Wheel:** Feine Anpassung (±1 pro Tick)
- **Shift+Drag:** Feinere Kontrolle (1/4 Geschwindigkeit)
- **Alt+Click:** Zeigt Masking-Vorschau (bei Sharpening Masking)
- **Rechtsklick:** Reset / Copy Value / Paste Value
- **Zahl tippen:** Direkteingabe des Werts (z.B. Exposure → Typ "0.5" → Enter)
- **Tastatursteuerung:** Wenn Slider fokussiert: ← → für ±1, Shift+← → für ±0.1

## 6.4 Filmstrip

- Immer am unteren Rand (collapsible)
- Zeigt aktuelle Auswahl/Sammlung
- Drag & Drop für Reihenfolge
- Rechtsklick-Kontextmenü: Rating, Flag, Color, Rotate, Delete, Stack
- Hover: EXIF-Tooltip
- Aktives Foto hervorgehoben
- Second Monitor Support: Filmstrip kann auf zweitem Monitor angezeigt werden

## 6.5 Before/After-Vergleich

Vier Modi:
1. **Side-by-Side:** Horizontal nebeneinander
2. **Split:** Vertikaler oder horizontaler Split-Slider
3. **Top/Bottom:** Vertikal übereinander
4. **Toggle:** Vollbild-Wechsel (\-Taste)

---

# 7. LIGHTROOM-KOMPATIBILITÄT

## 7.1 Kompatibilitätsstrategie

```
PRIORITÄT 1: Import-Kompatibilität
→ Lightroom-User können sofort mit OCPS arbeiten

PRIORITÄT 2: Workflow-Kompatibilität
→ Gleiche Denkweise, ähnliche Shortcuts, vertraute Logik

PRIORITÄT 3: Preset-Kompatibilität
→ XMP-Presets aus Lightroom funktionieren (soweit möglich)

PRIORITÄT 4: Katalog-Kompatibilität
→ Import von Lightroom-Katalogen (read-only)

NICHT-ZIEL: Pixel-perfekte Replikation
→ Die RAW-Engine wird leicht andere Ergebnisse liefern. Das ist OK.
→ Farb-Interpretation ist Adobe-proprietär und nicht 1:1 replizierbar.
```

## 7.2 XMP-Sidecar-Kompatibilität

### Was wir lesen können (müssen)

Adobe XMP-Sidecars (.xmp) enthalten Develop-Einstellungen im `crs:` Namespace (Camera Raw Settings). Diese Felder sind technisch dokumentiert (XMP-Spezifikation) und können gelesen werden:

**Basis:**
```xml
<crs:Temperature>5500</crs:Temperature>
<crs:Tint>10</crs:Tint>
<crs:Exposure2012>0.50</crs:Exposure2012>
<crs:Contrast2012>25</crs:Contrast2012>
<crs:Highlights2012>-30</crs:Highlights2012>
<crs:Shadows2012>+20</crs:Shadows2012>
<crs:Whites2012>+10</crs:Whites2012>
<crs:Blacks2012>-5</crs:Blacks2012>
<crs:Clarity2012>+15</crs:Clarity2012>
<crs:Dehaze>+10</crs:Dehaze>
<crs:Vibrance>+20</crs:Vibrance>
<crs:Saturation>0</crs:Saturation>
```

**HSL:**
```xml
<crs:HueAdjustmentRed>0</crs:HueAdjustmentRed>
<crs:SaturationAdjustmentRed>0</crs:SaturationAdjustmentRed>
<crs:LuminanceAdjustmentRed>0</crs:LuminanceAdjustmentRed>
<!-- ... für alle 8 Farben -->
```

**Tone Curve:**
```xml
<crs:ToneCurvePV2012>
  <rdf:Seq>
    <rdf:li>0, 0</rdf:li>
    <rdf:li>128, 128</rdf:li>
    <rdf:li>255, 255</rdf:li>
  </rdf:Seq>
</crs:ToneCurvePV2012>
```

**Crop:**
```xml
<crs:CropTop>0.1</crs:CropTop>
<crs:CropLeft>0.05</crs:CropLeft>
<crs:CropBottom>0.9</crs:CropBottom>
<crs:CropRight>0.95</crs:CropRight>
<crs:CropAngle>1.5</crs:CropAngle>
```

### Was wir schreiben (sollten)

- OCPS speichert primär im eigenen JSON-Format (in der Katalog-DB)
- Optional: XMP-Sidecar-Export für Interoperabilität
- XMP-Sidecars werden im Adobe-kompatiblen Format geschrieben
- Namespace: `crs:` für Develop, `dc:` für Dublin Core, `xmp:` für allgemeine Metadaten

### Mapping-Tabelle OCPS ↔ Lightroom XMP

| OCPS Parameter | Lightroom XMP Tag | Anmerkung |
|---------------|-------------------|-----------|
| basic.exposure | crs:Exposure2012 | Direkte 1:1-Zuordnung |
| basic.contrast | crs:Contrast2012 | Direkte 1:1-Zuordnung |
| basic.highlights | crs:Highlights2012 | Direkte 1:1-Zuordnung |
| basic.shadows | crs:Shadows2012 | Direkte 1:1-Zuordnung |
| basic.whites | crs:Whites2012 | Direkte 1:1-Zuordnung |
| basic.blacks | crs:Blacks2012 | Direkte 1:1-Zuordnung |
| basic.clarity | crs:Clarity2012 | Algorithmus kann abweichen |
| basic.dehaze | crs:Dehaze | Algorithmus kann abweichen |
| basic.vibrance | crs:Vibrance | Direkte 1:1-Zuordnung |
| basic.saturation | crs:Saturation | Direkte 1:1-Zuordnung |
| basic.white_balance.temperature | crs:Temperature | Direkte 1:1-Zuordnung |
| basic.white_balance.tint | crs:Tint | Direkte 1:1-Zuordnung |
| tone_curve.point_curve.rgb | crs:ToneCurvePV2012 | Punkt-Format identisch |
| hsl.hue.red | crs:HueAdjustmentRed | Direkte 1:1-Zuordnung |
| detail.sharpening.amount | crs:Sharpness | Direkte 1:1-Zuordnung |
| detail.noise_reduction.luminance | crs:LuminanceSmoothing | Direkte 1:1-Zuordnung |
| crop.top/left/bottom/right | crs:CropTop/Left/Bottom/Right | Direkte 1:1-Zuordnung |
| crop.angle | crs:CropAngle | Direkte 1:1-Zuordnung |
| rating | xmp:Rating | Direkte 1:1-Zuordnung |
| color_label | xmp:Label | Direkte 1:1-Zuordnung |
| flag | crs:Flagged / crs:RejectedFlags | Mapping nötig |

### Bekannte Einschränkungen

1. **Process Version:** Lightroom hat verschiedene Process Versions (PV2010, PV2012, PV2023). OCPS muss PV2012+ unterstützen.
2. **Camera Profiles:** Adobe Camera Profiles (.dcp) sind teilweise proprietär. OCPS kann ICC-Profile verwenden und DCP-Import als Best-Effort anbieten.
3. **Masken/AI-Features:** Lightrooms "AI Masking" (Subject, Sky etc.) ist proprietär und nicht replizierbar. OCPS bietet eigene Masking-Algorithmen.
4. **Color Science:** Die RAW-Interpretation wird leicht unterschiedlich sein. OCPS strebt "visuell ähnlich" an, nicht "pixel-identisch".

## 7.3 Lightroom-Katalog-Import

**Ansatz:** Read-only Import von `.lrcat` Dateien (SQLite-Format)

**Was importiert wird:**
- Foto-Pfade und Metadaten
- Ratings, Flags, Color Labels
- Keywords und Keyword-Hierarchie
- Collections und Smart Collections
- Develop-Einstellungen (soweit sie im XMP-Format vorliegen)
- Virtual Copies
- Stacks

**Was NICHT importiert wird:**
- Lightroom-Plugin-Daten
- Publish-Services-Status
- Lightroom-spezifische History
- Face-Recognition-Daten (proprietär)

**Implementierung:**
```
1. Benutzer wählt .lrcat-Datei
2. OCPS liest SQLite-Datenbank (read-only)
3. Mapping der Lightroom-Tabellen auf OCPS-Schema
4. Für jedes Foto: XMP-Sidecar lesen (falls vorhanden) für Develop-Settings
5. Fortschrittsanzeige mit Zusammenfassung am Ende
6. "Migration Report": Was importiert wurde, was nicht, Warnungen
```

## 7.4 Preset-Kompatibilität

### Lightroom-Presets (.xmp / .lrtemplate)

**Älteres Format (.lrtemplate):** Lua-ähnliches Format. OCPS implementiert einen Parser.

**Neueres Format (.xmp):** Standard-XMP mit `crs:`-Tags. Direkt lesbar.

**Import-Workflow:**
1. Benutzer zieht Preset-Dateien in OCPS oder wählt "Import Presets"
2. OCPS parst die Datei und extrahiert Settings
3. Settings werden in OCPS-internes JSON-Format konvertiert
4. Preset wird in der lokalen Preset-Bibliothek gespeichert
5. Warnung bei inkompatiblen Settings (z.B. proprietäre Profile)

### OCPS-eigene Presets

**Format:** JSON (wie Edit-Daten, aber mit `applied_settings`-Maske)

```json
{
  "name": "Warm Sunset",
  "group": "Color",
  "version": 1,
  "applied_settings": ["basic.white_balance", "basic.vibrance", "color_grading", "tone_curve"],
  "settings": {
    "basic": {
      "white_balance": { "temperature": 6500, "tint": 15 },
      "vibrance": 30
    },
    "color_grading": {
      "highlights": { "hue": 40, "saturation": 20 },
      "shadows": { "hue": 240, "saturation": 15 }
    }
  }
}
```

**Vorteile:**
- `applied_settings` definiert, welche Settings das Preset setzt (partielle Anwendung)
- JSON ist menschenlesbar und versionierbar (Git-freundlich)
- Community kann Presets einfach teilen

---

# 8. RAW-PROCESSING-PIPELINE

## 8.1 Übersicht

Die RAW-Processing-Pipeline ist das Herzstück der Software. Sie wandelt Bayer-Sensordaten in ein fertig bearbeitetes Bild um.

```
┌────────────────────────────────────────────────────────────────┐
│                   RAW PROCESSING PIPELINE                       │
├────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. RAW DECODE (CPU)                                            │
│     ├── Container-Parser (CR3, ARW, NEF, RAF, ORF, DNG...)     │
│     ├── Decompress (Lossless JPEG, LZ77, Huffman etc.)         │
│     └── Output: Raw Bayer Data (16-bit linear)                  │
│                                                                  │
│  2. BLACK/WHITE LEVEL (CPU/GPU)                                 │
│     ├── Subtract black level (per-channel)                      │
│     └── Scale to white level → Normalize to [0, 1]             │
│                                                                  │
│  3. LENS CORRECTIONS (GPU)                                      │
│     ├── Distortion correction (LensFun profile)                 │
│     ├── Vignetting correction                                   │
│     └── Chromatic aberration correction                         │
│                                                                  │
│  4. DEMOSAIC (GPU)                                              │
│     ├── Algorithm: AMaZE (default), RCD, VNG, AHD              │
│     ├── Input: Bayer CFA pattern                                │
│     └── Output: Full RGB image (linear, camera space)           │
│                                                                  │
│  5. WHITE BALANCE (GPU)                                         │
│     ├── Apply temperature + tint                                │
│     └── Multiply per-channel gains                              │
│                                                                  │
│  6. COLOR SPACE CONVERSION (GPU)                                │
│     ├── Camera → Working Space (e.g., ProPhoto RGB)             │
│     ├── Use camera-specific color matrix                        │
│     └── ICC/DCP profile application                             │
│                                                                  │
│  7. EXPOSURE + TONE (GPU)                                       │
│     ├── Exposure compensation                                   │
│     ├── Highlight recovery                                      │
│     ├── Shadow recovery                                         │
│     ├── White/Black point                                       │
│     ├── Contrast                                                │
│     ├── Tone curve application                                  │
│     └── Gamma encoding                                          │
│                                                                  │
│  8. HSL ADJUSTMENTS (GPU)                                       │
│     ├── Per-channel Hue/Saturation/Luminance                    │
│     └── Vibrance (intelligent saturation)                       │
│                                                                  │
│  9. COLOR GRADING (GPU)                                         │
│     ├── Shadows/Midtones/Highlights color wheels                │
│     ├── Blending + Balance                                      │
│     └── Global tint                                             │
│                                                                  │
│  10. LOCAL ADJUSTMENTS (GPU)                                    │
│      ├── Apply masks (brush, gradient, radial)                  │
│      ├── Per-mask: exposure, contrast, color, etc.              │
│      └── Blend with global adjustments                          │
│                                                                  │
│  11. DETAIL (GPU)                                               │
│      ├── Sharpening (Unsharp Mask variant)                      │
│      ├── Noise Reduction (Luminance + Chroma)                   │
│      └── Moiré suppression                                      │
│                                                                  │
│  12. CLARITY / DEHAZE (GPU)                                     │
│      ├── Clarity: Local contrast enhancement                    │
│      └── Dehaze: Dark channel prior based                       │
│                                                                  │
│  13. EFFECTS (GPU)                                              │
│      ├── Post-crop vignette                                     │
│      └── Film grain simulation                                  │
│                                                                  │
│  14. CROP & TRANSFORM (GPU)                                     │
│      ├── Apply crop rectangle                                   │
│      ├── Rotation                                               │
│      └── Perspective correction                                 │
│                                                                  │
│  15. OUTPUT (GPU → CPU)                                         │
│      ├── Color space conversion (Working → Output space)        │
│      ├── Bit depth conversion (16 → 8 for JPEG)                │
│      ├── Soft proofing (if enabled)                             │
│      └── Final render to screen or file                         │
│                                                                  │
└────────────────────────────────────────────────────────────────┘
```

## 8.2 Unterstützte RAW-Formate

### Phase 1 (Launch)
| Format | Kameras | Parser |
|--------|---------|--------|
| DNG | Adobe DNG, Leica, Hasselblad, Pentax | rawloader + custom |
| CR2 | Canon (ältere: 5D III, 6D, 7D II) | rawloader |
| CR3 | Canon (neuere: R5, R6, R3) | Custom (ISOBMFF container) |
| NEF | Nikon (D850, Z6, Z7, Z8, Z9) | rawloader |
| ARW | Sony (A7 Serie, A9, A1) | rawloader |
| RAF | Fujifilm (X-T5, X-H2, GFX) | Custom (X-Trans Demosaic) |
| ORF | Olympus/OM System | rawloader |
| RW2 | Panasonic Lumix | rawloader |

### Phase 2
| Format | Kameras |
|--------|---------|
| PEF | Pentax |
| SRW | Samsung |
| 3FR/FFF | Hasselblad (ältere) |
| IIQ | Phase One |
| ERF | Epson |
| MOS | Leaf |

### Sonderfälle
- **X-Trans (Fujifilm):** Eigener Demosaic-Algorithmus nötig (kein Bayer-Pattern). AMaZE geht nicht → Markesteijn oder Frank Markesteijn's Algorithmus.
- **Pixel Shift:** Einige Kameras (Pentax, Sony, OM System) unterstützen Pixel-Shift-Resolution. Phase 3 Feature.

## 8.3 Demosaicing-Algorithmen

| Algorithmus | Qualität | Speed | Anwendung |
|------------|----------|-------|-----------|
| **AMaZE** | ★★★★★ | ★★★ | Default, beste Qualität |
| **RCD** | ★★★★ | ★★★★ | Schneller, leicht weniger Detail |
| **VNG** | ★★★ | ★★★★★ | Schnell, für Vorschau |
| **Bilinear** | ★★ | ★★★★★ | Nur für Thumbnails |
| **Markesteijn** | ★★★★★ | ★★ | Nur für X-Trans (Fuji) |

**Strategie:** Bilinear für Thumbnails → VNG für schnelle Vorschau → AMaZE/RCD für finale Vorschau und Export.

## 8.4 Noise Reduction

**Ansatz: Wavelet-based + Non-Local Means (NLM) Hybrid**

1. **Luminance NR:** Wavelet-Decomposition, dann Thresholding pro Scale. Erhält Schärfe besser als reines Gaussian.
2. **Chroma NR:** Aggressiver, da Chroma-Noise weniger sichtbar bei Korrektur. NLM im Chroma-Kanal.
3. **GPU-Implementierung:** Wavelet-Transform als Compute Shader. NLM ist parallelisierbar.

**Qualitäts-Ziel:** Vergleichbar mit DxO PureRAW oder Lightroom's AI NR (Phase 3: ML-basierte NR als Plugin).

## 8.5 Color Science

### Working Color Space
- **ProPhoto RGB** als interner Working Space (wie Lightroom)
- 16-bit Float pro Kanal (GPU-intern 32-bit float)
- Grösster Gamut, verhindert Clipping bei extremen Edits

### Camera Color Profiles
- **DNG Color Matrices:** Jede Kamera hat eine Farbmatrix (in DNG-Spec dokumentiert). Wir verwenden die dcp-extrahierten Matrizen.
- **ICC-Profile:** Import und Anwendung von ICC-Kamera-Profilen
- **DCP-Import (Best-Effort):** Adobe DCP-Dateien können gelesen werden (HSV-Lookup-Tabellen). Nicht pixel-identisch, aber visuell nah.

### Output Color Space
- sRGB (Default für Web/Screen)
- Adobe RGB (für Print-Workflows)
- ProPhoto RGB (für weitere Bearbeitung)
- Display P3 (für moderne Displays)
- Custom ICC Profile

---

# 9. PLUGIN- & PRESET-SYSTEM

## 9.1 Plugin-Architektur

### Designziele
1. **Sandboxed:** Plugins können die Hauptanwendung nicht crashen
2. **Performant:** Plugin-Aufrufe dürfen den Haupt-Thread nicht blockieren
3. **Versioniert:** Plugins haben SemVer, Kompatibilität wird geprüft
4. **Discoverable:** Plugin-Registry/Marketplace für Installation mit einem Klick
5. **Multi-Language:** Plugins können in Rust, TypeScript oder jeder WASM-kompatiblen Sprache geschrieben werden

### Plugin-Typen

| Typ | Beschreibung | Beispiele |
|-----|-------------|-----------|
| **Image Filter** | Zusätzliche Bildbearbeitungs-Filter | Luminosity Masks, Freq. Separation, Custom LUT |
| **Import/Export** | Neue Formate oder Export-Ziele | HEIF Export, FTP Upload, S3 Sync |
| **Metadata** | Metadaten-Erweiterungen | GPS-Reverse-Geocoding, IPTC-Templates, AI Captioning |
| **UI Panel** | Eigene Panels in der UI | Histogramm-Varianten, Waveform, Vectorscope |
| **Catalog** | Katalog-Erweiterungen | Duplikat-Finder, Smart Album Logic, Backup |
| **Integration** | Drittanbieter-Anbindungen | Flickr, 500px, SmugMug, Adobe Portfolio |
| **AI/ML** | Machine-Learning-Features | Face Detection, Scene Recognition, Auto-Tagging |
| **Tethering** | Kamera-Anbindung | Live View, Remote Capture |

### Plugin-API (Konzept)

```rust
// Plugin-Manifest (plugin.toml)
[plugin]
name = "luminosity-masks"
version = "1.0.0"
api_version = "1"
type = "image_filter"
author = "Community"
license = "MIT"
description = "Advanced luminosity mask generation"

[permissions]
read_image = true
write_image = true
read_catalog = false
write_catalog = false
network = false
filesystem = false

// Plugin-Interface (Rust/WASM)
#[plugin_export]
fn process_image(input: &ImageBuffer, params: &PluginParams) -> Result<ImageBuffer, PluginError> {
    // Plugin-Logik
}

#[plugin_export]
fn get_parameters() -> Vec<ParameterDefinition> {
    vec![
        ParameterDefinition::slider("intensity", 0.0, 1.0, 0.5, "Mask Intensity"),
        ParameterDefinition::choice("zone", &["shadows", "midtones", "highlights"], "Zone"),
    ]
}

#[plugin_export]
fn get_ui() -> Option<PluginUI> {
    // Optional: Custom HTML/CSS UI
    None
}
```

### Plugin-Sandbox (WASM)

```
┌──────────────────────────────────────────┐
│          OCPS Main Process                │
│                                            │
│  ┌──────────────────────────────────────┐│
│  │         Plugin Host (wasmtime)       ││
│  │                                      ││
│  │  ┌────────────┐  ┌────────────┐    ││
│  │  │ Plugin A   │  │ Plugin B   │    ││
│  │  │ (WASM)     │  │ (WASM)     │    ││
│  │  │            │  │            │    ││
│  │  │ Memory:    │  │ Memory:    │    ││
│  │  │ Isolated   │  │ Isolated   │    ││
│  │  │            │  │            │    ││
│  │  │ Caps:      │  │ Caps:      │    ││
│  │  │ read_image │  │ read_image │    ││
│  │  │ write_img  │  │ network    │    ││
│  │  └────────────┘  └────────────┘    ││
│  │                                      ││
│  │  Communication: Shared Memory +     ││
│  │  Message Passing (no direct access) ││
│  └──────────────────────────────────────┘│
│                                            │
└──────────────────────────────────────────┘
```

### Plugin-Lifecycle

```
1. Discovery    → Scan plugin directories
2. Load         → Parse manifest, check API version
3. Validate     → Verify WASM module, check permissions
4. Initialize   → Call plugin init(), register capabilities
5. Ready        → Plugin appears in UI, callable
6. Execute      → Called on demand (lazy)
7. Suspend      → After inactivity, free WASM memory
8. Unload       → User disables or update
```

## 9.2 Preset-System

### Preset-Speicherorte

```
~/.ocps/
├── presets/
│   ├── builtin/           → Mit der App geliefert (read-only)
│   │   ├── color/
│   │   ├── bw/
│   │   └── creative/
│   ├── user/              → Vom Benutzer erstellt
│   │   ├── wedding/
│   │   └── landscape/
│   ├── imported/          → Aus Lightroom oder Drittanbieter
│   │   └── vsco/
│   └── community/         → Aus dem Marketplace
│       └── portra-400/
├── export-presets/
├── import-presets/
└── keyword-sets/
```

### Preset-Sharing

- Presets sind JSON-Dateien → Git-freundlich, teilbar
- Export als `.ocps-preset` (ZIP mit JSON + optional Vorschaubild)
- Community-Marketplace: Upload/Download von Presets
- Preset-Bundles: Mehrere Presets als Paket

### Builtin-Presets (mit der App geliefert)

**Color:**
- Natural Light, Warm Tone, Cool Tone
- High Contrast Color, Low Contrast Matte
- Vivid Landscape, Soft Portrait
- Golden Hour, Blue Hour
- Film Simulation: Portra 400, Fuji 400H, Ektar 100 (eigene Interpretation)

**Black & White:**
- Classic B&W, High Contrast B&W
- Film B&W: Tri-X, HP5, Delta 100
- Infrared Simulation
- Selenium, Sepia, Cyanotype Toning

**Creative:**
- Cross-Process, Lomo, Faded Film
- Teal & Orange, Moody Dark
- Vintage, Retro 70s

---

# 10. IMPLEMENTIERUNGSPLAN & ROADMAP

## 10.1 Phasen-Übersicht

```
Phase 0: Foundation (Monate 1-3)
├── Projekt-Setup, CI/CD, Architektur
├── Rust Core mit Basis-RAW-Pipeline
├── Minimale Tauri-App (Proof of Concept)
└── SQLite-Katalog-Grundstruktur

Phase 1: MVP (Monate 4-8)
├── Library Module (Import, Grid, Loupe)
├── Develop Module (Basis-Adjustments)
├── RAW-Pipeline (Top-5-Kameras)
├── XMP-Sidecar lesen/schreiben
├── Copy/Paste Edits
├── Export (JPEG, TIFF)
└── Keyboard-Shortcuts (Lightroom-kompatibel)

Phase 2: Feature Parity (Monate 9-14)
├── Vollständiges Develop Module
├── Local Adjustments (Brush, Gradient, Radial)
├── Tone Curve, HSL, Color Grading
├── Plugin-System (v1)
├── Preset-System (Import + Eigene)
├── Map Module
├── Print Module
├── Lightroom-Katalog-Import
├── Erweiterte RAW-Format-Unterstützung
└── Performance-Optimierung

Phase 3: Polish & Ecosystem (Monate 15-20)
├── Community-Marketplace für Plugins/Presets
├── AI/ML Plugins (Denoise, Masking, Auto-Tag)
├── Tethered Shooting Plugin
├── HDR Merge, Panorama Merge
├── Vim-Mode
├── Second Monitor Support
├── Soft Proofing
├── Internationalisierung
└── Beta-Release

Phase 4: Launch & Growth (Monate 21-24)
├── Public v1.0 Release
├── Dokumentation & Tutorials
├── Commercial License Launch
├── Community-Building
└── Roadmap für v1.x
```

## 10.2 Phase 0: Foundation (Detail)

### Woche 1-2: Projekt-Setup
- [ ] GitHub-Repository erstellen
- [ ] Monorepo-Struktur (Cargo Workspace + pnpm Workspace)
- [ ] CI/CD (GitHub Actions): Build, Test, Lint für alle Plattformen
- [ ] Lizenz-Dateien (PolyForm Noncommercial, COMMERCIAL.md)
- [ ] CONTRIBUTING.md, GOVERNANCE.md, CODE_OF_CONDUCT.md
- [ ] CLA-Bot einrichten
- [ ] README.md mit Vision und Status

### Woche 3-6: Core Engine
- [ ] RAW-Decode für 3 Formate (DNG, CR3, ARW)
- [ ] Demosaicing (Bilinear + AMaZE)
- [ ] GPU-Pipeline Setup (wgpu)
- [ ] Basis-Adjustments als GPU-Shader (Exposure, WB, Contrast)
- [ ] Katalog-Datenbank (SQLite Schema)
- [ ] XMP-Parser (Lesen von Adobe-Sidecars)
- [ ] Cache-System (L2 RAM, L3 Disk)

### Woche 7-12: Tauri-App Skeleton
- [ ] Tauri v2 App-Shell
- [ ] SolidJS UI-Framework-Setup
- [ ] IPC-Layer (Rust ↔ TypeScript)
- [ ] Basis-Layout (Library View mit Grid)
- [ ] Thumbnail-Rendering (Virtual List)
- [ ] Shortcut-Engine (Basics)
- [ ] Foto anzeigen (Loupe View, rudimentär)
- [ ] Dark UI Theme

### Milestone: Phase 0 Complete
- App startet, zeigt Ordner mit RAW-Fotos als Thumbnails
- Ein Foto kann geöffnet und rudimentär bearbeitet werden (Exposure, WB)
- Änderungen werden in SQLite gespeichert
- XMP-Sidecar einer Lightroom-Datei kann gelesen werden

## 10.3 Monorepo-Struktur

```
openclaw-photo-studio/
├── Cargo.toml                    # Rust workspace
├── package.json                  # pnpm workspace root
├── LICENSE                       # PolyForm Noncommercial
├── LICENSE-POLYFORM.md           # Volltext
├── COMMERCIAL.md                 # Kommerzielle Lizenz-Info
├── CONTRIBUTING.md
├── GOVERNANCE.md
├── CODE_OF_CONDUCT.md
├── CLA.md
├── README.md
├── CHANGELOG.md
│
├── crates/                       # Rust Crates
│   ├── ocps-core/               # Kern-Engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── raw/             # RAW-Decoding
│   │   │   │   ├── mod.rs
│   │   │   │   ├── decoder.rs
│   │   │   │   ├── formats/     # Pro Format ein Modul
│   │   │   │   │   ├── dng.rs
│   │   │   │   │   ├── cr3.rs
│   │   │   │   │   ├── arw.rs
│   │   │   │   │   ├── nef.rs
│   │   │   │   │   └── raf.rs
│   │   │   │   └── demosaic/
│   │   │   │       ├── amaze.rs
│   │   │   │       ├── rcd.rs
│   │   │   │       ├── vng.rs
│   │   │   │       └── xtrans.rs
│   │   │   ├── pipeline/        # Processing Pipeline
│   │   │   │   ├── mod.rs
│   │   │   │   ├── stage.rs
│   │   │   │   └── gpu/
│   │   │   │       ├── shaders/  # WGSL Shader
│   │   │   │       │   ├── exposure.wgsl
│   │   │   │       │   ├── white_balance.wgsl
│   │   │   │       │   ├── tone_curve.wgsl
│   │   │   │       │   ├── hsl.wgsl
│   │   │   │       │   ├── color_grading.wgsl
│   │   │   │       │   ├── sharpen.wgsl
│   │   │   │       │   ├── noise_reduce.wgsl
│   │   │   │       │   └── output.wgsl
│   │   │   │       └── context.rs
│   │   │   ├── color/           # Color Management
│   │   │   │   ├── icc.rs
│   │   │   │   ├── dcp.rs
│   │   │   │   └── spaces.rs
│   │   │   ├── edit/            # Edit Data Model
│   │   │   │   ├── mod.rs
│   │   │   │   ├── types.rs
│   │   │   │   ├── history.rs
│   │   │   │   └── snapshot.rs
│   │   │   └── cache/           # Cache Management
│   │   │       ├── mod.rs
│   │   │       ├── lru.rs
│   │   │       └── thumbnail.rs
│   │   └── Cargo.toml
│   │
│   ├── ocps-catalog/            # Katalog-Engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db.rs            # SQLite Operations
│   │   │   ├── schema.rs        # Table Definitions
│   │   │   ├── migrations/      # Schema Migrations
│   │   │   ├── photo.rs
│   │   │   ├── collection.rs
│   │   │   ├── keyword.rs
│   │   │   ├── search.rs        # FTS5 Search
│   │   │   ├── import.rs        # Import Logic
│   │   │   └── lightroom.rs     # Lightroom Catalog Import
│   │   └── Cargo.toml
│   │
│   ├── ocps-xmp/                # XMP/IPTC/EXIF
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reader.rs
│   │   │   ├── writer.rs
│   │   │   ├── lightroom.rs     # Lightroom XMP Mapping
│   │   │   ├── iptc.rs
│   │   │   └── exif.rs
│   │   └── Cargo.toml
│   │
│   ├── ocps-export/             # Export Engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── jpeg.rs
│   │   │   ├── tiff.rs
│   │   │   ├── png.rs
│   │   │   ├── webp.rs
│   │   │   ├── dng.rs
│   │   │   ├── resize.rs
│   │   │   ├── sharpen.rs
│   │   │   └── watermark.rs
│   │   └── Cargo.toml
│   │
│   ├── ocps-plugin-host/        # Plugin System
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── host.rs          # WASM Host (wasmtime)
│   │   │   ├── manifest.rs      # Plugin Manifest Parser
│   │   │   ├── api.rs           # Plugin API Definition
│   │   │   ├── sandbox.rs       # Permission Sandbox
│   │   │   └── registry.rs      # Plugin Discovery
│   │   └── Cargo.toml
│   │
│   └── ocps-cli/                # CLI für Headless/Batch
│       ├── src/
│       │   ├── main.rs
│       │   ├── import.rs
│       │   ├── export.rs
│       │   ├── batch.rs
│       │   └── catalog.rs
│       └── Cargo.toml
│
├── app/                          # Tauri Desktop App
│   ├── src-tauri/               # Tauri Rust Backend
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/        # IPC Commands
│   │   │   │   ├── library.rs
│   │   │   │   ├── develop.rs
│   │   │   │   ├── export.rs
│   │   │   │   ├── catalog.rs
│   │   │   │   └── plugin.rs
│   │   │   └── state.rs         # App State
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   │
│   └── src/                     # SolidJS Frontend
│       ├── index.html
│       ├── index.tsx
│       ├── App.tsx
│       ├── styles/
│       │   ├── global.css
│       │   └── theme.css
│       ├── components/
│       │   ├── layout/
│       │   │   ├── AppShell.tsx
│       │   │   ├── Sidebar.tsx
│       │   │   ├── Toolbar.tsx
│       │   │   ├── Filmstrip.tsx
│       │   │   └── Panel.tsx
│       │   ├── library/
│       │   │   ├── GridView.tsx
│       │   │   ├── LoupeView.tsx
│       │   │   ├── CompareView.tsx
│       │   │   ├── PhotoCard.tsx
│       │   │   ├── FolderTree.tsx
│       │   │   ├── CollectionList.tsx
│       │   │   └── KeywordPanel.tsx
│       │   ├── develop/
│       │   │   ├── DevelopView.tsx
│       │   │   ├── Slider.tsx
│       │   │   ├── BasicPanel.tsx
│       │   │   ├── ToneCurve.tsx
│       │   │   ├── HSLPanel.tsx
│       │   │   ├── DetailPanel.tsx
│       │   │   ├── ColorGrading.tsx
│       │   │   ├── LensPanel.tsx
│       │   │   ├── EffectsPanel.tsx
│       │   │   ├── CropTool.tsx
│       │   │   ├── HistoryPanel.tsx
│       │   │   └── PresetPanel.tsx
│       │   ├── export/
│       │   │   └── ExportDialog.tsx
│       │   ├── common/
│       │   │   ├── Histogram.tsx
│       │   │   ├── ColorWheel.tsx
│       │   │   ├── CommandPalette.tsx
│       │   │   ├── VirtualList.tsx
│       │   │   └── Modal.tsx
│       │   └── map/
│       │       └── MapView.tsx
│       ├── stores/
│       │   ├── catalog.ts
│       │   ├── develop.ts
│       │   ├── ui.ts
│       │   ├── shortcuts.ts
│       │   └── plugins.ts
│       ├── hooks/
│       │   ├── useShortcuts.ts
│       │   ├── useIPC.ts
│       │   └── useImagePreview.ts
│       └── lib/
│           ├── ipc.ts           # Tauri IPC wrapper
│           ├── shortcuts.ts     # Shortcut engine
│           ├── theme.ts
│           └── i18n.ts
│
├── plugins/                      # Offizielle Plugins (Beispiele)
│   ├── ocps-plugin-hdr/
│   ├── ocps-plugin-panorama/
│   ├── ocps-plugin-face-detect/
│   └── ocps-plugin-tether-gphoto/
│
├── presets/                      # Builtin Presets
│   ├── color/
│   ├── bw/
│   └── creative/
│
├── rfcs/                         # RFC Dokumente
│   └── 0001-template.md
│
├── docs/                         # Dokumentation
│   ├── architecture.md
│   ├── plugin-dev-guide.md
│   ├── preset-format.md
│   ├── xmp-compatibility.md
│   ├── build-guide.md
│   └── user-guide/
│
├── tests/                        # Integration Tests
│   ├── fixtures/                # Test-Bilder (verschiedene RAW-Formate)
│   ├── integration/
│   └── visual-regression/
│
└── scripts/                      # Build & Dev Scripts
    ├── build.sh
    ├── dev.sh
    ├── generate-test-fixtures.sh
    └── release.sh
```

## 10.4 Teamstruktur (empfohlen)

### Minimal Viable Team (Phase 0-1)

| Rolle | Anzahl | Fokus |
|-------|--------|-------|
| **Core Engine Lead** | 1 | Rust, RAW-Pipeline, GPU-Shaders |
| **UI Lead** | 1 | SolidJS, Tauri, UX |
| **Color Science** | 0.5 | Demosaicing, Color Management (kann extern sein) |

### Growth Team (Phase 2+)

| Rolle | Anzahl | Fokus |
|-------|--------|-------|
| **Core Engine** | 2-3 | RAW-Formate, Pipeline, Performance |
| **UI/UX** | 2 | Frontend, Interaction Design |
| **Plugin System** | 1 | WASM Host, Plugin API |
| **QA** | 1 | Testing, Visual Regression, RAW-Format-Kompatibilität |
| **Community** | 1 | Docs, Support, Marketplace |
| **Color Science** | 1 | Forschung, Algorithmen |

## 10.5 Technische Risiken

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|--------------------| -------|------------|
| GPU-Kompatibilität (ältere GPUs) | Mittel | Hoch | CPU-Fallback-Pipeline, wgpu-Abstraktion |
| RAW-Format-Coverage | Hoch | Hoch | Priorisierung der Top-10-Kameras, Community-Beiträge |
| Color Science Qualität | Mittel | Hoch | Vergleichstests mit Lightroom/darktable, Experten-Review |
| Performance bei grossen Katalogen | Mittel | Mittel | Early Benchmarking, SQLite-Optimierung, Indexing |
| XMP-Kompatibilität | Mittel | Mittel | Umfangreiche Testdaten, Community-Feedback |
| Tauri v2 Stabilität | Niedrig | Mittel | Tauri-Community ist aktiv, WebView-Fallbacks |
| WASM-Plugin Performance | Niedrig | Niedrig | Shared Memory für Bilddaten, nicht kopieren |
| CLA-Akzeptanz bei Contributors | Mittel | Mittel | Transparente Fairness-Klausel, klar kommunizieren |

---

# 11. REFERENZDOKUMENTE

## 11.1 Zu erstellende Dateien

| Datei | Status | Beschreibung |
|-------|--------|-------------|
| LICENSE | TODO | PolyForm Noncommercial 1.0.0 |
| LICENSE-POLYFORM.md | TODO | Volltext der Lizenz |
| COMMERCIAL.md | Draft ✓ | Kommerzielle Lizenz-Info |
| CONTRIBUTING.md | Draft ✓ | Beitragsrichtlinien |
| GOVERNANCE.md | Draft ✓ | Governance-Struktur |
| CLA.md | TODO | Contributor License Agreement |
| CODE_OF_CONDUCT.md | TODO | Contributor Covenant |
| TRADEMARK.md | TODO | Trademark Policy |
| README.md | TODO | Projekt-Übersicht |
| CHANGELOG.md | TODO | Änderungsprotokoll |

## 11.2 Externe Referenzen

- [PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/)
- [Business Source License](https://mariadb.com/bsl11/)
- [XMP Specification](https://www.adobe.com/devnet/xmp.html)
- [DNG Specification](https://helpx.adobe.com/photoshop/digital-negative.html)
- [LensFun Database](https://lensfun.github.io/)
- [wgpu (Rust GPU)](https://wgpu.rs/)
- [Tauri v2](https://v2.tauri.app/)
- [SolidJS](https://www.solidjs.com/)
- [rawloader (Rust)](https://github.com/nicola-spieser/rawloader) ← zu evaluieren
- [AMaZE Demosaicing](https://www.rawtherapee.com/) ← Referenz-Implementierung
- [PolyForm Project](https://polyformproject.org/)
- [CLA Assistant](https://cla-assistant.io/)

## 11.3 Vergleichs-Software (für Forschung)

| Software | Quellcode | Lizenz | Relevant für |
|---------|-----------|--------|-------------|
| darktable | Ja | GPL-3.0 | RAW-Pipeline, Algorithmen |
| RawTherapee | Ja | GPL-3.0 | Demosaicing (AMaZE), Color Science |
| Ansel (darktable fork) | Ja | GPL-3.0 | UX-Verbesserungen |
| LibRaw | Ja | LGPL/CDDL | RAW-Decoding-Referenz |
| dcraw | Ja | Public Domain | RAW-Decoding-Grundlagen |
| vkdt | Ja | BSD-2 | Vulkan-basierte Pipeline |

---

# APPENDIX A: GLOSSAR

| Begriff | Erklärung |
|---------|-----------|
| **Bayer Pattern** | Farbfilter-Mosaik auf dem Sensor (RGGB). Jeder Pixel sieht nur eine Farbe. |
| **CFA** | Color Filter Array — das physische Farbmuster auf dem Sensor |
| **Demosaicing** | Algorithmus, der aus dem Bayer-Pattern ein vollfarbiges Bild rekonstruiert |
|