use crate::db;
use crate::error::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Contexte de la vue affichée (fixé par le frontend via `set_front_view`) :
/// permet de placer les fichiers glissés dans le cours ou le dossier ouvert.
pub static FRONT_DOC: Mutex<Option<String>> = Mutex::new(None);
pub static FRONT_FOLDER: Mutex<Option<String>> = Mutex::new(None);

pub fn set_front_view(folder_id: Option<String>, doc_id: Option<String>) {
    *FRONT_FOLDER.lock().unwrap_or_else(|e| e.into_inner()) = folder_id;
    *FRONT_DOC.lock().unwrap_or_else(|e| e.into_inner()) = doc_id;
}

pub fn media_type_of(name: &str, mime: Option<&str>) -> &'static str {
    let ext = Path::new(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "avif" | "tif" | "tiff" | "heic" | "heif" => {
            "image"
        }
        "pdf" => "pdf",
        "docx" => "docx",
        "pptx" => "pptx",
        "md" | "markdown" => "md",
        "txt" => "txt",
        _ => match mime.unwrap_or("") {
            m if m.starts_with("image/") => "image",
            "application/pdf" => "pdf",
            _ => "other",
        },
    }
}

fn guess_mime(name: &str) -> Option<&'static str> {
    let ext = Path::new(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    Some(match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        _ => return None,
    })
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || (c as u32) < 32
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim().to_string();
    let cleaned = if cleaned.is_empty() { "fichier".to_string() } else { cleaned };
    cleaned.chars().take(100).collect()
}

fn unique_name(dir: &Path, name: &str) -> String {
    let p = Path::new(name);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "fichier".into());
    let ext = p
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut candidate = if ext.is_empty() { stem.clone() } else { format!("{stem}.{ext}") };
    let mut i = 2;
    while dir.join(&candidate).exists() {
        candidate = if ext.is_empty() {
            format!("{stem}-{i}")
        } else {
            format!("{stem}-{i}.{ext}")
        };
        i += 1;
    }
    candidate
}

/* ---------- DTO ---------- */

pub const SUPPORT_SELECT: &str = "SELECT s.id, s.doc_id, s.file_name, s.media_type, s.mime, s.size, \
     s.rel_path, s.created_at, \
     (SELECT COUNT(*) FROM transcriptions t WHERE t.support_id = s.id) \
     FROM supports s";

pub fn map_support(r: &rusqlite::Row) -> rusqlite::Result<db::SupportDto> {
    let has_tr: i64 = r.get(8)?;
    Ok(db::SupportDto {
        id: r.get(0)?,
        doc_id: r.get(1)?,
        file_name: r.get(2)?,
        media_type: r.get(3)?,
        mime: r.get(4)?,
        size: r.get(5)?,
        rel_path: r.get(6)?,
        created_at: r.get(7)?,
        has_transcription: has_tr > 0,
    })
}

pub fn support_dto_by_id(conn: &Connection, id: &str) -> Result<db::SupportDto> {
    let sql = format!("{SUPPORT_SELECT} WHERE s.id = ?1");
    conn.query_row(&sql, [id], map_support).map_err(Into::into)
}

pub fn create_doc_row(
    conn: &Connection,
    files_dir: &Path,
    folder_id: &str,
    title: &str,
) -> Result<db::DocDto> {
    let id = db::new_id();
    let t = db::now();
    conn.execute(
        "INSERT INTO docs(id, folder_id, title, status, created_at, updated_at) \
         VALUES(?1, ?2, ?3, 'importe', ?4, ?4)",
        params![id, folder_id, sanitize(title), t, t],
    )?;
    fs::create_dir_all(files_dir.join("uploads").join(&id))?;
    doc_dto_by_id(conn, &id)
}

pub const DOC_SELECT: &str = "SELECT d.id, d.folder_id, d.title, d.exam_date, d.excluded, \
     d.use_transcription, d.status, d.created_at, d.updated_at, \
     (SELECT COUNT(*) FROM supports s WHERE s.doc_id = d.id), \
     (SELECT COUNT(*) FROM questions q WHERE q.doc_id = d.id), 0 \
     FROM docs d";

pub fn map_doc(r: &rusqlite::Row) -> rusqlite::Result<db::DocDto> {
    Ok(db::DocDto {
        id: r.get(0)?,
        folder_id: r.get(1)?,
        title: r.get(2)?,
        exam_date: r.get(3)?,
        excluded: r.get::<_, i64>(4)? != 0,
        use_transcription: r.get::<_, i64>(5)? != 0,
        status: r.get(6)?,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
        support_count: r.get(9)?,
        question_count: r.get(10)?,
        due_count: r.get(11)?,
    })
}

pub fn doc_dto_by_id(conn: &Connection, id: &str) -> Result<db::DocDto> {
    let sql = format!("{DOC_SELECT} WHERE d.id = ?1");
    conn.query_row(&sql, [id], map_doc).map_err(Into::into)
}

/* ---------- Import ---------- */

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub created: Vec<db::SupportDto>,
    pub created_docs: Vec<db::DocDto>,
    pub duplicates: usize,
    pub skipped: Vec<String>,
}

fn folder_exists(conn: &Connection, id: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE id = ?1",
        [id],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Cours destination : FRONT_DOC > FRONT_FOLDER (nouveau cours) > Racine.
fn resolve_target_doc(
    conn: &Connection,
    files_dir: &Path,
) -> Result<(String, Option<db::DocDto>)> {
    let front = FRONT_DOC.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(id) = front {
        let exists: bool = conn
            .query_row("SELECT COUNT(*) FROM docs WHERE id = ?1", [&id], |r| r.get::<_, i64>(0))
            .unwrap_or(0)
            > 0;
        if exists {
            return Ok((id, None));
        }
    }
    let folder = {
        let f = FRONT_FOLDER.lock().unwrap_or_else(|e| e.into_inner()).clone();
        f.filter(|id| folder_exists(conn, id)).unwrap_or_else(|| "root".into())
    };
    let doc = create_doc_row(conn, files_dir, &folder, "Nouveau cours")?;
    Ok((doc.id.clone(), Some(doc)))
}

/// Copie les fichiers dans le dépôt et les rattache à un cours.
/// Tous les fichiers d'un même dépôt vont dans le même cours (cas 2 images + 1 PDF).
pub fn import_paths(conn: &Connection, app_data: &Path, paths: &[String]) -> Result<ImportReport> {
    let files_dir = app_data.join("files");
    let mut report = ImportReport::default();
    let mut target: Option<String> = None;
    for raw in paths {
        let src = Path::new(raw);
        let fname = match src.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => {
                report.skipped.push(raw.clone());
                continue;
            }
        };
        if !src.is_file() {
            report.skipped.push(fname);
            continue;
        }
        let bytes = fs::read(src)?;
        let digest: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        let dup: i64 = conn
            .query_row("SELECT COUNT(*) FROM supports WHERE hash = ?1", [&digest], |r| r.get(0))
            .unwrap_or(0);
        if dup > 0 {
            report.duplicates += 1;
            continue;
        }
        let doc_id = match target.clone() {
            Some(d) => d,
            None => {
                let (doc_id, created) = resolve_target_doc(conn, &files_dir)?;
                if let Some(d) = created {
                    report.created_docs.push(d);
                }
                target = Some(doc_id.clone());
                doc_id
            }
        };
        let doc_dir = files_dir.join("uploads").join(&doc_id);
        fs::create_dir_all(&doc_dir)?;
        let stored = unique_name(&doc_dir, &sanitize(&fname));
        fs::write(doc_dir.join(&stored), &bytes)?;
        let rel = format!("files/uploads/{doc_id}/{stored}");
        let id = db::new_id();
        let sort: i64 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM supports WHERE doc_id = ?1",
            [&doc_id],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO supports(id, doc_id, rel_path, file_name, media_type, mime, size, hash, sort_order, created_at) \
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id, doc_id, rel, fname,
                media_type_of(&stored, guess_mime(&stored)),
                guess_mime(&stored),
                bytes.len() as i64, digest, sort, db::now()
            ],
        )?;
        db::touch_doc(conn, &doc_id)?;
        report.created.push(support_dto_by_id(conn, &id)?);
    }
    Ok(report)
}

/// Supprime le dossier d'upload d'un cours (après suppression en BDD).
pub fn delete_upload_dir(app_data: &Path, doc_id: &str) {
    let dir: PathBuf = app_data.join("files").join("uploads").join(doc_id);
    let _ = fs::remove_dir_all(dir);
}