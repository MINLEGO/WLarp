import { create } from "zustand";
import { api } from "./lib/ipc";
import type { Settings } from "./lib/types";

interface AppState {
  /** Sélecteur de version : incrémenté pour invalider les vues (arbre, cours…). */
  rev: number;
  bump: () => void;

  selectedFolderId: string;
  selectFolder: (id: string) => void;
  openDocId: string | null;
  openDoc: (id: string | null) => void;
  /** dossiers dépliés dans l'arbre */
  expanded: Record<string, boolean>;
  toggleExpanded: (id: string) => void;

  settings: Settings | null;
  hasKey: boolean;
  loadSettings: () => Promise<void>;

  toast: string | null;
  setToast: (t: string | null) => void;
}

export const useApp = create<AppState>((set, get) => ({
  rev: 0,
  bump: () => set({ rev: get().rev + 1 }),

  selectedFolderId: "root",
  selectFolder: (id) => set({ selectedFolderId: id, openDocId: null }),
  openDocId: null,
  openDoc: (id) => set({ openDocId: id }),
  expanded: { root: true },
  toggleExpanded: (id) =>
    set({ expanded: { ...get().expanded, [id]: !get().expanded[id] } }),

  settings: null,
  hasKey: false,
  loadSettings: async () => {
    const [settings, hasKey] = await Promise.all([api.getSettings(), api.hasApiKey()]);
    set({ settings, hasKey });
  },

  toast: null,
  setToast: (t) => set({ toast: t }),
}));