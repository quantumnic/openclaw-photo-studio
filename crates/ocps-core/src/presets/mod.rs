//! Preset system for saving and loading edit recipes
//!
//! Presets allow users to save and reuse editing configurations. The system includes
//! builtin presets and supports user-created presets stored in a user directory.

use crate::pipeline::EditRecipe;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A preset containing an edit recipe and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub group: String, // "Color", "B&W", "Creative", "User"
    pub description: Option<String>,
    pub recipe: EditRecipe,
    pub applied_modules: Vec<String>,
    pub is_builtin: bool,
}

/// Preset library manager
pub struct PresetLibrary {
    presets: Vec<Preset>,
    user_dir: PathBuf,
}

impl PresetLibrary {
    /// Create a new preset library with the given user directory
    pub fn new(user_dir: PathBuf) -> Self {
        let mut library = Self {
            presets: Vec::new(),
            user_dir,
        };

        // Load builtin presets
        library.presets.extend(Self::load_builtin());

        library
    }

    /// Load builtin presets
    pub fn load_builtin() -> Vec<Preset> {
        vec![
            // 1. Warm Tone
            Preset {
                id: "builtin_warm_tone".to_string(),
                name: "Warm Tone".to_string(),
                group: "Color".to_string(),
                description: Some("Adds warmth with increased temperature, vibrance, and lifted shadows".to_string()),
                recipe: {
                    let mut recipe = EditRecipe::default();
                    recipe.white_balance.temperature = 6000; // +500 from default 5500
                    recipe.vibrance = 20;
                    recipe.shadows = 10;
                    recipe
                },
                applied_modules: vec!["white_balance".to_string(), "vibrance".to_string(), "shadows".to_string()],
                is_builtin: true,
            },
            // 2. Cool Mist
            Preset {
                id: "builtin_cool_mist".to_string(),
                name: "Cool Mist".to_string(),
                group: "Color".to_string(),
                description: Some("Creates a cool, ethereal look with reduced temperature and clarity".to_string()),
                recipe: {
                    let mut recipe = EditRecipe::default();
                    recipe.white_balance.temperature = 5000; // -500 from default
                    recipe.highlights = -20;
                    recipe.clarity = 10;
                    recipe
                },
                applied_modules: vec!["white_balance".to_string(), "highlights".to_string(), "clarity".to_string()],
                is_builtin: true,
            },
            // 3. High Contrast
            Preset {
                id: "builtin_high_contrast".to_string(),
                name: "High Contrast".to_string(),
                group: "Creative".to_string(),
                description: Some("Bold, punchy look with increased contrast and clarity".to_string()),
                recipe: EditRecipe {
                    contrast: 40,
                    clarity: 20,
                    blacks: -20,
                    ..Default::default()
                },
                applied_modules: vec!["contrast".to_string(), "clarity".to_string(), "blacks".to_string()],
                is_builtin: true,
            },
            // 4. Soft Portrait
            Preset {
                id: "builtin_soft_portrait".to_string(),
                name: "Soft Portrait".to_string(),
                group: "Color".to_string(),
                description: Some("Flattering portrait look with reduced clarity and boosted skin tones".to_string()),
                recipe: EditRecipe {
                    clarity: -20,
                    shadows: 20,
                    highlights: -10,
                    vibrance: 15,
                    ..Default::default()
                },
                applied_modules: vec!["clarity".to_string(), "shadows".to_string(), "highlights".to_string(), "vibrance".to_string()],
                is_builtin: true,
            },
            // 5. B&W Classic
            Preset {
                id: "builtin_bw_classic".to_string(),
                name: "B&W Classic".to_string(),
                group: "B&W".to_string(),
                description: Some("Classic black and white with enhanced contrast".to_string()),
                recipe: EditRecipe {
                    saturation: -100,
                    contrast: 20,
                    clarity: 10,
                    ..Default::default()
                },
                applied_modules: vec!["saturation".to_string(), "contrast".to_string(), "clarity".to_string()],
                is_builtin: true,
            },
            // 6. Faded Film
            Preset {
                id: "builtin_faded_film".to_string(),
                name: "Faded Film".to_string(),
                group: "Creative".to_string(),
                description: Some("Vintage film look with lifted blacks and muted colors".to_string()),
                recipe: EditRecipe {
                    blacks: 30,
                    highlights: -20,
                    saturation: -30,
                    contrast: -10,
                    ..Default::default()
                },
                applied_modules: vec!["blacks".to_string(), "highlights".to_string(), "saturation".to_string(), "contrast".to_string()],
                is_builtin: true,
            },
        ]
    }

    /// Load user presets from the user directory
    pub fn load_user_presets(&mut self) -> Result<usize, std::io::Error> {
        if !self.user_dir.exists() {
            fs::create_dir_all(&self.user_dir)?;
            return Ok(0);
        }

        let mut count = 0;
        for entry in fs::read_dir(&self.user_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(preset) = serde_json::from_str::<Preset>(&contents) {
                        self.presets.push(preset);
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Save a preset to the user directory
    pub fn save_preset(&self, preset: &Preset) -> Result<(), std::io::Error> {
        if !self.user_dir.exists() {
            fs::create_dir_all(&self.user_dir)?;
        }

        let filename = format!("{}.json", preset.id);
        let path = self.user_dir.join(filename);
        let json = serde_json::to_string_pretty(preset)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(path, json)?;
        Ok(())
    }

    /// Delete a preset from the user directory
    pub fn delete_preset(&self, id: &str) -> Result<(), std::io::Error> {
        let filename = format!("{}.json", id);
        let path = self.user_dir.join(filename);

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Get all presets
    pub fn all(&self) -> Vec<&Preset> {
        self.presets.iter().collect()
    }

    /// Get presets by group
    pub fn by_group(&self, group: &str) -> Vec<&Preset> {
        self.presets
            .iter()
            .filter(|p| p.group == group)
            .collect()
    }

    /// Apply a preset to a target recipe
    ///
    /// Returns a new recipe with the preset's values applied
    pub fn apply(preset: &Preset, target: &EditRecipe) -> EditRecipe {
        let mut result = target.clone();

        // Apply only the modules specified in the preset
        for module in &preset.applied_modules {
            match module.as_str() {
                "white_balance" => {
                    result.white_balance = preset.recipe.white_balance.clone();
                }
                "exposure" => {
                    result.exposure = preset.recipe.exposure;
                }
                "contrast" => {
                    result.contrast = preset.recipe.contrast;
                }
                "highlights" => {
                    result.highlights = preset.recipe.highlights;
                }
                "shadows" => {
                    result.shadows = preset.recipe.shadows;
                }
                "whites" => {
                    result.whites = preset.recipe.whites;
                }
                "blacks" => {
                    result.blacks = preset.recipe.blacks;
                }
                "clarity" => {
                    result.clarity = preset.recipe.clarity;
                }
                "dehaze" => {
                    result.dehaze = preset.recipe.dehaze;
                }
                "vibrance" => {
                    result.vibrance = preset.recipe.vibrance;
                }
                "saturation" => {
                    result.saturation = preset.recipe.saturation;
                }
                _ => {} // Unknown module, skip
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_builtin_presets_load() {
        let presets = PresetLibrary::load_builtin();
        assert!(presets.len() >= 6, "Should have at least 6 builtin presets");

        // Check that all presets have required fields
        for preset in &presets {
            assert!(!preset.id.is_empty());
            assert!(!preset.name.is_empty());
            assert!(!preset.group.is_empty());
            assert!(preset.is_builtin);
        }
    }

    #[test]
    fn test_preset_apply_warm_tone() {
        let presets = PresetLibrary::load_builtin();
        let warm_tone = presets.iter().find(|p| p.id == "builtin_warm_tone").unwrap();

        let base_recipe = EditRecipe::default();
        let applied = PresetLibrary::apply(warm_tone, &base_recipe);

        // Temperature should increase
        assert!(applied.white_balance.temperature > base_recipe.white_balance.temperature);
        // Vibrance should be set
        assert_eq!(applied.vibrance, 20);
        // Shadows should be lifted
        assert_eq!(applied.shadows, 10);
    }

    #[test]
    fn test_preset_apply_bw() {
        let presets = PresetLibrary::load_builtin();
        let bw = presets.iter().find(|p| p.id == "builtin_bw_classic").unwrap();

        let base_recipe = EditRecipe::default();
        let applied = PresetLibrary::apply(bw, &base_recipe);

        // Saturation should be -100 (full desaturation)
        assert_eq!(applied.saturation, -100);
        // Contrast should be increased
        assert_eq!(applied.contrast, 20);
    }

    #[test]
    fn test_preset_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let library = PresetLibrary::new(temp_dir.path().to_path_buf());

        // Create a custom preset
        let preset = Preset {
            id: "test_preset".to_string(),
            name: "Test Preset".to_string(),
            group: "User".to_string(),
            description: Some("Test description".to_string()),
            recipe: EditRecipe::default(),
            applied_modules: vec!["exposure".to_string()],
            is_builtin: false,
        };

        // Save it
        library.save_preset(&preset).unwrap();

        // Load it back
        let mut library2 = PresetLibrary::new(temp_dir.path().to_path_buf());
        let loaded_count = library2.load_user_presets().unwrap();

        assert_eq!(loaded_count, 1);
        let all_presets = library2.all();
        let loaded_preset = all_presets
            .iter()
            .find(|p| p.id == "test_preset")
            .unwrap();
        assert_eq!(loaded_preset.name, "Test Preset");
    }

    #[test]
    fn test_preset_delete() {
        let temp_dir = TempDir::new().unwrap();
        let library = PresetLibrary::new(temp_dir.path().to_path_buf());

        let preset = Preset {
            id: "delete_test".to_string(),
            name: "Delete Test".to_string(),
            group: "User".to_string(),
            description: None,
            recipe: EditRecipe::default(),
            applied_modules: vec![],
            is_builtin: false,
        };

        // Save and then delete
        library.save_preset(&preset).unwrap();
        library.delete_preset("delete_test").unwrap();

        // Verify it's gone
        let path = temp_dir.path().join("delete_test.json");
        assert!(!path.exists());
    }

    #[test]
    fn test_preset_by_group() {
        let library = PresetLibrary::new(PathBuf::from("/tmp"));
        let color_presets = library.by_group("Color");
        let bw_presets = library.by_group("B&W");
        let creative_presets = library.by_group("Creative");

        assert!(color_presets.len() >= 2); // Warm Tone, Cool Mist, Soft Portrait
        assert!(bw_presets.len() >= 1);    // B&W Classic
        assert!(creative_presets.len() >= 2); // High Contrast, Faded Film
    }

    #[test]
    fn test_preset_apply_partial() {
        // Test that applying a preset only changes specified modules
        let presets = PresetLibrary::load_builtin();
        let preset = presets.iter().find(|p| p.id == "builtin_warm_tone").unwrap();

        let mut base_recipe = EditRecipe::default();
        base_recipe.contrast = 50; // Set contrast manually
        base_recipe.exposure = 2.0; // Set exposure manually

        let applied = PresetLibrary::apply(preset, &base_recipe);

        // Modules in preset should be changed
        assert_eq!(applied.vibrance, 20);

        // Modules NOT in preset should be preserved
        assert_eq!(applied.contrast, 50);
        assert_eq!(applied.exposure, 2.0);
    }
}
