# OpenClaw Photo Studio — Datenmodelle

> Version: 0.1.0-draft | 2026-03-19

---

## Modell-Übersicht

Alle Modelle sind als Rust-Structs mit serde-Serialisierung definiert. JSON-Schema wird automatisch generiert. Versionierung über `schema_version` Feld.

---

## 1. Asset

```rust
/// Repräsentiert eine einzelne Bilddatei im Katalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub file_path: PathBuf,          // Absolut oder relativ zum Katalog-Root
    pub file_name: String,
    pub file_size: u64,              // Bytes
    pub file_hash: Option<String>,   // SHA-256
    pub mime_type: String,           // image/x-canon-cr3, image/x-nikon-nef etc.
    pub media_type: MediaType,       // RAW, JPEG, TIFF, DNG, Video
    pub width: u32,
    pub height: u32,
    pub orientation: u8,             // EXIF Orientation 1-8
    pub bit_depth: u8,               // 12, 14, 16
    pub color_space: Option<String>, // sRGB, AdobeRGB, ProPhotoRGB
    pub date_taken: Option<DateTime<Utc>>,
    pub date_imported: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub camera: Option<CameraInfo>,
    pub lens: Option<LensInfo>,
    pub exposure: Option<ExposureInfo>,
    pub gps: Option<GpsInfo>,
    pub rating: Rating,              // 0-5
    pub color_label: ColorLabel,
    pub flag: Flag,
    pub has_edits: bool,
    pub edit_count: u32,
    pub virtual_copy_of: Option<Uuid>,
    pub stack_id: Option<Uuid>,
    pub stack_position: Option<u32>,
    pub folder_id: Uuid,
    pub sidecar_path: Option<PathBuf>,
    pub sidecar_last_modified: Option<DateTime<Utc>>,
    pub preview_status: PreviewStatus,
    pub keywords: Vec<Uuid>,         // Keyword IDs
    pub collections: Vec<Uuid>,      // Collection IDs
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType { Raw, Jpeg, Tiff, Png, Dng, Heif, Video }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rating { None, One, Two, Three, Four, Five }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorLabel { None, Red, Yellow, Green, Blue, Purple }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Flag { Unflagged, Pick, Reject }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewStatus { None, Thumbnail, Preview, FullRes }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub make: String,                // "Canon", "Nikon", "Sony"
    pub model: String,               // "EOS R5", "Z8", "A7 IV"
    pub serial: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensInfo {
    pub name: String,                // "RF 24-70mm F2.8 L IS USM"
    pub focal_length: f32,           // mm
    pub focal_length_35mm: Option<f32>,
    pub max_aperture: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureInfo {
    pub aperture: Option<f32>,       // f-number
    pub shutter_speed: Option<String>, // "1/250", "1.3"
    pub shutter_speed_value: Option<f64>, // Sekunden
    pub iso: Option<u32>,
    pub exposure_comp: Option<f32>,  // EV
    pub flash: Option<bool>,
    pub metering_mode: Option<String>,
    pub focus_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,       // Meter
    pub direction: Option<f64>,      // Grad
    pub city: Option<String>,        // Reverse-Geocoded
    pub country: Option<String>,
    pub country_code: Option<String>,
}
```

**Validierungsregeln:**
- `file_path` muss existieren (bei Import) oder als "missing" markiert werden
- `rating` muss 0-5 sein
- `orientation` muss 1-8 sein
- `file_hash` wird beim Import berechnet
- `date_taken` wird aus EXIF extrahiert, Fallback auf Datei-Datum

---

## 2. SidecarReference

```rust
/// Referenz auf eine XMP-Sidecar-Datei.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarReference {
    pub asset_id: Uuid,
    pub sidecar_path: PathBuf,
    pub sidecar_hash: String,        // SHA-256 des Sidecar-Inhalts
    pub last_read: DateTime<Utc>,
    pub last_written: DateTime<Utc>,
    pub sync_status: SidecarSyncStatus,
    pub conflict: Option<SidecarConflict>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SidecarSyncStatus {
    InSync,                          // DB und Sidecar identisch
    DbNewer,                         // DB hat neuere Daten
    SidecarNewer,                    // Sidecar wurde extern geändert
    Conflict,                        // Beide geändert
    Missing,                         // Sidecar existiert nicht
    Orphaned,                        // Sidecar ohne Asset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarConflict {
    pub detected_at: DateTime<Utc>,
    pub db_values: serde_json::Value,
    pub sidecar_values: serde_json::Value,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepDb,
    KeepSidecar,
    Merged(serde_json::Value),
    Deferred,
}
```

---

## 3. CatalogEntry

```rust
/// Datenbank-Repräsentation eines Katalog-Eintrags.
/// Enthält alle DB-relevanten Felder (exkl. Bilddaten).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub asset: Asset,
    pub current_edit: Option<EditRecipe>,
    pub edit_history: Vec<EditHistoryEntry>,
    pub snapshots: Vec<Snapshot>,
    pub sidecar: Option<SidecarReference>,
    pub metadata: MetadataEnvelope,
    pub preview_paths: PreviewPaths,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPaths {
    pub thumbnail: Option<PathBuf>,   // 256px
    pub preview: Option<PathBuf>,     // 2048px
    pub full_res: Option<PathBuf>,    // 1:1
}
```

---

## 4. MetadataEnvelope

```rust
/// Container für alle Metadaten eines Fotos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEnvelope {
    pub exif: ExifData,
    pub iptc: IptcData,
    pub xmp: XmpData,
    pub custom: HashMap<String, serde_json::Value>,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    pub camera: Option<CameraInfo>,
    pub lens: Option<LensInfo>,
    pub exposure: Option<ExposureInfo>,
    pub gps: Option<GpsInfo>,
    pub date_original: Option<DateTime<Utc>>,
    pub date_digitized: Option<DateTime<Utc>>,
    pub software: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub raw_fields: HashMap<String, String>, // Unbekannte EXIF-Felder
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptcData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub creator: Option<String>,
    pub copyright: Option<String>,
    pub credit: Option<String>,
    pub source: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub headline: Option<String>,
    pub instructions: Option<String>,
    pub usage_terms: Option<String>,
    pub scene: Vec<String>,
    pub subject_code: Vec<String>,
    pub contact: Option<IptcContact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IptcContact {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmpData {
    pub rating: Option<u8>,
    pub label: Option<String>,
    pub develop_settings: Option<serde_json::Value>,
    pub namespaces: HashMap<String, serde_json::Value>, // Unbekannte Namespaces erhalten
}
```

---

## 5. EditRecipe

```rust
/// Vollständige Develop-Settings für ein Foto.
/// Bildet die gesamte nicht-destruktive Bearbeitung ab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditRecipe {
    pub schema_version: u32,         // Für Migrationen
    pub process_version: String,     // "ocps-1.0"
    pub basic: BasicSettings,
    pub tone_curve: ToneCurveSettings,
    pub hsl: HslSettings,
    pub color_grading: ColorGradingSettings,
    pub detail: DetailSettings,
    pub lens_corrections: LensCorrectionSettings,
    pub transform: TransformSettings,
    pub effects: EffectsSettings,
    pub crop: CropSettings,
    pub calibration: CalibrationSettings,
    pub local_adjustments: Vec<LocalAdjustment>,
    pub healing_spots: Vec<HealingSpot>,
    pub monochrome: Option<MonochromeSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicSettings {
    pub white_balance: WhiteBalance,
    pub exposure: f32,               // -5.0 to +5.0 EV
    pub contrast: i32,               // -100 to +100
    pub highlights: i32,             // -100 to +100
    pub shadows: i32,                // -100 to +100
    pub whites: i32,                 // -100 to +100
    pub blacks: i32,                 // -100 to +100
    pub clarity: i32,                // -100 to +100
    pub texture: i32,                // -100 to +100
    pub dehaze: i32,                 // -100 to +100
    pub vibrance: i32,               // -100 to +100
    pub saturation: i32,             // -100 to +100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteBalance {
    pub mode: WbMode,
    pub temperature: u32,            // 2000 - 50000 Kelvin
    pub tint: i32,                   // -150 to +150
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WbMode {
    AsShot, Auto, Daylight, Cloudy, Shade,
    Tungsten, Fluorescent, Flash, Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneCurveSettings {
    pub mode: ToneCurveMode,
    pub parametric: ParametricCurve,
    pub point_curve: PointCurves,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToneCurveMode { Parametric, Point, Both }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricCurve {
    pub highlights: i32,
    pub lights: i32,
    pub darks: i32,
    pub shadows: i32,
    pub highlight_split: u32,        // 0-100, default 75
    pub midtone_split: u32,          // 0-100, default 50
    pub shadow_split: u32,           // 0-100, default 25
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCurves {
    pub rgb: Vec<CurvePoint>,
    pub red: Option<Vec<CurvePoint>>,
    pub green: Option<Vec<CurvePoint>>,
    pub blue: Option<Vec<CurvePoint>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f32,                      // 0.0 - 1.0
    pub y: f32,                      // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HslSettings {
    pub hue: ChannelValues,
    pub saturation: ChannelValues,
    pub luminance: ChannelValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelValues {
    pub red: i32,
    pub orange: i32,
    pub yellow: i32,
    pub green: i32,
    pub aqua: i32,
    pub blue: i32,
    pub purple: i32,
    pub magenta: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingSettings {
    pub shadows: ColorWheelSettings,
    pub midtones: ColorWheelSettings,
    pub highlights: ColorWheelSettings,
    pub global: ColorWheelSettings,
    pub blending: u32,               // 0-100
    pub balance: i32,                // -100 to +100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorWheelSettings {
    pub hue: u32,                    // 0-360
    pub saturation: u32,             // 0-100
    pub luminance: i32,              // -100 to +100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailSettings {
    pub sharpening: SharpeningSettings,
    pub noise_reduction: NoiseReductionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpeningSettings {
    pub amount: u32,                 // 0-150
    pub radius: f32,                 // 0.5-3.0
    pub detail: u32,                 // 0-100
    pub masking: u32,                // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseReductionSettings {
    pub luminance: u32,              // 0-100
    pub luminance_detail: u32,       // 0-100
    pub luminance_contrast: u32,     // 0-100
    pub color: u32,                  // 0-100
    pub color_detail: u32,           // 0-100
    pub color_smoothness: u32,       // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensCorrectionSettings {
    pub profile_enabled: bool,
    pub profile_name: Option<String>,
    pub distortion: f32,             // -100 to +100
    pub chromatic_aberration: bool,
    pub vignetting_amount: f32,      // -100 to +100
    pub vignetting_midpoint: u32,    // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformSettings {
    pub vertical: f32,               // -100 to +100
    pub horizontal: f32,             // -100 to +100
    pub rotate: f32,                 // -45 to +45 Grad
    pub aspect: f32,                 // -100 to +100
    pub scale: f32,                  // 50-150%
    pub offset_x: f32,              // -100 to +100
    pub offset_y: f32,              // -100 to +100
    pub auto_upright: UprightMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UprightMode { Off, Auto, Level, Vertical, Full, Guided }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsSettings {
    pub vignette: VignetteSettings,
    pub grain: GrainSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VignetteSettings {
    pub amount: i32,                 // -100 to +100
    pub midpoint: u32,               // 0-100
    pub roundness: i32,              // -100 to +100
    pub feather: u32,                // 0-100
    pub style: VignetteStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VignetteStyle { HighlightPriority, ColorPriority, PaintOverlay }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrainSettings {
    pub amount: u32,                 // 0-100
    pub size: u32,                   // 0-100
    pub roughness: u32,              // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropSettings {
    pub enabled: bool,
    pub top: f64,                    // 0.0-1.0
    pub left: f64,                   // 0.0-1.0
    pub bottom: f64,                 // 0.0-1.0
    pub right: f64,                  // 0.0-1.0
    pub angle: f64,                  // Grad
    pub aspect_ratio: Option<AspectRatio>,
    pub constrain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AspectRatio {
    Original, Square, R4x3, R3x2, R16x9, R5x4, R7x5,
    Custom(f64),                     // width/height
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSettings {
    pub shadows_tint: i32,           // -100 to +100
    pub red_hue: i32,
    pub red_saturation: i32,
    pub green_hue: i32,
    pub green_saturation: i32,
    pub blue_hue: i32,
    pub blue_saturation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonochromeSettings {
    pub enabled: bool,
    pub filter_type: MonochromeFilter,
    pub mix: ChannelValues,          // Kanal-Mixer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonochromeFilter { None, Red, Orange, Yellow, Green, Blue }
```

**Validierung:** Alle numerischen Felder haben definierte Ranges. Parser erzwingt Clamping.
**Versionierung:** `schema_version` wird bei jedem Struktur-Change inkrementiert. Migration-Funktionen konvertieren alte Versionen.

---

## 6. DevelopModuleState

```rust
/// UI-State des Develop-Moduls (nicht persistiert im Katalog).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopModuleState {
    pub active_photo_id: Option<Uuid>,
    pub active_tool: Option<DevelopTool>,
    pub open_panels: Vec<DevelopPanel>,
    pub solo_mode: bool,
    pub auto_sync: bool,
    pub show_clipping: bool,         // J-Taste
    pub before_after_mode: BeforeAfterMode,
    pub zoom_level: ZoomLevel,
    pub zoom_position: Option<(f64, f64)>,
    pub clipboard: Option<EditClipboard>,
    pub selected_mask_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopTool { None, Crop, AdjustmentBrush, GraduatedFilter, RadialFilter, Healing, WhiteBalancePicker, Straighten }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopPanel { Basic, ToneCurve, Hsl, ColorGrading, Detail, LensCorrections, Transform, Effects, Calibration, Presets, Snapshots, History }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeforeAfterMode { Off, Toggle, SideBySide, Split, TopBottom }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoomLevel { Fit, Fill, OneToOne, TwoToOne, Custom(f64) }
```

---

## 7. Mask & LocalAdjustment

```rust
/// Eine lokale Korrektur mit Maske.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAdjustment {
    pub id: Uuid,
    pub name: Option<String>,
    pub mask: Mask,
    pub settings: LocalSettings,
    pub enabled: bool,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mask {
    Brush(BrushMask),
    Gradient(GradientMask),
    Radial(RadialMask),
    Range(RangeMask),
    Ai(AiMask),
    Intersection(Vec<Mask>),         // Kombination
    Subtraction { base: Box<Mask>, subtract: Box<Mask> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushMask {
    pub strokes: Vec<BrushStroke>,
    pub auto_mask: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushStroke {
    pub points: Vec<BrushPoint>,
    pub size: f32,
    pub feather: f32,
    pub flow: f32,
    pub density: f32,
    pub erase: bool,                 // true = Radierer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushPoint {
    pub x: f64,                      // 0.0-1.0 (relativ zum Bild)
    pub y: f64,
    pub pressure: f32,               // 0.0-1.0 (Stifttablett)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientMask {
    pub start: (f64, f64),           // Startpunkt (relativ)
    pub end: (f64, f64),             // Endpunkt (relativ)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialMask {
    pub center: (f64, f64),
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation: f64,               // Grad
    pub feather: f32,
    pub invert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeMask {
    pub range_type: RangeType,
    pub range_min: f32,
    pub range_max: f32,
    pub smoothness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeType { Luminance, Color(RgbColor) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8, pub g: u8, pub b: u8,
    pub tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMask {
    pub mask_type: AiMaskType,
    pub refinement: Option<BrushMask>, // Manuelle Verfeinerung
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiMaskType { Subject, Sky, Background, Person(u32) }

/// Settings die auf eine lokale Korrektur angewendet werden können.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSettings {
    pub exposure: f32,
    pub contrast: i32,
    pub highlights: i32,
    pub shadows: i32,
    pub clarity: i32,
    pub dehaze: i32,
    pub saturation: i32,
    pub sharpness: i32,
    pub noise: i32,
    pub moire: i32,
    pub defringe: i32,
    pub temperature: Option<i32>,    // Relative WB-Verschiebung
    pub tint: Option<i32>,
    pub color: Option<RgbColor>,     // Farb-Overlay
}

/// Healing/Clone Spots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingSpot {
    pub id: Uuid,
    pub spot_type: SpotType,
    pub target: (f64, f64),          // Position (relativ)
    pub source: (f64, f64),          // Quellposition
    pub radius: f64,
    pub feather: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpotType { Heal, Clone }
```

---

## 8. Preset

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: Uuid,
    pub name: String,
    pub group: Option<String>,       // "Color", "B&W", "Creative"
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: u32,
    pub schema_version: u32,
    pub source: PresetSource,
    pub applied_modules: HashSet<EditModule>,
    pub settings: Partial<EditRecipe>,
    pub is_favorite: bool,
    pub preview_image: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub compatible_cameras: Option<Vec<String>>, // None = alle
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetSource {
    Builtin,                         // Mit App geliefert
    User,                            // Vom Nutzer erstellt
    Imported { original_format: String }, // Aus Lightroom etc.
    Community { marketplace_id: Option<String> },
}
```

---

## 9. Snapshot

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub name: String,
    pub edit_recipe: EditRecipe,
    pub created_at: DateTime<Utc>,
}
```

---

## 10. VirtualCopy

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualCopy {
    pub id: Uuid,
    pub original_asset_id: Uuid,
    pub copy_name: Option<String>,   // "Color Version", "B&W"
    pub copy_number: u32,            // 1, 2, 3...
    pub edit_recipe: EditRecipe,
    pub rating: Rating,
    pub color_label: ColorLabel,
    pub flag: Flag,
    pub created_at: DateTime<Utc>,
}
```

---

## 11. Collection & SmartCollectionRule

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub collection_type: CollectionType,
    pub parent_id: Option<Uuid>,     // Collection Set
    pub sort_order: u32,
    pub smart_rules: Option<SmartCollectionRules>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionType { Manual, Smart, QuickCollection, CollectionSet }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCollectionRules {
    pub match_type: MatchType,       // All (AND) oder Any (OR)
    pub rules: Vec<SmartRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType { All, Any }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRule {
    pub field: SmartRuleField,
    pub operator: SmartRuleOperator,
    pub value: SmartRuleValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartRuleField {
    Rating, Flag, ColorLabel, HasEdits,
    CameraMake, CameraModel, LensName,
    FocalLength, Aperture, Iso, ShutterSpeed,
    DateTaken, DateImported,
    Keyword, Title, Description, FileName, FilePath,
    FileType, FileSize,
    Width, Height,
    GpsLatitude, GpsLongitude, GpsCity, GpsCountry,
    Collection, Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartRuleOperator {
    Equals, NotEquals,
    GreaterThan, GreaterOrEqual,
    LessThan, LessOrEqual,
    Contains, NotContains,
    StartsWith, EndsWith,
    IsEmpty, IsNotEmpty,
    InRange,                         // value = (min, max)
    InLast,                          // value = Duration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartRuleValue {
    Integer(i64),
    Float(f64),
    String(String),
    Date(DateTime<Utc>),
    Duration(Duration),              // "last 7 days"
    Range(f64, f64),
    Bool(bool),
    Enum(String),
}
```

---

## 12. ExportRecipe

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecipe {
    pub id: Uuid,
    pub name: String,
    pub format: ExportFormat,
    pub quality: u32,                // 0-100 (JPEG/WebP)
    pub resize: Option<ResizeSettings>,
    pub output_sharpening: Option<OutputSharpening>,
    pub color_space: ExportColorSpace,
    pub metadata_mode: MetadataExportMode,
    pub watermark: Option<WatermarkSettings>,
    pub naming_template: String,     // "{date}_{original}_{seq}"
    pub destination: PathBuf,
    pub subfolder_template: Option<String>,
    pub post_actions: Vec<PostExportAction>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Jpeg, Png, Tiff8, Tiff16, WebP, Avif, Heif, Dng,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeSettings {
    pub mode: ResizeMode,
    pub value: u32,                  // Pixel oder Prozent
    pub dont_enlarge: bool,
    pub resolution: Option<u32>,     // PPI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResizeMode {
    LongEdge, ShortEdge, Width, Height, Megapixels, Percentage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSharpening {
    pub target: SharpenTarget,
    pub amount: SharpenAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharpenTarget { Screen, MattePaper, GlossyPaper }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharpenAmount { Low, Standard, High }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportColorSpace { SRGB, AdobeRGB, ProPhotoRGB, DisplayP3, Custom(String) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataExportMode {
    All, AllExceptCamera, CopyrightOnly, None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatermarkSettings {
    pub watermark_type: WatermarkType,
    pub text: Option<String>,
    pub image_path: Option<PathBuf>,
    pub position: WatermarkPosition,
    pub opacity: f32,
    pub size: f32,                   // Relativ zum Bild
    pub inset: f32,                  // Abstand vom Rand
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatermarkType { Text, Image }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatermarkPosition {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, Center, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostExportAction {
    OpenInFinder,
    RunScript(PathBuf),
    PluginAction { plugin_id: String, action: String },
}
```

---

## 13. ShortcutBinding

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub action: String,              // "develop.copy_settings"
    pub key: String,                 // "cmd+c"
    pub context: ShortcutContext,
    pub when: Option<String>,        // Optionale Bedingung
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShortcutContext {
    Global, Library, Develop, Map, Print, Export, Dialog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapProfile {
    pub name: String,
    pub version: String,
    pub description: String,
    pub platform: Platform,
    pub bindings: Vec<ShortcutBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform { All, MacOS, Windows, Linux }
```

---

## 14. PluginManifest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,                  // "com.example.my-plugin"
    pub name: String,
    pub version: String,             // SemVer
    pub api_version: String,         // OCPS Plugin API Version
    pub plugin_type: PluginType,
    pub author: String,
    pub license: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub entry_point: String,         // "plugin.wasm"
    pub permissions: PluginPermissions,
    pub ui: Option<PluginUi>,
    pub dependencies: Vec<PluginDependency>,
    pub min_ocps_version: Option<String>,
    pub max_ocps_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    ImageFilter, ImportExport, Metadata, UiPanel,
    Catalog, Integration, AiMl, Tethering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub read_image: bool,
    pub write_image: bool,
    pub read_catalog: bool,
    pub write_catalog: bool,
    pub read_metadata: bool,
    pub write_metadata: bool,
    pub network: bool,
    pub filesystem: bool,
    pub gpu: bool,
    pub notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUi {
    pub panel_title: String,
    pub panel_location: PanelLocation,
    pub html_entry: String,          // "ui/panel.html"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelLocation { RightSidebar, LeftSidebar, BottomBar, Dialog }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version_range: String,       // SemVer range
}
```

---

## 15. LicenseState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseState {
    pub license_type: LicenseType,
    pub license_key: Option<String>,
    pub licensed_to: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub features: HashSet<String>,
    pub seat_count: Option<u32>,
    pub last_validated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseType {
    Community,                       // PolyForm Noncommercial
    IndieCommercial,
    Enterprise,
    Oem,
    Saas,
    Education,
    Trial { expires: DateTime<Utc> },
}
```

---

## 16. CompatibilityReport

```rust
/// Bericht über die Kompatibilität eines importierten Katalogs/Sidecars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub source: CompatibilitySource,
    pub total_items: u32,
    pub imported_successfully: u32,
    pub imported_partially: u32,
    pub import_failed: u32,
    pub warnings: Vec<CompatibilityWarning>,
    pub unsupported_features: Vec<String>,
    pub level: CompatibilityLevel,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilitySource {
    LightroomCatalog { version: String, path: PathBuf },
    XmpSidecar { path: PathBuf },
    LightroomPreset { path: PathBuf },
    DarktableXmp { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityWarning {
    pub asset_id: Option<Uuid>,
    pub field: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity { Info, Warning, Error }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    Level1_MetadataOnly,             // Nur Metadaten, keine Develop-Settings
    Level2_PartialDevelop,           // Basis-Settings importiert
    Level3_ExperimentalDevelop,      // Erweiterte Settings, best-effort
    Level4_Roundtrip,                // Volle Roundtrip-Kompatibilität
}
```

---

## Migration Strategy

### Schema-Versionierung
- Jedes Modell hat ein `schema_version` Feld
- Beim Laden: Prüfe Version, führe Migration aus wenn nötig
- Migrationen sind Funktionen: `fn migrate_v1_to_v2(old: serde_json::Value) -> Result<serde_json::Value>`
- Migrationen sind vorwärts-only (kein Downgrade)
- Migrations-Kette: v1 → v2 → v3 → ... → current

### DB-Migrationen (SQLite)
- `migrations/` Ordner mit nummerierten SQL-Dateien
- Format: `0001_initial.sql`, `0002_add_stacks.sql` etc.
- Migration-Tracking in `_migrations` Tabelle
- Migrations laufen beim App-Start (vor UI)
- Backup vor jeder Migration (automatisch)

### JSON-Migrationen (Presets, Keymaps)
- Eingebettete Migration-Logik in den Parsern
- Unbekannte Felder werden ignoriert (forward-compatible)
- Fehlende Felder bekommen Defaults (backward-compatible)
