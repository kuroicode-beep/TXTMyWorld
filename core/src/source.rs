// core/src/source.rs — 소스 클라이언트: 공통 API 파싱·스키마 방어·병합·폴백 격리 + HTTP(X1 소비)
// 근거: PRD §6.1, 통합 스펙 §2.5. 원칙: 소스에 쓰기 요청 금지(GET only), 본문 필드 무시.

use chrono::NaiveDate;

use crate::discovery::KeywordRecord;
use crate::models::{
    check_schema_version, HealthResponse, KeywordsResponse, SchemaCheck, VectorsResponse,
};
use crate::{CoreError, Result};

/// 페어링 토큰을 실어 보내는 인증 헤더 스킴.
/// 패밀리 앱마다 다르다 — TXTBrain/TXTDiary는 `Authorization: Bearer`, TXTAIMemory는 `X-Pairing-Token`.
/// (TXTSpace hub adapters.rs의 apply_auth와 동일 규약)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthHeader {
    #[default]
    Bearer,
    XPairingToken,
}

/// 소스 연결 설정 (base_url은 반드시 127.0.0.1 — localhost 전용 원칙)
#[derive(Debug, Clone)]
pub struct SourceConfig {
    pub base_url: String,
    /// 페어링 토큰. 호출측이 OS 보안 저장소에서 꺼내 전달
    pub pairing_token: Option<String>,
    /// 토큰 전송 헤더 스킴 (소스별로 다름)
    pub auth_header: AuthHeader,
}

impl SourceConfig {
    /// 기본(Bearer) 헤더로 생성
    pub fn new(base_url: impl Into<String>, pairing_token: Option<String>) -> Self {
        Self { base_url: base_url.into(), pairing_token, auth_header: AuthHeader::Bearer }
    }

    /// 헤더 스킴을 지정해 생성
    pub fn with_header(base_url: impl Into<String>, pairing_token: Option<String>, auth_header: AuthHeader) -> Self {
        Self { base_url: base_url.into(), pairing_token, auth_header }
    }
}

/// 소스별 조회 결과 — 폴백 격리를 위해 소스 단위 Result로 유지
#[derive(Debug)]
pub enum SourceFetch<T> {
    /// 정상 응답
    Ok(T),
    /// 상위 스키마 버전 — 오류 대신 안내 (앱 비중단)
    UpdateRequired,
    /// 미실행/네트워크 오류 — 캐시 폴백 대상
    Offline(String),
}

/// JSON 문자열을 /health 응답으로 파싱 (스키마 방어 포함)
pub fn parse_health(body: &str) -> Result<SourceFetch<HealthResponse>> {
    let resp: HealthResponse =
        serde_json::from_str(body).map_err(|e| CoreError::SchemaInvalid(e.to_string()))?;
    match check_schema_version(&resp.schema_version)? {
        SchemaCheck::Ok => Ok(SourceFetch::Ok(resp)),
        SchemaCheck::UpdateRequired => Ok(SourceFetch::UpdateRequired),
    }
}

/// JSON 문자열을 /keywords 응답으로 파싱 (스키마 방어 포함)
pub fn parse_keywords(body: &str) -> Result<SourceFetch<KeywordsResponse>> {
    let resp: KeywordsResponse =
        serde_json::from_str(body).map_err(|e| CoreError::SchemaInvalid(e.to_string()))?;
    match check_schema_version(&resp.schema_version)? {
        SchemaCheck::Ok => Ok(SourceFetch::Ok(resp)),
        SchemaCheck::UpdateRequired => Ok(SourceFetch::UpdateRequired),
    }
}

/// JSON 문자열을 /vectors 응답으로 파싱 (X1, dim 선언 검증 포함 — 불일치 레코드는 스킵 정책)
pub fn parse_vectors(body: &str) -> Result<SourceFetch<VectorsResponse>> {
    let mut resp: VectorsResponse =
        serde_json::from_str(body).map_err(|e| CoreError::SchemaInvalid(e.to_string()))?;
    match check_schema_version(&resp.schema_version)? {
        SchemaCheck::Ok => {
            // 선언 dim과 다른 레코드는 스킵 (통합 스펙 §2.6: 비중단)
            let dim = resp.dim;
            resp.vectors.retain(|v| v.vector.len() == dim);
            Ok(SourceFetch::Ok(resp))
        }
        SchemaCheck::UpdateRequired => Ok(SourceFetch::UpdateRequired),
    }
}

/// "YYYY-MM-DD" 문자열을 NaiveDate로 (실패는 None — 비중단)
fn parse_date(s: &Option<String>) -> Option<NaiveDate> {
    s.as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
}

/// 여러 소스의 /keywords 응답을 발견 엔진 입력 레코드로 병합 (source별 별도 항목 유지 — PRD §6.1).
/// 항목이 자기 source를 직접 들고 있으면(TXTSpace-hub 같은 다중 소스 통합 응답) 그것을 쓰고,
/// 없으면 봉투(KeywordsResponse)의 source로 폴백한다.
pub fn merge_keywords(responses: &[KeywordsResponse]) -> Vec<KeywordRecord> {
    let mut out = Vec::new();
    for resp in responses {
        for kw in &resp.keywords {
            let normalized_text =
                if kw.normalized_text.is_empty() { kw.text.clone() } else { kw.normalized_text.clone() };
            out.push(KeywordRecord {
                source: kw.source.clone().unwrap_or_else(|| resp.source.clone()),
                text: kw.text.clone(),
                normalized_text,
                frequency: kw.frequency,
                avg_emotion_score: kw.avg_emotion_score,
                first_seen: parse_date(&kw.first_seen),
                last_seen: parse_date(&kw.last_seen),
            });
        }
        // TXTAIMemory 레거시 형태(items) — v2.0 `keywords`를 이미 함께 보낸 응답(전환기 dual-schema)이면
        // 중복 집계를 막기 위해 건너뛴다. weight를 frequency로, normalized_text는 keyword로 폴백.
        if resp.keywords.is_empty() {
            for item in &resp.items {
                out.push(KeywordRecord {
                    source: resp.source.clone(),
                    text: item.keyword.clone(),
                    normalized_text: item.keyword.clone(),
                    frequency: item.weight,
                    avg_emotion_score: 0.0,
                    first_seen: None,
                    last_seen: None,
                });
            }
        }
    }
    // 결정적 순서: (source, normalized_text)
    out.sort_by(|a, b| a.source.cmp(&b.source).then_with(|| a.normalized_text.cmp(&b.normalized_text)));
    out
}

/// HTTP GET — read-only 원칙의 유일한 메서드. 실패는 Offline으로 격리
fn http_get(config: &SourceConfig, path: &str) -> SourceFetch<String> {
    let url = format!("{}{}", config.base_url, path);
    let mut req = ureq::get(&url);
    if let Some(token) = &config.pairing_token {
        req = match config.auth_header {
            AuthHeader::Bearer => req.set("Authorization", &format!("Bearer {token}")),
            AuthHeader::XPairingToken => req.set("X-Pairing-Token", token),
        };
    }
    match req.call() {
        Ok(resp) => match resp.into_string() {
            Ok(body) => SourceFetch::Ok(body),
            Err(e) => SourceFetch::Offline(e.to_string()),
        },
        Err(e) => SourceFetch::Offline(e.to_string()),
    }
}

/// GET /health — 소스 상태·벡터 능력 확인
pub fn fetch_health(config: &SourceConfig) -> Result<SourceFetch<HealthResponse>> {
    match http_get(config, "/health") {
        SourceFetch::Ok(body) => parse_health(&body),
        SourceFetch::Offline(e) => Ok(SourceFetch::Offline(e)),
        SourceFetch::UpdateRequired => unreachable!(),
    }
}

/// GET /keywords — 키워드 통합 조회 (쿼리는 호출측 조립)
pub fn fetch_keywords(config: &SourceConfig, query: &str) -> Result<SourceFetch<KeywordsResponse>> {
    let path = if query.is_empty() { "/keywords".to_string() } else { format!("/keywords?{query}") };
    match http_get(config, &path) {
        SourceFetch::Ok(body) => parse_keywords(&body),
        SourceFetch::Offline(e) => Ok(SourceFetch::Offline(e)),
        SourceFetch::UpdateRequired => unreachable!(),
    }
}

/// GET /vectors — X1 배치·증분 동기화 (since는 ISO8601)
pub fn fetch_vectors(
    config: &SourceConfig,
    since: Option<&str>,
    cursor: Option<&str>,
) -> Result<SourceFetch<VectorsResponse>> {
    let mut params: Vec<String> = Vec::new();
    if let Some(s) = since {
        params.push(format!("since={s}"));
    }
    if let Some(c) = cursor {
        params.push(format!("cursor={c}"));
    }
    let path = if params.is_empty() {
        "/vectors".to_string()
    } else {
        format!("/vectors?{}", params.join("&"))
    };
    match http_get(config, &path) {
        SourceFetch::Ok(body) => parse_vectors(&body),
        SourceFetch::Offline(e) => Ok(SourceFetch::Offline(e)),
        SourceFetch::UpdateRequired => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceId;

    // 상위 major 스키마는 UpdateRequired로 격리 (앱 비중단)
    #[test]
    fn schema_defense_isolates_update_required() {
        let body = r#"{"schema_version": "2.0", "source": "txtdiary", "keywords": []}"#;
        match parse_keywords(body).unwrap() {
            SourceFetch::UpdateRequired => {}
            _ => panic!("2.0은 UpdateRequired여야 함"),
        }
    }

    // 손상 JSON은 SchemaInvalid 오류 (호출측이 오프라인 캐시 폴백)
    #[test]
    fn corrupted_body_is_invalid() {
        assert!(parse_keywords("{not json").is_err());
    }

    // /vectors에서 선언 dim과 다른 레코드는 스킵되고 나머지는 유지
    #[test]
    fn vectors_dim_mismatch_records_skipped() {
        let body = r#"{
            "schema_version": "1.1", "source": "txtbrain", "model": "bge-m3", "dim": 3, "normalized": true,
            "vectors": [
                {"normalized_text": "ok", "vector": [0.1, 0.2, 0.3]},
                {"normalized_text": "bad", "vector": [0.1, 0.2]}
            ]
        }"#;
        match parse_vectors(body).unwrap() {
            SourceFetch::Ok(resp) => {
                assert_eq!(resp.vectors.len(), 1);
                assert_eq!(resp.vectors[0].normalized_text, "ok");
            }
            _ => panic!("정상 파싱이어야 함"),
        }
    }

    // TXTAIMemory 실제 /keywords 응답 형태(items + keyword/weight/ai_id)를 그대로 파싱·병합
    #[test]
    fn parse_and_merge_aimemory_items_schema() {
        // 실제 47531/keywords 응답 캡처와 동일 형태
        let resp: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"1.0","source":"txtaimemory","items":[
                {"keyword":"관측자","weight":7.5,"ai_id":null},
                {"keyword":"observer effect","weight":3.0,"ai_id":"yumi"}]}"#,
        )
        .unwrap();
        assert_eq!(resp.source, SourceId::TxtAiMemory);
        assert!(resp.keywords.is_empty(), "aimemory는 keywords가 아니라 items를 쓴다");
        assert_eq!(resp.items.len(), 2);

        let merged = merge_keywords(&[resp]);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().all(|r| r.source == SourceId::TxtAiMemory));
        let kw = merged.iter().find(|r| r.text == "관측자").unwrap();
        assert_eq!(kw.frequency, 7.5, "weight가 frequency로 매핑돼야 함");
        assert_eq!(kw.normalized_text, "관측자", "normalized_text는 keyword로 폴백");
    }

    // TXTAIMemory v0.9.3+가 하위호환을 위해 keywords(v2.0)와 items(v1.0)를 같은 응답에 동시에
    // 보낸다 — items까지 또 병합하면 이중 집계가 되므로 keywords가 있으면 items는 무시해야 한다.
    #[test]
    fn merge_prefers_keywords_over_items_when_both_present() {
        let resp: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"2.0","source":"txtaimemory",
                "keywords":[{"text":"관측자","normalized_text":"관측자","frequency":7.5,
                             "first_seen":"2026-07-01","last_seen":"2026-07-10"}],
                "items":[{"keyword":"관측자","weight":7.5,"ai_id":null}]}"#,
        )
        .unwrap();
        let merged = merge_keywords(&[resp]);
        assert_eq!(merged.len(), 1, "keywords와 items가 같은 데이터를 겹쳐 보내면 한 번만 집계돼야 함");
        assert!(merged[0].first_seen.is_some(), "keywords 쪽의 first_seen이 살아있어야 함");
    }

    // 병합: source별 별도 항목 + 결정적 순서 + 날짜 파싱
    #[test]
    fn merge_is_deterministic_and_per_source() {
        let diary: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"1.0","source":"txtdiary","keywords":[
                {"text":"관측자","normalized_text":"관측자","frequency":12,"avg_emotion_score":0.3,
                 "first_seen":"2026-05-01","last_seen":"2026-07-10"}]}"#,
        )
        .unwrap();
        let brain: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"1.1","source":"txtbrain","keywords":[
                {"text":"관측자","normalized_text":"관측자","frequency":4,"avg_emotion_score":0.0}]}"#,
        )
        .unwrap();

        let merged = merge_keywords(&[brain, diary]);
        assert_eq!(merged.len(), 2, "같은 normalized_text라도 소스별 별도 항목");
        // SourceId 정렬 순서(TxtDiary < TxtBrain 선언 순)
        assert_eq!(merged[0].source, SourceId::TxtDiary);
        assert!(merged[0].first_seen.is_some());
        assert_eq!(merged[1].source, SourceId::TxtBrain);
        assert!(merged[1].first_seen.is_none());
    }

    // 허브형 통합 응답(항목별 source, 봉투 source는 허브 자신)에서도 올바른 소스로 태깅되는지 검증
    #[test]
    fn merge_prefers_per_item_source_over_envelope() {
        let hub: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"1.1","source":"txtspace-hub","keywords":[
                {"keyword":"양자역학","normalized_text":"양자역학","frequency":2.0,"avg_emotion_score":0.45,"source":"txtdiary"},
                {"keyword":"측정 문제","normalized_text":"측정문제","frequency":3.0,"avg_emotion_score":0.1,"source":"txtbrain"},
                {"keyword":"observer effect","normalized_text":"observereffect","frequency":1.0,"avg_emotion_score":0.0,"source":"txtaimemory"}
            ]}"#,
        )
        .unwrap();

        let merged = merge_keywords(&[hub]);
        assert_eq!(merged.len(), 3);
        // 봉투 source("txtspace-hub"→Unknown)가 아니라 항목별 source로 태깅돼야 함
        let sources: std::collections::BTreeSet<_> = merged.iter().map(|r| r.source.clone()).collect();
        assert_eq!(
            sources,
            std::collections::BTreeSet::from([SourceId::TxtDiary, SourceId::TxtBrain, SourceId::TxtAiMemory])
        );
        assert!(!sources.contains(&SourceId::Unknown), "허브 자신의 source가 새어나오면 안 됨");
    }

    // normalized_text가 아예 없는 실제 사례(허브 응답 일부) — text로 폴백해야 파싱이 안 깨진다
    #[test]
    fn merge_backfills_missing_normalized_text() {
        let resp: KeywordsResponse = serde_json::from_str(
            r#"{"schema_version":"1.1","source":"txtbrain","keywords":[
                {"keyword":"양자역학","frequency":1.0,"avg_emotion_score":0.0}]}"#,
        )
        .unwrap();
        let merged = merge_keywords(&[resp]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].normalized_text, "양자역학", "normalized_text 없으면 text로 폴백");
    }
}
