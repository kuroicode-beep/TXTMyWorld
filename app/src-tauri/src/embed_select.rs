// src-tauri/src/embed_select.rs — 활성 임베더 선택 (Ollama에 실제 설치된 임베딩 모델 감지 → 로컬 해시 폴백)
// 근거: PRD §3.4.1 전략 B(로컬 자체 임베딩). bge-m3가 정식/최선이나, 이 기기에 설치돼 있지 않을 수 있다.
// "Ollama가 켜져있으면 무조건 bge-m3"로 가정하면 모델 미설치 시 임베딩이 전부 실패해 발견이 0건이 된다
// (실측 버그, 2026-07-14). 그래서 /api/tags로 **실제 설치된** 임베딩 모델을 우선순위로 고르고,
// 하나도 없으면 결정적 해시 임베더로 다운그레이드해 앱이 항상 동작하게 한다.

use txtmyworld_core::embedding::{Embedder, HashEmbedder, OllamaEmbedder};
use txtmyworld_core::vector::VectorSpace;

/// 해시 폴백/기본 벡터 차원. 실제 모델을 쓸 땐 그 모델의 차원(SelectedEmbedder.dim)을 따른다.
pub const EMBED_DIM: usize = 1024;

/// 알려진 임베딩 모델과 출력 차원 (우선순위 순). Ollama 태그 이름은 "name:tag" 형태이므로 접두 매칭한다.
/// bge-m3(다국어, 최선) → nomic-embed-text(다국어, 768) → mxbai/arctic(1024) → all-minilm(영어 위주, 384).
const KNOWN_EMBED_MODELS: &[(&str, usize)] = &[
    ("bge-m3", 1024),
    ("nomic-embed-text", 768),
    ("mxbai-embed-large", 1024),
    ("snowflake-arctic-embed", 1024),
    ("all-minilm", 384),
];

/// 실제 사용할 임베더 + 그 차원·모델명·진짜모델 여부.
pub struct SelectedEmbedder {
    pub embedder: Box<dyn Embedder + Send + Sync>,
    pub is_real_model: bool,
    pub model_name: String,
    pub dim: usize,
}

/// Ollama /api/tags에서 설치된 모델 이름 목록을 가져온다 (짧은 타임아웃, 실패 시 빈 목록).
fn installed_models(ollama_base_url: &str) -> Vec<String> {
    let url = format!("{ollama_base_url}/api/tags");
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_millis(1200)).build();
    let Ok(resp) = agent.get(&url).call() else { return Vec::new() };
    let Ok(body) = resp.into_json::<serde_json::Value>() else { return Vec::new() };
    body.get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// 설치된 임베딩 모델을 우선순위로 고른다. 없으면 해시 폴백(EMBED_DIM).
pub fn select_embedder(ollama_base_url: &str) -> SelectedEmbedder {
    let installed = installed_models(ollama_base_url);
    for (name, dim) in KNOWN_EMBED_MODELS {
        // "nomic-embed-text:latest"처럼 태그가 붙으므로 접두 매칭
        if let Some(full) = installed.iter().find(|m| m.as_str() == *name || m.starts_with(&format!("{name}:"))) {
            let ollama = OllamaEmbedder {
                base_url: ollama_base_url.to_string(),
                model: full.clone(),
                dim: *dim,
            };
            return SelectedEmbedder {
                embedder: Box::new(ollama),
                is_real_model: true,
                model_name: full.clone(),
                dim: *dim,
            };
        }
    }
    // 설치된 임베딩 모델이 없거나 Ollama 미가동 → 해시 폴백(항상 동작, 의미 품질은 낮음)
    SelectedEmbedder {
        embedder: Box::new(HashEmbedder::new(EMBED_DIM)),
        is_real_model: false,
        model_name: "hash-fallback".into(),
        dim: EMBED_DIM,
    }
}

/// 로컬 기준 벡터 공간 (X1 소스측 공유 벡터와의 정합 판정 기준). 현재 선택 모델 기준으로 만든다.
pub fn local_space_for(model: &str, dim: usize) -> VectorSpace {
    VectorSpace { model: model.to_string(), dim, normalized: true }
}
