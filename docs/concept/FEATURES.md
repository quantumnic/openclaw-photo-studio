# OpenClaw Photo Studio — Vollständige Feature-Liste

> Ergänzung zu CONCEPT.md
> Version: 0.1.0-draft | 2026-03-19

---

# FEATURE-KATEGORIEN MIT BEWERTUNG

Legende:
- **Prio:** P0 = MVP, P1 = V1.0, P2 = V1.5, P3 = V2.0, P4 = Experimental/Future
- **Schwierigkeit:** 1-5 (1 = trivial, 5 = Forschungsprojekt)
- **Risiko:** L = Low, M = Medium, H = High

---

## A. DATEI- UND BIBLIOTHEKSVERWALTUNG

### A.01 Import Engine
**Beschreibung:** Import von Ordnern, Karten, Kameras. Copy/Move/Add-in-Place. Rename-Templates, Duplikat-Erkennung.
**Nutzen:** Grundfunktion — ohne Import keine Software.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Catalog DB, File System Watcher, EXIF Parser
**Teststrategie:** Unit-Tests für Rename-Templates, Integration-Tests mit echten Ordnerstrukturen (100, 1'000, 10'000 Dateien), Duplikat-Tests mit identischen und near-identical Hashes.

### A.02 Watch Folders
**Beschreibung:** Überwachte Ordner, die bei neuen Dateien automatisch importieren.
**Nutzen:** Studio-Workflow — Kamera-Ordner wird automatisch synchronisiert.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** M (Race Conditions bei schnellem Schreiben)
**Abhängigkeiten:** Import Engine, File System Watcher (notify crate)
**Teststrategie:** Stress-Tests mit schnellem Dateischreiben, Tests mit Netzwerk-Ordnern, Debounce-Tests.

### A.03 Copy/Move/Add Modes
**Beschreibung:** Beim Import: Dateien kopieren (in Katalog-Ordner), verschieben, oder am Ort belassen.
**Nutzen:** Flexibilität für verschiedene Workflows (mobil vs. Studio vs. NAS).
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** Import Engine
**Teststrategie:** Tests für alle drei Modi, Tests mit schreibgeschützten Quellen, Tests mit vollen Zielen.

### A.04 Folder Sync
**Beschreibung:** Bidirektionale Synchronisation zwischen Dateisystem und Katalog. Erkennt gelöschte, umbenannte, verschobene Dateien.
**Nutzen:** Katalog bleibt aktuell, auch wenn Dateien ausserhalb der App bewegt werden.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (Rename-Detection ist fuzzy)
**Abhängigkeiten:** Catalog DB, File System Watcher, File Hash Service
**Teststrategie:** Tests mit umbenannten, verschobenen, gelöschten Dateien. Tests mit Batch-Operationen. Tests mit externen Tools (Finder, Explorer).

### A.05 Catalog Database
**Beschreibung:** SQLite-basierter Katalog mit FTS5-Suche, Indizes, Migrations-System.
**Nutzen:** Schnelle Suche und Filterung bei 100k+ Fotos.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** rusqlite, Schema-Design
**Teststrategie:** Performance-Tests mit 10k, 100k, 500k Einträgen. Migration-Tests. Concurrent-Access-Tests. Crash-Recovery.

### A.06 Sidecar Sync
**Beschreibung:** Automatische Synchronisation zwischen Katalog-DB und XMP-Sidecar-Dateien. Konfigurierbar: Auto-Write, Manual, Read-Only.
**Nutzen:** Interoperabilität mit anderen Tools (Lightroom, darktable, Photo Mechanic).
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M (Konflikte, Race Conditions)
**Abhängigkeiten:** XMP Engine, Catalog DB, File Watcher
**Teststrategie:** Roundtrip-Tests (schreiben → lesen → vergleichen). Konflikt-Tests (gleichzeitige Änderung in DB und Sidecar). Encoding-Tests (UTF-8, Sonderzeichen).

### A.07 Duplicate Detection
**Beschreibung:** Hash-basierte (SHA-256) und EXIF-basierte Duplikat-Erkennung.
**Nutzen:** Verhindert doppelte Imports, spart Speicherplatz.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Import Engine, Catalog DB
**Teststrategie:** Tests mit identischen Dateien, near-duplicates (gleicher EXIF, anderer Crop), verschiedene Dateiformate desselben Fotos.

### A.08 Missing File Recovery
**Beschreibung:** Erkennung und Wiederherstellung fehlender Dateien. Suche in alternativen Pfaden, manuelle Zuordnung.
**Nutzen:** Wenn Festplatten umbenannt oder Ordner verschoben werden.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Catalog DB, File Hash Service
**Teststrategie:** Tests mit verschobenen Volumes, umbenannten Ordnern, gelöschten Dateien.

### A.09 Versioning / Edit History
**Beschreibung:** Voller Edit-History-Stack pro Foto. Benannte Snapshots.
**Nutzen:** Nicht-destruktiver Workflow. Jederzeit zu jedem Zustand zurückkehren.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Edit Data Model
**Teststrategie:** Undo/Redo-Tests mit komplexen Edit-Sequenzen. Snapshot-Tests. Speicherverbrauchs-Tests bei langer History.

### A.10 Virtual Copies
**Beschreibung:** Mehrere Bearbeitungsversionen eines Fotos ohne Dateikopie.
**Nutzen:** Verschiedene Looks testen (Color vs. B&W, verschiedene Crops).
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Catalog DB, Edit Data Model
**Teststrategie:** Tests mit Erstellen, Löschen, Bearbeiten von Virtual Copies. Tests mit Export aller Copies.

### A.11 Stacks
**Beschreibung:** Fotos gruppieren (Belichtungsreihe, Burst, HDR-Quellen).
**Nutzen:** Aufgeräumte Bibliothek bei vielen ähnlichen Aufnahmen.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Catalog DB
**Teststrategie:** Auto-Stack by Time. Manuelles Stacking. Unstacking. Stack-Collapse/Expand in Grid.

### A.12 Collections (Manual)
**Beschreibung:** Manuelle Sammlungen (wie Playlists). Fotos können in mehreren Sammlungen sein.
**Nutzen:** Projektbasierte Organisation (Hochzeit A, Portfolio, Instagram-Favoriten).
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** Catalog DB
**Teststrategie:** CRUD-Tests. Drag & Drop. Sortierung. Collection Sets (verschachtelt).

### A.13 Smart Collections
**Beschreibung:** Regelbasierte, automatisch aktualisierte Sammlungen.
**Nutzen:** "Alle 5-Sterne-Fotos mit Sony A7IV von 2025" — automatisch.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (Query-Builder-Komplexität)
**Abhängigkeiten:** Catalog DB, Query Engine
**Teststrategie:** Tests mit komplexen Regeln (AND/OR/NOT). Performance-Tests mit grossen Katalogen. Update-Tests bei neuen Imports.

### A.14 Project Folders
**Beschreibung:** Projektorientierte Ordnerstruktur innerhalb des Katalogs.
**Nutzen:** Studio-Workflow: Projekt = Kunde + Shooting + Deliverables.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Catalog DB, Collections
**Teststrategie:** Erstellen, Umbenennen, Verschieben, Löschen von Projekten.

### A.15 Archive Mode
**Beschreibung:** Fotos als "archiviert" markieren. Werden nicht in Standardansichten gezeigt, bleiben aber im Katalog.
**Nutzen:** Langzeitarchivierung ohne Löschen.
**Prio:** P2 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** Catalog DB
**Teststrategie:** Archivieren, Wiederherstellen, Suche in Archiv.

### A.16 Offline Mode / Smart Previews
**Beschreibung:** Generierung von Smart Previews (komprimierte DNG oder lossy RAW). Bearbeitung möglich, wenn Originale offline sind.
**Nutzen:** Laptop-Workflow: Fotos auf externer Festplatte, bearbeiten unterwegs.
**Prio:** P2 | **Schwierigkeit:** 4 | **Risiko:** M
**Abhängigkeiten:** RAW Engine, Cache System, Export Engine
**Teststrategie:** Bearbeitung offline → Sync wenn online. Qualitätsvergleich Smart Preview vs. Original.

### A.17 Preview / Proxy Pipeline
**Beschreibung:** Mehrstufige Vorschau-Generierung: Thumbnail (256px), Preview (2048px), 1:1 Preview.
**Nutzen:** Schnelle Navigation auch bei grossen RAW-Dateien (50-100MB).
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M (Speicherverbrauch)
**Abhängigkeiten:** RAW Engine, Cache System, GPU Pipeline
**Teststrategie:** Generierungsgeschwindigkeit. Speicherverbrauch. Cache-Invalidierung bei Edit-Änderung.

### A.18 Remote Library Index
**Beschreibung:** Katalog-Index für Fotos auf Netzwerk-Speicher (NAS, Cloud-Mount).
**Nutzen:** Zentralisierte Bibliothek im Heimnetzwerk.
**Prio:** P3 | **Schwierigkeit:** 4 | **Risiko:** H (Netzwerk-Latenz, Offline-Handling)
**Abhängigkeiten:** Catalog DB, Smart Previews, File System Abstraction
**Teststrategie:** Tests mit simulierter Latenz. Tests mit Verbindungsabbrüchen. Tests mit Concurrent Access.

---

## B. CULLING / AUSWAHL

### B.01 Fullscreen Culling
**Beschreibung:** Dedizierter Vollbild-Modus für schnelles Sichten und Bewerten.
**Nutzen:** Hochzeits-/Event-Fotografen sichten 3'000+ Fotos schnell.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Loupe View, Rating System, Shortcut Engine
**Teststrategie:** Geschwindigkeits-Tests (Fotos/Minute). Tastatur-Only-Tests. Prefetch-Tests.

### B.02 Compare Mode
**Beschreibung:** 2 Fotos nebeneinander vergleichen. Sync-Zoom, Sync-Pan.
**Nutzen:** Beste Aufnahme aus einer Serie wählen.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Loupe View, GPU Rendering
**Teststrategie:** Tests mit verschiedenen Seitenverhältnissen. Sync-Tests. Performance bei 1:1 Zoom.

### B.03 Survey Mode
**Beschreibung:** N Bilder gleichzeitig im Raster anzeigen (3-12 Bilder).
**Nutzen:** Vergleich mehrerer Varianten einer Szene.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Grid View, GPU Rendering
**Teststrategie:** Tests mit 3, 6, 9, 12 Bildern. Performance. Memory.

### B.04 Burst Grouping
**Beschreibung:** Automatisches Gruppieren von Burst-Aufnahmen nach Zeitstempel.
**Nutzen:** 20 Burst-Fotos werden zu einer Gruppe, nur das beste wird bewertet.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Stacks, EXIF Parser
**Teststrategie:** Tests mit verschiedenen Burst-Geschwindigkeiten. Tests mit fehlenden EXIF-Daten.

### B.05 Focus Check / Zoom-to-Eye
**Beschreibung:** Schnelles Zoomen auf Fokuspunkt oder Augen bei Portraits.
**Nutzen:** Sofortige Schärfe-Prüfung ohne manuelles Zoomen.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (AF-Point-Daten sind kamera-spezifisch)
**Abhängigkeiten:** EXIF Parser (AF-Point-Daten), optional Face Detection
**Teststrategie:** Tests mit verschiedenen Kameras. Tests mit fehlenden AF-Daten.

### B.06 Face Grouping (Plugin)
**Beschreibung:** Gesichtserkennung und -gruppierung für Personen-Organisation.
**Nutzen:** "Zeige alle Fotos von Person X."
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** H (ML-Modell-Qualität, Privacy)
**Abhängigkeiten:** Plugin System, ONNX Runtime oder ähnlich
**Teststrategie:** Precision/Recall-Tests. Tests mit verschiedenen Ethnien. Privacy-Tests (keine Cloud).

### B.07 Similarity Grouping
**Beschreibung:** Gruppierung visuell ähnlicher Fotos (z.B. gleiche Szene, leicht andere Perspektive).
**Nutzen:** Schnelleres Culling bei Event-Fotografie.
**Prio:** P3 | **Schwierigkeit:** 4 | **Risiko:** M
**Abhängigkeiten:** Perceptual Hashing, optional ML
**Teststrategie:** Tests mit bekannten Similar/Different-Paaren.

### B.08 Rating by Keyboard
**Beschreibung:** 0-5 Sterne, P/X/U Flags, 6-9 Color Labels — alles per Tastatur.
**Nutzen:** Schnellstes Culling möglich.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** Shortcut Engine, Catalog DB
**Teststrategie:** Alle Tastenkombinationen. Auto-Advance nach Rating.

### B.09 Reject Workflow
**Beschreibung:** Rejected-Fotos sammeln, Review, Bulk-Delete mit Bestätigung.
**Nutzen:** Sicheres Aussortieren schlechter Aufnahmen.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** Rating System, Catalog DB
**Teststrategie:** Reject → Review → Delete-Flow. Undo-Tests. Trash vs. Delete.

### B.10 AI Sharpness Estimation (Plugin)
**Beschreibung:** ML-basierte Schärfe-Bewertung pro Foto.
**Nutzen:** Automatisches Aussortieren unscharfer Bilder.
**Prio:** P4 | **Schwierigkeit:** 4 | **Risiko:** M
**Abhängigkeiten:** Plugin System, ML Runtime
**Teststrategie:** Tests mit bekannten scharf/unscharf-Paaren. False Positive Rate.

### B.11 Blink / Closed-Eye Detection (Plugin)
**Beschreibung:** Erkennung geschlossener Augen bei Portraits.
**Nutzen:** Sofortiges Aussortieren von "Blinzlern" bei Gruppenfotos.
**Prio:** P4 | **Schwierigkeit:** 4 | **Risiko:** M
**Abhängigkeiten:** Face Detection Plugin, ML Runtime
**Teststrategie:** Precision/Recall. Tests mit Sonnenbrillen, Halbprofil.

### B.12 Duplicate Clustering
**Beschreibung:** Gruppierung von Duplikaten/Near-Duplicates für Review.
**Nutzen:** Aufräumen grosser Bibliotheken.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M
**Abhängigkeiten:** Perceptual Hashing, Catalog DB
**Teststrategie:** Tests mit echten Duplikaten, Near-Duplicates, False Positives.

---

## C. RAW-DEVELOPMENT

### C.01 Exposure
**Beschreibung:** Belichtungskorrektur (-5 bis +5 EV). Linear im Szenen-Raum.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Linearity-Tests. Clipping-Tests. Roundtrip XMP.

### C.02 Tone Controls (Highlights/Shadows/Whites/Blacks)
**Beschreibung:** Getrennte Kontrolle der Tonwertbereiche.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Vergleich mit Lightroom-Output. Histogram-Konsistenz.

### C.03 Contrast
**Beschreibung:** Globaler Kontrast mit S-Kurve.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Linearity-Tests bei 0. Symmetrie bei ±50.

### C.04 Tone Curve (Parametric + Point)
**Beschreibung:** Parametrische Kurve (4 Regionen) und frei editierbare Punkt-Kurve (RGB + einzelne Kanäle).
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** L
**Teststrategie:** Interpolations-Tests (Spline-Qualität). XMP-Roundtrip. Edge Cases (Kreuzungen).

### C.05 HSL / Color Mixer
**Beschreibung:** Hue, Saturation, Luminance pro Farbkanal (8 Kanäle).
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Pro-Kanal-Tests. Overlap-Tests an Farbgrenzen. XMP-Kompatibilität.

### C.06 White Balance
**Beschreibung:** Temperature + Tint. Presets (Daylight, Cloudy etc.). Picker (Click auf neutrales Grau).
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Picker-Tests. Preset-Werte. XMP-Roundtrip.

### C.07 Camera Profiles / Color Matching
**Beschreibung:** Kamera-spezifische Farbprofile. Match Camera Standard, Adobe Color/Landscape/Portrait-Äquivalente.
**Prio:** P1 | **Schwierigkeit:** 4 | **Risiko:** H (DCP ist semi-proprietär)
**Abhängigkeiten:** Color Management, DCP Parser
**Teststrategie:** Vergleich mit Kamera-JPEG. Vergleich mit Lightroom-Output pro Profil.

### C.08 DCP / LUT Support
**Beschreibung:** Import von Adobe DCP-Profilen und 3D-LUTs (.cube, .3dl).
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (DCP-Spec ist komplex)
**Abhängigkeiten:** Color Management
**Teststrategie:** Import-Tests mit verschiedenen DCP-Dateien. LUT-Interpolations-Tests.

### C.09 Lens Corrections
**Beschreibung:** Profil-basiert (LensFun DB), manuell. Distortion, Vignetting, CA.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (LensFun-Coverage)
**Abhängigkeiten:** LensFun-Integration, GPU Pipeline
**Teststrategie:** Tests mit bekannten Objektiven. Before/After-Vergleich. Edge-Case: unbekanntes Objektiv.

### C.10 Geometry / Transform / Upright
**Beschreibung:** Perspective Correction, Vertical/Horizontal, Rotate, Aspect, Scale. Auto-Upright.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (Auto-Detection-Qualität)
**Abhängigkeiten:** GPU Pipeline, optional Line Detection
**Teststrategie:** Tests mit Architekturfotos. Auto-Upright vs. manuell. XMP-Roundtrip.

### C.11 Denoise
**Beschreibung:** Luminance + Chroma Noise Reduction. Wavelet + NLM Hybrid.
**Prio:** P0 | **Schwierigkeit:** 4 | **Risiko:** M (Qualität vs. Performance)
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Tests bei verschiedenen ISO (100, 3200, 12800, 51200). Performance-Tests. A/B mit Lightroom.

### C.12 Sharpening
**Beschreibung:** Amount, Radius, Detail, Masking. Unsharp-Mask-Variante.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Tests bei verschiedenen Auflösungen. Masking-Vorschau. XMP-Roundtrip.

### C.13 Dehaze
**Beschreibung:** Dunst-Entfernung basierend auf Dark Channel Prior.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (Artefakte bei nicht-nebligen Bildern)
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Tests mit nebligen und klaren Bildern. Negative-Werte (Nebel hinzufügen). XMP.

### C.14 Clarity / Texture
**Beschreibung:** Clarity = Midtone Contrast. Texture = feinere Version (Lightroom CC Feature).
**Prio:** P0 (Clarity) / P2 (Texture) | **Schwierigkeit:** 3 | **Risiko:** M
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Frequency-Separation-Tests. Vergleich mit Lightroom.

### C.15 Grain
**Beschreibung:** Film-Grain-Simulation. Amount, Size, Roughness.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Determinismus (gleiche Settings → gleicher Output). XMP-Roundtrip.

### C.16 Post-Crop Vignette
**Beschreibung:** Vignettierung nach dem Crop. Amount, Midpoint, Roundness, Feather.
**Prio:** P1 | **Schwierigkeit:** 1 | **Risiko:** L
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** Tests mit verschiedenen Crops. XMP-Roundtrip.

### C.17 Calibration
**Beschreibung:** Shadow Tint, RGB Primary Hue/Saturation Shifts.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** GPU Pipeline, Color Management
**Teststrategie:** XMP-Roundtrip. Vergleich mit Lightroom.

### C.18 Monochrome / B&W Mix
**Beschreibung:** Schwarz-Weiss-Konvertierung mit Kanal-Mixer (wie Lightroom B&W Mix).
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** HSL Module, GPU Pipeline
**Teststrategie:** Vergleich mit Lightroom B&W. Auto-Mix-Tests.

### C.19 Color Grading
**Beschreibung:** 3-Wege Color Grading (Shadows, Midtones, Highlights) + Global. Blending + Balance.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** GPU Pipeline
**Teststrategie:** XMP-Roundtrip (ersetzt Split Toning). Vergleich mit Lightroom.

### C.20 HDR Merge (Plugin)
**Beschreibung:** Belichtungsreihe → 32-bit HDR. Deghosting, Auto-Align.
**Prio:** P2 | **Schwierigkeit:** 4 | **Risiko:** M
**Abhängigkeiten:** Plugin System, RAW Engine
**Teststrategie:** Tests mit 3, 5, 7 Belichtungen. Deghosting. Alignment.

### C.21 Panorama Merge (Plugin)
**Beschreibung:** Mehrere Bilder → Panorama. Cylindrical, Spherical, Perspective.
**Prio:** P2 | **Schwierigkeit:** 5 | **Risiko:** H (Stitching-Qualität)
**Abhängigkeiten:** Plugin System, Image Registration
**Teststrategie:** Tests mit 2, 5, 10 Bildern. Verschiedene Projektionen. Boundary Warp.

### C.22 Focus Stacking Prep
**Beschreibung:** Nicht das Stacking selbst, sondern: Erkennung, Gruppierung, Export für externe Stacking-Tools.
**Prio:** P3 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** EXIF, Stacks
**Teststrategie:** Erkennung von Focus-Brackets.

### C.23 Local Adjustments (Brush, Gradient, Radial)
**Beschreibung:** Lokale Korrekturen mit Masken. Jede Maske hat eigene Develop-Settings.
**Prio:** P1 | **Schwierigkeit:** 4 | **Risiko:** M (GPU-Performance bei vielen Masken)
**Abhängigkeiten:** GPU Pipeline, Mask Rendering
**Teststrategie:** Tests mit 1, 5, 20 Masken. Performance. Feathering. XMP-Import.

### C.24 AI Masks (Sky, Subject, Background)
**Beschreibung:** ML-basierte Maskenerkennung. Lokal, kein Cloud.
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** H (Modell-Qualität, Size, Speed)
**Abhängigkeiten:** Plugin System, ONNX Runtime, Segmentation Model
**Teststrategie:** IoU-Tests mit Ground Truth. Speed-Tests. Edge-Case-Tests.

### C.25 Healing / Clone / Content-Aware
**Beschreibung:** Spot-Healing (Staub-Entfernung), Clone-Tool, Content-Aware Fill (begrenzt).
**Prio:** P2 (Healing/Clone) / P4 (Content-Aware) | **Schwierigkeit:** 3/5 | **Risiko:** M/H
**Abhängigkeiten:** GPU Pipeline, optional ML
**Teststrategie:** Dust-Spot-Tests. Clone-Alignment. Content-Aware bei verschiedenen Szenen.

### C.26 Soft Proofing
**Beschreibung:** Vorschau, wie das Foto auf einem bestimmten Drucker/Papier/Monitor aussehen wird.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (ICC-Profil-Handling)
**Abhängigkeiten:** Color Management, ICC Profile Engine
**Teststrategie:** Tests mit verschiedenen ICC-Profilen. Gamut-Warning-Tests.

---

## D. METADATEN

### D.01 EXIF Reader
**Beschreibung:** Vollständiges EXIF-Parsing. Kamera, Objektiv, Belichtung, GPS, Datum.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Tests mit RAW-Dateien von 20+ Kameras. Edge Cases (fehlende Felder, korrupte EXIF).

### D.02 IPTC Editor
**Beschreibung:** Editierbare IPTC-Felder: Titel, Beschreibung, Keywords, Copyright, Kontakt, Ort.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Schreib-/Lese-Roundtrip. UTF-8-Tests. Batch-Edit-Tests.

### D.03 XMP Read/Write
**Beschreibung:** Vollständiger XMP-Parser und -Writer. Adobe-kompatibel.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Roundtrip mit Lightroom-XMP. Unbekannte Namespaces erhalten. Encoding-Tests.

### D.04 Keyword Hierarchies
**Beschreibung:** Hierarchische Keywords (Tier > Säugetier > Katze). Import/Export.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Verschachtelungs-Tests. Import von Lightroom-Keywords. Export als Tab-getrennte Liste.

### D.05 People Tags
**Beschreibung:** Personen-Tags mit optionaler Gesichtserkennung-Integration.
**Prio:** P2 | **Schwierigkeit:** 2 (ohne ML) / 5 (mit ML) | **Risiko:** M
**Teststrategie:** Manuelles Taggen. Integration mit Face Detection Plugin.

### D.06 Location Tagging
**Beschreibung:** GPS-Koordinaten, Reverse-Geocoding (→ Stadt, Land). Kartenansicht.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Abhängigkeiten:** Map Module, Geocoding Service (lokal oder Nominatim)
**Teststrategie:** Tests mit GPS-Daten. Reverse-Geocoding-Genauigkeit. Offline-Fallback.

### D.07 Copyright Presets
**Beschreibung:** Speicherbare Copyright-Templates (Name, URL, Rechte). Automatisch beim Import anwendbar.
**Prio:** P1 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Preset-CRUD. Import-Integration. XMP-Export-Test.

### D.08 Metadata Templates
**Beschreibung:** Vollständige Metadaten-Templates (Copyright, Kontakt, IPTC, Keywords).
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Template-Anwendung auf Batch. Partial-Apply-Tests.

### D.09 Batch Metadata Editing
**Beschreibung:** Metadaten auf viele Fotos gleichzeitig anwenden.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Tests mit 10, 100, 1000 Fotos. Append vs. Replace für Keywords.

### D.10 Sidecar Conflict Detection
**Beschreibung:** Erkennung von Konflikten zwischen DB und Sidecar (z.B. extern modifiziert).
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M
**Abhängigkeiten:** Sidecar Sync, File Watcher
**Teststrategie:** Gleichzeitige Änderung in DB und Sidecar. Merge-Strategien.

### D.11 Metadata Diff Viewer
**Beschreibung:** Anzeige der Unterschiede zwischen DB-Metadaten und Sidecar/EXIF.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Diff-Anzeige bei bekannten Unterschieden.

### D.12 Metadata Merge Assistant
**Beschreibung:** Tool zum intelligenten Zusammenführen von Metadaten aus verschiedenen Quellen.
**Prio:** P3 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Merge-Strategien (neuere gewinnt, manuell, per-Feld).

---

## E. PERFORMANCE

### E.01 GPU Acceleration
**Beschreibung:** Gesamte Processing-Pipeline auf GPU (wgpu: Vulkan/Metal/DX12).
**Prio:** P0 | **Schwierigkeit:** 4 | **Risiko:** M (GPU-Kompatibilität)
**Teststrategie:** Tests auf 5+ GPU-Generationen. CPU-Fallback-Tests. Benchmark-Suite.

### E.02 Tile Rendering
**Beschreibung:** Bild in Tiles aufteilen für progressive Anzeige und 1:1-Zoom.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Tile-Boundary-Artefakt-Tests. Performance bei verschiedenen Zoom-Stufen.

### E.03 Cache Hierarchy (L1-L4)
**Beschreibung:** GPU Cache → RAM Cache → Disk Cache → Original.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Cache-Hit-Rate-Tests. Invalidierungs-Tests. Speicher-Limit-Tests.

### E.04 Preview Pipelines
**Beschreibung:** Separate Pipelines für Thumbnails, Previews und Full-Quality.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Geschwindigkeits-Tests pro Pipeline-Stufe.

### E.05 Lazy Loading
**Beschreibung:** Nur sichtbare Elemente laden (Grid, Filmstrip, Panels).
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Scroll-Performance-Tests mit 100k Thumbnails.

### E.06 Memory Control
**Beschreibung:** Konfigurierbarer RAM-Verbrauch. Cache-Limits. GC für Preview-Cache.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** M (Memory-Leaks)
**Teststrategie:** Langzeit-Tests (8h-Session). Leak-Detection. Memory-Profiling.

### E.07 Background Job System
**Beschreibung:** Job-Queue für Import, Export, Preview-Generierung, Batch-Operationen.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Concurrent-Jobs. Cancel-Tests. Priority-Tests. Crash-Recovery.

### E.08 Batching / Job Queue
**Beschreibung:** Batch-Verarbeitung mit Fortschritt, Pause, Cancel, Retry.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** 1000-Foto-Batch. Cancel mid-batch. Retry nach Fehler.

### E.09 Crash Recovery
**Beschreibung:** Automatische Wiederherstellung nach Crash. Unsaved Edits Recovery. Journal-basiert.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Simulierte Crashes mid-edit, mid-import, mid-export. DB-Recovery.

### E.10 Massive Library Scaling (100k+)
**Beschreibung:** Performance-Optimierungen für Kataloge mit 100'000+ Fotos.
**Prio:** P1 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** Benchmark mit 10k, 50k, 100k, 500k Einträgen. Query-Performance. Scroll-Performance.

### E.11 Telemetry-Free Diagnostics
**Beschreibung:** Lokale Performance-Diagnose ohne Datenversand. Export von Diagnose-Berichten (manuell).
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Diagnose-Report-Generierung. Kein Netzwerk-Verkehr.

---

## F. EXPORT

### F.01 Format-Support
**Beschreibung:** JPEG, PNG, TIFF (8/16-bit), WebP, AVIF, HEIF, DNG.
**Prio:** P0 (JPEG/TIFF/PNG) / P1 (WebP/AVIF) / P2 (HEIF/DNG) | **Schwierigkeit:** 2-3 | **Risiko:** L
**Teststrategie:** Roundtrip-Tests. Qualitätsvergleich. Metadaten-Erhaltung.

### F.02 Naming Templates
**Beschreibung:** Konfigurierbare Export-Dateinamen: `{date}_{camera}_{sequence}_{rating}`.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Template-Parser-Tests. Edge Cases (Sonderzeichen, Umlaute).

### F.03 Watermarking
**Beschreibung:** Text- und Bild-Wasserzeichen. Position, Grösse, Transparenz konfigurierbar.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Tests mit verschiedenen Positionen. Skalierung bei verschiedenen Auflösungen.

### F.04 Resize Rules
**Beschreibung:** Long Edge, Short Edge, Width, Height, Megapixels, Percentage.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Tests mit verschiedenen Seitenverhältnissen. Don't-Enlarge-Option.

### F.05 Output Sharpening
**Beschreibung:** Schärfung für Screen, Matte Paper, Glossy Paper. Low/Standard/High.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Vergleich mit Lightroom-Output-Sharpening.

### F.06 Contact Sheets
**Beschreibung:** Übersichtsblätter mit Thumbnails (PDF oder JPEG). Konfigurierbare Layouts.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Layout-Tests. PDF-Generierung. Metadaten-Overlay.

### F.07 Print Templates
**Beschreibung:** Druckvorlagen für verschiedene Papierformate.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** DPI-Tests. Color-Management-Tests.

### F.08 Web Gallery Export
**Beschreibung:** Generierung einer statischen HTML-Galerie aus einer Sammlung.
**Prio:** P3 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** HTML-Validierung. Responsiveness. Verschiedene Template-Styles.

### F.09 Publish Targets (Plugin)
**Beschreibung:** Direct-Publish zu Flickr, 500px, SmugMug, WordPress etc.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (API-Abhängigkeit)
**Teststrategie:** Mock-API-Tests. Auth-Tests. Upload-Tests.

### F.10 Export Recipes
**Beschreibung:** Gespeicherte Export-Konfigurationen. Schneller Re-Export.
**Prio:** P0 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** CRUD. Batch-Export mit Recipe.

### F.11 Queue Retries
**Beschreibung:** Fehlgeschlagene Exports erneut versuchen. Fehlerprotokoll.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Simulierte Fehler (Disk Full, Permission Denied). Retry-Logik.

### F.12 Color-Space-Aware Export
**Beschreibung:** Automatische Color-Space-Konvertierung (ProPhoto → sRGB/Adobe RGB/P3) beim Export.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Gamut-Tests. Profil-Embedding-Tests.

---

## G. TETHERING / PRO WORKFLOWS

### G.01 Tethered Capture (Plugin)
**Beschreibung:** Live-Verbindung zur Kamera. Fotos werden direkt importiert.
**Prio:** P2 | **Schwierigkeit:** 4 | **Risiko:** H (Kamera-SDK-Abhängigkeit)
**Abhängigkeiten:** Plugin System, gPhoto2 oder Kamera-SDK
**Teststrategie:** Tests mit Canon, Nikon, Sony (gPhoto2-kompatibel). Latenz-Tests.

### G.02 Session Mode
**Beschreibung:** Shooting-Session mit definiertem Ordner, Naming, Metadaten.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Session-Erstellung. Auto-Import. Session-Archivierung.

### G.03 Live Ingest
**Beschreibung:** Sofortige Anzeige neuer Fotos während eines Shootings.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M
**Abhängigkeiten:** Watch Folders, Preview Pipeline
**Teststrategie:** Latenz vom Auslösen bis zur Anzeige (<3s Ziel).

### G.04 Backup on Import
**Beschreibung:** Automatische Backup-Kopie beim Import (zweites Ziel).
**Prio:** P1 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Backup-Verifikation. Fehler bei vollem Backup-Ziel.

### G.05 Studio Tagging
**Beschreibung:** Schnelles Taggen während eines Shootings (Set, Look, Model).
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Speed-Tests. Preset-Tags.

### G.06 Shot List
**Beschreibung:** Vordefinierte Aufnahmeliste. Shots abhaken.
**Prio:** P3 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** CRUD. Zuordnung Foto → Shot.

### G.07 Client Review Mode
**Beschreibung:** Vereinfachter Modus für Kunden. Nur Bewerten, kein Bearbeiten.
**Prio:** P3 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Berechtigungs-Tests. UI-Reduzierung.

### G.08 Live Select Mode
**Beschreibung:** Echtzeit-Auswahl während eines Shootings. Fotos sofort bewertbar.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Performance bei schnellem Shutter.

### G.09 Collaborative Tagging
**Beschreibung:** Mehrere Personen taggen gleichzeitig (Netzwerk-Sync).
**Prio:** P4 | **Schwierigkeit:** 5 | **Risiko:** H
**Teststrategie:** Concurrent-Edit-Tests. Conflict-Resolution.

---

## H. AI-FEATURES (alle als Plugins)

### H.01 Semantic Search
**Beschreibung:** "Zeige mir alle Fotos mit Sonnenuntergang am Meer."
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** H
**Teststrategie:** Recall/Precision-Tests mit bekannten Szenen.

### H.02 Duplicate Reasoning
**Beschreibung:** ML-basierte Erkennung visueller Duplikate (nicht nur Hash).
**Prio:** P3 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** Tests mit Known-Duplicate-Paaren.

### H.03 Scene Detection
**Beschreibung:** Automatische Szenen-Klassifikation (Landschaft, Portrait, Architektur, Sport).
**Prio:** P3 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** Accuracy pro Szenen-Typ.

### H.04 Subject Masking (AI)
**Beschreibung:** ML-basierte Subjekt-Segmentierung. SAM oder vergleichbar.
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** H (Modell-Grösse, Inferenz-Speed)
**Teststrategie:** IoU auf Testset. Speed-Benchmark.

### H.05 Auto Keywording
**Beschreibung:** Automatische Keyword-Vorschläge basierend auf Bildinhalten.
**Prio:** P3 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** Precision/Recall. Relevanz-Tests.

### H.06 Smart Preset Recommendation
**Beschreibung:** Preset-Vorschläge basierend auf Bildinhalt und Stil.
**Prio:** P4 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** Relevanz-Bewertung durch Fotografen.

### H.07 Style Transfer Suggestions
**Beschreibung:** "Mach dieses Foto wie [Referenzbild]." Analyse und Parameter-Matching.
**Prio:** P4 | **Schwierigkeit:** 5 | **Risiko:** H
**Teststrategie:** A/B-Vergleich mit manueller Anpassung.

### H.08 AI Denoise (Advanced)
**Beschreibung:** ML-basierte Rauschentfernung (à la DxO PureRAW).
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** M
**Teststrategie:** PSNR/SSIM-Tests. ISO-Stufentests. Speed.

### H.09 Dust Spot Detection
**Beschreibung:** Automatische Erkennung von Sensorflecken.
**Prio:** P3 | **Schwierigkeit:** 3 | **Risiko:** L
**Teststrategie:** Tests mit bekannten Dust-Spot-Bildern.

### H.10 Natural Language Command Palette
**Beschreibung:** "Erhöhe die Belichtung um 0.5 und füge etwas Wärme hinzu."
**Prio:** P4 | **Schwierigkeit:** 4 | **Risiko:** M (LLM-Abhängigkeit)
**Teststrategie:** Intent-Recognition-Tests. Parameter-Parsing.

---

## I. ACCESSIBILITY

### I.01 Complete Keyboard Flow
**Beschreibung:** Jede Aktion per Tastatur erreichbar. Focus-Management.
**Prio:** P0 | **Schwierigkeit:** 3 | **Risiko:** M (umfangreich)
**Teststrategie:** Tab-Order-Tests. Keyboard-Only-Sessions.

### I.02 Screen Reader Support
**Beschreibung:** ARIA-Labels, Rollen, Status-Updates für Screen-Reader.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (WebView-Einschränkungen)
**Teststrategie:** Tests mit VoiceOver (macOS), NVDA (Windows).

### I.03 High Contrast Mode
**Beschreibung:** Erhöhter Kontrast für UI-Elemente.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** WCAG-Kontrast-Checks.

### I.04 Large UI Mode
**Beschreibung:** Grössere Schrift, grössere Controls für Touchscreens oder Sehschwäche.
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Layout-Tests bei 150%, 200% Skalierung.

### I.05 Color-Blind Safe Overlays
**Beschreibung:** Farb-Labels und Overlays auch für farbenblinde Nutzer erkennbar (Muster, Icons).
**Prio:** P2 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Simulation von Protanopie, Deuteranopie, Tritanopie.

### I.06 Motion Reduction
**Beschreibung:** Reduzierte Animationen (prefers-reduced-motion). Keine blinkenden Elemente.
**Prio:** P2 | **Schwierigkeit:** 1 | **Risiko:** L
**Teststrategie:** Prüfung aller Animationen.

### I.07 Remappable Shortcuts
**Beschreibung:** Alle Shortcuts frei belegbar. Import/Export von Shortcut-Maps.
**Prio:** P0 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** Konflikt-Detection. Import/Export-Roundtrip.

---

## J. ECOSYSTEM

### J.01 Plugin API
**Beschreibung:** Stabile, versionierte Plugin-API. WASM-Sandbox.
**Prio:** P1 | **Schwierigkeit:** 4 | **Risiko:** M
**Teststrategie:** API-Stabilitäts-Tests. Sandbox-Escape-Tests. Version-Kompatibilität.

### J.02 Preset Marketplace
**Beschreibung:** Community-Marketplace für Presets. Upload, Download, Bewertung.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Upload/Download-Flow. Validierung. Virus-Scan.

### J.03 Camera/Lens Profile Packs
**Beschreibung:** Community-gepflegte Kamera- und Objektivprofile.
**Prio:** P1 | **Schwierigkeit:** 3 | **Risiko:** M (Qualitätssicherung)
**Teststrategie:** Profil-Validierung gegen Referenzdaten.

### J.04 Scripting / Automation
**Beschreibung:** Scripting-API (Lua, JS oder WASM) für Automatisierung. Batch-Scripte.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Script-Execution-Tests. Sandbox-Tests.

### J.05 CLI Export
**Beschreibung:** Headless Export via CLI. Für CI/CD-Pipelines und Automation.
**Prio:** P1 | **Schwierigkeit:** 2 | **Risiko:** L
**Teststrategie:** CLI-Integration-Tests. Batch-Tests.

### J.06 Headless Processing
**Beschreibung:** Vollständige RAW-Pipeline ohne UI. Server-Einsatz.
**Prio:** P2 | **Schwierigkeit:** 3 | **Risiko:** M (GPU-Zugang auf Servern)
**Teststrategie:** Docker-Tests. CPU-Fallback-Tests.

### J.07 Web Review Companion
**Beschreibung:** Leichtgewichtige Web-App für Client-Review (nur Anschauen + Bewerten).
**Prio:** P3 | **Schwierigkeit:** 3 | **Risiko:** M
**Teststrategie:** Mobile-Tests. Rating-Sync-Tests.

### J.08 Mobile Companion
**Beschreibung:** Mobile App für Quick-Review, Rating, Metadata.
**Prio:** P4 | **Schwierigkeit:** 5 | **Risiko:** H
**Teststrategie:** iOS/Android-Tests. Sync-Tests.

### J.09 Cloud Sync (Optional)
**Beschreibung:** Optionale Synchronisation von Katalog/Settings/Presets. Nie Pflicht. E2E-encrypted.
**Prio:** P3 | **Schwierigkeit:** 5 | **Risiko:** H
**Teststrategie:** Sync-Conflict-Tests. Encryption-Tests. Offline-Tests.

---

# FEATURE-PRIORITÄTS-MATRIX (Top 25)

| # | Feature | Prio | Phase |
|---|---------|------|-------|
| 1 | RAW Decode + Demosaic (Top-5 Kameras) | P0 | MVP |
| 2 | GPU Processing Pipeline (Basis) | P0 | MVP |
| 3 | Import Engine (Copy/Move/Add) | P0 | MVP |
| 4 | Catalog Database (SQLite) | P0 | MVP |
| 5 | Library Grid + Loupe View | P0 | MVP |
| 6 | Basic Develop (Exposure, WB, Contrast, HSL) | P0 | MVP |
| 7 | Rating/Flagging/Color Labels per Keyboard | P0 | MVP |
| 8 | Copy/Paste Edits | P0 | MVP |
| 9 | XMP Sidecar Read | P0 | MVP |
| 10 | Export (JPEG, TIFF) | P0 | MVP |
| 11 | Keyboard Shortcuts (LR-kompatibel) | P0 | MVP |
| 12 | Histogram | P0 | MVP |
| 13 | Before/After | P0 | MVP |
| 14 | Filmstrip | P0 | MVP |
| 15 | Tone Curve + Color Grading | P0 | V1.0 |
| 16 | Noise Reduction + Sharpening | P0 | V1.0 |
| 17 | Local Adjustments (Brush/Gradient/Radial) | P1 | V1.0 |
| 18 | Lens Corrections (LensFun) | P1 | V1.0 |
| 19 | XMP Sidecar Write | P1 | V1.0 |
| 20 | Smart Collections | P1 | V1.0 |
| 21 | Preset System (Import + Create) | P1 | V1.0 |
| 22 | Compare + Survey Mode | P1 | V1.0 |
| 23 | Command Palette | P1 | V1.0 |
| 24 | Plugin System (v1) | P1 | V1.0 |
| 25 | CLI Export | P1 | V1.0 |
