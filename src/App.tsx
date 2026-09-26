import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useApp } from "./stores";
import { api, errText } from "./lib/ipc";
import Sidebar from "./components/Sidebar";
import CoursesPane from "./components/CoursesPane";
import EditorPane from "./components/EditorPane";
import SettingsModal from "./components/SettingsModal";
import { Toast } from "./components/ui";

export default function App() {
  const rev = useApp((s) => s.rev);
  const bump = useApp((s) => s.bump);
  const loadSettings = useApp((s) => s.loadSettings);
  const setToast = useApp((s) => s.setToast);
  const selectedFolderId = useApp((s) => s.selectedFolderId);
  const openDocId = useApp((s) => s.openDocId);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [due, setDue] = useState(0);
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    loadSettings().catch(() => {});
  }, []); // eslint-disable-line

  useEffect(() => {
    api.appInfo().then((i) => setDue(i.dueCount)).catch(() => {});
  }, [rev]);

  /** Dit au backend où poser les fichiers importés (cours ouvert, sinon dossier sélectionné). */
  useEffect(() => {
    const folder =
      selectedFolderId === "root" ? null : selectedFolderId;
    api.setFrontView(openDocId ? null : folder, openDocId).catch(() => {});
  }, [selectedFolderId, openDocId]);

  /** Drop natif OS : les chemins de fichiers arrivent par Tauri, pas par la webview. */
  useEffect(() => {
    const reg = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "enter" || e.payload.type === "over") {
        setDragging(true);
      } else if (e.payload.type === "leave") {
        setDragging(false);
      } else if (e.payload.type === "drop") {
        setDragging(false);
        const paths = e.payload.paths;
        void (async () => {
          try {
            const rep = await api.importPaths(paths);
            setToast(
              `Import : ${rep.created.length} support(s)` +
                (rep.createdDocs.length ? `, ${rep.createdDocs.length} cours créé(s)` : "") +
                (rep.duplicates ? `, ${rep.duplicates} doublon(s)` : "") +
                (rep.skipped.length ? `, ${rep.skipped.length} ignoré(s)` : ""),
            );
            bump();
          } catch (err) {
            setToast(errText(err));
          }
        })();
      }
    });
    return () => {
      void reg.then((f) => f());
    };
  }, [bump, setToast]);

  return (
    <div className="flex h-full flex-col">
      <header
        data-tauri-drag-region
        className="flex h-11 shrink-0 items-center gap-3 border-b border-line bg-surface px-4"
      >
        <span className="text-sm font-bold select-none">WLarp</span>
        <span className="text-xs text-muted select-none">Révisions espacées</span>
        <div className="ml-auto flex items-center gap-2">
          <span
            className="rounded-full bg-amber-100 px-2.5 py-0.5 text-xs font-medium text-amber-800"
            title="Cartes prêtes à réviser"
          >
            🔔 {due} à réviser
          </span>
          <button
            className="rounded-lg bg-surface-2 px-3 py-1 text-sm hover:bg-line"
            onClick={() => setSettingsOpen(true)}
          >
            ⚙️ Réglages
          </button>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <CoursesPane />
        <EditorPane />
      </div>

      {settingsOpen && <SettingsModal onClose={() => setSettingsOpen(false)} />}

      {dragging && (
        <div className="pointer-events-none fixed inset-4 z-40 flex items-center justify-center rounded-2xl border-2 border-dashed border-accent bg-accent/10 text-lg font-medium text-accent">
          📥 Dépose pour importer dans la vue actuelle
        </div>
      )}

      <Toast />
    </div>
  );
}