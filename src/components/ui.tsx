import { useEffect, useState, type ReactNode } from "react";

export function Modal(props: {
  title: string;
  onClose: () => void;
  children: ReactNode;
  width?: string;
}) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <div
        className="max-h-[86vh] w-full overflow-auto rounded-xl bg-surface p-5 shadow-2xl"
        style={{ maxWidth: props.width ?? "560px" }}
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold">{props.title}</h2>
          <button
            className="rounded-md px-2 py-1 text-muted hover:bg-surface-2"
            onClick={props.onClose}
            title="Fermer"
          >
            ✕
          </button>
        </div>
        {props.children}
      </div>
    </div>
  );
}

export function Field(props: { label: string; children: ReactNode; hint?: string }) {
  return (
    <label className="mb-3 block">
      <span className="mb-1 block text-xs font-medium text-muted">
        {props.label}
        {props.hint && <span className="ml-2 font-normal opacity-70">{props.hint}</span>}
      </span>
      {props.children}
    </label>
  );
}

export function TextInput(props: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  type?: string;
  autoFocus?: boolean;
  onEnter?: () => void;
}) {
  return (
    <input
      className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
      type={props.type ?? "text"}
      value={props.value}
      placeholder={props.placeholder}
      autoFocus={props.autoFocus}
      onChange={(e) => props.onChange(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && props.onEnter) props.onEnter();
      }}
    />
  );
}

export function Button(props: {
  children: ReactNode;
  onClick?: () => void;
  kind?: "primary" | "ghost" | "danger";
  disabled?: boolean;
  title?: string;
}) {
  const k = props.kind ?? "primary";
  const cls =
    k === "primary"
      ? "bg-accent text-white hover:opacity-90"
      : k === "danger"
        ? "bg-red-600 text-white hover:opacity-90"
        : "bg-surface-2 text-ink hover:bg-line";
  return (
    <button
      className={`rounded-lg px-3 py-1.5 text-sm font-medium disabled:opacity-40 ${cls}`}
      onClick={props.onClick}
      disabled={props.disabled}
      title={props.title}
    >
      {props.children}
    </button>
  );
}

export function Checkbox(props: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  hint?: string;
}) {
  return (
    <label className="mb-2 flex cursor-pointer items-start gap-2 text-sm">
      <input
        type="checkbox"
        className="mt-1 accent-[var(--color-accent)]"
        checked={props.checked}
        onChange={(e) => props.onChange(e.target.checked)}
      />
      <span>
        {props.label}
        {props.hint && <span className="block text-xs text-muted">{props.hint}</span>}
      </span>
    </label>
  );
}

/** Toast global, cliquable pour fermer. */
export function Toast() {
  const toast = useToastValue();
  const setToast = useToastSetter();
  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 5000);
    return () => clearTimeout(t);
  }, [toast, setToast]);
  if (!toast) return null;
  return (
    <div
      className="fixed bottom-4 left-1/2 z-[60] max-w-[70vw] -translate-x-1/2 rounded-lg bg-ink px-4 py-2 text-sm text-white shadow-xl"
      onClick={() => setToast(null)}
    >
      {toast}
    </div>
  );
}

import { useApp } from "../stores";
function useToastValue() {
  return useApp((s) => s.toast);
}
function useToastSetter() {
  return useApp((s) => s.setToast);
}

/** prompt() natif indisponible dans la webview : petit modal de saisie. */
export function Prompt(props: {
  title: string;
  initial: string;
  onSubmit: (v: string) => void;
  onClose: () => void;
}) {
  const [v, setV] = useState(props.initial);
  return (
    <Modal title={props.title} onClose={props.onClose} width="380px">
      <TextInput autoFocus value={v} onChange={setV} onEnter={() => v.trim() && props.onSubmit(v.trim())} />
      <div className="mt-2 flex justify-end gap-2">
        <Button kind="ghost" onClick={props.onClose}>Annuler</Button>
        <Button disabled={!v.trim()} onClick={() => props.onSubmit(v.trim())}>Valider</Button>
      </div>
    </Modal>
  );
}

/** confirm() natif indisponible : modal de confirmation. */
export function Confirm(props: {
  title: string;
  body: ReactNode;
  danger?: boolean;
  onYes: () => void;
  onClose: () => void;
}) {
  return (
    <Modal title={props.title} onClose={props.onClose} width="420px">
      <div className="mb-4 text-sm">{props.body}</div>
      <div className="flex justify-end gap-2">
        <Button kind="ghost" onClick={props.onClose}>Annuler</Button>
        <Button kind={props.danger ? "danger" : "primary"} onClick={props.onYes}>Confirmer</Button>
      </div>
    </Modal>
  );
}