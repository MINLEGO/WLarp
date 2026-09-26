import { useEffect, useState } from "react";
import { api, errText } from "../lib/ipc";
import type { ModelInfo, Settings } from "../lib/types";
import { useApp } from "../stores";
import { Button, Checkbox, Field, Modal, TextInput } from "./ui";

function Num({
  label,
  value,
  onChange,
  min,
  max,
}: {
  label: string;
  value: number;
  onChange: (n: number) => void;
  min?: number;
  max?: number;
}) {
  return (
    <Field label={label}>
      <input
        type="number"
        className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm outline-none focus:border-accent"
        value={value}
        min={min}
        max={max}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </Field>
  );
}

export default function SettingsModal({ onClose }: { onClose: () => void }) {
  const settings = useApp((s) => s.settings);
  const hasKey = useApp((s) => s.hasKey);
  const loadSettings = useApp((s) => s.loadSettings);
  const setToast = useApp((s) => s.setToast);
  const [s, setS] = useState<Settings | null>(settings);
  const [key, setKey] = useState("");
  const [models, setModels] = useState<ModelInfo[] | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);
  const [tab, setTab] = useState<"api" | "study" | "sms">("api");

  useEffect(() => setS(settings), [settings]);
  useEffect(() => {
    if (hasKey && models === null) loadModels();
  }, [hasKey]); // eslint-disable-line

  async function loadModels() {
    setLoadingModels(true);
    try {
      setModels(await api.listModels());
    } catch (e) {
      setToast(errText(e));
      setModels([]);
    } finally {
      setLoadingModels(false);
    }
  }

  async function saveKey() {
    if (!key.trim()) return;
    try {
      await api.setApiKey(key.trim());
      setKey("");
      await loadSettings();
      loadModels();
      setToast("Clé API enregistrée 🔑");
    } catch (e) {
      setToast(errText(e));
    }
  }

  async function save(patch: Partial<Settings>) {
    if (!s) return;
    const next = { ...s, ...patch };
    setS(next);
    try {
      await api.setSettings(next);
      await loadSettings();
    } catch (e) {
      setToast(errText(e));
    }
  }

  if (!s) return null;
  const visionOpts = models?.filter((m) => m.vision) ?? [];

  return (
    <Modal title="Réglages" onClose={onClose} width="600px">
      <div className="mb-4 flex gap-1 border-b border-line">
        {(
          [
            ["api", "🔑 API & modèles"],
            ["study", "📆 Révisions"],
            ["sms", "📩 Notification SMS"],
          ] as const
        ).map(([k, l]) => (
          <button
            key={k}
            className={`rounded-t-lg px-3 py-1.5 text-sm ${tab === k ? "border-b-2 border-accent font-medium" : "text-muted hover:bg-surface-2"}`}
            onClick={() => setTab(k)}
          >
            {l}
          </button>
        ))}
      </div>

      {tab === "api" && (
        <div>
          {hasKey ? (
            <div className="mb-4 flex items-center gap-2">
              <span className="text-sm">✅ Clé OpenRouter enregistrée (Windows keychain)</span>
              <div className="ml-auto">
                <Button
                  kind="ghost"
                  onClick={async () => {
                    await api.deleteApiKey();
                    await loadSettings();
                    setModels(null);
                    setToast("Clé supprimée");
                  }}
                >
                  Supprimer
                </Button>
              </div>
            </div>
          ) : (
            <Field label="Clé API OpenRouter" hint="https://openrouter.ai/settings/keys">
              <div className="flex gap-2">
                <TextInput
                  type="password"
                  value={key}
                  onChange={setKey}
                  placeholder="sk-or-v1-…"
                />
                <Button disabled={!key.trim()} onClick={saveKey}>
                  Enregistrer
                </Button>
              </div>
            </Field>
          )}

          {hasKey && (
            <>
              <Field label="Modèle vision (PDF/images → Markdown)" hint={loadingModels ? "chargement…" : `${visionOpts.length} modèles`}>
                <ModelSelect
                  models={models}
                  value={s.visionModel}
                  onlyVision
                  onChange={(v) => save({ visionModel: v })}
                />
              </Field>
              <Field label="Modèle texte (questions, rappels)">
                <ModelSelect models={models} value={s.textModel} onChange={(v) => save({ textModel: v })} />
              </Field>
            </>
          )}
        </div>
      )}
      {tab === "study" && (
        <div>
          <div className="grid grid-cols-2 gap-x-4">
            <Num label="Objectif / jour (cartes)" value={s.dailyTarget} min={1} onChange={(v) => save({ dailyTarget: v })} />
            <Num label="Nouvelles max / jour" value={s.maxNewPerDay} min={0} onChange={(v) => save({ maxNewPerDay: v })} />
            <Num label="Durée max session (min)" value={s.sessionMax} min={5} onChange={(v) => save({ sessionMax: v })} />
            <Num label="% aperçu nouvelles cartes" value={s.newPreviewPct} min={0} max={100} onChange={(v) => save({ newPreviewPct: v })} />
          </div>
          <Checkbox
            checked={s.reinforceEnabled}
            onChange={(v) => save({ reinforceEnabled: v })}
            label="Renforcement en fin de session"
            hint="Rappeler les cartes les plus fragiles tant que le score est sous le seuil"
          />
          {s.reinforceEnabled && (
            <div className="grid grid-cols-2 gap-x-4">
              <Num label="Seuil score (%)" value={s.reinforceThreshold} min={50} max={100} onChange={(v) => save({ reinforceThreshold: v })} />
              <Num label="Passes max" value={s.reinforceMax} min={1} max={6} onChange={(v) => save({ reinforceMax: v })} />
            </div>
          )}
          <div className="grid grid-cols-2 gap-x-4">
            <Field label="Heure limite quotidienne" hint="au-delà, session allégée">
              <TextInput value={s.sessionDeadline} onChange={(v) => save({ sessionDeadline: v })} />
            </Field>
            <Field label="Heure de notification">
              <TextInput value={s.notifyTime} onChange={(v) => save({ notifyTime: v })} />
            </Field>
          </div>
          <Field label="Rappels (snooze)" hint="heures séparées par des virgules">
            <TextInput value={s.snoozeTimes} onChange={(v) => save({ snoozeTimes: v })} />
          </Field>
        </div>
      )}

      {tab === "sms" && (
        <div>
          <Checkbox
            checked={s.useCustomSms}
            onChange={(v) => save({ useCustomSms: v })}
            label="Utiliser un fournisseur SMS personnalisé"
            hint="Sinon : Smsfactor (par défaut)"
          />
          <Field label="Fournisseur">
            <select
              className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm"
              value={s.smsProvider}
              onChange={(e) => save({ smsProvider: e.target.value })}
            >
              <option value="smsfactor">Smsfactor</option>
              <option value="other">Autre (endpoint custom)</option>
            </select>
          </Field>
          <Field label="Clé API SMS" hint="stockée en clair pour l'instant — keychain en M2">
            <TextInput type="password" value={s.smsApiKey} onChange={(v) => save({ smsApiKey: v })} />
          </Field>
          <Field label="Endpoint (si autre)">
            <TextInput value={s.smsApiBase} onChange={(v) => save({ smsApiBase: v })} />
          </Field>
          <div className="grid grid-cols-2 gap-x-4">
            <Field label="Sender ID">
              <TextInput value={s.smsSenderId} onChange={(v) => save({ smsSenderId: v })} />
            </Field>
            <Field label="De (numéro)">
              <TextInput value={s.smsFromNumber} onChange={(v) => save({ smsFromNumber: v })} />
            </Field>
          </div>
          <Field label="Numéro destination (toi)">
            <TextInput value={s.smsToNumber} onChange={(v) => save({ smsToNumber: v })} />
          </Field>
        </div>
      )}
    </Modal>
  );
}

function ModelSelect({
  models,
  value,
  onChange,
  onlyVision,
}: {
  models: ModelInfo[] | null;
  value: string;
  onChange: (v: string) => void;
  onlyVision?: boolean;
}) {
  const list = (models ?? []).filter((m) => !onlyVision || m.vision);
  const known = list.some((m) => m.id === value);
  return (
    <>
      <select
        className="w-full rounded-lg border border-line bg-surface px-3 py-2 text-sm"
        value={known ? value : "__custom"}
        onChange={(e) => e.target.value !== "__custom" && onChange(e.target.value)}
      >
        {models === null && <option value={value}>{value}</option>}
        {models !== null && !known && <option value="__custom">{value} (hors liste)</option>}
        {list.map((m) => (
          <option key={m.id} value={m.id}>
            {m.name} ({m.id})
          </option>
        ))}
      </select>
      <TextInput value={value} onChange={onChange} placeholder="ou saisir un id manuellement" />
    </>
  );
}