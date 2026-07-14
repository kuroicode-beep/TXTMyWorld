// core/src/vector_sqlite.rs — sqlite-vec 기반 VectorStore 구현 (feature = "sqlitevec")
// 근거: PRD §3.4.3 "벡터 수가 커지면 HNSW(ANN) 인덱스로 승격". sqlite-vec의 vec0 가상 테이블은
// 브루트포스 KNN을 SIMD로 가속하며, 같은 크레이트가 추후 ANN(예: partitioning) 확장을 제공한다.
// v0.1 규모(수천~수만 키워드)에서는 InMemoryVectorStore로 충분하지만, 이 구현체는 동일 트레이트로
// "구현체만 교체"가 가능함을 보장하는 v0.1 증거이자 RC 승격 경로다.

use std::sync::Once;

use rusqlite::Connection;

use crate::vector::{VecKey, VectorStore};
use crate::{CoreError, Result};

static VEC_EXT_INIT: Once = Once::new();

/// sqlite-vec 확장을 프로세스 전역에 1회 등록한다 (auto_extension — 이후 모든 Connection에 적용).
fn ensure_extension_loaded() {
    VEC_EXT_INIT.call_once(|| unsafe {
        // 공식 바인딩 패턴(asg017/sqlite-vec bindings/rust): auto_extension으로 등록.
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    });
}

/// f32 벡터 ↔ vec0가 기대하는 리틀엔디언 바이트 블롭 상호 변환 (store.rs의 임베딩 BLOB 포맷과 동일)
fn vector_to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// sqlite-vec(vec0) 기반 벡터 저장소. 고정 차원(dim)의 벡터만 upsert 가능.
pub struct SqliteVecStore {
    conn: Connection,
    dim: usize,
}

impl SqliteVecStore {
    /// 인메모리 sqlite-vec 저장소 생성 (테스트/임시 세션용)
    pub fn open_in_memory(dim: usize) -> Result<Self> {
        ensure_extension_loaded();
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn, dim)?;
        Ok(Self { conn, dim })
    }

    /// 파일 경로에 저장소를 연다 (앱 SQLite와 별도 파일로 두거나, 같은 DB에 ATTACH해도 됨)
    pub fn open(path: &str, dim: usize) -> Result<Self> {
        ensure_extension_loaded();
        let conn = Connection::open(path)?;
        Self::init_schema(&conn, dim)?;
        Ok(Self { conn, dim })
    }

    fn init_schema(conn: &Connection, dim: usize) -> Result<()> {
        conn.execute_batch(&format!(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(embedding float[{dim}]);
            CREATE TABLE IF NOT EXISTS vec_keys (
                rowid INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                normalized_text TEXT NOT NULL,
                UNIQUE(source, normalized_text)
            );
            "#
        ))?;
        Ok(())
    }

    fn key_rowid(&self, key: &VecKey) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT rowid FROM vec_keys WHERE source=?1 AND normalized_text=?2")?;
        let mut rows = stmt.query(rusqlite::params![key.source.as_str(), key.normalized_text])?;
        Ok(rows.next()?.map(|r| r.get(0)).transpose()?)
    }
}

impl VectorStore for SqliteVecStore {
    fn upsert(&mut self, key: VecKey, vector: Vec<f32>) -> Result<()> {
        if vector.len() != self.dim {
            return Err(CoreError::DimMismatch { expected: self.dim, actual: vector.len() });
        }

        if let Some(rowid) = self.key_rowid(&key)? {
            self.conn.execute(
                "UPDATE vec_items SET embedding = ?1 WHERE rowid = ?2",
                rusqlite::params![vector_to_bytes(&vector), rowid],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO vec_items(embedding) VALUES (?1)",
                rusqlite::params![vector_to_bytes(&vector)],
            )?;
            let rowid = self.conn.last_insert_rowid();
            self.conn.execute(
                "INSERT INTO vec_keys(rowid, source, normalized_text) VALUES (?1, ?2, ?3)",
                rusqlite::params![rowid, key.source.as_str(), key.normalized_text],
            )?;
        }
        Ok(())
    }

    fn get(&self, key: &VecKey) -> Option<Vec<f32>> {
        // vec_items에서 rowid로 저장된 임베딩 블롭을 읽어 f32 벡터로 복원한다.
        // (discovery 엔진이 시드 벡터를 get()으로 가져오므로 반드시 실제 값을 반환해야 한다 —
        //  예전 None 스텁은 모든 발견을 조용히 skip시키는 버그였다.)
        let rowid = self.key_rowid(key).ok().flatten()?;
        let blob: Vec<u8> = self
            .conn
            .query_row("SELECT embedding FROM vec_items WHERE rowid=?1", rusqlite::params![rowid], |r| r.get(0))
            .ok()?;
        Some(blob.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
    }

    fn knn(&self, query: &[f32], k: usize, exclude: Option<&VecKey>) -> Result<Vec<(VecKey, f32)>> {
        if query.len() != self.dim {
            return Err(CoreError::DimMismatch { expected: self.dim, actual: query.len() });
        }
        let exclude_rowid = match exclude {
            Some(k) => self.key_rowid(k)?,
            None => None,
        };
        // vec0 MATCH는 거리(distance, L2) 오름차순을 반환한다. 코사인 유사도로 환산하려면
        // 저장 전 L2 정규화된 벡터를 전제로 하고, distance^2 = 2 - 2*cos_sim (정규화 벡터 간)를 사용한다.
        let mut stmt = self.conn.prepare(
            r#"SELECT vk.source, vk.normalized_text, vi.distance
               FROM vec_items vi JOIN vec_keys vk ON vk.rowid = vi.rowid
               WHERE vi.embedding MATCH ?1 AND k = ?2
               ORDER BY vi.distance"#,
        )?;
        let rows = stmt.query_map(rusqlite::params![vector_to_bytes(query), (k + 1) as i64], |row| {
            let source: String = row.get(0)?;
            let text: String = row.get(1)?;
            let dist: f64 = row.get(2)?;
            Ok((source, text, dist as f32))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (source_str, normalized_text, dist) = row?;
            let source = crate::models::SourceId::parse_lenient(&source_str);
            let vk = VecKey { source, normalized_text };
            if Some(&vk) == exclude {
                continue;
            }
            if exclude_rowid.is_some() && out.len() >= k {
                break;
            }
            // L2 distance(정규화 벡터) → 코사인 유사도로 환산, 오차로 인한 범위 이탈은 클램프
            let cos_sim = (1.0 - (dist * dist) / 2.0).clamp(-1.0, 1.0);
            out.push((vk, cos_sim));
            if out.len() >= k {
                break;
            }
        }
        Ok(out)
    }

    fn keys(&self) -> Vec<VecKey> {
        let mut stmt = match self.conn.prepare("SELECT source, normalized_text FROM vec_keys") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let rows = stmt.query_map([], |row| {
            let source_str: String = row.get(0)?;
            let normalized_text: String = row.get(1)?;
            Ok((source_str, normalized_text))
        });
        let Ok(rows) = rows else { return Vec::new() };
        rows.filter_map(|r| r.ok())
            .map(|(s, t)| VecKey { source: crate::models::SourceId::parse_lenient(&s), normalized_text: t })
            .collect()
    }

    fn len(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM vec_keys", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceId;
    use crate::vector::l2_normalize;

    // sqlite-vec upsert·KNN이 인메모리 구현과 동일한 순서 계약을 지키는지 검증
    #[test]
    fn sqlite_vec_upsert_and_knn() {
        let mut store = SqliteVecStore::open_in_memory(4).unwrap();
        let ka = VecKey::new(SourceId::TxtDiary, "a");
        let kb = VecKey::new(SourceId::TxtBrain, "b");
        let kc = VecKey::new(SourceId::TxtAiMemory, "c");

        store.upsert(ka.clone(), l2_normalize(&[1.0, 0.0, 0.0, 0.0])).unwrap();
        store.upsert(kb.clone(), l2_normalize(&[0.9, 0.1, 0.0, 0.0])).unwrap();
        store.upsert(kc.clone(), l2_normalize(&[0.0, 1.0, 0.0, 0.0])).unwrap();

        assert_eq!(store.len(), 3);

        let query = l2_normalize(&[1.0, 0.0, 0.0, 0.0]);
        let out = store.knn(&query, 2, Some(&ka)).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0.normalized_text, "b", "b가 a에 가장 가까워야 함");
        assert!(out[0].1 > out[1].1);

        // upsert 갱신(같은 키 재삽입 시 rowid 재사용)
        store.upsert(kb.clone(), l2_normalize(&[0.0, 0.0, 1.0, 0.0])).unwrap();
        assert_eq!(store.len(), 3, "갱신은 행 수를 늘리지 않아야 함");
    }

    // 차원 불일치는 오류
    #[test]
    fn dim_mismatch_rejected() {
        let mut store = SqliteVecStore::open_in_memory(4).unwrap();
        let err = store.upsert(VecKey::new(SourceId::TxtDiary, "a"), vec![1.0, 0.0]).unwrap_err();
        assert!(matches!(err, CoreError::DimMismatch { .. }));
    }

    // get()이 저장한 벡터를 실제로 복원하는지 검증 — 예전 None 스텁은 discovery 엔진이 시드 벡터를
    // 못 가져와 모든 발견을 조용히 skip시키는 프로덕션 버그였다(2026-07-14).
    #[test]
    fn get_returns_stored_vector() {
        let mut store = SqliteVecStore::open_in_memory(4).unwrap();
        let k = VecKey::new(SourceId::TxtDiary, "a");
        let v = l2_normalize(&[1.0, 2.0, 3.0, 4.0]);
        store.upsert(k.clone(), v.clone()).unwrap();

        let got = store.get(&k).expect("get()이 None을 반환하면 안 됨");
        assert_eq!(got.len(), 4);
        for (a, b) in got.iter().zip(&v) {
            assert!((a - b).abs() < 1e-6, "복원된 벡터가 저장값과 같아야 함");
        }
        assert!(store.get(&VecKey::new(SourceId::TxtBrain, "none")).is_none());
    }

    // SqliteVecStore를 실제 저장소로 쓴 발견 엔진 E2E — get() 버그가 있으면 여기서 브리지 0건으로 실패.
    #[test]
    fn discovery_engine_works_with_sqlite_store() {
        use crate::discovery::{DiscoveryConfig, DiscoveryEngine, KeywordRecord};
        use crate::vector::VectorStore;

        let recs = vec![
            KeywordRecord { source: SourceId::TxtDiary, text: "관측자".into(), normalized_text: "관측자".into(), frequency: 5.0, avg_emotion_score: 0.0, first_seen: None, last_seen: None },
            KeywordRecord { source: SourceId::TxtBrain, text: "관측자".into(), normalized_text: "관측자".into(), frequency: 4.0, avg_emotion_score: 0.0, first_seen: None, last_seen: None },
        ];
        // 동일 키워드 → 동일 벡터(유사도 1.0)로 색인
        let mut store = SqliteVecStore::open_in_memory(4).unwrap();
        let v = l2_normalize(&[1.0, 1.0, 0.0, 0.0]);
        store.upsert(recs[0].key(), v.clone()).unwrap();
        store.upsert(recs[1].key(), v.clone()).unwrap();

        let engine = DiscoveryEngine::new(DiscoveryConfig { bridge_sim_cut: 0.8, ..Default::default() });
        let store_ref: &dyn VectorStore = &store;
        let bridges = engine.detect_bridges(&recs, store_ref).unwrap();
        assert!(!bridges.is_empty(), "SqliteVecStore로도 교차소스 브리지를 찾아야 함(get 버그 회귀 방지)");
    }
}
