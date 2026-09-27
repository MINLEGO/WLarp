import { invoke, convertFileSrc } from "@tauri-apps/api/core";

/** true seulement dans la webview Tauri (F12/navigateur pur → false). */
export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

import type {
  DocDetail,
  DocDto,
  FolderDto,
  ImportReport,
  ModelInfo,
  Settings,
} from "./types";

/** Chemin absolu d'un support → URL affichable par la webview (asset: scope $APPDATA/**). */
export async function supportUrl(supportId: string): Promise<string> {
  const abs = await invoke<string>("support_abs_path", { id: supportId });
  return convertFileSrc(abs);
}

export const api = {
  appInfo: () => invoke<{ appDataDir: string; dueCount: number }>("app_info"),
  setFrontView: (folderId: string | null, docId: string | null) =>
    invoke("set_front_view", { folderId, docId }),

  listFolders: () => invoke<FolderDto[]>("list_folders"),
  createFolder: (parentId: string, name: string) =>
    invoke<FolderDto>("create_folder", { parentId, name }),
  renameFolder: (id: string, name: string) => invoke("rename_folder", { id, name }),
  moveFolder: (id: string, newParentId: string) =>
    invoke("move_folder", { id, newParentId }),
  deleteFolder: (id: string, mode: "recycle" | "content") =>
    invoke("delete_folder", { id, mode }),

  listDocs: (folderId: string) => invoke<DocDto[]>("list_docs", { folderId }),
  getDoc: (id: string) => invoke<DocDetail>("get_doc", { id }),
  createDoc: (folderId: string, title: string) =>
    invoke<DocDto>("create_doc", { folderId, title }),
  renameDoc: (id: string, title: string) => invoke("rename_doc", { id, title }),
  moveDoc: (id: string, folderId: string) => invoke("move_doc", { id, folderId }),
  setDocFlags: (id: string, excluded: boolean, useTranscription: boolean) =>
    invoke("set_doc_flags", { id, excluded, useTranscription }),
  setExamDate: (id: string, date: string | null) => invoke("set_exam_date", { id, date }),
  deleteDoc: (id: string) => invoke("delete_doc", { id }),

  importPaths: (paths: string[]) => invoke<ImportReport>("import_paths", { paths }),
  deleteSupport: (id: string) => invoke("delete_support", { id }),
  mergeDocs: (sourceIds: string[], targetId: string) =>
    invoke<DocDto>("merge_docs", { sourceIds, targetId }),
  splitSupport: (supportId: string, title: string) =>
    invoke<DocDto>("split_support", { supportId, title }),

  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke("set_settings", { settings }),
  setApiKey: (key: string) => invoke("set_api_key", { key }),
  hasApiKey: () => invoke<boolean>("has_api_key"),
  deleteApiKey: () => invoke("delete_api_key"),
  listModels: () => invoke<ModelInfo[]>("list_models"),

  hideWindow: () => invoke("hide_window"),
};

/** Erreur lisible depuis un rejet d'invoke (AppError sérialisé en chaîne ou objet). */
export function errText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e && typeof e === "object") {
    const v = e as Record<string, unknown>;
    if (typeof v.message === "string") return v.message;
  }
  return String(e);
}