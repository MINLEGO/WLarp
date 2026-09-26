import { useEffect, useState } from "react";
import { api, errText, supportUrl } from "../lib/ipc";
import type { DocDetail, SupportDto } from "../lib/types";
import { useApp } from "../stores";
import { Button, Confirm, Prompt } from "./ui";

function SupportView({ s }: { s: SupportDto }) {
  const [url, setUrl] = useState<string | null>(null);
  const [text, setText] = useState<string | null>(null);
  const setToast = useApp((st) => st.setToast);

  useEffect(() => {
    setUrl(null);
    setText(null);
    let dead = false;
    supportUrl(s.id)
      .then(async (u) => {
        if (dead) return;
        setUrl(u);
        if (s.mediaType === "mdx") {
          try {
            const r = await fetch(u);
            setText(await r.text());
          } catch {
            setText("(lecture du fichier impossible)");
          }
        }
      })
      .catch((e) => !dead && setToast(errText(e)));
    return () => {
      dead = true;
    };
  }, [s.id, s.mediaType, setToast]);

  if (!url) return <div className="p-6 text-sm text-muted">Chargement…</div>;
  switch (s.mediaType) {
    case "pdf":
      return <object data={url} type="application/pdf" className="h-[calc(100vh-160px)] w-full" />;
    case "image":
      return (
        <div className="flex h-full items-start justify-center overflow-auto p-3">
          <img src={url} alt={s.fileName} className="max-h-[calc(100vh-170px)] object-contain" />
        </div>
      );
    case "audio":
      return <audio controls src={url} className="m-4" />;
    case "video":
      return <video controls src={url} className="m-4 max-h-[70vh]" />;
    case "mdx":
      return <pre className="overflow-auto whitespace-pre-wrap p-4 font-mono text-xs">{text}</pre>;
    default:
      return (
        <p className="p-4 text-sm text-muted">
          Prévisualisation indisponible pour ce fichier ({s.mime}).
        </p>
      );
  }
}

export default function EditorPane() {
  const rev = useApp((s) => s.rev);
  const bump = useApp((s) => s.bump);
  const openDocId = useApp((s) => s.openDocId);
  const openDoc = useApp((s) => s.openDoc);
  const setToast = useApp((s) => s.setToast);
  const [detail, setDetail] = useState<DocDetail | null>(null);
  const [tab, setTab] = useState<"supports" | "note">("supports");
  const [active, setActive] = useState(0);
  const [delSup, setDelSup] = useState<SupportDto | null>(null);
  const [split, setSplit] = useState<SupportDto | null>(null);

  useEffect(() => {
    setDetail(null);
    setActive(0);
    setTab("supports");
    if (!openDocId) return;
    api
      .getDoc(openDocId)
      .then(setDetail)
      .catch((e) => setToast(errText(e)));
  }, [openDocId, rev]); // eslint-disable-line

  useEffect(() => {
    if (!openDocId) return;
    api.setFrontView(null, openDocId).catch(() => {});
  }, [openDocId]); // eslint-disable-line

  const supports = detail?.supports ?? [];
  const cur = supports[Math.min(active, supports.length - 1)];

  if (!openDocId)
    return (
      <div className="flex h-full flex-1 items-center justify-center text-sm text-muted">
        Sélectionne un cours
      </div>
    );
  if (!detail) return <div className="flex-1 p-6 text-sm text-muted">Chargement…</div>;
  const doc = detail.doc;

  return (
    <div className="flex h-full min-w-0 flex-1 flex-col bg-surface">
      <div className="flex flex-wrap items-center gap-2 border-b border-line px-3 py-2">
        <h2 className="truncate text-sm font-semibold">{doc.title}</h2>
        <div className="flex gap-1">
          <button
            className={`rounded px-2 py-0.5 text-xs ${tab === "supports" ? "bg-accent-soft font-medium" : "hover:bg-surface-2"}`}
            onClick={() => setTab("supports")}
          >
            Supports ({supports.length})
          </button>
          <button
            className={`rounded px-2 py-0.5 text-xs ${tab === "note" ? "bg-accent-soft font-medium" : "hover:bg-surface-2"}`}
            onClick={() => setTab("note")}
          >
            Note
          </button>
        </div>
        <label className="ml-auto flex items-center gap-1 text-xs text-muted" title="Date d'examen — notifications & priorité">
          📅 examen
          <input
            type="date"
            className="rounded border border-line bg-surface px-1 text-xs"
            value={doc.examDate ?? ""}
            onChange={(e) =>
              api
                .setExamDate(doc.id, e.target.value || null)
                .then(bump)
                .catch((er) => setToast(errText(er)))
            }
          />
        </label>
        <Button
          kind="ghost"
          title="Renvoyer vers la liste des cours"
          onClick={() => openDoc(null)}
        >
          ✕
        </Button>
      </div>
      {tab === "supports" && (
        <div className="flex min-h-0 flex-1">
          <div className="w-56 shrink-0 overflow-auto border-r border-line p-2">
            {supports.map((sp, i) => (
              <div
                key={sp.id}
                className={`group mb-1 flex cursor-pointer items-center gap-1 rounded-md px-2 py-1 text-xs hover:bg-surface-2 ${
                  active === i ? "bg-accent-soft font-medium" : ""
                }`}
                onClick={() => setActive(i)}
                title={sp.fileName}
              >
                <span className="flex-1 truncate">
                  {sp.mediaType === "pdf" ? "📄" : sp.mediaType === "image" ? "🖼️" : sp.mediaType === "mdx" ? "📝" : sp.mediaType === "audio" ? "🎧" : "🎬"}{" "}
                  {sp.fileName}
                </span>
                <button
                  title="Détacher dans un nouveau cours"
                  className="opacity-0 group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation();
                    setSplit(sp);
                  }}
                >
                  ↗
                </button>
                <button
                  title="Supprimer le support"
                  className="opacity-0 group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation();
                    setDelSup(sp);
                  }}
                >
                  🗑
                </button>
              </div>
            ))}
            {supports.length === 0 && (
              <p className="p-2 text-xs text-muted">
                Glisse des fichiers (PDF, images…) sur la fenêtre pour les importer ici.
              </p>
            )}
          </div>
          <div className="min-w-0 flex-1 overflow-auto">
            {cur ? <SupportView s={cur} /> : <p className="p-6 text-sm text-muted">Aucun support sélectionné.</p>}
          </div>
        </div>
      )}

      {tab === "note" && (
        <div className="flex flex-1 items-center justify-center p-6 text-sm text-muted">
          L’éditeur de note Markdown arrive en M2 — les supports restent visibles dans l’onglet
          « Supports ».
        </div>
      )}

      {delSup && (
        <Confirm
          title={`Supprimer « ${delSup.fileName} » ?`}
          body="Le fichier importé, sa transcription et les questions liées à ce support seront supprimés."
          danger
          onClose={() => setDelSup(null)}
          onYes={async () => {
            setDelSup(null);
            try {
              await api.deleteSupport(delSup.id);
              bump();
            } catch (e) {
              setToast(errText(e));
            }
          }}
        />
      )}

      {split && (
        <Prompt
          title={`Détacher « ${split.fileName} » dans un nouveau cours`}
          initial={split.fileName.replace(/\.[^.]+$/, "")}
          onClose={() => setSplit(null)}
          onSubmit={async (t) => {
            setSplit(null);
            try {
              const d = await api.splitSupport(split.id, t);
              bump();
              openDoc(d.id);
            } catch (e) {
              setToast(errText(e));
            }
          }}
        />
      )}
    </div>
  );
}