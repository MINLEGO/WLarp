import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errText, isTauri } from "../lib/ipc";
import type { DocDto } from "../lib/types";
import { useApp } from "../stores";
import { Button, Confirm, Modal, Prompt } from "./ui";

const today = () => new Date().toISOString().slice(0, 10);

export default function CoursesPane() {
  const rev = useApp((s) => s.rev);
  const bump = useApp((s) => s.bump);
  const selectedFolderId = useApp((s) => s.selectedFolderId);
  const openDoc = useApp((s) => s.openDoc);
  const openDocId = useApp((s) => s.openDocId);
  const setToast = useApp((s) => s.setToast);

  const [docs, setDocs] = useState<DocDto[]>([]);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [newPrompt, setNewPrompt] = useState(false);
  const [mergeOpen, setMergeOpen] = useState(false);
  const [mergeTarget, setMergeTarget] = useState("");
  const [delDoc, setDelDoc] = useState<DocDto | null>(null);
  const [rename, setRename] = useState<DocDto | null>(null);

  useEffect(() => {
    setSel(new Set());
    if (!selectedFolderId || selectedFolderId === "root") {
      setDocs([]);
      return;
    }
    api.listDocs(selectedFolderId).then(setDocs).catch((e) => setToast(errText(e)));
  }, [rev, selectedFolderId]); // eslint-disable-line

  const upcoming = useMemo(() => {
    const dated = docs
      .filter((d) => d.examDate && d.examDate >= today())
      .sort((a, b) => (a.examDate! < b.examDate! ? -1 : 1));
    return dated[0];
  }, [docs]);

  async function importHere() {
    if (!isTauri) {
      setToast("Importation disponible dans l’application desktop");
      return;
    }
    const picked = await open({
      multiple: true,
      title: "Importer des supports",
      filters: [
        { name: "Documents", extensions: ["pdf", "docx", "pptx", "md", "txt"] },
        { name: "Images", extensions: ["jpg", "jpeg", "png", "webp", "gif", "avif"] },
      ],
    });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
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

  const selDocs = docs.filter((d) => sel.has(d.id));

  async function doMerge() {
    const sources = selDocs.filter((d) => d.id !== mergeTarget).map((d) => d.id);
    if (!mergeTarget || sources.length === 0) return;
    setMergeOpen(false);
    try {
      await api.mergeDocs(sources, mergeTarget);
      setToast("Cours fusionnés");
      setSel(new Set());
      bump();
    } catch (e) {
      setToast(errText(e));
    }
  }

  return (
    <div
      className="flex h-full min-w-72 flex-col bg-surface"
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => {
        const docId = e.dataTransfer.getData("text/wlarp-doc");
        if (docId && selectedFolderId && selectedFolderId !== "root") {
          api.moveDoc(docId, selectedFolderId).then(bump).catch((err) => setToast(errText(err)));
        }
      }}
    >
      <div className="flex items-center gap-2 border-b border-line px-4 py-2">
        <h2 className="truncate font-semibold">
          {selectedFolderId === "root" || !selectedFolderId ? "Choisir un dossier" : "Cours"}
        </h2>
        <div className="ml-auto flex gap-2">
          <Button kind="ghost" onClick={importHere} disabled={!selectedFolderId || selectedFolderId === "root"}>
            📥 Importer
          </Button>
          <Button onClick={() => setNewPrompt(true)} disabled={!selectedFolderId || selectedFolderId === "root"}>
            ＋ Cours
          </Button>
        </div>
      </div>

      {upcoming && (
        <button
          className="border-b border-line bg-amber-50 px-4 py-2 text-left text-sm hover:bg-amber-100"
          onClick={() => openDoc(upcoming.id)}
        >
          📅 Examen prévu : <b>{upcoming.title}</b> le {upcoming.examDate}
        </button>
      )}

      {sel.size >= 2 && (
        <div className="flex items-center gap-2 border-b border-line bg-accent-soft px-4 py-2 text-sm">
          {sel.size} cours sélectionnés
          <div className="ml-auto">
            <Button onClick={() => { setMergeTarget(selDocs[0]?.id ?? ""); setMergeOpen(true); }}>
              🔀 Fusionner…
            </Button>
          </div>
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-auto">
        {docs.map((d) => (
          <div
            key={d.id}
            draggable
            onDragStart={(e) => e.dataTransfer.setData("text/wlarp-doc", d.id)}
            className={`flex cursor-grab items-center gap-2 border-b border-line px-4 py-2 text-sm hover:bg-surface-2 ${
              openDocId === d.id ? "bg-accent-soft" : ""
            } ${d.excluded ? "opacity-50" : ""}`}
          >
            <input
              type="checkbox"
              checked={sel.has(d.id)}
              onChange={(e) => {
                const next = new Set(sel);
                if (e.target.checked) next.add(d.id);
                else next.delete(d.id);
                setSel(next);
              }}
            />
            <button className="truncate font-medium hover:underline" onClick={() => openDoc(d.id)}>
              {d.title}
            </button>
            {d.examDate && <span className="text-xs text-amber-700">📅 {d.examDate}</span>}
            <span className="text-xs text-muted">📎 {d.supportCount}</span>
            {d.questionCount > 0 && <span className="text-xs text-muted">❓ {d.questionCount}</span>}
            {d.dueCount > 0 && (
              <span className="rounded bg-amber-100 px-1.5 text-xs text-amber-800">due {d.dueCount}</span>
            )}
            <div className="ml-auto flex gap-1">
              <button
                title={d.excluded ? "Réinclure" : "Exclure"}
                className="rounded px-1 hover:bg-surface"
                onClick={() => api.setDocFlags(d.id, !d.excluded, d.useTranscription).then(bump)}
              >
                {d.excluded ? "🚩" : "🏳️"}
              </button>
              <button title="Renommer" className="rounded px-1 hover:bg-surface" onClick={() => setRename(d)}>
                ✏️
              </button>
              <button title="Supprimer" className="rounded px-1 hover:bg-surface" onClick={() => setDelDoc(d)}>
                🗑️
              </button>
            </div>
          </div>
        ))}
        {docs.length === 0 && selectedFolderId && selectedFolderId !== "root" && (
          <p className="p-6 text-center text-sm text-muted">
            Aucun cours ici. Glissez des fichiers PDF / images, ou cliquez sur « Importer ».
          </p>
        )}
      </div>
      {newPrompt && (
        <Prompt
          title="Nouveau cours"
          initial=""
          onClose={() => setNewPrompt(false)}
          onSubmit={async (title) => {
            setNewPrompt(false);
            try {
              const d = await api.createDoc(selectedFolderId!, title);
              bump();
              openDoc(d.id);
            } catch (e) {
              setToast(errText(e));
            }
          }}
        />
      )}
      {rename && (
        <Prompt
          title="Renommer le cours"
          initial={rename.title}
          onClose={() => setRename(null)}
          onSubmit={async (title) => {
            setRename(null);
            await api.renameDoc(rename.id, title);
            bump();
          }}
        />
      )}
      {delDoc && (
        <Confirm
          title={`Supprimer « ${delDoc.title} » ?`}
          body="Le cours et tous ses supports/fichiers importés seront supprimés définitivement (questions et révisions incluses)."
          danger
          onClose={() => setDelDoc(null)}
          onYes={async () => {
            setDelDoc(null);
            await api.deleteDoc(delDoc.id);
            bump();
          }}
        />
      )}
      {mergeOpen && (
        <Modal title="Fusionner les cours" width="440px" onClose={() => setMergeOpen(false)}>
          <p className="mb-3 text-sm text-muted">
            Sélectionner le cours cible — les supports et questions des autres seront déplacés
            dedans, et le contenu fusionné dans la note.
          </p>
          {selDocs.map((d) => (
            <label key={d.id} className="mb-1 flex items-center gap-2 rounded-md px-2 py-1 text-sm hover:bg-surface-2">
              <input
                type="radio"
                name="merge-target"
                checked={mergeTarget === d.id}
                onChange={() => setMergeTarget(d.id)}
              />
              {d.title}
            </label>
          ))}
          <div className="mt-4 flex justify-end gap-2">
            <Button kind="ghost" onClick={() => setMergeOpen(false)}>Annuler</Button>
            <Button onClick={doMerge} disabled={!mergeTarget}>Fusionner</Button>
          </div>
        </Modal>
      )}
    </div>
  );
}