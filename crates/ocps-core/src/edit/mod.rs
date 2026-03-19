//! Edit copy/paste system for applying adjustments across photos

use crate::pipeline::types::EditRecipe;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Edit modules that can be copied and pasted individually
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditModule {
    WhiteBalance,
    Exposure,
    Contrast,
    Highlights,
    Shadows,
    Whites,
    Blacks,
    Clarity,
    Dehaze,
    Vibrance,
    Saturation,
    Sharpening,
    NoiseReduction,
    Crop,
    ColorGrading,
}

impl EditModule {
    /// Get all available edit modules
    pub fn all() -> Vec<EditModule> {
        vec![
            EditModule::WhiteBalance,
            EditModule::Exposure,
            EditModule::Contrast,
            EditModule::Highlights,
            EditModule::Shadows,
            EditModule::Whites,
            EditModule::Blacks,
            EditModule::Clarity,
            EditModule::Dehaze,
            EditModule::Vibrance,
            EditModule::Saturation,
            EditModule::Sharpening,
            EditModule::NoiseReduction,
            EditModule::Crop,
            EditModule::ColorGrading,
        ]
    }

    /// Get safe default modules (excludes image-specific adjustments like crop)
    pub fn safe_defaults() -> Vec<EditModule> {
        Self::all()
            .into_iter()
            .filter(|m| *m != EditModule::Crop)
            .collect()
    }
}

/// Edit clipboard containing copied settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditClipboard {
    /// Source photo ID that was copied from
    pub source_photo_id: String,
    /// The copied recipe
    pub recipe: EditRecipe,
    /// Which modules are included in the clipboard
    pub modules: Vec<EditModule>,
    /// Timestamp when copied
    pub copied_at: DateTime<Utc>,
}

/// Copy/paste operations for edit settings
pub struct EditCopyPaste;

impl EditCopyPaste {
    /// Copy all edit settings from a photo
    pub fn copy_all(photo_id: &str, recipe: &EditRecipe) -> EditClipboard {
        EditClipboard {
            source_photo_id: photo_id.to_string(),
            recipe: recipe.clone(),
            modules: EditModule::all(),
            copied_at: Utc::now(),
        }
    }

    /// Copy selected edit modules from a photo
    pub fn copy_selected(
        photo_id: &str,
        recipe: &EditRecipe,
        modules: Vec<EditModule>,
    ) -> EditClipboard {
        EditClipboard {
            source_photo_id: photo_id.to_string(),
            recipe: recipe.clone(),
            modules,
            copied_at: Utc::now(),
        }
    }

    /// Paste clipboard settings to a target recipe
    /// Applies all modules present in the clipboard
    pub fn paste(clipboard: &EditClipboard, target: &mut EditRecipe) {
        Self::paste_selected(clipboard, target, &clipboard.modules);
    }

    /// Paste only selected modules from clipboard to target recipe
    pub fn paste_selected(
        clipboard: &EditClipboard,
        target: &mut EditRecipe,
        modules: &[EditModule],
    ) {
        for module in modules {
            match module {
                EditModule::WhiteBalance => {
                    target.white_balance = clipboard.recipe.white_balance.clone();
                }
                EditModule::Exposure => {
                    target.exposure = clipboard.recipe.exposure;
                }
                EditModule::Contrast => {
                    target.contrast = clipboard.recipe.contrast;
                }
                EditModule::Highlights => {
                    target.highlights = clipboard.recipe.highlights;
                }
                EditModule::Shadows => {
                    target.shadows = clipboard.recipe.shadows;
                }
                EditModule::Whites => {
                    target.whites = clipboard.recipe.whites;
                }
                EditModule::Blacks => {
                    target.blacks = clipboard.recipe.blacks;
                }
                EditModule::Clarity => {
                    target.clarity = clipboard.recipe.clarity;
                }
                EditModule::Dehaze => {
                    target.dehaze = clipboard.recipe.dehaze;
                }
                EditModule::Vibrance => {
                    target.vibrance = clipboard.recipe.vibrance;
                }
                EditModule::Saturation => {
                    target.saturation = clipboard.recipe.saturation;
                }
                EditModule::Sharpening => {
                    target.sharpening = clipboard.recipe.sharpening.clone();
                }
                EditModule::NoiseReduction => {
                    target.noise_reduction = clipboard.recipe.noise_reduction.clone();
                }
                EditModule::Crop => {
                    target.crop = clipboard.recipe.crop.clone();
                }
                EditModule::ColorGrading => {
                    target.color_grading = clipboard.recipe.color_grading.clone();
                }
            }
        }
    }

    /// Sync settings from source to multiple target recipes
    /// Applies specified modules to all targets
    pub fn sync_to_many(
        source: &EditRecipe,
        targets: &mut [EditRecipe],
        modules: &[EditModule],
    ) {
        // Create a temporary clipboard
        let clipboard = EditClipboard {
            source_photo_id: String::new(), // Not used for sync
            recipe: source.clone(),
            modules: modules.to_vec(),
            copied_at: Utc::now(),
        };

        // Apply to all targets
        for target in targets {
            Self::paste_selected(&clipboard, target, modules);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::WhiteBalance;

    #[test]
    fn test_copy_all_captures_exposure() {
        let mut recipe = EditRecipe::default();
        recipe.exposure = 1.5;

        let clipboard = EditCopyPaste::copy_all("photo123", &recipe);

        assert_eq!(clipboard.source_photo_id, "photo123");
        assert_eq!(clipboard.recipe.exposure, 1.5);
        assert_eq!(clipboard.modules.len(), EditModule::all().len());
    }

    #[test]
    fn test_paste_overwrites_exposure() {
        let mut source = EditRecipe::default();
        source.exposure = 2.0;

        let clipboard = EditCopyPaste::copy_all("source", &source);

        let mut target = EditRecipe::default();
        target.exposure = 0.5;

        EditCopyPaste::paste(&clipboard, &mut target);

        assert_eq!(target.exposure, 2.0);
    }

    #[test]
    fn test_paste_selected_skips_unselected() {
        let mut source = EditRecipe::default();
        source.exposure = 2.0;
        source.contrast = 50;

        let clipboard =
            EditCopyPaste::copy_selected("source", &source, vec![EditModule::Exposure]);

        let mut target = EditRecipe::default();
        target.exposure = 0.0;
        target.contrast = 10;

        // Paste only exposure
        EditCopyPaste::paste(&clipboard, &mut target);

        // Exposure should be updated
        assert_eq!(target.exposure, 2.0);
        // Contrast should remain unchanged
        assert_eq!(target.contrast, 10);
    }

    #[test]
    fn test_sync_to_many_updates_all() {
        let mut source = EditRecipe::default();
        source.exposure = 1.5;
        source.contrast = 30;

        let target1 = EditRecipe::default();
        let target2 = EditRecipe::default();
        let target3 = EditRecipe::default();

        let mut targets = vec![target1, target2, target3];

        EditCopyPaste::sync_to_many(
            &source,
            &mut targets,
            &[EditModule::Exposure, EditModule::Contrast],
        );

        // All targets should have the same exposure and contrast
        for target in &targets {
            assert_eq!(target.exposure, 1.5);
            assert_eq!(target.contrast, 30);
        }
    }

    #[test]
    fn test_safe_defaults_excludes_crop() {
        let safe = EditModule::safe_defaults();

        assert!(!safe.contains(&EditModule::Crop));
        assert!(safe.contains(&EditModule::Exposure));
        assert!(safe.contains(&EditModule::WhiteBalance));
    }

    #[test]
    fn test_copy_paste_roundtrip() {
        let mut original = EditRecipe::default();
        original.exposure = 1.0;
        original.contrast = 25;
        original.highlights = -50;
        original.shadows = 40;
        original.white_balance = WhiteBalance {
            temperature: 6500,
            tint: 10,
        };

        // Copy all
        let clipboard = EditCopyPaste::copy_all("test", &original);

        // Paste to new recipe
        let mut result = EditRecipe::default();
        EditCopyPaste::paste(&clipboard, &mut result);

        // Should match original
        assert_eq!(result.exposure, original.exposure);
        assert_eq!(result.contrast, original.contrast);
        assert_eq!(result.highlights, original.highlights);
        assert_eq!(result.shadows, original.shadows);
        assert_eq!(
            result.white_balance.temperature,
            original.white_balance.temperature
        );
        assert_eq!(result.white_balance.tint, original.white_balance.tint);
    }
}
