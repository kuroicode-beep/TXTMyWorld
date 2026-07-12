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
                frequency INTEGER NOT NULL DEFAULT 0,
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
                serde_json::to_string(&rec.source)?.trim_matches('"'),
                rec.normalized_text,
                rec.text,
                rec.frequency as i64,
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
            frequency: 12,
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
}
