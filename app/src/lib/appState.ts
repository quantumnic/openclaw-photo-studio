export interface AppState {
  lastCatalogPath?: string;
  activeModule: "library" | "develop" | "map" | "print";
  leftPanelOpen: boolean;
  rightPanelOpen: boolean;
  filmstripOpen: boolean;
  thumbnailSize: number;
  lastFilter?: {
    rating_min?: number;
    flag?: string;
    color_label?: string;
    search?: string;
  };
}

const DEFAULT_STATE: AppState = {
  activeModule: "library",
  leftPanelOpen: true,
  rightPanelOpen: true,
  filmstripOpen: true,
  thumbnailSize: 200,
};

export function loadAppState(): Partial<AppState> {
  try {
    const stored = localStorage.getItem("ocps_app_state");
    if (stored) {
      return { ...DEFAULT_STATE, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error("Failed to load app state:", error);
  }
  return DEFAULT_STATE;
}

export function saveAppState(state: Partial<AppState>): void {
  try {
    localStorage.setItem("ocps_app_state", JSON.stringify(state));
  } catch (error) {
    console.error("Failed to save app state:", error);
  }
}
