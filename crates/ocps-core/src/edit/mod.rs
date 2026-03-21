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

/// Entry in the edit history stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHistoryEntry {
    pub recipe: EditRecipe,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

/// Edit history for undo/redo functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditHistory {
    pub entries: Vec<EditHistoryEntry>,
    pub current_index: usize,
    pub max_entries: usize,
}

impl EditHistory {
    /// Create a new edit history with an initial recipe
    pub fn new(initial: EditRecipe) -> Self {
        Self {
            entries: vec![EditHistoryEntry {
                recipe: initial,
                description: "Initial state".to_string(),
                timestamp: Utc::now(),
            }],
            current_index: 0,
            max_entries: 50,
        }
    }

    /// Push a new recipe to history
    /// Truncates any redo entries (entries after current_index)
    /// If at max_entries, removes oldest entry
    pub fn push(&mut self, recipe: EditRecipe, description: String) {
        // Truncate redo entries (anything after current_index)
        self.entries.truncate(self.current_index + 1);

        // Add new entry
        self.entries.push(EditHistoryEntry {
            recipe,
            description,
            timestamp: Utc::now(),
        });

        // Move current index to the new entry
        self.current_index = self.entries.len() - 1;

        // If we exceeded max entries, remove the oldest
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
            self.current_index = self.entries.len() - 1;
        }
    }

    /// Undo: move back in history and return the previous recipe
    pub fn undo(&mut self) -> Option<&EditRecipe> {
        if self.can_undo() {
            self.current_index -= 1;
            Some(&self.entries[self.current_index].recipe)
        } else {
            None
        }
    }

    /// Redo: move forward in history and return the next recipe
    pub fn redo(&mut self) -> Option<&EditRecipe> {
        if self.can_redo() {
            self.current_index += 1;
            Some(&self.entries[self.current_index].recipe)
        } else {
            None
        }
    }

    /// Check if we can undo (not at the beginning)
    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    /// Check if we can redo (not at the end)
    pub fn can_redo(&self) -> bool {
        self.current_index < self.entries.len() - 1
    }

    /// Get the current recipe
    pub fn current(&self) -> &EditRecipe {
        &self.entries[self.current_index].recipe
    }

    /// Get entries for display with (description, is_current) pairs
    pub fn entries_for_display(&self) -> Vec<(String, bool)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (entry.description.clone(), i == self.current_index))
            .collect()
    }

    /// Generate description by comparing old and new recipes
    pub fn auto_describe(old: &EditRecipe, new: &EditRecipe) -> String {
        let mut changes = Vec::new();

        if old.exposure != new.exposure {
            changes.push(format!(
                "Exposure ({:.1} → {:.1})",
                old.exposure, new.exposure
            ));
        }

        if old.contrast != new.contrast {
            changes.push(format!(
                "Contrast ({} → {})",
                old.contrast, new.contrast
            ));
        }

        if old.highlights != new.highlights {
            changes.push(format!(
                "Highlights ({} → {})",
                old.highlights, new.highlights
            ));
        }

        if old.shadows != new.shadows {
            changes.push(format!("Shadows ({} → {})", old.shadows, new.shadows));
        }

        if old.whites != new.whites {
            changes.push(format!("Whites ({} → {})", old.whites, new.whites));
        }

        if old.blacks != new.blacks {
            changes.push(format!("Blacks ({} → {})", old.blacks, new.blacks));
        }

        if old.clarity != new.clarity {
            changes.push(format!("Clarity ({} → {})", old.clarity, new.clarity));
        }

        if old.vibrance != new.vibrance {
            changes.push(format!("Vibrance ({} → {})", old.vibrance, new.vibrance));
        }

        if old.saturation != new.saturation {
            changes.push(format!(
                "Saturation ({} → {})",
                old.saturation, new.saturation
            ));
        }

        if old.white_balance.temperature != new.white_balance.temperature {
            changes.push(format!(
                "WB Temp ({} → {})",
                old.white_balance.temperature, new.white_balance.temperature
            ));
        }

        if changes.is_empty() {
            "Edit".to_string()
        } else if changes.len() == 1 {
            changes[0].clone()
        } else {
            format!("{} changes", changes.len())
        }
    }
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

    // ===== EDIT HISTORY TESTS =====

    #[test]
    fn test_history_push_and_undo() {
        let recipe0 = EditRecipe::default();
        let mut history = EditHistory::new(recipe0.clone());

        // Push 3 more states
        let mut recipe1 = recipe0.clone();
        recipe1.exposure = 1.0;
        history.push(recipe1.clone(), "Exposure +1.0".to_string());

        let mut recipe2 = recipe1.clone();
        recipe2.contrast = 20;
        history.push(recipe2.clone(), "Contrast +20".to_string());

        let mut recipe3 = recipe2.clone();
        recipe3.shadows = 30;
        history.push(recipe3.clone(), "Shadows +30".to_string());

        // Should be at state 3
        assert_eq!(history.current_index, 3);
        assert_eq!(history.current().shadows, 30);

        // Undo once -> should be at state 2
        let prev = history.undo().unwrap();
        assert_eq!(prev.shadows, 0);
        assert_eq!(prev.contrast, 20);
        assert_eq!(history.current_index, 2);

        // Undo again -> should be at state 1
        let prev = history.undo().unwrap();
        assert_eq!(prev.exposure, 1.0);
        assert_eq!(prev.contrast, 0);
        assert_eq!(history.current_index, 1);

        // Undo again -> should be at state 0
        let prev = history.undo().unwrap();
        assert_eq!(prev.exposure, 0.0);
        assert_eq!(history.current_index, 0);

        // Can't undo anymore
        assert!(!history.can_undo());
        assert!(history.undo().is_none());
    }

    #[test]
    fn test_history_undo_redo() {
        let recipe0 = EditRecipe::default();
        let mut history = EditHistory::new(recipe0.clone());

        let mut recipe1 = recipe0.clone();
        recipe1.exposure = 1.0;
        history.push(recipe1.clone(), "State 1".to_string());

        let mut recipe2 = recipe1.clone();
        recipe2.exposure = 2.0;
        history.push(recipe2.clone(), "State 2".to_string());

        let mut recipe3 = recipe2.clone();
        recipe3.exposure = 3.0;
        history.push(recipe3.clone(), "State 3".to_string());

        // At index 3
        assert_eq!(history.current_index, 3);

        // Undo twice
        history.undo();
        history.undo();
        assert_eq!(history.current_index, 1);

        // Redo once
        let next = history.redo().unwrap();
        assert_eq!(next.exposure, 2.0);
        assert_eq!(history.current_index, 2);

        // Verify we can redo one more time
        assert!(history.can_redo());
        let next = history.redo().unwrap();
        assert_eq!(next.exposure, 3.0);
        assert_eq!(history.current_index, 3);

        // Can't redo anymore
        assert!(!history.can_redo());
        assert!(history.redo().is_none());
    }

    #[test]
    fn test_history_max_entries() {
        let mut recipe = EditRecipe::default();
        let mut history = EditHistory::new(recipe.clone());

        // Push 55 entries (initial + 55 = 56 total, but max is 50)
        for i in 1..=55 {
            recipe.exposure = i as f32;
            history.push(recipe.clone(), format!("State {}", i));
        }

        // Should only have 50 entries
        assert_eq!(history.entries.len(), 50);

        // Current index should be at the last entry
        assert_eq!(history.current_index, 49);

        // Oldest entries should have been removed
        // The first entry should now be state 6 (initial 0 + states 1-5 were removed)
        assert_eq!(history.entries[0].recipe.exposure, 6.0);
    }

    #[test]
    fn test_history_redo_cleared_on_new_push() {
        let recipe0 = EditRecipe::default();
        let mut history = EditHistory::new(recipe0.clone());

        let mut recipe1 = recipe0.clone();
        recipe1.exposure = 1.0;
        history.push(recipe1.clone(), "State 1".to_string());

        let mut recipe2 = recipe1.clone();
        recipe2.exposure = 2.0;
        history.push(recipe2.clone(), "State 2".to_string());

        let mut recipe3 = recipe2.clone();
        recipe3.exposure = 3.0;
        history.push(recipe3.clone(), "State 3".to_string());

        // Undo twice -> at state 1
        history.undo();
        history.undo();
        assert_eq!(history.current_index, 1);

        // Push a new state -> should clear states 2 and 3
        let mut recipe_new = recipe1.clone();
        recipe_new.contrast = 50;
        history.push(recipe_new.clone(), "New branch".to_string());

        // Should have only 3 entries now (0, 1, new)
        assert_eq!(history.entries.len(), 3);
        assert_eq!(history.current_index, 2);

        // Can't redo because we branched
        assert!(!history.can_redo());

        // Current state should have contrast 50
        assert_eq!(history.current().contrast, 50);
    }

    #[test]
    fn test_history_auto_describe() {
        let old = EditRecipe::default();
        let mut new = old.clone();

        // Single change
        new.exposure = 1.5;
        let desc = EditHistory::auto_describe(&old, &new);
        assert!(desc.contains("Exposure"));
        assert!(desc.contains("0.0") || desc.contains("1.5"));

        // Multiple changes
        new.contrast = 20;
        new.shadows = 30;
        let desc = EditHistory::auto_describe(&old, &new);
        assert!(desc.contains("changes"));

        // No changes
        let desc = EditHistory::auto_describe(&old, &old);
        assert_eq!(desc, "Edit");
    }

    #[test]
    fn test_history_1000_pushes_respects_max_entries() {
        // Edge case: ensure history doesn't grow beyond max_entries even after 1000 pushes
        let mut recipe = EditRecipe::default();
        let mut history = EditHistory::new(recipe.clone());

        // Push 1000 entries
        for i in 1..=1000 {
            recipe.exposure = i as f32;
            history.push(recipe.clone(), format!("State {}", i));
        }

        // Should still only have max_entries (50)
        assert_eq!(history.entries.len(), 50);
        assert_eq!(history.current_index, 49);

        // The oldest entries should have been removed
        // First entry should be state 951 (1000 - 50 + 1)
        assert_eq!(history.entries[0].recipe.exposure, 951.0);
        // Last entry should be state 1000
        assert_eq!(history.entries[49].recipe.exposure, 1000.0);
    }

    #[test]
    fn test_paste_on_empty_clipboard() {
        // Edge case: pasting when clipboard is conceptually empty
        // Since EditClipboard always has a recipe, we'll test with default recipe
        let clipboard = EditClipboard {
            source_photo_id: String::new(),
            recipe: EditRecipe::default(),
            modules: vec![], // Empty modules list
            copied_at: Utc::now(),
        };

        let mut target = EditRecipe::default();
        target.exposure = 2.0;

        // Paste with no modules should not change anything
        EditCopyPaste::paste(&clipboard, &mut target);

        // Target should remain unchanged
        assert_eq!(target.exposure, 2.0);
    }

    #[test]
    fn test_paste_selected_with_empty_modules() {
        // Test paste_selected with empty module list
        let mut source = EditRecipe::default();
        source.exposure = 1.5;
        source.contrast = 50;

        let clipboard = EditCopyPaste::copy_all("source", &source);

        let mut target = EditRecipe::default();
        target.exposure = 0.5;
        target.contrast = 10;

        // Paste with empty module list
        EditCopyPaste::paste_selected(&clipboard, &mut target, &[]);

        // Nothing should change
        assert_eq!(target.exposure, 0.5);
        assert_eq!(target.contrast, 10);
    }
}
