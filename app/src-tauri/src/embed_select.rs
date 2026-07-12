// src-tauri/src/embed_select.rs — 활성 임베더 선택 (Ollama bge-m3 우선, 미가동 시 로컬 해시 폴백)
// 근거: PRD §3.4.1 전략 B(로컬 자체 임베딩)의 "로컬" 구현 선택 지점. bge-m3가 정식 경로이며,
// Ollama가 이 개발 환경에 없을 수 있으므로 결정적 해시 임베더로 다운그레이드해 앱이 항상 동작하게 한다.
// (해시 임베더는 core::embedding 문서대로 "테스트·오프라인 데모 전용" — 의미 품질 보장 없음)

use txtmyworld_core::embedding::{Embedder, HashEmbedder, OllamaEmbedder};
use txtmyworld_core::vector::VectorSpace;

/// 앱 전역 임베딩 차원 — bge-m3 실사용/해시 폴백 모두 이 차원으로 통일해
/// sqlite-vec 테이블(vec_items float[dim])과 dim 정합을 유지한다.
pub const EMBED_DIM: usize = 1024;

/// 실제 사용할 임베더와, 그것이 진짜 bge-m3인지 여부를 함께 반환한다.
pub struct SelectedEmbedder {
    pub embedder: Box<dyn Embedder + Send + Sync>,
    pub is_real_model: bool,
    pub model_name: String,
}

/// Ollama가 응답하면 bge-m3, 아니면 해시 폴백 (짧은 타임아웃으로 앱 시작을 막지 않음)
pub fn select_embedder(ollama_base_url: &str) -> SelectedEmbedder {
    let health_url = format!("{ollama_base_url}/api/tags");
    let reachable = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_millis(800))
        .build()
        .get(&health_url)
        .call()
        .is_ok();

    if reachable {
        let ollama = OllamaEmbedder { base_url: ollama_base_url.to_string(), model: "bge-m3".into(), dim: EMBED_DIM };
        SelectedEmbedder { embedder: Box::new(ollama), is_real_model: true, model_name: "bge-m3".into() }
    } else {
        SelectedEmbedder {
            embedder: Box::new(HashEmbedder::new(EMBED_DIM)),
            is_real_model: false,
            model_name: "hash-fallback".into(),
        }
    }
}

/// 로컬 기준 벡터 공간 (X1 소스측 공유 벡터와의 정합 판정 기준)
pub fn local_space() -> VectorSpace {
    VectorSpace { model: "bge-m3".into(), dim: EMBED_DIM, normalized: true }
}
