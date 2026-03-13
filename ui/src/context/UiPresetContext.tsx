import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type UiPreset =
  | "default"
  | "blue"
  | "green"
  | "lavender"
  | "teal"
  | "coral"
  | "sage"
  | "mint";

const PRESET_IDS: UiPreset[] = [
  "default",
  "blue",
  "green",
  "lavender",
  "teal",
  "coral",
  "sage",
  "mint",
];

interface UiPresetContextValue {
  preset: UiPreset;
  setPreset: (preset: UiPreset) => void;
}

const STORAGE_KEY = "paperclip.uiPreset";
const UiPresetContext = createContext<UiPresetContextValue | undefined>(undefined);

function loadPreset(): UiPreset {
  if (typeof document === "undefined") return "default";
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (PRESET_IDS.includes(stored as UiPreset)) return stored as UiPreset;
  } catch {
    // ignore
  }
  return "default";
}

function applyPreset(preset: UiPreset) {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-ui-preset", preset);
}

export function UiPresetProvider({ children }: { children: ReactNode }) {
  const [preset, setPresetState] = useState<UiPreset>(loadPreset);

  const setPreset = useCallback((next: UiPreset) => {
    setPresetState(next);
  }, []);

  useEffect(() => {
    applyPreset(preset);
    try {
      localStorage.setItem(STORAGE_KEY, preset);
    } catch {
      // ignore
    }
  }, [preset]);

  const value = useMemo(() => ({ preset, setPreset }), [preset, setPreset]);

  return (
    <UiPresetContext.Provider value={value}>
      {children}
    </UiPresetContext.Provider>
  );
}

export function useUiPreset() {
  const context = useContext(UiPresetContext);
  if (!context) {
    throw new Error("useUiPreset must be used within UiPresetProvider");
  }
  return context;
}
