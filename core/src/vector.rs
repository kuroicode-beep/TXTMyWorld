// core/src/vector.rs — 벡터 공간 정합·코사인 유사도·VectorStore 트레이트 + 인메모리 KNN
// 근거: PRD §3.4.3, 통합 스펙 §2.5 (공간 정합 판정). sqlite-vec/HNSW는 같은 트레이트로 후속 교체.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::models::SourceId;
use crate::{CoreError, Result};

/// 벡터 공간 식별 — (model, dim, normalized)가 모두 같아야 직접 비교 가능
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorSpace {
    pub model: String,
    pub dim: usize,
    pub normalized: bool,
}

impl VectorSpace {
    /// 기본 공간 — bge-m3 / 1024-dim / L2 정규화 (PRD §3.4.2 D2)
    pub fn default_bge_m3() -> Self {
        Self { model: "bge-m3".into(), dim: 1024, normalized: true }
    }

    /// 다른 공간과 직접 비교 가능한지 판정 (통합 스펙 §2.5-3)
    pub fn is_compatible(&self, other: &VectorSpace) -> bool {
        self.model == other.model && self.dim == other.dim && self.normalized == other.normalized
    }
}

/// 벡터 키 — 소스 + 정규화 키워드로 유일 식별
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct VecKey {
    pub source: SourceId,
    pub normalized_text: String,
}

impl VecKey {
    /// 편의 생성자
    pub fn new(source: SourceId, normalized_text: impl Into<String>) -> Self {
        Self { source, normalized_text: normalized_text.into() }
    }
}

/// 벡터를 L2 정규화한다 (0 벡터는 그대로 반환)
pub fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        v.to_vec()
    } else {
        v.iter().map(|x| x / norm).collect()
    }
}

/// 코사인 유사도 (차원 불일치는 오류)
pub fn cosine(a: &[f32], b: &[f32]) -> Result<f32> {
    if a.len() != b.len() {
        return Err(CoreError::DimMismatch { expected: a.len(), actual: b.len() });
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        return Ok(0.0);
    }
    Ok(dot / (na * nb))
}

/// 벡터 저장소 트레이트 — v0.1 인메모리, 후속 sqlite-vec/HNSW가 동일 인터페이스로 교체
pub trait VectorStore {
    /// 벡터 upsert (dim 불일치는 오류 반환 — 레코드 스킵 정책은 호출측에서)
    fn upsert(&mut self, key: VecKey, vector: Vec<f32>) -> Result<()>;
    /// 키의 벡터 조회(소유값). sqlite-vec처럼 참조를 못 주는 구현도 있으므로 owned로 반환한다.
    fn get(&self, key: &VecKey) -> Option<Vec<f32>>;
    /// top-k 최근접 이웃 (자기 자신 제외, 유사도 내림차순)
    fn knn(&self, query: &[f32], k: usize, exclude: Option<&VecKey>) -> Result<Vec<(VecKey, f32)>>;
    /// 저장된 모든 키 (결정적 순서)
    fn keys(&self) -> Vec<VecKey>;
    /// 벡터 수
    fn len(&self) -> usize;
    /// 비어 있는지
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 인메모리 브루트포스 KNN 저장소 (BTreeMap으로 결정적 순회 보장)
#[derive(Debug, Default)]
pub struct InMemoryVectorStore {
    dim: Option<usize>,
    map: BTreeMap<VecKey, Vec<f32>>,
}

impl InMemoryVectorStore {
    /// 빈 저장소 생성
    pub fn new() -> Self {
        Self::default()
    }
}

impl VectorStore for InMemoryVectorStore {
    fn upsert(&mut self, key: VecKey, vector: Vec<f32>) -> Result<()> {
        match self.dim {
            None => self.dim = Some(vector.len()),
            Some(d) if d != vector.len() => {
                return Err(CoreError::DimMismatch { expected: d, actual: vector.len() })
            }
            _ => {}
        }
        self.map.insert(key, vector);
        Ok(())
    }

    fn get(&self, key: &VecKey) -> Option<Vec<f32>> {
        self.map.get(key).cloned()
    }

    fn knn(&self, query: &[f32], k: usize, exclude: Option<&VecKey>) -> Result<Vec<(VecKey, f32)>> {
        let mut scored: Vec<(VecKey, f32)> = Vec::with_capacity(self.map.len());
        for (key, vec) in &self.map {
            if Some(key) == exclude {
                continue;
            }
            scored.push((key.clone(), cosine(query, vec)?));
        }
        // 유사도 내림차순, 동률은 키 순서(결정적)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
        scored.truncate(k);
        Ok(scored)
    }

    fn keys(&self) -> Vec<VecKey> {
        self.map.keys().cloned().collect()
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 코사인·정규화 기본 성질 검증
    #[test]
    fn cosine_basics() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let c = [1.0, 0.0];
        assert!((cosine(&a, &b).unwrap() - 0.0).abs() < 1e-6);
        assert!((cosine(&a, &c).unwrap() - 1.0).abs() < 1e-6);
        let n = l2_normalize(&[3.0, 4.0]);
        assert!((n[0] - 0.6).abs() < 1e-6 && (n[1] - 0.8).abs() < 1e-6);
    }

    // 차원 불일치는 명시적 오류 (비중단 스킵 정책은 상위에서)
    #[test]
    fn dim_mismatch_is_error() {
        assert!(cosine(&[1.0], &[1.0, 2.0]).is_err());
        let mut s = InMemoryVectorStore::new();
        s.upsert(VecKey::new(SourceId::TxtDiary, "a"), vec![1.0, 0.0]).unwrap();
        assert!(s.upsert(VecKey::new(SourceId::TxtBrain, "b"), vec![1.0, 0.0, 0.0]).is_err());
    }

    // KNN이 자기 자신 제외·유사도 내림차순으로 동작하는지 검증
    #[test]
    fn knn_ordering_and_exclude() {
        let mut s = InMemoryVectorStore::new();
        let ka = VecKey::new(SourceId::TxtDiary, "a");
        s.upsert(ka.clone(), vec![1.0, 0.0]).unwrap();
        s.upsert(VecKey::new(SourceId::TxtBrain, "b"), vec![0.9, 0.1]).unwrap();
        s.upsert(VecKey::new(SourceId::TxtAiMemory, "c"), vec![0.0, 1.0]).unwrap();

        let out = s.knn(&[1.0, 0.0], 2, Some(&ka)).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0.normalized_text, "b");
        assert!(out[0].1 > out[1].1);
    }

    // 공간 정합 판정 (model/dim/normalized 모두 일치해야 호환)
    #[test]
    fn space_compat() {
        let base = VectorSpace::default_bge_m3();
        assert!(base.is_compatible(&VectorSpace::default_bge_m3()));
        assert!(!base.is_compatible(&VectorSpace { model: "e5".into(), dim: 1024, normalized: true }));
        assert!(!base.is_compatible(&VectorSpace { model: "bge-m3".into(), dim: 768, normalized: true }));
    }
}
