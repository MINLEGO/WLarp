import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errText, isTauri } from "../lib/ipc";
import type { FolderDto } from "../lib/types";
import { useApp } from "../stores";
import { Button, Prompt } from "./ui";

interface Menu {
  x: number;
  y: number;
  folder: FolderDto;
}

const ROOT: FolderDto = {
  id: "root",
  parentId: null,
  name: "Racine",
  createdAt: "",
  docCount: 0,
};

function subtreeIds(all: FolderDto[], root: string): Set<string> {
  const out = new Set<string>([root]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const f of all) {
      if (f.parentId && out.has(f.parentId) && !out.has(f.id)) {
        out.add(f.id);
        changed = true;
      }
    }
  }
  return out;
}

export default function Sidebar() {
  const rev = useApp((s) => s.rev);
  const bump = useApp((s) => s.bump);
  const selectedFolderId = useApp((s) => s.selectedFolderId);
  const selectFolder = useApp((s) => s.selectFolder);
  const expanded = useApp((s) => s.expanded);
  const toggleExpanded = useApp((s) => s.toggleExpanded);
  const setToast = useApp((s) => s.setToast);

  const [folders, setFolders] = useState<FolderDto[]>([]);
  const [menu, setMenu] = useState<Menu | null>(null);
  const [prompt, setPrompt] = useState<{ kind: "new" | "rename"; folder: FolderDto } | null>(null);
  const [del, setDel] = useState<FolderDto | null>(null);
  const [move, setMove] = useState<FolderDto | null>(null);

  useEffect(() => {
    api.listFolders().then(setFolders).catch((e) => setToast(errText(e)));
  }, [rev]);

  const children = useMemo(() => {
    const map: Record<string, FolderDto[]> = {};
    for (const f of folders) {
      if (f.id === "root") continue; // la ligne synthétique « root » de la BDD EST la racine de l'arbre
      (map[f.parentId ?? "root"] ??= []).push(f);
    }
    return map;
  }, [folders]);

  /** Ligne réelle de « Racine » (nom + comptage fournis par le backend), sinon le vide statique. */
  const rootFolder = folders.find((f) => f.id === ROOT.id) ?? ROOT;

  async function importInto(folderId: string) {
    if (!isTauri) {
      setToast("Importation disponible dans l’application desktop");
      return;
    }
    const sel = await open({
      multiple: true,
      title: "Importer des supports",
      filters: [
        { name: "Documents", extensions: ["pdf", "docx", "pptx", "md", "txt"] },
        { name: "Images", extensions: ["jpg", "jpeg", "png", "webp", "gif", "avif"] },
      ],
    });
    if (!sel) return;
    const paths = Array.isArray(sel) ? sel : [sel];
    await api.setFrontView(folderId, null);
    try {
      const rep = await api.importPaths(paths);
      setToast(
        `Import : ${rep.created.length} support(s)` +
          (rep.createdDocs.length ? `, ${rep.createdDocs.length} cours créé(s)` : "") +
          (rep.duplicates ? `, ${rep.duplicates} doublon(s)` : ""),
      );
      bump();
    } catch (e) {
      setToast(errText(e));
    }
  }

  function row(f: FolderDto, depth: number) {
    if (depth > 32) return null; // garde-fou anti-cycle dans la hierarchie
    const kids = children[f.id] ?? [];
    const isOpen = expanded[f.id];
    return (
      <div key={f.id}>
        <div
          className={`flex cursor-pointer items-center gap-1 rounded-md px-2 py-1 text-sm hover:bg-surface-2 ${
            selectedFolderId === f.id ? "bg-accent-soft font-medium" : ""
          }`}
          style={{ paddingLeft: 8 + depth * 14 }}
          onClick={() => selectFolder(f.id)}
          onContextMenu={(e) => {
            e.preventDefault();
            setMenu({ x: e.clientX, y: e.clientY, folder: f });
          }}
          onDragOver={(e) => e.preventDefault()}
          onDrop={(e) => {
            e.preventDefault();
            const docId = e.dataTransfer.getData("text/wlarp-doc");
            if (docId) api.moveDoc(docId, f.id).then(bump).catch((err) => setToast(errText(err)));
          }}
        >
          <button
            className="w-4 text-xs text-muted"
            onClick={(e) => {
              e.stopPropagation();
              toggleExpanded(f.id);
            }}
          >
            {kids.length ? (isOpen ? "▾" : "▸") : ""}
          </button>
          <span className="truncate">📁 {f.name}</span>
          {f.docCount > 0 && <span className="ml-auto text-xs text-muted">{f.docCount}</span>}
        </div>
        {isOpen && kids.map((k) => row(k, depth + 1))}
      </div>
    );
  }

  return (
    <aside className="flex h-full w-60 shrink-0 flex-col border-r border-line bg-surface p-2">
      <div className="mb-1 flex items-center justify-between px-1">
        <span className="text-xs font-semibold uppercase tracking-wide text-muted">Dossiers</span>
        <button
          className="rounded px-1.5 text-sm hover:bg-surface-2"
          title="Nouveau dossier à la racine"
          onClick={() => setPrompt({ kind: "new", folder: ROOT })}
        >
          ＋
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-auto">
        {row(rootFolder, 0)}
      </div>
      {menu && (
        <div
          className="fixed z-50 min-w-48 rounded-lg border border-line bg-surface py-1 shadow-xl"
          style={{ left: menu.x, top: menu.y }}
          onMouseLeave={() => setMenu(null)}
        >
          {([
            ["➕ Nouveau sous-dossier", () => setPrompt({ kind: "new", folder: menu.folder })],
            ["📥 Importer des fichiers", () => importInto(menu.folder.id)],
            ...(menu.folder.id !== "root"
              ? ([
                  ["✏️ Renommer", () => setPrompt({ kind: "rename", folder: menu.folder })],
                  ["📦 Déplacer vers…", () => setMove(menu.folder)],
                  ["🗑️ Supprimer…", () => setDel(menu.folder)],
                ] as [string, () => void][])
              : []),
          ] as [string, () => void][]).map(([label, fn]) => (
            <button
              key={label}
              className="block w-full px-3 py-1.5 text-left text-sm hover:bg-surface-2"
              onClick={() => {
                setMenu(null);
                fn();
              }}
            >
              {label}
            </button>
          ))}
        </div>
      )}

      {prompt && (
        <Prompt
          title={
            prompt.kind === "new"
              ? `Nouveau dossier dans « ${prompt.folder.name} »`
              : "Renommer le dossier"
          }
          initial={prompt.kind === "rename" ? prompt.folder.name : ""}
          onClose={() => setPrompt(null)}
          onSubmit={async (name) => {
            setPrompt(null);
            try {
              if (prompt.kind === "new") await api.createFolder(prompt.folder.id, name);
              else await api.renameFolder(prompt.folder.id, name);
              bump();
            } catch (e) {
              setToast(errText(e));
            }
          }}
        />
      )}
      {del && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onMouseDown={(e) => e.target === e.currentTarget && setDel(null)}
        >
          <div className="w-[460px] rounded-xl bg-surface p-5 shadow-2xl">
            <h2 className="mb-2 text-lg font-semibold">Supprimer « {del.name} » ?</h2>
            <p className="mb-4 text-sm text-muted">
              Le dossier (et ses sous-dossiers, rattachés au parent) sera supprimé. Que deviennent
              ses cours ?
            </p>
            <div className="flex flex-wrap justify-end gap-2">
              <Button kind="ghost" onClick={() => setDel(null)}>Annuler</Button>
              <Button
                onClick={async () => {
                  setDel(null);
                  await api.deleteFolder(del.id, "recycle");
                  if (selectedFolderId === del.id) selectFolder("root");
                  bump();
                }}
              >
                Garder les cours
              </Button>
              <Button
                kind="danger"
                onClick={async () => {
                  setDel(null);
                  await api.deleteFolder(del.id, "content");
                  if (selectedFolderId === del.id) selectFolder("root");
                  bump();
                }}
              >
                Tout supprimer
              </Button>
            </div>
          </div>
        </div>
      )}

      {move && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onMouseDown={(e) => e.target === e.currentTarget && setMove(null)}
        >
          <div className="w-[380px] rounded-xl bg-surface p-5 shadow-2xl">
            <h2 className="mb-3 text-lg font-semibold">Déplacer « {move.name} » vers…</h2>
            <div className="mb-3 max-h-72 overflow-auto">
              {[ROOT, ...folders]
                .filter((f) => f.id !== move.id && !subtreeIds(folders, move.id).has(f.id))
                .map((f) => (
                  <button
                    key={f.id}
                    className="block w-full rounded-md px-3 py-1.5 text-left text-sm hover:bg-surface-2"
                    onClick={async () => {
                      setMove(null);
                      try {
                        await api.moveFolder(move.id, f.id);
                        if (!expanded[f.id]) toggleExpanded(f.id);
                        bump();
                      } catch (e) {
                        setToast(errText(e));
                      }
                    }}
                  >
                    📁 {f.name}
                  </button>
                ))}
            </div>
            <div className="flex justify-end">
              <Button kind="ghost" onClick={() => setMove(null)}>Annuler</Button>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
}