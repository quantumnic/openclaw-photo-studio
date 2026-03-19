import { createStore } from "solid-js/store";

export interface Preferences {
  // General
  language: string;
  dateFormat: "YYYY-MM-DD" | "MM-DD-YYYY" | "DD-MM-YYYY";
  autoAdvanceAfterRating: boolean;

  // Performance
  previewCacheSize: "512MB" | "1GB" | "2GB" | "4GB";
  backgroundThreads: number | "auto";
  gpuAcceleration: boolean;

  // Metadata & Sidecars
  xmpSidecarMode: "auto" | "manual" | "read-only" | "disabled";
  defaultCopyright: string;
  defaultCreator: string;
}

const DEFAULT_PREFERENCES: Preferences = {
  language: "English",
  dateFormat: "YYYY-MM-DD",
  autoAdvanceAfterRating: true,
  previewCacheSize: "1GB",
  backgroundThreads: "auto",
  gpuAcceleration: false,
  xmpSidecarMode: "auto",
  defaultCopyright: "",
  defaultCreator: "",
};

function loadPreferences(): Preferences {
  const stored = localStorage.getItem("ocps_preferences");
  if (stored) {
    try {
      return { ...DEFAULT_PREFERENCES, ...JSON.parse(stored) };
    } catch {
      return DEFAULT_PREFERENCES;
    }
  }
  return DEFAULT_PREFERENCES;
}

function savePreferences(prefs: Preferences) {
  localStorage.setItem("ocps_preferences", JSON.stringify(prefs));
}

export function createPreferences() {
  const [preferences, setPreferences] = createStore<Preferences>(loadPreferences());

  const updatePreferences = (updates: Partial<Preferences>) => {
    setPreferences(updates);
    savePreferences({ ...preferences });
  };

  return { preferences, updatePreferences };
}

// Global preferences store
const preferencesStore = createPreferences();

export function usePreferences() {
  return preferencesStore;
}
