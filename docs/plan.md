

Très bien — c'est une évolution importante du point « Génération des questions ». Voici comment je l'intègre au plan (à la place du « ~15 questions, mix fixe des 4 types »).

## Questions à nombre **et** types dynamiques (par analyse du contenu)

### Pipeline en 2 passes (remplace la génération directe)

**Passe 1 — Analyse (rapide, peu coûteuse)** : le markdown du cours est analysé par l'IA qui retourne un *inventaire structuré* :
- **unités de connaissance** détectées (fait daté, définition/nom, relation de cause à effet « pourquoi… », liste/comparaison classable, définition à choix multiples plausibles…) avec le **type de question suggéré** pour chacune ;
- métriques : volume, densité d'information, thèmes principaux ;
- répartition des types **découlant naturellement du contenu** : un cours d'histoire produira beaucoup de « date », un cours de philosophie surtout du « développé », un cours de biologie beaucoup de « courte » + QCM — aucun type n'est imposé ni obligatoire.

**Passe 2 — Budgétisation puis génération** :
```
nb_questions_cible = clamp( f(densité × volume,            ← contenu
                              proximité(exam_date),        ← examen dans 5 j → plus de questions
                              couverture actuelle + faiblesses),  ← rattrapage ciblé
                         min_global , max_global )          ← plafond de coût réglable
```
- puis génération des questions chunk par chunk en suivant l'inventaire ;
- **avant lancement : estimation affichée** « ≈ 23 questions prévues (8 date · 6 courte · 5 QCM · 4 développée) · ≈ X jetons ≈ ≈ Y $ » avec possibilité d'ajuster via un curseur d'ambition : *Économe / Standard / Riche*.
- Surcharge manuelle toujours possible par cours (forcer un nombre ou une répartition), mais le mode par défaut est 100 % dynamique.

**Renfort en session (hybride)** : lui aussi devient dynamique — le nombre de questions régénérées à la volée (0 à N) dépend du déficit réel des thèmes faibles, dans la limite du budget restant.

### Répercussions sur le plan
- Table `questions` inchangée, plus une table `analyses` (inventaire par document, réutilisé pour éviter de repayer l'analyse si on régénère).
- Réglages ajoutés : ambition par défaut (économe→riche), plafond de questions globales/coût, cases « ajuster selon date d'examen » et « ajuster selon faiblesses ».
- Coût IA : la passe d'analyse est bon marché (1 appel court par document) ; le surcoût face au plan précédent est négligeable, et la dynamisation **évite de générer des questions inutiles** sur les documents denses.

### Jalons révisés (seul M3 change)
| # | Contenu |
|---|---|
| M1 | Fondations : schéma SQLite, réglages + clé OpenRouter (trousseau), arborescence CRUD, drag & drop, regroupement/fusion de supports |
| M2 | Conversion : docx/pptx/PDF→md local, transcription IA, aperçu + bascule transcription/original |
| **M3** | **Banque de questions adaptative : passe d'analyse → inventaire + types suggérés → budgétisation dynamique (volume, densité, examen, faiblesses) → génération ; estimation de coût avant lancement ; renfort ciblé lui aussi dynamique ; revue/édition des questions** |
| M4 | Révision : session de 10 questions, 4 formats de réponse, corrections, SM-2, priorités/examens, exclusions, ciblage des faiblesses |
| M5 | Notifications `schtasks` (mode `--remind`, zéro résident), statistiques légères, import de `wlarp_files/`, nettoyage |

Le plan est à jour avec vos deux arbitres : **mode hybride** + **nombre et types de questions dynamiques**. Rien d'autre n'est en suspens de mon côté.

**Basculez en mode Act** quand vous êtes prêt : je commencerai par le **M1 — Fondations** (schéma de base complet, squelette des commandes Rust, écran de réglages avec clé OpenRouter stockée dans le trousseau Windows, explorateur de dossiers avec glisser-déposer et copié des fichiers dans `%APPDATA%\wlarping`).