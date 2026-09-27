//! SQLite local : schéma complet (migration en cascade simple), réglages, helpers.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{msg, AppError, Result};

pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/* ---------- DTO ---------- */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderDto {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub created_at: String,
    pub doc_count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocDto {
    pub id: String,
    pub folder_id: String,
    pub title: String,
    pub exam_date: Option<String>,
    pub excluded: bool,
    pub use_transcription: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub support_count: i64,
    pub question_count: i64,
    pub due_count: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupportDto {
    pub id: String,
    pub doc_id: String,
    pub file_name: String,
    pub media_type: String,
    pub mime: String,
    pub size: i64,
    pub rel_path: String,
    pub created_at: String,
    pub has_transcription: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocDetail {
    pub doc: DocDto,
    pub supports: Vec<SupportDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub vision_model: String,
    pub text_model: String,
    pub daily_target: i64,
    pub new_preview_pct: i64,
    pub max_new_per_day: i64,
    pub session_max: i64,
    pub reinforce_enabled: bool,
    pub reinforce_threshold: i64,
    pub reinforce_max: i64,
    pub session_deadline: String,
    pub notify_time: String,
    pub snooze_times: String,
    pub use_custom_sms: bool,
    pub sms_provider: String,
    pub sms_api_key: String,
    pub sms_api_base: String,
    pub sms_sender_id: String,
    pub sms_from_number: String,
    pub sms_to_number: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            vision_model: "google/gemini-2.5-flash".into(),
            text_model: "anthropic/claude-sonnet-4.5".into(),
            daily_target: 30,
            new_preview_pct: 40,
            max_new_per_day: 20,
            session_max: 40,
            reinforce_enabled: true,
            reinforce_threshold: 80,
            reinforce_max: 3,
            session_deadline: "22:30".into(),
            notify_time: "19:00".into(),
            snooze_times: "20:30,21:30".into(),
            use_custom_sms: false,
            sms_provider: "smsfactor".into(),
            sms_api_key: String::new(),
            sms_api_base: "https://rest.smsfactor.com/sms/1/SMS/Message/SMS".into(),
            sms_sender_id: String::new(),
            sms_from_number: String::new(),
            sms_to_number: String::new(),
        }
    }
}

fn get_str(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .ok()
}

macro_rules! field {
    ($conn:expr, $s:expr, $key:expr, $f:ident) => {
        if let Some(v) = get_str($conn, $key) {
            $s.$f = v;
        }
    };
    (bool $conn:expr, $s:expr, $key:expr, $f:ident) => {
        if let Some(v) = get_str($conn, $key) {
            $s.$f = v == "1";
        }
    };
    (int $conn:expr, $s:expr, $key:expr, $f:ident) => {
        if let Some(v) = get_str($conn, $key) {
            if let Ok(n) = v.parse() {
                $s.$f = n;
            }
        }
    };
}

pub fn load_settings(conn: &Connection) -> Settings {
    let mut s = Settings::default();
    field!(conn, s, "vision_model", vision_model);
    field!(conn, s, "text_model", text_model);
    field!(int conn, s, "daily_target", daily_target);
    field!(int conn, s, "new_preview_pct", new_preview_pct);
    field!(int conn, s, "max_new_per_day", max_new_per_day);
    field!(int conn, s, "session_max", session_max);
    field!(bool conn, s, "reinforce_enabled", reinforce_enabled);
    field!(int conn, s, "reinforce_threshold", reinforce_threshold);
    field!(int conn, s, "reinforce_max", reinforce_max);
    field!(conn, s, "session_deadline", session_deadline);
    field!(conn, s, "notify_time", notify_time);
    field!(conn, s, "snooze_times", snooze_times);
    field!(bool conn, s, "use_custom_sms", use_custom_sms);
    field!(conn, s, "sms_provider", sms_provider);
    field!(conn, s, "sms_api_key", sms_api_key);
    field!(conn, s, "sms_api_base", sms_api_base);
    field!(conn, s, "sms_sender_id", sms_sender_id);
    field!(conn, s, "sms_from_number", sms_from_number);
    field!(conn, s, "sms_to_number", sms_to_number);
    s
}

pub fn save_settings(conn: &Connection, s: &Settings) -> rusqlite::Result<()> {
    fn flag(b: bool) -> String {
        if b { "1".into() } else { "0".into() }
    }
    let rows: Vec<(&str, String)> = vec![
        ("vision_model", s.vision_model.clone()),
        ("text_model", s.text_model.clone()),
        ("daily_target", s.daily_target.to_string()),
        ("new_preview_pct", s.new_preview_pct.to_string()),
        ("max_new_per_day", s.max_new_per_day.to_string()),
        ("session_max", s.session_max.to_string()),
        ("reinforce_enabled", flag(s.reinforce_enabled)),
        ("reinforce_threshold", s.reinforce_threshold.to_string()),
        ("reinforce_max", s.reinforce_max.to_string()),
        ("session_deadline", s.session_deadline.clone()),
        ("notify_time", s.notify_time.clone()),
        ("snooze_times", s.snooze_times.clone()),
        ("use_custom_sms", flag(s.use_custom_sms)),
        ("sms_provider", s.sms_provider.clone()),
        ("sms_api_key", s.sms_api_key.clone()),
        ("sms_api_base", s.sms_api_base.clone()),
        ("sms_sender_id", s.sms_sender_id.clone()),
        ("sms_from_number", s.sms_from_number.clone()),
        ("sms_to_number", s.sms_to_number.clone()),
    ];
    for (k, v) in rows {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES(?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )?;
    }
    Ok(())
}
/* ---------- Schéma ---------- */

const SCHEMA: &str = r#"
CREATE TABLE folders (
    id TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES folders(id),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE docs (
    id TEXT PRIMARY KEY,
    folder_id TEXT NOT NULL REFERENCES folders(id),
    title TEXT NOT NULL,
    exam_date TEXT,
    excluded INTEGER NOT NULL DEFAULT 0,
    use_transcription INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'importe',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE supports (
    id TEXT PRIMARY KEY,
    doc_id TEXT NOT NULL REFERENCES docs(id),
    rel_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    media_type TEXT NOT NULL,
    mime TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    hash TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_supports_doc ON supports(doc_id);
CREATE TABLE transcriptions (
    id TEXT PRIMARY KEY,
    support_id TEXT NOT NULL REFERENCES supports(id),
    engine TEXT NOT NULL,
    text TEXT NOT NULL,
    model TEXT NOT NULL DEFAULT '',
    lang TEXT NOT NULL DEFAULT '',
    duration_s REAL NOT NULL DEFAULT 0,
    cost_usd REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX uniq_tr_support ON transcriptions(support_id, engine);
CREATE TABLE images (
    support_id TEXT PRIMARY KEY REFERENCES supports(id),
    data BLOB NOT NULL,
    mime TEXT NOT NULL DEFAULT 'image/png',
    source TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);
CREATE TABLE questions (
    id TEXT PRIMARY KEY,
    doc_id TEXT NOT NULL REFERENCES docs(id),
    front_md TEXT NOT NULL,
    back_md TEXT NOT NULL,
    question_type TEXT NOT NULL DEFAULT 'flashcard',
    options_json TEXT NOT NULL DEFAULT '[]',
    answer_json TEXT NOT NULL DEFAULT '{}',
    support_id TEXT,
    support_bbox TEXT,
    source TEXT NOT NULL DEFAULT 'ia',
    ai_model TEXT NOT NULL DEFAULT '',
    srs_state TEXT NOT NULL DEFAULT 'new',
    srs_ease REAL NOT NULL DEFAULT 2.5,
    srs_interval REAL NOT NULL DEFAULT 0,
    srs_due_at TEXT NOT NULL DEFAULT '',
    srs_reps INTEGER NOT NULL DEFAULT 0,
    srs_lapses INTEGER NOT NULL DEFAULT 0,
    srs_last_rating INTEGER,
    ai_pending INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_questions_doc ON questions(doc_id);
CREATE INDEX idx_questions_due ON questions(srs_due_at);
CREATE TABLE answers (
    id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES questions(id),
    session_id TEXT NOT NULL,
    given_md TEXT NOT NULL DEFAULT '',
    rating INTEGER NOT NULL,
    ai_verdict TEXT,
    ai_expected TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_answers_q ON answers(question_id);
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    scope TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    answered INTEGER NOT NULL DEFAULT 0,
    correct INTEGER NOT NULL DEFAULT 0,
    new_count INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    read_at TEXT
);
"#;

pub const SCHEMA_VERSION: i64 = 1;

fn pragmas() -> [&'static str; 3] {
    [
        "PRAGMA foreign_keys = ON",
        "PRAGMA journal_mode = WAL",
        "PRAGMA busy_timeout = 5000",
    ]
}

pub fn open(app_data: &Path) -> Result<Connection> {
    fs::create_dir_all(app_data.join("files").join("uploads"))?;
    let mut conn = Connection::open(app_data.join("data.sqlite"))?;
    for p in pragmas() {
        conn.execute_batch(p)?;
    }
    open_schema(&mut conn)?;
    Ok(conn)
}

pub fn open_schema(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    )?;
    let current: i64 = conn
        .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| r.get(0))
        .ok()
        .and_then(|v: String| v.parse().ok())
        .unwrap_or(0);
    let tx = conn.transaction()?;
    migrate(&tx, current, SCHEMA_VERSION)?;
    tx.execute(
        "INSERT INTO meta(key, value) VALUES('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![SCHEMA_VERSION.to_string()],
    )?;
    tx.execute(
        "INSERT OR IGNORE INTO folders(id, parent_id, name, created_at) VALUES('root', NULL, 'Racine', ?1)",
        params![now()],
    )?;
    tx.commit()?;
    Ok(())
}

/// Migrations successives ; les numéros manquants sont comblés par le schéma complet.
fn migrate(conn: &Connection, from: i64, to: i64) -> Result<()> {
    if from > to {
        return msg("Base créée par une version plus récente de WLarp.");
    }
    for v in (from + 1)..=to {
        match v {
            1 => conn.execute_batch(SCHEMA).map_err(|e| {
                AppError(format!("Initialisation de la base impossible : {e}"))
            })?,
            _ => {}
        }
    }
    Ok(())
}
/* ---------- Requêtes utilitaires ---------- */

pub fn touch_doc(conn: &Connection, doc_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE docs SET updated_at = ?2, status = CASE WHEN status = 'new' THEN 'importe' ELSE status END WHERE id = ?1",
        params![doc_id, now()],
    )?;
    Ok(())
}

pub fn due_count(conn: &Connection) -> rusqlite::Result<i64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM questions q JOIN docs d ON d.id = q.doc_id
         WHERE d.excluded = 0 AND q.srs_state != 'new' AND q.srs_due_at <= ?1",
        [&now()],
        |r| r.get(0),
    )?;
    Ok(n)
}

/// Supprime un cours en cascade : réponses → questions → transcriptions/images → supports → doc.
pub fn doc_delete_children(conn: &Connection, doc_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM answers WHERE question_id IN (SELECT id FROM questions WHERE doc_id = ?1)",
        [doc_id],
    )?;
    conn.execute("DELETE FROM questions WHERE doc_id = ?1", [doc_id])?;
    conn.execute(
        "DELETE FROM transcriptions WHERE support_id IN (SELECT id FROM supports WHERE doc_id = ?1)",
        [doc_id],
    )?;
    conn.execute(
        "DELETE FROM images WHERE support_id IN (SELECT id FROM supports WHERE doc_id = ?1)",
        [doc_id],
    )?;
    conn.execute("DELETE FROM supports WHERE doc_id = ?1", [doc_id])?;
    Ok(())
}

/// Mode « Tout supprimer » : supprime les dossiers (sous-arbre) ET leur contenu.
/// Ordre : réponses → questions → transcriptions/images → supports → docs → folders.
/// Retourne les ids des cours supprimés pour que l'appelant purge les fichiers.
pub fn delete_folders_content(conn: &Connection, subtree: &[String]) -> rusqlite::Result<Vec<String>> {
    let placeholders = subtree.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let doc_ids: Vec<String> = conn
        .prepare(&format!("SELECT id FROM docs WHERE folder_id IN ({placeholders})"))?
        .query_map(rusqlite::params_from_iter(subtree.iter()), |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for d in &doc_ids {
        doc_delete_children(conn, d)?;
    }
    conn.execute(
        &format!("DELETE FROM docs WHERE folder_id IN ({placeholders})"),
        rusqlite::params_from_iter(subtree.iter()),
    )?;
    conn.execute(
        &format!("DELETE FROM folders WHERE id IN ({placeholders})"),
        rusqlite::params_from_iter(subtree.iter()),
    )?;
    Ok(doc_ids)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_creates_all_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        for p in pragmas() {
            conn.execute_batch(p).unwrap();
        }
        open_schema(&mut conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(count >= 11, "tables manquantes ({count})");
        let root: i64 = conn
            .query_row("SELECT COUNT(*) FROM folders WHERE id='root'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(root, 1);
    }

    #[test]
    fn idempotent_reopen() {
        let mut conn = Connection::open_in_memory().unwrap();
        open_schema(&mut conn).unwrap();
        conn.execute("INSERT INTO folders VALUES('a','root','Cours 1','2026')", []).unwrap();
        open_schema(&mut conn).unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn fk_cascade_blocks_orphan_support() {
        let mut conn = Connection::open_in_memory().unwrap();
        for p in pragmas() {
            conn.execute_batch(p).unwrap();
        }
        open_schema(&mut conn).unwrap();
        let r = conn.execute(
            "INSERT INTO supports(id, doc_id, rel_path, file_name, media_type, mime, size, hash, sort_order, created_at)
             VALUES('s','inexistant','x','f','pdf','application/pdf',1,'h',0,'t')",
            [],
        );
        assert!(r.is_err());
    }

    #[test]
    fn delete_folders_content_cascades_all_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        for p in pragmas() {
            conn.execute_batch(p).unwrap();
        }
        open_schema(&mut conn).unwrap();
        conn.execute("INSERT INTO folders VALUES('f1','root','L2','t')", []).unwrap();
        conn.execute("INSERT INTO folders VALUES('f2','f1','L3','t')", []).unwrap();
        conn.execute(
            "INSERT INTO docs(id,folder_id,title,status,created_at,updated_at) VALUES('d1','f2','C1','importe','t','t')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO supports(id,doc_id,rel_path,file_name,media_type,mime,size,hash,sort_order,created_at) VALUES('s1','d1','r','f','pdf','application/pdf',1,'h',0,'t')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO questions(id,doc_id,front_md,back_md,created_at,updated_at) VALUES('q1','d1','q','a','t','t')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO answers(id,question_id,session_id,given_md,rating,created_at) VALUES('a1','q1','sess','',4,'t')",
            [],
        )
        .unwrap();
        let deleted = delete_folders_content(&conn, &["f1".into(), "f2".into()]).unwrap();
        assert_eq!(deleted, vec!["d1".to_string()]);
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM folders WHERE id != 'root'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "dossiers restants");
        for t in ["docs", "supports", "questions", "answers"] {
            let n: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| r.get(0)).unwrap();
            assert_eq!(n, 0, "lignes restantes dans {t}");
        }
    }


    #[test]
    fn settings_roundtrip() {
        let mut conn = Connection::open_in_memory().unwrap();
        open_schema(&mut conn).unwrap();
        let mut s = Settings::default();
        s.text_model = "meta/llama-4".into();
        s.daily_target = 50;
        s.reinforce_enabled = false;
        save_settings(&conn, &s).unwrap();
        let back = load_settings(&conn);
        assert_eq!(back.text_model, "meta/llama-4");
        assert_eq!(back.daily_target, 50);
        assert!(!back.reinforce_enabled);
        assert_eq!(back.notify_time, "19:00");
    }
}
