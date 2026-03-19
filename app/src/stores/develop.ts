/**
 * Develop module shared state
 *
 * This store manages the current photo selection and edit recipe,
 * allowing DevelopView and RightSidebar to share state.
 */

import { createStore } from "solid-js/store";
import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

export interface EditRecipe {
  white_balance: {
    temperature: number;
    tint: number;
  };
  exposure: number;
  contrast: number;
  highlights: number;
  shadows: number;
  whites: number;
  blacks: number;
  clarity: number;
  dehaze: number;
  vibrance: number;
  saturation: number;
  sharpening: {
    amount: number;
    radius: number;
  };
  noise_reduction: {
    luminance: number;
    color: number;
  };
  crop: {
    left: number;
    top: number;
    right: number;
    bottom: number;
    angle: number;
  };
  color_grading_new: {
    enabled: boolean;
    shadows: { r: number; g: number; b: number };
    midtones: { r: number; g: number; b: number };
    highlights: { r: number; g: number; b: number };
  };
  hsl: {
    red: { hue: number; saturation: number; luminance: number };
    orange: { hue: number; saturation: number; luminance: number };
    yellow: { hue: number; saturation: number; luminance: number };
    green: { hue: number; saturation: number; luminance: number };
    aqua: { hue: number; saturation: number; luminance: number };
    blue: { hue: number; saturation: number; luminance: number };
    purple: { hue: number; saturation: number; luminance: number };
    magenta: { hue: number; saturation: number; luminance: number };
  };
  tone_curve_rgb: {
    points: Array<{ x: number; y: number }>;
  };
  lens_corrections: {
    distortion: number;
    vignetting: number;
  };
}

// Default recipe (identity adjustments)
function defaultRecipe(): EditRecipe {
  return {
    white_balance: {
      temperature: 5500,
      tint: 0,
    },
    exposure: 0.0,
    contrast: 0,
    highlights: 0,
    shadows: 0,
    whites: 0,
    blacks: 0,
    clarity: 0,
    dehaze: 0,
    vibrance: 0,
    saturation: 0,
    sharpening: {
      amount: 0,
      radius: 1.0,
    },
    noise_reduction: {
      luminance: 0,
      color: 0,
    },
    crop: {
      left: 0.0,
      top: 0.0,
      right: 1.0,
      bottom: 1.0,
      angle: 0.0,
    },
    color_grading_new: {
      enabled: false,
      shadows: { r: 0, g: 0, b: 0 },
      midtones: { r: 0, g: 0, b: 0 },
      highlights: { r: 0, g: 0, b: 0 },
    },
    hsl: {
      red: { hue: 0, saturation: 0, luminance: 0 },
      orange: { hue: 0, saturation: 0, luminance: 0 },
      yellow: { hue: 0, saturation: 0, luminance: 0 },
      green: { hue: 0, saturation: 0, luminance: 0 },
      aqua: { hue: 0, saturation: 0, luminance: 0 },
      blue: { hue: 0, saturation: 0, luminance: 0 },
      purple: { hue: 0, saturation: 0, luminance: 0 },
      magenta: { hue: 0, saturation: 0, luminance: 0 },
    },
    tone_curve_rgb: {
      points: [],
    },
    lens_corrections: {
      distortion: 0.0,
      vignetting: 0.0,
    },
  };
}

// Global state
const [developRecipe, setDevelopRecipe] = createStore<EditRecipe>(defaultRecipe());
const [selectedPhotoId, setSelectedPhotoId] = createSignal<string | null>(null);
const [isDirty, setIsDirty] = createSignal(false);

// Callback registry for recipe changes
let onRecipeChangeCallbacks: Array<(recipe: EditRecipe) => void> = [];

/**
 * Register a callback to be called when the recipe changes
 * Returns an unregister function
 */
export function registerRecipeChangeCallback(cb: (recipe: EditRecipe) => void): () => void {
  onRecipeChangeCallbacks.push(cb);
  return () => {
    onRecipeChangeCallbacks = onRecipeChangeCallbacks.filter(c => c !== cb);
  };
}

/**
 * Update a single field in the recipe
 */
export function updateRecipeField(path: string[], value: any) {
  // Navigate to the nested field and update it
  if (path.length === 1) {
    setDevelopRecipe(path[0] as any, value);
  } else if (path.length === 2) {
    setDevelopRecipe(path[0] as any, path[1] as any, value);
  } else if (path.length === 3) {
    setDevelopRecipe(path[0] as any, path[1] as any, path[2] as any, value);
  }

  setIsDirty(true);

  // Trigger callbacks
  onRecipeChangeCallbacks.forEach(cb => cb(developRecipe));
}

/**
 * Load recipe for a photo from the catalog
 */
export async function loadRecipeForPhoto(photoId: string) {
  try {
    const recipe = await invoke<EditRecipe>("load_edit_recipe", { photoId });
    setDevelopRecipe(recipe);
    setSelectedPhotoId(photoId);
    setIsDirty(false);
  } catch (e) {
    console.error("Failed to load recipe:", e);
    // Fall back to default
    setDevelopRecipe(defaultRecipe());
    setSelectedPhotoId(photoId);
    setIsDirty(false);
  }
}

/**
 * Save the current recipe to the catalog
 */
export async function saveRecipe() {
  const photoId = selectedPhotoId();
  if (!photoId) {
    console.warn("No photo selected, cannot save recipe");
    return;
  }

  try {
    await invoke("save_edit_recipe", {
      photoId,
      recipe: developRecipe,
    });
    setIsDirty(false);
  } catch (e) {
    console.error("Failed to save recipe:", e);
    throw e;
  }
}

/**
 * Reset the recipe to default
 */
export function resetRecipe() {
  setDevelopRecipe(defaultRecipe());
  setIsDirty(true);
  onRecipeChangeCallbacks.forEach(cb => cb(developRecipe));
}

export {
  developRecipe,
  setDevelopRecipe,
  selectedPhotoId,
  setSelectedPhotoId,
  isDirty,
  defaultRecipe,
};
