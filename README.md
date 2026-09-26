# WLarp

Application de bureau (Windows, Tauri 2 + React) pour réviser des cours avec la **répétition espacée** (SM-2), assistée par IA via **OpenRouter**.

> Importer ses cours (PDF, images, Word, PowerPoint) → conversion en markdown → banques de questions générées par IA → un clic = une session de 10 questions choisies automatiquement (points faibles + examens proches prioritaires) → on recommence.

## Fonctionnalités prévues

- 📁 **Gestionnaire de cours** : arborescence de dossiers ; un « cours » = groupe de supports (2 photos + 1 PDF = 1 cours) ; exclusion optionnelle ; date d'examen optionnelle → priorité de révision.
- 🔄 **Conversion en markdown** : docx/pptx/pdf localement (sans IA) ; images transcrits par modèle vision OpenRouter (option activable). Bascule par cours « transcription / original » (les graphiques ne sont jamais remplacés d'office).
- 🧠 **Banques de questions adaptatives** (hybride) : analyse du cours (inventaire des connaissances + types suggérés) → nombre de questions **dynamique** (densité, volume, proximité examen, plafond réglable, ambition économe→riche) → génération ; renfort régénéré pendant les sessions sur les thèmes faibles, puis mis en cache.
- ✍️ **4 formats de questions** : QCM · développée (1–4 lignes) · date à précision variable · réponse courte. Correction automatique + auto-notation Anki pour le développé ; tolérances de réponse paramétrables.
- 🎯 **Ciblage des faiblesses** : l'historique d'erreurs alimente un profil par thème qui pondère les tirages.
- 🔔 **Notifications quotidiennes sans processus résident** : tâche planifiée Windows (`schtasks`) lançant `WLarp.exe --remind` (toast + quit en ~1 s). Tray, fenêtre cachée à la fermeture, single-instance qui ramène la fenêtre.
- ⚙️ **IA configurable** : clé OpenRouter dans le trousseau Windows (jamais en clair), modèle « traitement » (vision) et modèle « quiz » distincts, choix limité aux modèles compatibles (métadonnées `/api/v1/models`).

## Développement

Prérequis : Rust (MSVC) + Visual Studio Build Tools (workload Desktop C++), Node 18+.

```bash
npm install
npm run tauri dev      # développement (1ʳᵉ compilation longue)
npm run tauri build    # exécutable + installeur
npx tsc --noEmit       # typecheck frontend
cargo check --manifest-path src-tauri/Cargo.toml
```

## Données & sécurité

- Stockage : `%APPDATA%\com.wlarp.app\` → `data.sqlite` + `files/uploads/<doc_id>/`.
- Schéma : `folders`, `docs` (cours), `supports` (fichiers), `transcriptions`, `analyses`, `questions` (+ champs SM-2), `attempts`, `settings`.
- Clé API : Credential Manager Windows (cible `WLarp.OpenRouter`).
- Zéro appel réseau sans action explicite de l'utilisateur.

## Suivi d'avancement

Plan détaillé validé : [docs/plan.md](docs/plan.md).

| Jalon | Statut | Contenu |
|---|---|---|
| **M1 — Fondations** | 🔄 en cours | Schéma SQLite complet · explorateur dossiers CRUD · drag & drop + copie des fichiers · fusion/split de supports · réglages + clé OpenRouter (trousseau) · test connexion + liste des modèles · tray + single-instance + cache de fenêtre |
| **M2 — Conversion** | ⬜ | docx/pptx/pdf → markdown local (par défaut) · transcription IA images (option activable) · aperçu markdown + visionnage originaux · bascule transcription/original · tolérances de réponse |
| **M3 — Banques de questions** | ⬜ | passe d'analyse (inventaire + types suggérés) · budget dynamique (densité × volume × examen × plafond) · estimation de coût avant génération · revue/édition/suppression des questions · renfort ciblé en session |
| **M4 — Révision** | ⬜ | session de 10 questions (due + retard + urgence examen + faiblesses) · 4 formats de réponse · corrections + SM-2 · sessions à la demande sur dossier/cours · notation développé (auto ou IA) |
| **M5 — Notifications & finitions** | ⬜ | `schtasks` + mode `--remind` (zéro résident) · statistiques légères · import de `wlarp_files/` · nettoyage |

## Choix d'architecture (arbitres validés)

- Génération **hybride** : base pré-générée à l'import + renfort à la volée (0 à N selon le déficit des thèmes faibles).
- **Nombre et types de questions 100 % dynamiques** par défaut, surcharge manuelle possible par cours.
- PDF scannés (sans texte) : hors périmètre v1 — message invitant à passer par la capture écran ; texte extrai­ble sinon.
- Conversions locales **par défaut** (docx/pptx/pdf) ; IA pour images (transcription optionnelle) et quiz.
- Notifications via `schtasks` → aucun processus résident ; tray + single-instance pour ne jamais tuer le process.
- v1 Windows ; réponses développées auto-notées (correction IA optionnelle) ; non gérés : `.odt`, `.odp`, audio/vidéo.

*Projet complet quand : MVP 100 % implémenté, et utilisé 2 semaines sans erreur critique.*
