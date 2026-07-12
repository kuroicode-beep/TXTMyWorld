// src-tauri/src/feedback_client.rs — X2 환류 전송 클라이언트 (best-effort HTTP 브리지)
//
// 통합 스펙 §3.2는 "TXTAIMemory MCP 쓰기 인터페이스(memory_write)"를 채널로 규정한다. MCP는 보통
// stdio JSON-RPC로 호스트(예: Claude)가 구동하는 프로토콜이라, 독립 데스크톱 앱이 일반적인 MCP 클라이언트
// 역할을 온전히 구현하려면 별도의 프로세스 관리·핸드셰이크가 필요하다. TXTAIMemory 쪽 실제 수신 방식
// (stdio vs. 로컬 HTTP 게이트웨이)은 아직 패밀리 차원 합의 전(PRD §16 X2-a, cross-project dependency)이므로,
// v0.1은 **설정 가능한 로컬 HTTP 엔드포인트로 페이로드를 POST**하는 트랜스포트를 기본 구현으로 둔다.
// FeedbackTransport 트레이트로 분리했으니, 실제 MCP 브리지가 확정되면 구현체만 교체하면 된다.

use serde_json::Value;

/// 환류 응답
pub struct FeedbackAck {
    pub ok: bool,
    pub memory_id: Option<String>,
}

/// 환류 전송 채널 — 구현체 교체 지점 (HTTP 브리지 → 실제 MCP 클라이언트로 후속 교체 가능)
pub trait FeedbackTransport {
    fn send(&self, payload: &Value) -> Result<FeedbackAck, String>;
}

/// 로컬 HTTP POST 브리지. TXTAIMemory가 `memory_write`를 HTTP로 노출한다는 가정 하의 기본 구현.
pub struct HttpFeedbackTransport {
    pub endpoint: String,
}

impl FeedbackTransport for HttpFeedbackTransport {
    fn send(&self, payload: &Value) -> Result<FeedbackAck, String> {
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(5)).build();
        let resp = agent
            .post(&self.endpoint)
            .set("Content-Type", "application/json")
            .send_json(payload.clone())
            .map_err(|e| e.to_string())?;
        let body: Value = resp.into_json().map_err(|e| e.to_string())?;
        Ok(FeedbackAck {
            ok: body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
            memory_id: body.get("memory_id").and_then(|v| v.as_str()).map(String::from),
        })
    }
}

/// 기본 엔드포인트 — 설정에서 사용자가 재정의 가능 (X2-a 확정 전까지는 잠정값)
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8765/mcp/tools/memory_write";
