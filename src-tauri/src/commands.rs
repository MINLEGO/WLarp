use crate::db;
use crate::error::{msg, AppError, Result};
use crate::files;
use crate::keychain;
use crate::state::AppState;
use rusqlite::params;
use tauri::{AppHandle, Manager, State};

fn lock(state: &AppState) -> std::sync::MutexGuard<'_, rusqlite::Connection> {
    state.db.lock().unwrap_or_else(|e| e.into_inner())
}

/* ---------- Divers ---------- */

#[tauri::command]
pub fn app_info(state: State<AppState>) -> Result<serde_json::Value> {
    let conn = lock(&state);
    Ok(serde_json::json!({
        "appDataDir": state.app_data.display().to_string(),
        "dueCount": db::due_count(&conn)?,
    }))
}

/// Fixe le contexte de destination pour les imports (vue affichée).
#[tauri::command]
pub fn set_front_view(folder_id: Option<String>, doc_id: Option<String>) {
    files::set_front_view(folder_id, doc_id);
}

/// Chemin absolu d'un support (pour `convertFileSrc` côté frontend).
#[tauri::command]
pub fn support_abs_path(id: String, state: State<AppState>) -> Result<String> {
    let conn = lock(&state);
    let rel: String = conn.query_row("SELECT rel_path FROM supports WHERE id = ?1", [&id], |r| r.get(0))?;
    Ok(state.app_data.join(rel).display().to_string())
}

/* ---------- Dossiers ---------- */

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> Result<Vec<db::FolderDto>> {
    let conn = lock(&state);
    let mut stmt = conn.prepare(
        "SELECT f.id, f.parent_id, f.name, f.created_at,
                (SELECT COUNT(*) FROM docs d WHERE d.folder_id = f.id)
         FROM folders f ORDER BY f.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(db::FolderDto {
            id: r.get(0)?,
            parent_id: r.get(1)?,
            name: r.get(2)?,
            created_at: r.get(3)?,
            doc_count: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn create_folder(
    parent_id: String,
    name: String,
    state: State<AppState>,
) -> Result<db::FolderDto> {
    let conn = lock(&state);
    let id = db::new_id();
    conn.execute(
        "INSERT INTO folders(id, parent_id, name, created_at) VALUES(?1, ?2, ?3, ?4)",
        params![id, parent_id, name.trim(), db::now()],
    )?;
    conn.query_row(
        "SELECT id, parent_id, name, created_at, 0 FROM folders WHERE id = ?1",
        [&id],
        |r| {
            Ok(db::FolderDto {
                id: r.get(0)?,
                parent_id: r.get(1)?,
                name: r.get(2)?,
                created_at: r.get(3)?,
                doc_count: 0,
            })
        },
    )
    .map_err(Into::into)
}

#[tauri::command]
pub fn rename_folder(id: String, name: String, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    conn.execute("UPDATE folders SET name = ?2 WHERE id = ?1", params![id, name.trim()])?;
    Ok(())
}

fn collect_subtree(conn: &rusqlite::Connection, root: &str) -> Result<Vec<String>> {
    let mut out = vec![root.to_string()];
    let mut queue = vec![root.to_string()];
    while let Some(cur) = queue.pop() {
        let mut stmt = conn.prepare("SELECT id FROM folders WHERE parent_id = ?1")?;
        let kids: Vec<String> = stmt
            .query_map([&cur], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for k in kids {
            out.push(k.clone());
            queue.push(k);
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn move_folder(id: String, new_parent_id: String, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    if id == "root" {
        return msg("Le dossier racine ne peut pas être déplacé.");
    }
    let subtree = collect_subtree(&conn, &id)?;
    if subtree.contains(&new_parent_id) {
        return msg("Impossible de déplacer un dossier dans l'un de ses descendants.");
    }
    conn.execute("UPDATE folders SET parent_id = ?2 WHERE id = ?1", params![id, new_parent_id])?;
    Ok(())
}

/// mode = "recycle" (contenu remonté au parent) ou "content" (tout supprimé).
#[tauri::command]
pub fn delete_folder(id: String, mode: String, state: State<AppState>) -> Result<()> {
    if id == "root" {
        return msg("Le dossier racine ne peut pas être supprimé.");
    }
    let app_data = state.app_data.clone();
    let conn = lock(&state);
    let subtree = collect_subtree(&conn, &id)?;
    let parent: Option<String> =
        conn.query_row("SELECT parent_id FROM folders WHERE id = ?1", [&id], |r| r.get(0))?;
    let placeholders = subtree.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    if mode == "recycle" {
        let dest = parent.clone().unwrap_or_else(|| "root".into());
        conn.execute(
            &format!("UPDATE folders SET parent_id = ? WHERE parent_id IN ({placeholders})"),
            rusqlite::params_from_iter(std::iter::once(&dest).chain(subtree.iter())),
        )?;
        conn.execute(
            &format!("UPDATE docs SET folder_id = ? WHERE folder_id IN ({placeholders})"),
            rusqlite::params_from_iter(std::iter::once(&dest).chain(subtree.iter())),
        )?;
    } else {
        let doc_ids: Vec<String> = conn
            .prepare(&format!(
                "SELECT id FROM docs WHERE folder_id IN ({placeholders})"
            ))?
            .query_map(rusqlite::params_from_iter(subtree.iter()), |r| r.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        conn.execute(
            &format!("DELETE FROM folders WHERE id IN ({placeholders})"),
            rusqlite::params_from_iter(subtree.iter()),
        )?;
        for d in doc_ids {
            files::delete_upload_dir(&app_data, &d);
        }
    }
    Ok(())
}

/* ---------- Cours ---------- */

#[tauri::command]
pub fn list_docs(folder_id: String, state: State<AppState>) -> Result<Vec<db::DocDto>> {
    let conn = lock(&state);
    let sql = format!("{} WHERE d.folder_id = ?1 ORDER BY d.title COLLATE NOCASE", files::DOC_SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([&folder_id], files::map_doc)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn get_doc(id: String, state: State<AppState>) -> Result<db::DocDetail> {
    let conn = lock(&state);
    let doc = files::doc_dto_by_id(&conn, &id)?;
    let sql = format!("{} WHERE s.doc_id = ?1 ORDER BY s.sort_order", files::SUPPORT_SELECT);
    let mut stmt = conn.prepare(&sql)?;
    let supports = stmt
        .query_map([&id], files::map_support)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(db::DocDetail { doc, supports })
}

#[tauri::command]
pub fn create_doc(
    folder_id: String,
    title: String,
    state: State<AppState>,
) -> Result<db::DocDto> {
    let conn = lock(&state);
    files::create_doc_row(&conn, &state.app_data.join("files"), &folder_id, &title)
}

#[tauri::command]
pub fn rename_doc(id: String, title: String, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    conn.execute("UPDATE docs SET title = ?2 WHERE id = ?1", params![title.trim(), id])?;
    db::touch_doc(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub fn move_doc(id: String, folder_id: String, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    conn.execute("UPDATE docs SET folder_id = ?2 WHERE id = ?1", params![id, folder_id])?;
    db::touch_doc(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub fn set_doc_flags(
    id: String,
    excluded: bool,
    use_transcription: bool,
    state: State<AppState>,
) -> Result<()> {
    let conn = lock(&state);
    conn.execute(
        "UPDATE docs SET excluded = ?2, use_transcription = ?3 WHERE id = ?1",
        params![id, excluded as i64, use_transcription as i64],
    )?;
    db::touch_doc(&conn, &id)?;
    Ok(())
}

/// `date = None` retire la date d'examen.
#[tauri::command]
pub fn set_exam_date(id: String, date: Option<String>, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    conn.execute("UPDATE docs SET exam_date = ?2 WHERE id = ?1", params![id, date])?;
    db::touch_doc(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub fn delete_doc(id: String, state: State<AppState>) -> Result<()> {
    let app_data = state.app_data.clone();
    let conn = lock(&state);
    db::doc_delete_children(&conn, &id)?;
    let n = conn.execute("DELETE FROM docs WHERE id = ?1", [&id])?;
    if n > 0 {
        files::delete_upload_dir(&app_data, &id);
    }
    Ok(())
}

/* ---------- Supports ---------- */

/// Importe des fichiers côté app (drag & drop, sélecteur). Retourne le rapport d'import.
#[tauri::command]
pub fn import_paths(paths: Vec<String>, state: State<AppState>) -> Result<files::ImportReport> {
    let conn = lock(&state);
    files::import_paths(&conn, &state.app_data, &paths)
}

#[tauri::command]
pub fn delete_support(id: String, state: State<AppState>) -> Result<()> {
    let app_data = state.app_data.clone();
    let conn = lock(&state);
    if let Ok((doc_id, rel)) = conn.query_row(
        "SELECT doc_id, rel_path FROM supports WHERE id = ?1",
        [&id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    ) {
        conn.execute("DELETE FROM transcriptions WHERE support_id = ?1", [&id])?;
        conn.execute("DELETE FROM images WHERE support_id = ?1", [&id])?;
        conn.execute("DELETE FROM supports WHERE id = ?1", [&id])?;
        let _ = std::fs::remove_file(app_data.join(rel));
        db::touch_doc(&conn, &doc_id)?;
    }
    Ok(())
}

/// Fusionne des cours : tous les supports de `source_ids` rejoignent `target_id`.
#[tauri::command]
pub fn merge_docs(source_ids: Vec<String>, target_id: String, state: State<AppState>) -> Result<db::DocDto> {
    let app_data = state.app_data.clone();
    let conn = lock(&state);
    let mut sort: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM supports WHERE doc_id = ?1",
        [&target_id],
        |r| r.get(0),
    )?;
    for src in &source_ids {
        if src == &target_id {
            continue;
        }
        conn.execute(
            "UPDATE supports SET doc_id = ?1, sort_order = sort_order + ?3 WHERE doc_id = ?2",
            params![target_id, src, sort],
        )?;
        sort += 10;
    }
    // re-numérotation propre des supports du cours cible
    conn.execute(
        "UPDATE supports SET sort_order = \
           (SELECT COUNT(*) FROM supports x WHERE x.doc_id = ?1 AND x.rowid < supports.rowid) \
         WHERE doc_id = ?1",
        [&target_id],
    )?;
    for src in &source_ids {
        if src != &target_id {
            conn.execute("DELETE FROM docs WHERE id = ?1", [src])?;
            files::delete_upload_dir(&app_data, src);
        }
    }
    db::touch_doc(&conn, &target_id)?;
    files::doc_dto_by_id(&conn, &target_id)
}

/// Sépare un support dans un nouveau cours (même dossier).
#[tauri::command]
pub fn split_support(support_id: String, title: String, state: State<AppState>) -> Result<db::DocDto> {
    let conn = lock(&state);
    let (old_doc, file_name): (String, String) = conn.query_row(
        "SELECT doc_id, file_name FROM supports WHERE id = ?1",
        [&support_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let folder_id: String =
        conn.query_row("SELECT folder_id FROM docs WHERE id = ?1", [&old_doc], |r| r.get(0))?;
    let new_doc = files::create_doc_row(
        &conn,
        &state.app_data.join("files"),
        &folder_id,
        if title.trim().is_empty() { &file_name } else { &title },
    )?;
    conn.execute(
        "UPDATE supports SET doc_id = ?2 WHERE id = ?1",
        params![support_id, new_doc.id],
    )?;
    db::touch_doc(&conn, &old_doc)?;
    files::doc_dto_by_id(&conn, &new_doc.id)
}

/* ---------- Réglages & clé API ---------- */

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<db::Settings> {
    let conn = lock(&state);
    Ok(db::load_settings(&conn))
}

#[tauri::command]
pub fn set_settings(settings: db::Settings, state: State<AppState>) -> Result<()> {
    let conn = lock(&state);
    db::save_settings(&conn, &settings)?;
    Ok(())
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<()> {
    keychain::set_secret("OpenRouter", key.trim()).map_err(AppError)
}

#[tauri::command]
pub fn has_api_key() -> bool {
    keychain::get_secret("OpenRouter").is_some()
}

#[tauri::command]
pub fn delete_api_key() {
    keychain::delete_secret("OpenRouter");
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub vision: bool,
    pub context_length: i64,
}

/// Liste les modèles OpenRouter (triés par id), avec le drapeau vision.
#[tauri::command]
pub fn list_models() -> Result<Vec<ModelInfo>> {
    let key = keychain::get_secret("OpenRouter").ok_or_else(|| AppError("Aucune clé API enregistrée.".into()))?;
    let resp = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError(e.to_string()))?
        .get("https://openrouter.ai/api/v1/models")
        .header("Authorization", format!("Bearer {key}"))
        .send();
    let resp = match resp {
        Ok(r) => r,
        Err(e) => return msg(format!("Impossible de joindre OpenRouter : {e}")),
    };
    if !resp.status().is_success() {
        return msg(format!("OpenRouter a répondu {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().map_err(|e| AppError(e.to_string()))?;
    let mut out = Vec::new();
    if let Some(arr) = body.get("data").and_then(|d| d.as_array()) {
        for m in arr {
            let id = m.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let name = m
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| id.clone());
            let mods = m
                .pointer("/architecture/input_modalities")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let vision = mods.iter().any(|x| x.as_str() == Some("image"));
            let ctx = m
                .pointer("/context_length")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            out.push(ModelInfo { id, name, vision, context_length: ctx });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/* ---------- Fenêtre (tray) ---------- */

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

#[tauri::command]
pub fn show_window(app: AppHandle) {
    crate::tray::show_main(&app);
}