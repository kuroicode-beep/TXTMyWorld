// core/src/topic.rs — 주제 카드: TXTMyWorld가 소유하는 유일한 원본 데이터 (PRD §6.4)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::discovery::{Discovery, Evidence, KeywordRecord};

/// 카드 이름의 출처 (사용자 직접 / AI 제안 수락)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LabelSource {
    User,
    Ai,
}

/// 카드 상태 (초안/확정/보관)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    Draft,
    Confirmed,
    Archived,
}

/// 주제 카드 — 발견을 채택·명명한 결과물. 근거는 "그때의 관찰" 스냅샷으로 보존
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCard {
    pub id: Uuid,
    pub name: String,
    pub label_source: LabelSource,
    /// 원 발견 후보 id (추적용)
    pub discovery_id: Uuid,
    pub members: Vec<KeywordRecord>,
    pub evidence_snapshot: Evidence,
    pub note: Option<String>,
    pub status: CardStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// soft delete — 값이 있으면 삭제 상태
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TopicCard {
    /// 발견 후보를 채택해 카드로 만든다 (PRD §3.3 흐름 4~5단계)
    pub fn adopt(discovery: &Discovery, name: impl Into<String>, label_source: LabelSource) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            label_source,
            discovery_id: discovery.id,
            members: discovery.members.clone(),
            evidence_snapshot: discovery.evidence.clone(),
            note: None,
            status: CardStatus::Draft,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// X2 환류 멱등 키 — 통합 스펙 §3.3 external_id 규칙
    pub fn external_id(&self) -> String {
        format!("topic_card:{}", self.id)
    }

    /// 구성 키워드의 소스 딥링크 목록 (마스터 §3.4)
    pub fn deeplinks(&self) -> Vec<String> {
        self.members
            .iter()
            .map(|m| m.source.deeplink(&m.normalized_text))
            .collect()
    }

    /// 소프트 삭제 처리
    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveryType, Evidence};
    use crate::models::SourceId;

    // 채택 → external_id·딥링크·soft delete 동작 검증
    #[test]
    fn adopt_and_lifecycle() {
        let d = Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Cluster,
            members: vec![KeywordRecord {
                source: SourceId::TxtDiary,
                text: "관측자".into(),
                normalized_text: "관측자".into(),
                frequency: 12.0,
                avg_emotion_score: 0.3,
                first_seen: None,
                last_seen: None,
            }],
            evidence: Evidence {
                semantic_sim: 0.82,
                temporal_overlap: 0.5,
                frequency_signal: 0.7,
                period_from: None,
                period_to: None,
                note: None,
            },
            score: 0.71,
            weak_signal: false,
        };

        let mut card = TopicCard::adopt(&d, "관측 문제: 나의 세 갈래", LabelSource::User);
        assert_eq!(card.status, CardStatus::Draft);
        assert!(card.external_id().starts_with("topic_card:"));
        assert_eq!(card.deeplinks(), vec!["txtdiary://search?keyword=관측자".to_string()]);
        assert!(card.deleted_at.is_none());

        card.soft_delete();
        assert!(card.deleted_at.is_some());
    }
}
