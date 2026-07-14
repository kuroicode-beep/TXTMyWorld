// core/src/models.rs — 공통 Keyword/Context API 스키마 v1.0/v1.1 타입 정의 + 스키마 방어
// 근거: TXT 패밀리 마스터 §3.2, TXTMyWorld 통합 스펙 §2 (X1 벡터 공유 확장)

use serde::{Deserialize, Serialize};

use crate::{CoreError, Result};

/// 소비 가능한 스키마 상한 (major.minor). 상위 major는 "업데이트 필요" 처리
pub const SUPPORTED_SCHEMA_MAJOR: u32 = 1;

/// 소스 식별자 — 공통 스키마의 `source` 필드. 모르는 값은 Unknown으로 무해 처리
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceId {
    TxtDiary,
    TxtBrain,
    TxtAiMemory,
    #[serde(other)]
    Unknown,
}

impl SourceId {
    /// 딥링크 스킴 접두어 반환 (마스터 §3.4)
    pub fn deeplink(&self, keyword: &str) -> String {
        match self {
            SourceId::TxtDiary => format!("txtdiary://search?keyword={keyword}"),
            SourceId::TxtBrain => format!("txtbrain://search?keyword={keyword}"),
            SourceId::TxtAiMemory => format!("txtaimemory://recall?keyword={keyword}"),
            SourceId::Unknown => format!("unknown://search?keyword={keyword}"),
        }
    }

    /// 사람이 읽는 한국어 라벨 (접근성 문장용)
    pub fn label_ko(&self) -> &'static str {
        match self {
            SourceId::TxtDiary => "일기",
            SourceId::TxtBrain => "문서",
            SourceId::TxtAiMemory => "AI 대화",
            SourceId::Unknown => "알 수 없는 소스",
        }
    }

    /// 안정적인 소문자 문자열 표현 (DB 컬럼 값 등 non-JSON 직렬화 지점에 사용)
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceId::TxtDiary => "txtdiary",
            SourceId::TxtBrain => "txtbrain",
            SourceId::TxtAiMemory => "txtaimemory",
            SourceId::Unknown => "unknown",
        }
    }

    /// 문자열에서 관대하게 파싱한다 (모르는 값은 Unknown — graceful)
    pub fn parse_lenient(s: &str) -> Self {
        match s {
            "txtdiary" => SourceId::TxtDiary,
            "txtbrain" => SourceId::TxtBrain,
            "txtaimemory" => SourceId::TxtAiMemory,
            _ => SourceId::Unknown,
        }
    }
}

/// /health 응답의 벡터 능력 광고 (X1, schema v1.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCapability {
    pub supported: bool,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub dim: Option<usize>,
    #[serde(default)]
    pub normalized: Option<bool>,
    #[serde(default)]
    pub count: Option<u64>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// /health 응답 (v1.0 호환 + v1.1 vector_capability 확장)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub schema_version: String,
    pub source: SourceId,
    #[serde(default)]
    pub app_version: Option<String>,
    #[serde(default)]
    pub vector_capability: Option<VectorCapability>,
}

/// 키워드 동시 출현 항목.
/// 실서비스(TXTSpace-hub 등)는 이름을 `keyword`로, count를 정수 아닌 부동소수(예: 1.0)로 보낸다 —
/// alias와 f64로 두 표기를 모두 수용한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooccurrence {
    #[serde(alias = "keyword")]
    pub text: String,
    pub count: f64,
}

/// 키워드 인라인 임베딩 (X1, `include=embedding` 옵트인)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPayload {
    pub model: String,
    pub dim: usize,
    #[serde(default)]
    pub normalized: bool,
    pub vector: Vec<f32>,
}

/// 공통 스키마의 키워드 항목 (모르는 확장 필드는 serde가 무시 — graceful).
///
/// PRD 원안 스키마(`text`/`cooccurrence`/정수 `frequency`)와 실서비스(TXTSpace-hub 등)가 실제로
/// 내보내는 형태(`keyword`/`cooccurrences`/부동소수 `frequency`, 항목별 `source`)가 갈라져 있어
/// alias와 관대한 타입으로 양쪽을 모두 수용한다. 항목별 `source`는 다중 소스를 한 번에 통합
/// 제공하는 허브(예: TXTSpace-hub)를 위한 필드 — 없으면 봉투(KeywordsResponse)의 `source`를 쓴다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    #[serde(alias = "keyword")]
    pub text: String,
    /// 실서비스 일부 항목은 이 필드를 아예 안 보낸다 — 없으면 병합 단계(merge_keywords)에서
    /// text로부터 채운다(비어 있는 채로 두지 않는다: normalized_text가 KNN/캐시 키이기 때문).
    #[serde(default)]
    pub normalized_text: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub frequency: f64,
    #[serde(default)]
    pub avg_emotion_score: f32,
    #[serde(default)]
    pub first_seen: Option<String>,
    #[serde(default)]
    pub last_seen: Option<String>,
    #[serde(default, alias = "cooccurrences")]
    pub cooccurrence: Vec<Cooccurrence>,
    /// 항목별 소스(허브 응답용). 없으면 KeywordsResponse.source로 폴백.
    #[serde(default)]
    pub source: Option<SourceId>,
    /// v1.1 인라인 임베딩 (없으면 None → 전략 B 폴백)
    #[serde(default)]
    pub embedding: Option<EmbeddingPayload>,
}

/// /keywords 응답의 date_range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
}

/// /keywords 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordsResponse {
    pub schema_version: String,
    pub source: SourceId,
    #[serde(default)]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub date_range: Option<DateRange>,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
}

/// /vectors 응답의 개별 벡터 레코드 (X1 배치·증분 동기화)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub normalized_text: String,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// /vectors 응답 (통합 스펙 §2.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorsResponse {
    pub schema_version: String,
    pub source: SourceId,
    pub model: String,
    pub dim: usize,
    #[serde(default)]
    pub normalized: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub vectors: Vec<VectorRecord>,
}

/// 스키마 버전 검사 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaCheck {
    /// 지원 범위 — 그대로 소비
    Ok,
    /// 상위 major — 오류 대신 "업데이트 필요" 안내 (마스터 방어 원칙)
    UpdateRequired,
}

/// schema_version 문자열("1.0", "1.1", …)을 검사한다. 파싱 불가면 SchemaInvalid
pub fn check_schema_version(version: &str) -> Result<SchemaCheck> {
    let major = version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok())
        .ok_or_else(|| CoreError::SchemaInvalid(format!("schema_version 형식 오류: {version}")))?;
    if major > SUPPORTED_SCHEMA_MAJOR {
        Ok(SchemaCheck::UpdateRequired)
    } else {
        Ok(SchemaCheck::Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // v1.0 응답(스펙 원문 형태)이 그대로 파싱되는지 검증
    #[test]
    fn parse_keywords_v1_0() {
        let json = r#"{
            "schema_version": "1.0",
            "source": "txtdiary",
            "generated_at": "2026-07-12T00:00:00Z",
            "date_range": {"from": "2026-05-01", "to": "2026-07-10"},
            "keywords": [{
                "text": "관측자",
                "normalized_text": "관측자",
                "category": "topic",
                "frequency": 12,
                "avg_emotion_score": 0.3,
                "first_seen": "2026-05-01",
                "last_seen": "2026-07-10",
                "cooccurrence": [{"text": "양자역학", "count": 5}]
            }]
        }"#;
        let resp: KeywordsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.source, SourceId::TxtDiary);
        assert_eq!(resp.keywords.len(), 1);
        assert_eq!(resp.keywords[0].frequency, 12.0);
        assert!(resp.keywords[0].embedding.is_none());
    }

    // v1.1 인라인 임베딩 + 모르는 확장 필드(ai_id)가 무시되는지 검증
    #[test]
    fn parse_keywords_v1_1_with_embedding_and_unknown_fields() {
        let json = r#"{
            "schema_version": "1.1",
            "source": "txtaimemory",
            "ai_id": "yumi",
            "keywords": [{
                "text": "observer effect",
                "normalized_text": "observereffect",
                "frequency": 5,
                "avg_emotion_score": 0.0,
                "unknown_future_field": true,
                "embedding": {"model": "bge-m3", "dim": 4, "normalized": true, "vector": [0.5, 0.5, 0.5, 0.5]}
            }]
        }"#;
        let resp: KeywordsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.source, SourceId::TxtAiMemory);
        let emb = resp.keywords[0].embedding.as_ref().unwrap();
        assert_eq!(emb.dim, 4);
    }

    // /health의 vector_capability 유무에 따른 파싱 검증
    #[test]
    fn parse_health_capability() {
        let v10 = r#"{"schema_version": "1.0", "source": "txtbrain", "app_version": "1.8.0"}"#;
        let h: HealthResponse = serde_json::from_str(v10).unwrap();
        assert!(h.vector_capability.is_none());

        let v11 = r#"{"schema_version": "1.1", "source": "txtdiary",
            "vector_capability": {"supported": true, "model": "bge-m3", "dim": 1024, "normalized": true, "count": 10}}"#;
        let h: HealthResponse = serde_json::from_str(v11).unwrap();
        assert!(h.vector_capability.unwrap().supported);
    }

    // 스키마 방어: 상위 major는 오류가 아니라 UpdateRequired
    #[test]
    fn schema_defense() {
        assert_eq!(check_schema_version("1.0").unwrap(), SchemaCheck::Ok);
        assert_eq!(check_schema_version("1.1").unwrap(), SchemaCheck::Ok);
        assert_eq!(check_schema_version("2.0").unwrap(), SchemaCheck::UpdateRequired);
        assert!(check_schema_version("abc").is_err());
    }

    // 모르는 source 값은 Unknown으로 무해 처리
    #[test]
    fn unknown_source_is_graceful() {
        let json = r#"{"schema_version": "1.0", "source": "txtfuture", "keywords": []}"#;
        let resp: KeywordsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.source, SourceId::Unknown);
    }

    // 실제 TXTSpace-hub(127.0.0.1:47540)가 보내는 형태 그대로를 픽스처로 검증한다.
    // text 대신 keyword, cooccurrence 대신 cooccurrences, frequency/count가 부동소수,
    // source가 항목마다(봉투가 아니라) 붙는다 — 전부 실제 응답에서 그대로 캡처한 값.
    #[test]
    fn parse_real_hub_response_shape() {
        let json = r#"{
            "keywords": [{
                "avg_emotion_score": 0.45,
                "category": "topic",
                "cooccurrences": [
                    {"count": 1.0, "keyword": "생각"},
                    {"count": 1.0, "keyword": "우주 맵"}
                ],
                "first_seen": "2026-07-08",
                "frequency": 2.0,
                "keyword": "양자역학",
                "last_seen": "2026-07-09",
                "meta": {},
                "normalized_text": "양자역학",
                "source": "txtdiary"
            }],
            "schema_version": "1.1",
            "source": "txtspace-hub",
            "sources": {"txtdiary": {"status": "live"}}
        }"#;
        let resp: KeywordsResponse = serde_json::from_str(json).unwrap();
        // 봉투 source는 허브 자신("txtspace-hub") — SourceId가 모르는 값이라 Unknown으로 무해 처리
        assert_eq!(resp.source, SourceId::Unknown);
        assert_eq!(resp.keywords.len(), 1);
        let kw = &resp.keywords[0];
        assert_eq!(kw.text, "양자역학", "keyword 별칭이 text로 매핑돼야 함");
        assert_eq!(kw.frequency, 2.0, "부동소수 frequency를 그대로 받아야 함");
        assert_eq!(kw.cooccurrence.len(), 2, "cooccurrences 별칭이 매핑돼야 함");
        assert_eq!(kw.cooccurrence[0].text, "생각");
        assert_eq!(kw.source, Some(SourceId::TxtDiary), "항목별 source가 파싱돼야 함");
    }

    // as_str/parse_lenient 왕복 및 모르는 문자열의 graceful 처리
    #[test]
    fn source_id_str_roundtrip() {
        for s in [SourceId::TxtDiary, SourceId::TxtBrain, SourceId::TxtAiMemory] {
            assert_eq!(SourceId::parse_lenient(s.as_str()), s);
        }
        assert_eq!(SourceId::parse_lenient("nonsense"), SourceId::Unknown);
    }
}
