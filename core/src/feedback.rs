// core/src/feedback.rs — X2 환류: 주제 카드 → TXTAIMemory MCP 쓰기 페이로드 (통합 스펙 §3.3, 멱등·본문 미포함)

use serde_json::{json, Value};

use crate::discovery::DiscoveryType;
use crate::topic::TopicCard;

/// 환류 페이로드 스키마 버전 (통합 스펙 §3.3)
pub const PAYLOAD_SCHEMA_VERSION: &str = "1.0";

/// 주제 카드로 X2 MCP 쓰기 페이로드를 만든다.
/// 원칙: 소스 본문은 절대 포함하지 않는다 — 제목·키워드·근거 요약·딥링크만.
pub fn build_feedback_payload(card: &TopicCard) -> Value {
    let dtype = discovery_type_str(&card_discovery_type(card));
    json!({
        "payload_schema_version": PAYLOAD_SCHEMA_VERSION,
        "origin": "txtmyworld",
        "ai_id": "txtmyworld",
        "memory_tier": "consolidated",
        "external_id": card.external_id(),
        "title": card.name,
        "summary": build_summary(card),
        "discovery_type": dtype,
        "member_keywords": card.members.iter().map(|m| json!({
            "text": m.text,
            "normalized_text": m.normalized_text,
            "source": m.source,
        })).collect::<Vec<_>>(),
        "evidence": {
            "semantic_sim": card.evidence_snapshot.semantic_sim,
            "period": {
                "from": card.evidence_snapshot.period_from,
                "to": card.evidence_snapshot.period_to,
            },
            "frequency_signal": frequency_label(card.evidence_snapshot.frequency_signal),
        },
        "deeplinks": card.deeplinks(),
        "created_at": card.created_at.to_rfc3339(),
    })
}

/// 카드의 발견 유형 추정 — 카드에는 evidence만 스냅샷되므로 note 기반 보수적 판정 대신 cluster 기본
/// (저장 시 discoveries.type을 함께 조회할 수 있으면 그 값을 쓰는 것이 정확 — store 참조)
fn card_discovery_type(_card: &TopicCard) -> DiscoveryType {
    DiscoveryType::Cluster
}

/// DiscoveryType → 페이로드 문자열
pub fn discovery_type_str(t: &DiscoveryType) -> &'static str {
    match t {
        DiscoveryType::Bridge => "bridge",
        DiscoveryType::Gap => "gap",
        DiscoveryType::Cluster => "cluster",
        DiscoveryType::Drift => "drift",
    }
}

/// 진단 아닌 경향 서술 요약 생성 (마스터 §4 진단 금지 원칙)
fn build_summary(card: &TopicCard) -> String {
    let parts: Vec<String> = card
        .members
        .iter()
        .map(|m| format!("{}의 '{}'", m.source.label_ko(), m.text))
        .collect();
    format!("{}이(가) 하나의 주제로 수렴하는 경향.", parts.join(", "))
}

/// 빈도 신호 수치 → 사람이 읽는 라벨
fn frequency_label(f: f32) -> &'static str {
    if f >= 0.7 {
        "rising"
    } else if f >= 0.3 {
        "steady"
    } else {
        "weak"
    }
}

/// 발견 유형을 알고 있을 때의 페이로드 생성 (store에서 discoveries.type 조회 후 사용 권장)
pub fn build_feedback_payload_with_type(card: &TopicCard, dtype: &DiscoveryType) -> Value {
    let mut v = build_feedback_payload(card);
    v["discovery_type"] = json!(discovery_type_str(dtype));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{Discovery, DiscoveryType, Evidence, KeywordRecord};
    use crate::models::SourceId;
    use crate::topic::{LabelSource, TopicCard};
    use uuid::Uuid;

    // 스펙 예시 형태의 카드 생성 헬퍼
    fn sample_card() -> TopicCard {
        let d = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Cluster,
            members: vec![
                KeywordRecord {
                    source: SourceId::TxtDiary,
                    text: "관측자".into(),
                    normalized_text: "관측자".into(),
                    frequency: 12.0,
                    avg_emotion_score: 0.3,
                    first_seen: None,
                    last_seen: None,
                },
                KeywordRecord {
                    source: SourceId::TxtBrain,
                    text: "측정 문제".into(),
                    normalized_text: "측정문제".into(),
                    frequency: 8.0,
                    avg_emotion_score: 0.1,
                    first_seen: None,
                    last_seen: None,
                },
            ],
            evidence: Evidence {
                semantic_sim: 0.82,
                temporal_overlap: 0.6,
                frequency_signal: 0.8,
                period_from: None,
                period_to: None,
                note: None,
            },
            score: 0.77,
            weak_signal: false,
        };
        TopicCard::adopt(&d, "관측 문제: 나의 세 갈래", LabelSource::User)
    }

    // 페이로드가 스펙 §3.3 필수 필드를 갖추고 본문을 포함하지 않는지 검증
    #[test]
    fn payload_matches_spec_and_no_body() {
        let card = sample_card();
        let p = build_feedback_payload_with_type(&card, &DiscoveryType::Cluster);

        assert_eq!(p["payload_schema_version"], PAYLOAD_SCHEMA_VERSION);
        assert_eq!(p["origin"], "txtmyworld");
        assert_eq!(p["memory_tier"], "consolidated");
        assert_eq!(p["discovery_type"], "cluster");
        assert!(p["external_id"].as_str().unwrap().starts_with("topic_card:"));
        assert_eq!(p["member_keywords"].as_array().unwrap().len(), 2);
        assert_eq!(p["member_keywords"][0]["source"], "txtdiary");
        assert!(p["deeplinks"][0].as_str().unwrap().starts_with("txtdiary://search"));
        // 본문성 필드가 존재하지 않아야 함
        assert!(p.get("body").is_none() && p.get("content").is_none() && p.get("text").is_none());
        // 진단 금지: 요약은 "경향" 서술
        assert!(p["summary"].as_str().unwrap().contains("경향"));
    }

    // 멱등성: 같은 카드 → 같은 external_id (재환류는 update로 처리 가능)
    #[test]
    fn idempotent_external_id() {
        let card = sample_card();
        let p1 = build_feedback_payload(&card);
        let p2 = build_feedback_payload(&card);
        assert_eq!(p1["external_id"], p2["external_id"]);
    }
}
