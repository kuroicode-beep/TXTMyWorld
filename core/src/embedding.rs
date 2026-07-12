// core/src/embedding.rs — Embedder 트레이트 + Ollama(bge-m3) 클라이언트 + 결정적 해시 임베더 + 전략 A/B 선택
// 근거: PRD §3.4.1(이중 임베딩 전략 D1), §3.4.2(bge-m3 D2), 통합 스펙 §2.5

use serde_json::json;

use crate::models::VectorCapability;
use crate::vector::{l2_normalize, VectorSpace};
use crate::{CoreError, Result};

/// 임베딩 생성기 트레이트 — 로컬(bge-m3) 구현과 테스트 구현이 공유
pub trait Embedder {
    /// 텍스트 배치를 임베딩한다 (출력 길이 = 입력 길이)
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    /// 이 임베더가 생성하는 벡터 공간
    fn space(&self) -> VectorSpace;
}

/// 임베딩 출처 태그 — embeddings 테이블의 origin 컬럼과 대응
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingOrigin {
    /// 전략 A: 소스측 공유 벡터 (X1)
    Shared,
    /// 전략 B: 로컬 자체 임베딩
    Local,
}

/// 전략 선택 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Strategy {
    /// 소스 벡터를 그대로 사용 (공간 호환)
    UseShared,
    /// 소스 벡터가 있으나 공간 불일치 → 로컬 재임베딩으로 정렬 (권장 기본)
    RealignLocally,
    /// 소스 벡터 없음/미지원 → 로컬 임베딩 (전략 B)
    LocalOnly,
}

/// 소스의 vector_capability와 로컬 기준 공간으로 전략 A/B를 결정한다 (통합 스펙 §2.5)
pub fn choose_strategy(cap: Option<&VectorCapability>, local: &VectorSpace) -> Strategy {
    match cap {
        Some(c) if c.supported => {
            let shared = VectorSpace {
                model: c.model.clone().unwrap_or_default(),
                dim: c.dim.unwrap_or(0),
                normalized: c.normalized.unwrap_or(false),
            };
            if shared.is_compatible(local) {
                Strategy::UseShared
            } else {
                Strategy::RealignLocally
            }
        }
        _ => Strategy::LocalOnly,
    }
}

/// Ollama 임베딩 클라이언트 (bge-m3 등) — POST {base}/api/embed
pub struct OllamaEmbedder {
    pub base_url: String,
    pub model: String,
    pub dim: usize,
}

impl OllamaEmbedder {
    /// 기본 로컬 Ollama + bge-m3 구성
    pub fn default_local() -> Self {
        Self { base_url: "http://127.0.0.1:11434".into(), model: "bge-m3".into(), dim: 1024 }
    }
}

impl Embedder for OllamaEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let url = format!("{}/api/embed", self.base_url);
        let resp = ureq::post(&url)
            .send_json(json!({ "model": self.model, "input": texts }))
            .map_err(|e| CoreError::Http(e.to_string()))?;
        let body: serde_json::Value =
            resp.into_json().map_err(|e| CoreError::Http(e.to_string()))?;
        let arr = body
            .get("embeddings")
            .and_then(|v| v.as_array())
            .ok_or_else(|| CoreError::Http("embeddings 필드 없음".into()))?;
        let mut out = Vec::with_capacity(arr.len());
        for row in arr {
            let vec: Vec<f32> = row
                .as_array()
                .ok_or_else(|| CoreError::Http("embedding 행 형식 오류".into()))?
                .iter()
                .filter_map(|x| x.as_f64().map(|f| f as f32))
                .collect();
            if vec.len() != self.dim {
                return Err(CoreError::DimMismatch { expected: self.dim, actual: vec.len() });
            }
            out.push(l2_normalize(&vec));
        }
        Ok(out)
    }

    fn space(&self) -> VectorSpace {
        VectorSpace { model: self.model.clone(), dim: self.dim, normalized: true }
    }
}

/// 결정적 해시 임베더 — 테스트·오프라인 데모 전용 (의미 품질 없음, 동일 입력→동일 출력 보장)
pub struct HashEmbedder {
    pub dim: usize,
}

impl HashEmbedder {
    /// 소차원 테스트 임베더 생성
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// 문자 bigram을 dim 버킷으로 해싱 — 부분 문자열이 겹치면 유사도가 높아진다
    fn embed_one(&self, text: &str) -> Vec<f32> {
        let chars: Vec<char> = text.chars().collect();
        let mut v = vec![0.0f32; self.dim];
        if chars.is_empty() {
            return v;
        }
        // 단일 문자도 신호를 갖도록 unigram + bigram 해싱
        for w in 1..=2usize {
            for window in chars.windows(w) {
                let mut h: u64 = 1469598103934665603; // FNV offset
                for c in window {
                    h ^= *c as u64;
                    h = h.wrapping_mul(1099511628211);
                }
                v[(h % self.dim as u64) as usize] += 1.0;
            }
        }
        l2_normalize(&v)
    }
}

impl Embedder for HashEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|t| self.embed_one(t)).collect())
    }

    fn space(&self) -> VectorSpace {
        VectorSpace { model: "hash-test".into(), dim: self.dim, normalized: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::cosine;

    // 해시 임베더가 결정적이고, 유사 문자열이 상이 문자열보다 가까운지 검증
    #[test]
    fn hash_embedder_deterministic_and_ordered() {
        let e = HashEmbedder::new(64);
        let a1 = e.embed(&["observer effect".into()]).unwrap();
        let a2 = e.embed(&["observer effect".into()]).unwrap();
        assert_eq!(a1, a2);

        let vecs = e
            .embed(&["observer effect".into(), "observer bias".into(), "김치찌개".into()])
            .unwrap();
        let sim_near = cosine(&vecs[0], &vecs[1]).unwrap();
        let sim_far = cosine(&vecs[0], &vecs[2]).unwrap();
        assert!(sim_near > sim_far, "near={sim_near} far={sim_far}");
    }

    // 전략 선택: 호환 공유→UseShared, 불일치→RealignLocally, 미지원→LocalOnly
    #[test]
    fn strategy_selection() {
        let local = VectorSpace::default_bge_m3();
        let cap_ok = VectorCapability {
            supported: true,
            model: Some("bge-m3".into()),
            dim: Some(1024),
            normalized: Some(true),
            count: Some(10),
            updated_at: None,
        };
        assert_eq!(choose_strategy(Some(&cap_ok), &local), Strategy::UseShared);

        let cap_diff = VectorCapability { model: Some("e5".into()), ..cap_ok.clone() };
        assert_eq!(choose_strategy(Some(&cap_diff), &local), Strategy::RealignLocally);

        let cap_unsupported = VectorCapability { supported: false, ..cap_ok };
        assert_eq!(choose_strategy(Some(&cap_unsupported), &local), Strategy::LocalOnly);
        assert_eq!(choose_strategy(None, &local), Strategy::LocalOnly);
    }
}
