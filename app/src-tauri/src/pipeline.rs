// src-tauri/src/pipeline.rs — 동기화·발견 오케스트레이션 (fetch → merge → embed → index → discover)
// 코어(txtmyworld-core)는 순수 로직만 제공하고, 이 파일이 그것들을 실제 파이프라인으로 배선한다.
// 작업지시서 §10 인계 노트의 "동기화 오케스트레이션" 항목.

use txtmyworld_core::discovery::{DiscoveryConfig, DiscoveryEngine, KeywordRecord};
use txtmyworld_core::models::SourceId;
use txtmyworld_core::source::{
    fetch_health, fetch_keywords, fetch_vectors, merge_keywords, AuthHeader, SourceConfig, SourceFetch,
};
use txtmyworld_core::store::Store;
use txtmyworld_core::vector::{VecKey, VectorSpace, VectorStore};
use txtmyworld_core::vector_sqlite::SqliteVecStore;

use crate::dto::{DiscoveryRunSummaryDto, SyncResultDto};
use crate::embed_select::{local_space_for, select_embedder, SelectedEmbedder, EMBED_DIM};
use txtmyworld_core::embedding::HashEmbedder;

/// 실 임베딩 모델 실패 시 쓰는 해시 폴백 임베더 (항상 동작, 차원 EMBED_DIM).
fn fallback_hash_embedder() -> SelectedEmbedder {
    SelectedEmbedder {
        embedder: Box::new(HashEmbedder::new(EMBED_DIM)),
        is_real_model: false,
        model_name: "hash-fallback".into(),
        dim: EMBED_DIM,
    }
}
use crate::secure;

/// 소스별 인증 헤더 스킴 (TXTSpace hub adapters.rs와 동일 규약):
/// TXTAIMemory는 X-Pairing-Token, 나머지(diary/brain/hub)는 Authorization: Bearer.
fn auth_header_for(source_name: &str) -> AuthHeader {
    if source_name == "txtaimemory" {
        AuthHeader::XPairingToken
    } else {
        AuthHeader::Bearer
    }
}

/// 한 소스를 동기화한다: /health 확인 → /keywords 병합 저장 → (지원 시) /vectors 저장.
/// 실패는 오류를 전파하지 않고 상태 문자열로 격리한다 (한 소스 실패가 나머지에 영향 없게).
pub fn sync_source(store: &Store, source_name: &str, base_url: &str) -> SyncResultDto {
    // TXTMyWorld 자체 페어링 우선, 없으면 TXTSpace 공유 토큰 폴백. 허브는 토큰 불필요.
    let token = secure::resolve_token(source_name).map(|(t, _shared)| t);
    let cfg = SourceConfig::with_header(base_url.to_string(), token, auth_header_for(source_name));

    let health = match fetch_health(&cfg) {
        Ok(SourceFetch::Ok(h)) => h,
        Ok(SourceFetch::UpdateRequired) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "update_required".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some("소스의 스키마 버전이 앱보다 높습니다. 앱 업데이트가 필요합니다.".into()),
            }
        }
        Ok(SourceFetch::Offline(e)) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "offline".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some(format!("연결 실패: {e}")),
            }
        }
        Err(e) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "offline".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some(format!("응답 파싱 실패: {e}")),
            }
        }
    };

    let keyword_count = match fetch_keywords(&cfg, "") {
        Ok(SourceFetch::Ok(resp)) => {
            let records = merge_keywords(std::slice::from_ref(&resp));
            for rec in &records {
                let _ = store.upsert_keyword_cache(rec);
            }
            records.len()
        }
        Ok(SourceFetch::UpdateRequired) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "update_required".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some("/keywords 스키마 버전이 앱보다 높습니다. 앱 업데이트가 필요합니다.".into()),
            }
        }
        Ok(SourceFetch::Offline(e)) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "offline".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some(format!("/keywords 연결 실패: {e}")),
            }
        }
        // 조용히 삼키지 않는다 — 파싱 실패(스키마 불일치 등)는 반드시 사용자에게 보인다.
        Err(e) => {
            return SyncResultDto {
                source: source_name.into(),
                status: "offline".into(),
                keyword_count: 0,
                vector_count: 0,
                message: Some(format!("/keywords 응답 파싱 실패: {e}")),
            }
        }
    };

    // X1 소스측 임베딩 공유 — 통합 스펙 §2.5-3 공간 정합 판정: 로컬 기준 공간과 일치할 때만 그대로
    // 받아들인다("전략 A"). 불일치하면 저장하지 않고 조용히 건너뛴다 — discovery 단계에서 해당
    // 키워드는 캐시 미스로 처리되어 "전략 B(로컬 재임베딩)"로 자동 정렬된다.
    let mut vector_count = 0usize;
    if let Some(cap) = &health.vector_capability {
        if cap.supported {
            let source_space = VectorSpace {
                model: cap.model.clone().unwrap_or_default(),
                dim: cap.dim.unwrap_or(0),
                normalized: cap.normalized.unwrap_or(false),
            };
            // 우리가 선호하는 공간(bge-m3/1024)과 일치할 때만 소스 공유 벡터를 그대로 받는다.
            // 불일치하면 저장하지 않고 discovery 단계의 로컬 재임베딩(전략 B)에 맡긴다.
            if source_space.is_compatible(&local_space_for("bge-m3", EMBED_DIM)) {
                let mut cursor: Option<String> = None;
                while let Ok(SourceFetch::Ok(resp)) = fetch_vectors(&cfg, None, cursor.as_deref()) {
                    for v in &resp.vectors {
                        if v.vector.len() != resp.dim {
                            continue; // dim 선언 불일치 레코드는 스킵(비중단)
                        }
                        let _ = store.upsert_embedding(source_name, &v.normalized_text, "shared", &resp.model, &v.vector);
                        vector_count += 1;
                    }
                    match resp.next_cursor {
                        Some(next) if !next.is_empty() => cursor = Some(next),
                        _ => break,
                    }
                }
            }
        }
    }

    let fp = secure::resolve_token(source_name).map(|(t, _)| secure::fingerprint(&t));
    let _ = store.upsert_source(source_name, base_url, fp.as_deref());
    let _ = store.touch_source_synced(source_name);

    SyncResultDto { source: source_name.into(), status: "ok".into(), keyword_count, vector_count, message: None }
}

/// 등록된 모든 소스를 동기화한다 (소스 하나 실패해도 나머지는 계속)
pub fn sync_all(store: &Store) -> Vec<SyncResultDto> {
    let sources = store.list_sources().unwrap_or_default();
    sources.iter().map(|s| sync_source(store, &s.source, &s.base_url)).collect()
}

/// 3소스를 각 실측 포트로 직접 등록·동기화한다 (2026-07-14 실측). 공유 토큰 자동 재사용.
/// 각 소스는 sync_source 안에서 upsert_source로 등록되므로, 이후 sync_all에도 잡힌다.
pub fn connect_all_direct(store: &Store) -> Vec<SyncResultDto> {
    const DIRECT_SOURCES: &[(&str, &str)] = &[
        ("txtdiary", "http://127.0.0.1:47821"),
        ("txtbrain", "http://127.0.0.1:8811"),
        ("txtaimemory", "http://127.0.0.1:47531"),
    ];
    DIRECT_SOURCES.iter().map(|(name, url)| sync_source(store, name, url)).collect()
}

/// 발견 파이프라인: 캐시된 키워드 로드 → (미보유분) 임베딩 → sqlite-vec 인메모리 색인 → 3유형 발견 → 영속화
pub fn run_discovery(store: &Store, config: DiscoveryConfig, ollama_base_url: &str) -> DiscoveryRunSummaryDto {
    let records: Vec<KeywordRecord> = store.list_keyword_cache().unwrap_or_default();
    let mut selected = select_embedder(ollama_base_url);
    let mut model_name = selected.model_name.clone();
    let mut dim = selected.dim;

    let mut vec_store = SqliteVecStore::open_in_memory(dim).expect("sqlite-vec 인메모리 저장소 생성 실패");

    let mut embedded_count = 0usize;
    // 미보유 임베딩만 배치로 계산 (증분) — 이미 같은 모델 임베딩이 있으면 재사용.
    // pending_* 세 벡터는 반드시 같은 인덱스로 짝지어야 하므로 레코드 참조도 함께 모은다.
    let mut pending_texts: Vec<String> = Vec::new();
    let mut pending_keys: Vec<VecKey> = Vec::new();
    let mut pending_records: Vec<&KeywordRecord> = Vec::new();

    for rec in &records {
        let key = rec.key();
        if let Ok(Some(v)) = store.get_embedding(rec.source.as_str(), &rec.normalized_text, &model_name) {
            if v.len() == dim {
                let _ = vec_store.upsert(key, v);
                continue;
            }
        }
        pending_texts.push(rec.text.clone());
        pending_keys.push(key);
        pending_records.push(rec);
    }

    if !pending_texts.is_empty() {
        // 선택한 실제 모델(Ollama)로 임베딩 시도. 실패하면(모델 오류 등) 해시 폴백으로 전환해
        // 발견이 0건이 되는 것을 막는다 — 차원이 바뀌므로 벡터 저장소를 재생성하고 전량 재임베딩한다.
        let vectors = match selected.embedder.embed(&pending_texts) {
            Ok(v) => v,
            Err(_) => {
                selected = fallback_hash_embedder();
                model_name = selected.model_name.clone();
                dim = selected.dim;
                vec_store = SqliteVecStore::open_in_memory(dim).expect("sqlite-vec 재생성 실패");
                // 폴백 시엔 전량 재임베딩 (앞서 캐시로 채운 것도 차원이 다르므로 무효)
                pending_texts = records.iter().map(|r| r.text.clone()).collect();
                pending_keys = records.iter().map(|r| r.key()).collect();
                pending_records = records.iter().collect();
                selected.embedder.embed(&pending_texts).unwrap_or_default()
            }
        };
        for ((key, vec), rec) in pending_keys.into_iter().zip(vectors).zip(pending_records) {
            let _ = store.upsert_embedding(rec.source.as_str(), &rec.normalized_text, "local", &model_name, &vec);
            let _ = vec_store.upsert(key, vec);
            embedded_count += 1;
        }
    }

    let engine = DiscoveryEngine::new(config);
    let store_ref: &dyn VectorStore = &vec_store;

    let bridges = engine.detect_bridges(&records, store_ref).unwrap_or_default();
    let gaps = engine.detect_gaps(&records, store_ref).unwrap_or_default();
    let clusters = engine.detect_clusters(&records, store_ref).unwrap_or_default();

    let mut all = Vec::new();
    all.extend(bridges.iter().cloned());
    all.extend(gaps.iter().cloned());
    all.extend(clusters.iter().cloned());

    for d in &all {
        let _ = store.insert_discovery(d);
    }

    let mut dtos: Vec<crate::dto::DiscoveryDto> = all.iter().map(crate::dto::DiscoveryDto::from).collect();
    dtos.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    DiscoveryRunSummaryDto {
        total_keywords: records.len(),
        embedded_count,
        bridges: bridges.len(),
        gaps: gaps.len(),
        clusters: clusters.len(),
        discoveries: dtos,
    }
}

/// 데모 시드: 실서비스 소스가 없어도 발견 흐름을 체험할 수 있도록 큐레이션된 키워드 세트를 넣는다.
/// PRD §5 사용자 시나리오의 "관측자/측정 문제/observer effect" 예시를 그대로 사용 — 투명하게 데모 데이터로 라벨링.
pub fn seed_demo_data(store: &Store) -> usize {
    use chrono::NaiveDate;
    fn d(s: &str) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
    }
    let demo: Vec<KeywordRecord> = vec![
        KeywordRecord {
            source: SourceId::TxtDiary,
            text: "관측자".into(),
            normalized_text: "관측자".into(),
            frequency: 12.0,
            avg_emotion_score: 0.3,
            first_seen: d("2026-05-01"),
            last_seen: d("2026-07-10"),
        },
        KeywordRecord {
            source: SourceId::TxtBrain,
            text: "측정 문제".into(),
            normalized_text: "측정문제".into(),
            frequency: 8.0,
            avg_emotion_score: 0.1,
            first_seen: d("2026-05-15"),
            last_seen: d("2026-07-01"),
        },
        KeywordRecord {
            source: SourceId::TxtAiMemory,
            text: "observer effect".into(),
            normalized_text: "observereffect".into(),
            frequency: 5.0,
            avg_emotion_score: 0.0,
            first_seen: d("2026-06-01"),
            last_seen: d("2026-07-10"),
        },
        KeywordRecord {
            source: SourceId::TxtBrain,
            text: "quantum decoherence".into(),
            normalized_text: "quantumdecoherence".into(),
            frequency: 9.0,
            avg_emotion_score: 0.0,
            first_seen: d("2026-06-01"),
            last_seen: d("2026-06-30"),
        },
        KeywordRecord {
            source: SourceId::TxtDiary,
            text: "몰입".into(),
            normalized_text: "몰입".into(),
            frequency: 5.0,
            avg_emotion_score: -0.1,
            first_seen: d("2026-04-01"),
            last_seen: d("2026-04-30"),
        },
        KeywordRecord {
            source: SourceId::TxtDiary,
            text: "몰입".into(),
            normalized_text: "몰입".into(),
            frequency: 7.0,
            avg_emotion_score: 0.6,
            first_seen: d("2026-06-01"),
            last_seen: d("2026-06-30"),
        },
        KeywordRecord {
            source: SourceId::TxtDiary,
            text: "산책".into(),
            normalized_text: "산책".into(),
            frequency: 6.0,
            avg_emotion_score: 0.4,
            first_seen: d("2026-06-01"),
            last_seen: d("2026-06-30"),
        },
    ];
    let mut n = 0;
    for rec in &demo {
        if store.upsert_keyword_cache(rec).is_ok() {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use txtmyworld_core::models::SourceId;

    // 실제로 켜져 있는 TXTSpace-hub(127.0.0.1:47540, 토큰 불필요, 3소스 통합)를 상대로
    // sync_source가 진짜 키워드를 받아오는지 검증한다. 라이브 로컬 서비스 의존이라 기본
    // 테스트 스위트에서는 제외(#[ignore]) — `cargo test -- --ignored`로 수동 실행.
    #[test]
    #[ignore = "requires a live local TXTSpace-hub on 127.0.0.1:47540"]
    fn sync_source_against_live_txtspace_hub() {
        let store = Store::open_in_memory().unwrap();
        let result = sync_source(&store, "txtspace-hub", "http://127.0.0.1:47540");

        assert_eq!(result.status, "ok", "message={:?}", result.message);
        assert!(result.keyword_count > 0, "허브에서 키워드를 하나도 못 받아옴");

        let records = store.list_keyword_cache().unwrap();
        let sources: std::collections::BTreeSet<_> = records.iter().map(|r| r.source.clone()).collect();
        // 허브 자신(txtspace-hub→Unknown)이 아니라 진짜 origin 3종으로 태깅돼야 한다
        assert!(sources.contains(&SourceId::TxtBrain) || sources.contains(&SourceId::TxtAiMemory) || sources.contains(&SourceId::TxtDiary));
        assert!(!sources.contains(&SourceId::Unknown), "허브 자신의 source가 새어나오면 안 됨");

        println!("동기화된 키워드 {}개, 소스: {:?}", result.keyword_count, sources);
    }
}
