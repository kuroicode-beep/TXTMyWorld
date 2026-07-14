// core/src/store.rs — SQLite 저장소: PRD §8 테이블 스키마 + 리포지토리 (vec_index는 sqlite-vec 통합 시 확장)
// 원칙: topic_cards만 원본, 나머지는 재조회·재계산으로 복원 가능한 파생 데이터.

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::discovery::{Discovery, DiscoveryType, KeywordRecord};
use crate::topic::{CardStatus, LabelSource, TopicCard};
use crate::Result;

/// 앱 로컬 저장소 (SQLite)
pub struct Store {
    conn: Connection,
}

impl Store {
    /// 파일 경로로 열기 (없으면 생성 + 마이그레이션)
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// 인메모리 저장소 (테스트용)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// 스키마 마이그레이션 — PRD §8 테이블 (vec_index는 sqlite-vec 로드 후 별도 생성)
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL UNIQUE,
                base_url TEXT NOT NULL,
                pairing_token_hash TEXT,
                last_synced_at TEXT
            );
            CREATE TABLE IF NOT EXISTS keyword_cache (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                normalized_text TEXT NOT NULL,
                text TEXT NOT NULL,
                category TEXT,
                frequency REAL NOT NULL DEFAULT 0,
                avg_emotion_score REAL NOT NULL DEFAULT 0,
                first_seen TEXT,
                last_seen TEXT,
                fetched_at TEXT NOT NULL,
                UNIQUE(source, normalized_text)
            );
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                normalized_text TEXT NOT NULL,
                origin TEXT NOT NULL CHECK(origin IN ('local','shared')),
                model TEXT NOT NULL,
                dim INTEGER NOT NULL,
                normalized INTEGER NOT NULL DEFAULT 1,
                vector_blob BLOB NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(source, normalized_text, model)
            );
            CREATE TABLE IF NOT EXISTS clusters (
                id TEXT PRIMARY KEY,
                method TEXT NOT NULL,
                member_embedding_ids_json TEXT NOT NULL,
                centroid_blob BLOB,
                period TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS discoveries (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL CHECK(type IN ('bridge','gap','cluster','drift')),
                members_json TEXT NOT NULL,
                evidence_json TEXT NOT NULL,
                score REAL NOT NULL,
                weak_signal INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'new' CHECK(status IN ('new','dismissed','adopted')),
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS topic_cards (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                label_source TEXT NOT NULL,
                discovery_id TEXT NOT NULL,
                members_json TEXT NOT NULL,
                evidence_snapshot_json TEXT NOT NULL,
                note TEXT,
                status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','confirmed','archived')),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE IF NOT EXISTS feedbacks (
                id INTEGER PRIMARY KEY,
                topic_card_id TEXT NOT NULL,
                target TEXT NOT NULL DEFAULT 'txtaimemory',
                payload_summary TEXT NOT NULL,
                status TEXT NOT NULL,
                memory_id TEXT,
                sent_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_keyword_cache_source ON keyword_cache(source);
            CREATE INDEX IF NOT EXISTS idx_embeddings_key ON embeddings(source, normalized_text);
            CREATE INDEX IF NOT EXISTS idx_discoveries_status ON discoveries(status);
            "#,
        )?;
        Ok(())
    }

    /// 키워드 캐시 upsert (소스 재조회 스냅샷)
    pub fn upsert_keyword_cache(&self, rec: &KeywordRecord) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO keyword_cache (source, normalized_text, text, frequency, avg_emotion_score, first_seen, last_seen, fetched_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
               ON CONFLICT(source, normalized_text) DO UPDATE SET
                 text=excluded.text, frequency=excluded.frequency, avg_emotion_score=excluded.avg_emotion_score,
                 first_seen=excluded.first_seen, last_seen=excluded.last_seen, fetched_at=excluded.fetched_at"#,
            params![
                rec.source.as_str(),
                rec.normalized_text,
                rec.text,
                rec.frequency,
                rec.avg_emotion_score as f64,
                rec.first_seen.map(|d| d.to_string()),
                rec.last_seen.map(|d| d.to_string()),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 임베딩 upsert — f32 슬라이스를 리틀엔디언 BLOB으로 저장
    pub fn upsert_embedding(
        &self,
        source: &str,
        normalized_text: &str,
        origin: &str,
        model: &str,
        vector: &[f32],
    ) -> Result<()> {
        let blob: Vec<u8> = vector.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.conn.execute(
            r#"INSERT INTO embeddings (source, normalized_text, origin, model, dim, normalized, vector_blob, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)
               ON CONFLICT(source, normalized_text, model) DO UPDATE SET
                 origin=excluded.origin, dim=excluded.dim, vector_blob=excluded.vector_blob, created_at=excluded.created_at"#,
            params![source, normalized_text, origin, model, vector.len() as i64, blob, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// 저장된 임베딩 조회 (BLOB → Vec<f32>)
    pub fn get_embedding(&self, source: &str, normalized_text: &str, model: &str) -> Result<Option<Vec<f32>>> {
        let mut stmt = self.conn.prepare(
            "SELECT vector_blob FROM embeddings WHERE source=?1 AND normalized_text=?2 AND model=?3",
        )?;
        let mut rows = stmt.query(params![source, normalized_text, model])?;
        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            let vec: Vec<f32> = blob
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Ok(Some(vec))
        } else {
            Ok(None)
        }
    }

    /// 발견 후보 저장
    pub fn insert_discovery(&self, d: &Discovery) -> Result<()> {
        let type_str = match d.dtype {
            DiscoveryType::Bridge => "bridge",
            DiscoveryType::Gap => "gap",
            DiscoveryType::Cluster => "cluster",
            DiscoveryType::Drift => "drift",
        };
        self.conn.execute(
            r#"INSERT OR REPLACE INTO discoveries (id, type, members_json, evidence_json, score, weak_signal, status, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'new', ?7)"#,
            params![
                d.id.to_string(),
                type_str,
                serde_json::to_string(&d.members)?,
                serde_json::to_string(&d.evidence)?,
                d.score as f64,
                d.weak_signal as i64,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 발견 상태 변경 (dismissed/adopted)
    pub fn set_discovery_status(&self, id: &Uuid, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE discoveries SET status=?2 WHERE id=?1",
            params![id.to_string(), status],
        )?;
        Ok(())
    }

    /// 발견의 유형 조회 (X2 페이로드 생성 시 사용)
    pub fn get_discovery_type(&self, id: &Uuid) -> Result<Option<DiscoveryType>> {
        let mut stmt = self.conn.prepare("SELECT type FROM discoveries WHERE id=?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let t: String = row.get(0)?;
            let dtype = match t.as_str() {
                "bridge" => DiscoveryType::Bridge,
                "gap" => DiscoveryType::Gap,
                "cluster" => DiscoveryType::Cluster,
                _ => DiscoveryType::Drift,
            };
            Ok(Some(dtype))
        } else {
            Ok(None)
        }
    }

    /// 주제 카드 저장 (upsert — 편집 반영)
    pub fn upsert_topic_card(&self, card: &TopicCard) -> Result<()> {
        let label = match card.label_source {
            LabelSource::User => "user",
            LabelSource::Ai => "ai",
        };
        let status = match card.status {
            CardStatus::Draft => "draft",
            CardStatus::Confirmed => "confirmed",
            CardStatus::Archived => "archived",
        };
        self.conn.execute(
            r#"INSERT INTO topic_cards (id, name, label_source, discovery_id, members_json, evidence_snapshot_json, note, status, created_at, updated_at, deleted_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
               ON CONFLICT(id) DO UPDATE SET
                 name=excluded.name, label_source=excluded.label_source, note=excluded.note,
                 status=excluded.status, updated_at=excluded.updated_at, deleted_at=excluded.deleted_at"#,
            params![
                card.id.to_string(),
                card.name,
                label,
                card.discovery_id.to_string(),
                serde_json::to_string(&card.members)?,
                serde_json::to_string(&card.evidence_snapshot)?,
                card.note,
                status,
                card.created_at.to_rfc3339(),
                card.updated_at.to_rfc3339(),
                card.deleted_at.map(|d| d.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    /// 삭제되지 않은 주제 카드 수
    pub fn count_active_topic_cards(&self) -> Result<u64> {
        let n: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM topic_cards WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(n as u64)
    }

    /// 카드 이름 조회 (존재 검증용 간이 API — 전체 역직렬화는 UI 계층에서 확장)
    pub fn get_topic_card_name(&self, id: &Uuid) -> Result<Option<String>> {
        let mut stmt =
            self.conn.prepare("SELECT name FROM topic_cards WHERE id=?1 AND deleted_at IS NULL")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        Ok(rows.next()?.map(|row| row.get(0)).transpose()?)
    }

    /// 환류 이력 기록 (X2 — 투명성 보장)
    pub fn insert_feedback(
        &self,
        topic_card_id: &Uuid,
        payload_summary: &str,
        status: &str,
        memory_id: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO feedbacks (topic_card_id, target, payload_summary, status, memory_id, sent_at)
               VALUES (?1, 'txtaimemory', ?2, ?3, ?4, ?5)"#,
            params![topic_card_id.to_string(), payload_summary, status, memory_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// 설정 저장
    pub fn set_setting(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
               ON CONFLICT(key) DO UPDATE SET value_json=excluded.value_json, updated_at=excluded.updated_at"#,
            params![key, serde_json::to_string(value)?, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// 설정 조회
    pub fn get_setting(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let mut stmt = self.conn.prepare("SELECT value_json FROM settings WHERE key=?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let s: String = row.get(0)?;
            Ok(Some(serde_json::from_str(&s)?))
        } else {
            Ok(None)
        }
    }

    // ---- 아래는 UI 계층(Tauri commands)을 위한 조회 확장. 스키마·기존 시그니처 변경 없음. ----

    /// 캐시된 키워드 전체를 발견 엔진 입력 형태로 로드 (source, normalized_text 순 결정적)
    pub fn list_keyword_cache(&self) -> Result<Vec<KeywordRecord>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT source, text, normalized_text, frequency, avg_emotion_score, first_seen, last_seen
               FROM keyword_cache ORDER BY source, normalized_text"#,
        )?;
        let rows = stmt.query_map([], |row| {
            let source_str: String = row.get(0)?;
            let first_seen: Option<String> = row.get(5)?;
            let last_seen: Option<String> = row.get(6)?;
            Ok(KeywordRecord {
                source: crate::models::SourceId::parse_lenient(&source_str),
                text: row.get(1)?,
                normalized_text: row.get(2)?,
                frequency: row.get::<_, f64>(3)?,
                avg_emotion_score: row.get::<_, f64>(4)? as f32,
                first_seen: first_seen.and_then(|d| d.parse().ok()),
                last_seen: last_seen.and_then(|d| d.parse().ok()),
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// 발견 후보 전체를 status 필터로 조회 (근거·멤버 포함 완전 역직렬화)
    pub fn list_discoveries(&self, status: Option<&str>) -> Result<Vec<Discovery>> {
        let sql = match status {
            Some(_) => "SELECT id, type, members_json, evidence_json, score, weak_signal FROM discoveries WHERE status=?1 ORDER BY score DESC",
            None => "SELECT id, type, members_json, evidence_json, score, weak_signal FROM discoveries ORDER BY score DESC",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let map_row = |row: &rusqlite::Row| -> rusqlite::Result<(String, String, String, String, f64, i64)> {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        };
        let raw_rows: Vec<(String, String, String, String, f64, i64)> = match status {
            Some(s) => stmt.query_map(params![s], map_row)?.collect::<std::result::Result<Vec<_>, _>>()?,
            None => stmt.query_map([], map_row)?.collect::<std::result::Result<Vec<_>, _>>()?,
        };
        let mut out = Vec::with_capacity(raw_rows.len());
        for (id, type_str, members_json, evidence_json, score, weak) in raw_rows {
            out.push(Discovery {
                id: Uuid::parse_str(&id).map_err(|e| crate::CoreError::Other(e.to_string()))?,
                dtype: parse_discovery_type(&type_str),
                members: serde_json::from_str(&members_json)?,
                evidence: serde_json::from_str(&evidence_json)?,
                score: score as f32,
                weak_signal: weak != 0,
            });
        }
        Ok(out)
    }

    /// 발견 단건 조회 (id 기준)
    pub fn get_discovery(&self, id: &Uuid) -> Result<Option<Discovery>> {
        let mut stmt = self
            .conn
            .prepare("SELECT type, members_json, evidence_json, score, weak_signal FROM discoveries WHERE id=?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            let type_str: String = row.get(0)?;
            let members_json: String = row.get(1)?;
            let evidence_json: String = row.get(2)?;
            let score: f64 = row.get(3)?;
            let weak: i64 = row.get(4)?;
            Ok(Some(Discovery {
                id: *id,
                dtype: parse_discovery_type(&type_str),
                members: serde_json::from_str(&members_json)?,
                evidence: serde_json::from_str(&evidence_json)?,
                score: score as f32,
                weak_signal: weak != 0,
            }))
        } else {
            Ok(None)
        }
    }

    /// 주제 카드 전체 조회 (완전 역직렬화)
    pub fn list_topic_cards(&self, include_deleted: bool) -> Result<Vec<TopicCard>> {
        let sql = if include_deleted {
            "SELECT id, name, label_source, discovery_id, members_json, evidence_snapshot_json, note, status, created_at, updated_at, deleted_at FROM topic_cards ORDER BY created_at DESC"
        } else {
            "SELECT id, name, label_source, discovery_id, members_json, evidence_snapshot_json, note, status, created_at, updated_at, deleted_at FROM topic_cards WHERE deleted_at IS NULL ORDER BY created_at DESC"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], row_to_topic_card)?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// 주제 카드 단건 조회 (soft-delete 여부 무관하게 조회)
    pub fn get_topic_card(&self, id: &Uuid) -> Result<Option<TopicCard>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, label_source, discovery_id, members_json, evidence_snapshot_json, note, status, created_at, updated_at, deleted_at FROM topic_cards WHERE id=?1",
        )?;
        let mut rows = stmt.query_map(params![id.to_string()], row_to_topic_card)?;
        rows.next().transpose().map_err(Into::into)
    }

    /// 소스 연결 정보 upsert (토큰 자체가 아니라 지문만 저장 — 원문은 OS 보안 저장소)
    pub fn upsert_source(&self, source: &str, base_url: &str, token_fingerprint: Option<&str>) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO sources (source, base_url, pairing_token_hash, last_synced_at) VALUES (?1, ?2, ?3, NULL)
               ON CONFLICT(source) DO UPDATE SET base_url=excluded.base_url, pairing_token_hash=excluded.pairing_token_hash"#,
            params![source, base_url, token_fingerprint],
        )?;
        Ok(())
    }

    /// 소스 동기화 시각 갱신
    pub fn touch_source_synced(&self, source: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sources SET last_synced_at=?2 WHERE source=?1",
            params![source, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// 소스 연결 해제
    pub fn delete_source(&self, source: &str) -> Result<()> {
        self.conn.execute("DELETE FROM sources WHERE source=?1", params![source])?;
        Ok(())
    }

    /// 등록된 소스 목록
    pub fn list_sources(&self) -> Result<Vec<SourceRow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT source, base_url, pairing_token_hash, last_synced_at FROM sources ORDER BY source")?;
        let rows = stmt.query_map([], |row| {
            Ok(SourceRow {
                source: row.get(0)?,
                base_url: row.get(1)?,
                token_fingerprint: row.get(2)?,
                last_synced_at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    /// 카드의 환류 이력 조회
    pub fn list_feedbacks(&self, topic_card_id: &Uuid) -> Result<Vec<FeedbackRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT target, payload_summary, status, memory_id, sent_at FROM feedbacks WHERE topic_card_id=?1 ORDER BY sent_at DESC",
        )?;
        let rows = stmt.query_map(params![topic_card_id.to_string()], |row| {
            Ok(FeedbackRow {
                target: row.get(0)?,
                payload_summary: row.get(1)?,
                status: row.get(2)?,
                memory_id: row.get(3)?,
                sent_at: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

/// discoveries.type 컬럼 문자열 → DiscoveryType (알 수 없는 값은 Drift로 폴백하지 않고 Cluster로 — 보수적 기본)
fn parse_discovery_type(s: &str) -> DiscoveryType {
    match s {
        "bridge" => DiscoveryType::Bridge,
        "gap" => DiscoveryType::Gap,
        "drift" => DiscoveryType::Drift,
        _ => DiscoveryType::Cluster,
    }
}

fn row_to_topic_card(row: &rusqlite::Row) -> rusqlite::Result<TopicCard> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let label_source: String = row.get(2)?;
    let discovery_id: String = row.get(3)?;
    let members_json: String = row.get(4)?;
    let evidence_json: String = row.get(5)?;
    let note: Option<String> = row.get(6)?;
    let status: String = row.get(7)?;
    let created_at: String = row.get(8)?;
    let updated_at: String = row.get(9)?;
    let deleted_at: Option<String> = row.get(10)?;

    Ok(TopicCard {
        id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::nil()),
        name,
        label_source: if label_source == "ai" { LabelSource::Ai } else { LabelSource::User },
        discovery_id: Uuid::parse_str(&discovery_id).unwrap_or_else(|_| Uuid::nil()),
        members: serde_json::from_str(&members_json).unwrap_or_default(),
        evidence_snapshot: serde_json::from_str(&evidence_json).unwrap_or(crate::discovery::Evidence {
            semantic_sim: 0.0,
            temporal_overlap: 0.0,
            frequency_signal: 0.0,
            period_from: None,
            period_to: None,
            note: None,
        }),
        note,
        status: match status.as_str() {
            "confirmed" => CardStatus::Confirmed,
            "archived" => CardStatus::Archived,
            _ => CardStatus::Draft,
        },
        created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
        deleted_at: deleted_at.and_then(|d| d.parse().ok()),
    })
}

/// sources 테이블 조회 결과
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceRow {
    pub source: String,
    pub base_url: String,
    pub token_fingerprint: Option<String>,
    pub last_synced_at: Option<String>,
}

/// feedbacks 테이블 조회 결과
#[derive(Debug, Clone, serde::Serialize)]
pub struct FeedbackRow {
    pub target: String,
    pub payload_summary: String,
    pub status: String,
    pub memory_id: Option<String>,
    pub sent_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{Evidence, KeywordRecord};
    use crate::models::SourceId;
    use crate::topic::{LabelSource, TopicCard};

    // 테스트 레코드
    fn rec() -> KeywordRecord {
        KeywordRecord {
            source: SourceId::TxtDiary,
            text: "관측자".into(),
            normalized_text: "관측자".into(),
            frequency: 12.0,
            avg_emotion_score: 0.3,
            first_seen: None,
            last_seen: None,
        }
    }

    // 키워드 캐시·임베딩·설정 왕복 검증
    #[test]
    fn cache_embedding_settings_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        store.upsert_keyword_cache(&rec()).unwrap();
        store.upsert_keyword_cache(&rec()).unwrap(); // upsert 중복 무해

        let v = vec![0.1f32, 0.2, 0.3];
        store.upsert_embedding("txtdiary", "관측자", "local", "bge-m3", &v).unwrap();
        let loaded = store.get_embedding("txtdiary", "관측자", "bge-m3").unwrap().unwrap();
        assert_eq!(loaded.len(), 3);
        assert!((loaded[1] - 0.2).abs() < 1e-6);
        assert!(store.get_embedding("txtbrain", "관측자", "bge-m3").unwrap().is_none());

        store.set_setting("weights", &serde_json::json!({"w_s": 0.6})).unwrap();
        let s = store.get_setting("weights").unwrap().unwrap();
        assert_eq!(s["w_s"], 0.6);
    }

    // 발견 저장 → 채택 → 카드 저장 → 환류 이력까지 E2E 왕복
    #[test]
    fn discovery_to_card_to_feedback_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let d = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Bridge,
            members: vec![rec()],
            evidence: Evidence {
                semantic_sim: 0.8,
                temporal_overlap: 0.5,
                frequency_signal: 0.6,
                period_from: None,
                period_to: None,
                note: None,
            },
            score: 0.7,
            weak_signal: false,
        };
        store.insert_discovery(&d).unwrap();
        assert_eq!(store.get_discovery_type(&d.id).unwrap(), Some(DiscoveryType::Bridge));

        store.set_discovery_status(&d.id, "adopted").unwrap();

        let mut card = TopicCard::adopt(&d, "관측 브리지", LabelSource::User);
        store.upsert_topic_card(&card).unwrap();
        assert_eq!(store.count_active_topic_cards().unwrap(), 1);
        assert_eq!(store.get_topic_card_name(&card.id).unwrap().unwrap(), "관측 브리지");

        store.insert_feedback(&card.id, "관측 브리지 (키워드 1개)", "created", Some("aim-123")).unwrap();

        // soft delete 후 활성 카드 0
        card.soft_delete();
        store.upsert_topic_card(&card).unwrap();
        assert_eq!(store.count_active_topic_cards().unwrap(), 0);
        assert!(store.get_topic_card_name(&card.id).unwrap().is_none());
    }

    // list_keyword_cache가 소스별 결정적 순서로 완전 역직렬화되는지 검증
    #[test]
    fn list_keyword_cache_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        store.upsert_keyword_cache(&rec()).unwrap();
        let mut second = rec();
        second.source = SourceId::TxtBrain;
        second.normalized_text = "측정문제".into();
        second.text = "측정 문제".into();
        store.upsert_keyword_cache(&second).unwrap();

        let all = store.list_keyword_cache().unwrap();
        assert_eq!(all.len(), 2);
        // ORDER BY source: "txtbrain" < "txtdiary" (사전식)
        assert_eq!(all[0].source, SourceId::TxtBrain);
        assert_eq!(all[1].source, SourceId::TxtDiary);
    }

    // list_discoveries가 status 필터·score 내림차순으로 완전 역직렬화되는지, get_discovery 단건도 검증
    #[test]
    fn list_and_get_discoveries() {
        let store = Store::open_in_memory().unwrap();
        let low = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Gap,
            members: vec![rec()],
            evidence: Evidence { semantic_sim: 0.2, temporal_overlap: 0.0, frequency_signal: 0.1, period_from: None, period_to: None, note: Some("약함".into()) },
            score: 0.2,
            weak_signal: true,
        };
        let high = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Cluster,
            members: vec![rec()],
            evidence: Evidence { semantic_sim: 0.9, temporal_overlap: 0.8, frequency_signal: 0.7, period_from: None, period_to: None, note: None },
            score: 0.85,
            weak_signal: false,
        };
        store.insert_discovery(&low).unwrap();
        store.insert_discovery(&high).unwrap();
        store.set_discovery_status(&high.id, "adopted").unwrap();

        let all = store.list_discoveries(None).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, high.id, "score 내림차순이어야 함");

        let adopted_only = store.list_discoveries(Some("adopted")).unwrap();
        assert_eq!(adopted_only.len(), 1);
        assert_eq!(adopted_only[0].dtype, DiscoveryType::Cluster);

        let fetched = store.get_discovery(&low.id).unwrap().unwrap();
        assert!(fetched.weak_signal);
        assert_eq!(fetched.evidence.note.as_deref(), Some("약함"));
    }

    // list_topic_cards의 삭제 포함/제외 필터와 get_topic_card 단건 조회 검증
    #[test]
    fn list_and_get_topic_cards() {
        let store = Store::open_in_memory().unwrap();
        let d = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Bridge,
            members: vec![rec()],
            evidence: Evidence { semantic_sim: 0.7, temporal_overlap: 0.3, frequency_signal: 0.4, period_from: None, period_to: None, note: None },
            score: 0.6,
            weak_signal: false,
        };
        let mut card = TopicCard::adopt(&d, "테스트 카드", LabelSource::User);
        store.upsert_topic_card(&card).unwrap();

        assert_eq!(store.list_topic_cards(false).unwrap().len(), 1);
        let fetched = store.get_topic_card(&card.id).unwrap().unwrap();
        assert_eq!(fetched.name, "테스트 카드");
        assert_eq!(fetched.members[0].text, "관측자");

        card.soft_delete();
        store.upsert_topic_card(&card).unwrap();
        assert_eq!(store.list_topic_cards(false).unwrap().len(), 0, "삭제 제외 목록에서 빠져야 함");
        assert_eq!(store.list_topic_cards(true).unwrap().len(), 1, "삭제 포함 목록엔 남아야 함");
    }

    // 소스 페어링 정보(지문만) upsert/list/delete 및 feedbacks 이력 조회 검증
    #[test]
    fn sources_and_feedbacks_listing() {
        let store = Store::open_in_memory().unwrap();
        store.upsert_source("txtdiary", "http://127.0.0.1:4001", Some("fingerprint-abc")).unwrap();
        store.touch_source_synced("txtdiary").unwrap();

        let sources = store.list_sources().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source, "txtdiary");
        assert!(sources[0].last_synced_at.is_some());
        assert_eq!(sources[0].token_fingerprint.as_deref(), Some("fingerprint-abc"));

        store.delete_source("txtdiary").unwrap();
        assert!(store.list_sources().unwrap().is_empty());

        let d = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Cluster,
            members: vec![rec()],
            evidence: Evidence { semantic_sim: 0.5, temporal_overlap: 0.5, frequency_signal: 0.5, period_from: None, period_to: None, note: None },
            score: 0.5,
            weak_signal: false,
        };
        let card = TopicCard::adopt(&d, "환류 테스트", LabelSource::User);
        store.upsert_topic_card(&card).unwrap();
        store.insert_feedback(&card.id, "요약", "created", Some("aim-1")).unwrap();
        store.insert_feedback(&card.id, "요약(update)", "updated", Some("aim-1")).unwrap();

        let history = store.list_feedbacks(&card.id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].status, "updated", "최신순 정렬이어야 함");
    }
}
