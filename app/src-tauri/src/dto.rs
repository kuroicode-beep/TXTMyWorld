// src-tauri/src/dto.rs — 프론트엔드로 나가는 직렬화 전용 DTO. 코어 타입을 그대로 노출하지 않고
// 여기서 변환해, 코어의 RC 계약(모델/트레이트)과 UI 배선을 분리한다.

use serde::Serialize;
use txtmyworld_core::discovery::{Discovery, DiscoveryType, KeywordRecord};
use txtmyworld_core::store::{FeedbackRow, SourceRow};
use txtmyworld_core::topic::{CardStatus, LabelSource, TopicCard};

#[derive(Debug, Clone, Serialize)]
pub struct AppInfoDto {
    pub version: String,
    pub history: Vec<VersionEntryDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionEntryDto {
    pub version: String,
    pub date: String,
    pub summary: String,
}

pub fn app_info() -> AppInfoDto {
    AppInfoDto {
        version: txtmyworld_core::APP_VERSION.to_string(),
        history: txtmyworld_core::VERSION_HISTORY
            .iter()
            .map(|(v, d, s)| VersionEntryDto { version: v.to_string(), date: d.to_string(), summary: s.to_string() })
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordDto {
    pub source: String,
    pub source_label: String,
    pub text: String,
    pub normalized_text: String,
    pub frequency: u64,
    pub avg_emotion_score: f32,
    pub deeplink: String,
}

impl From<&KeywordRecord> for KeywordDto {
    fn from(r: &KeywordRecord) -> Self {
        Self {
            source: r.source.as_str().to_string(),
            source_label: r.source.label_ko().to_string(),
            text: r.text.clone(),
            normalized_text: r.normalized_text.clone(),
            frequency: r.frequency,
            avg_emotion_score: r.avg_emotion_score,
            deeplink: r.source.deeplink(&r.normalized_text),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryDto {
    pub id: String,
    pub dtype: String,
    pub dtype_label_ko: String,
    pub members: Vec<KeywordDto>,
    pub semantic_sim: f32,
    pub temporal_overlap: f32,
    pub frequency_signal: f32,
    pub note: Option<String>,
    pub score: f32,
    pub weak_signal: bool,
    /// 접근성용 완전 문장 — 리스트/스크린리더가 그대로 읽을 수 있게 (PRD §7)
    pub evidence_sentence: String,
}

fn dtype_str(t: DiscoveryType) -> (&'static str, &'static str) {
    match t {
        DiscoveryType::Bridge => ("bridge", "브리지"),
        DiscoveryType::Gap => ("gap", "갭"),
        DiscoveryType::Cluster => ("cluster", "이머전트 클러스터"),
        DiscoveryType::Drift => ("drift", "드리프트"),
    }
}

impl From<&Discovery> for DiscoveryDto {
    fn from(d: &Discovery) -> Self {
        let (dtype, dtype_label_ko) = dtype_str(d.dtype);
        Self {
            id: d.id.to_string(),
            dtype: dtype.to_string(),
            dtype_label_ko: dtype_label_ko.to_string(),
            members: d.members.iter().map(KeywordDto::from).collect(),
            semantic_sim: d.evidence.semantic_sim,
            temporal_overlap: d.evidence.temporal_overlap,
            frequency_signal: d.evidence.frequency_signal,
            note: d.evidence.note.clone(),
            score: d.score,
            weak_signal: d.weak_signal,
            evidence_sentence: d.evidence_sentence(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicCardDto {
    pub id: String,
    pub name: String,
    pub label_source: String,
    pub discovery_id: String,
    pub members: Vec<KeywordDto>,
    pub note: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub deeplinks: Vec<String>,
    pub external_id: String,
}

impl From<&TopicCard> for TopicCardDto {
    fn from(c: &TopicCard) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name.clone(),
            label_source: match c.label_source {
                LabelSource::User => "user".to_string(),
                LabelSource::Ai => "ai".to_string(),
            },
            discovery_id: c.discovery_id.to_string(),
            members: c.members.iter().map(KeywordDto::from).collect(),
            note: c.note.clone(),
            status: match c.status {
                CardStatus::Draft => "draft".to_string(),
                CardStatus::Confirmed => "confirmed".to_string(),
                CardStatus::Archived => "archived".to_string(),
            },
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
            deleted_at: c.deleted_at.map(|d| d.to_rfc3339()),
            deeplinks: c.deeplinks(),
            external_id: c.external_id(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceStatusDto {
    pub source: String,
    pub base_url: String,
    pub paired: bool,
    pub last_synced_at: Option<String>,
    pub online: bool,
    pub vector_capable: bool,
    pub message: Option<String>,
}

impl From<&SourceRow> for SourceStatusDto {
    fn from(r: &SourceRow) -> Self {
        Self {
            source: r.source.clone(),
            base_url: r.base_url.clone(),
            paired: r.token_fingerprint.is_some(),
            last_synced_at: r.last_synced_at.clone(),
            online: false,
            vector_capable: false,
            message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackRecordDto {
    pub target: String,
    pub payload_summary: String,
    pub status: String,
    pub memory_id: Option<String>,
    pub sent_at: String,
}

impl From<&FeedbackRow> for FeedbackRecordDto {
    fn from(r: &FeedbackRow) -> Self {
        Self {
            target: r.target.clone(),
            payload_summary: r.payload_summary.clone(),
            status: r.status.clone(),
            memory_id: r.memory_id.clone(),
            sent_at: r.sent_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncResultDto {
    pub source: String,
    pub status: String, // "ok" | "update_required" | "offline"
    pub keyword_count: usize,
    pub vector_count: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryRunSummaryDto {
    pub total_keywords: usize,
    pub embedded_count: usize,
    pub bridges: usize,
    pub gaps: usize,
    pub clusters: usize,
    pub discoveries: Vec<DiscoveryDto>,
}
