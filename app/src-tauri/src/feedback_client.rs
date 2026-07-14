// src-tauri/src/feedback_client.rs — X2 환류 전송 클라이언트
//
// TXTAIMemory의 control API(POST /write, 127.0.0.1:47530)가 실제 X2 수신처로 확정됐다(2026-07-15
// 실측·구현: WriteRequest{content,source,ai_id,summary,importance,origin_app,external_id},
// external_id 재전송 시 멱등 upsert — TXT 패밀리 연결프로토콜 v2.0 Phase 4). MCP stdio는 이 앱처럼
// 독립 데스크톱 프로세스가 호스트 없이 구동하기엔 무겁고, control API가 이미 같은 계약을 HTTP로
// 노출하므로 그쪽을 정식 채널로 쓴다. FeedbackTransport 트레이트로 분리해 뒀으니, 필요해지면
// 구현체만 교체하면 된다.

use serde_json::{json, Value};

/// 환류 응답
pub struct FeedbackAck {
    pub ok: bool,
    pub memory_id: Option<String>,
}

/// 환류 전송 채널 — 구현체 교체 지점
pub trait FeedbackTransport {
    fn send(&self, payload: &Value) -> Result<FeedbackAck, String>;
}

/// 주제 카드 환류 페이로드(feedback::build_feedback_payload_with_type의 풍부한 형태)를
/// TXTAIMemory의 실제 POST /write 계약(WriteRequest)으로 눌러 담는다. 본문은 다른 앱의 원문이
/// 아니라 TXTMyWorld가 만든 카드 설명이므로 "본문 미포함" 원칙과 무관하다.
fn to_write_request(payload: &Value) -> Value {
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let summary = payload.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    let deeplinks = payload
        .get("deeplinks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    let content = if deeplinks.is_empty() {
        format!("[TXTMyWorld 발견] {title}\n{summary}")
    } else {
        format!("[TXTMyWorld 발견] {title}\n{summary}\n관련: {deeplinks}")
    };
    json!({
        "content": content,
        "source": "txtmyworld",
        "summary": summary,
        "origin_app": "txtmyworld",
        "external_id": payload.get("external_id"),
    })
}

/// TXTAIMemory control API로 POST. 페어링 토큰이 있으면 Bearer로 첨부(Phase 1 인증 통일 이후
/// 토큰 발급 상태에서는 없으면 401).
pub struct HttpFeedbackTransport {
    pub endpoint: String,
    pub token: Option<String>,
}

impl FeedbackTransport for HttpFeedbackTransport {
    fn send(&self, payload: &Value) -> Result<FeedbackAck, String> {
        let write_req = to_write_request(payload);
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(15)).build();
        let mut req = agent.post(&self.endpoint).set("Content-Type", "application/json");
        if let Some(token) = &self.token {
            req = req.set("Authorization", &format!("Bearer {token}"));
        }
        let resp = req.send_json(write_req).map_err(|e| e.to_string())?;
        let body: Value = resp.into_json().map_err(|e| e.to_string())?;
        Ok(FeedbackAck {
            ok: body.get("id").and_then(|v| v.as_str()).is_some(),
            memory_id: body.get("id").and_then(|v| v.as_str()).map(String::from),
        })
    }
}

/// 기본 엔드포인트 — TXTAIMemory control API의 실제 /write (설정에서 재정의 가능)
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:47530/write";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_write_request_maps_rich_payload_to_real_contract() {
        let rich = json!({
            "payload_schema_version": "1.0",
            "origin": "txtmyworld",
            "external_id": "topic_card:abc-123",
            "title": "관측자 개념의 수렴",
            "summary": "일기의 '관측자'와 문서의 '관측자 효과'가 하나의 주제로 수렴하는 경향.",
            "discovery_type": "cluster",
            "member_keywords": [{"text": "관측자", "source": "txtdiary"}],
            "deeplinks": ["txtdiary://search?keyword=관측자", "txtbrain://search?keyword=관측자효과"],
        });
        let req = to_write_request(&rich);
        assert_eq!(req["source"], "txtmyworld");
        assert_eq!(req["origin_app"], "txtmyworld");
        assert_eq!(req["external_id"], "topic_card:abc-123");
        let content = req["content"].as_str().unwrap();
        assert!(content.contains("관측자 개념의 수렴"));
        assert!(content.contains("txtdiary://search"), "딥링크가 content에 포함돼야 함");
        // WriteRequest 필수 필드(content/source)가 채워져 있어야 실제 서버에서 422가 안 남
        assert!(!req["content"].as_str().unwrap().is_empty());
    }

    #[test]
    fn to_write_request_without_deeplinks_omits_related_line() {
        let rich = json!({"title": "t", "summary": "s", "external_id": "topic_card:x"});
        let req = to_write_request(&rich);
        assert!(!req["content"].as_str().unwrap().contains("관련:"));
    }

    #[test]
    fn default_endpoint_points_at_real_control_api_write() {
        // 예전엔 존재하지 않는 포트(8765)였다 — 실제 control API(47530)/write인지 고정.
        assert_eq!(DEFAULT_ENDPOINT, "http://127.0.0.1:47530/write");
    }
}
