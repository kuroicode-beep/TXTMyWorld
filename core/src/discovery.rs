// core/src/discovery.rs — 3축 융합 발견 엔진: 브리지/갭/이머전트 클러스터/드리프트 + 근거 문장(접근성)
// 근거: PRD §3(발견 유형)·§3.4.5(융합 스코어 score = w_s·sim + w_t·overlap + w_f·freq), §6.2(결정적·근거 투명성)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::models::SourceId;
use crate::vector::{VecKey, VectorStore};
use crate::Result;

/// 3축 융합 가중치 (기본 0.6/0.2/0.2 — 개발 중 튜닝, 설정으로 조절 가능)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionWeights {
    pub w_s: f32,
    pub w_t: f32,
    pub w_f: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self { w_s: 0.6, w_t: 0.2, w_f: 0.2 }
    }
}

/// 발견 엔진 설정 (임계값 기본값 — PRD §16 non-blocking, 설정으로 노출)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub weights: FusionWeights,
    /// 브리지로 인정할 최소 의미 유사도
    pub bridge_sim_cut: f32,
    /// 갭 판정: 이 유사도 이상 이웃이 상대 소스에 없으면 갭
    pub gap_sim_cut: f32,
    /// 갭 판정: 원 소스에서 이 빈도 이상이어야 "강한" 키워드
    pub gap_min_freq: u64,
    /// 클러스터 멤버 최소 유사도(시드 기준)
    pub cluster_sim_cut: f32,
    /// 클러스터 최소 멤버 수
    pub cluster_min_size: usize,
    /// 드리프트 판정: 감정 점수 이동 최소 절대값
    pub drift_min_delta: f32,
    /// KNN 후보 수 상한
    pub knn_k: usize,
    /// "약한 신호" 라벨 경계 (이하이면 weak)
    pub weak_signal_score: f32,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            weights: FusionWeights::default(),
            // 실제 의미 임베딩(nomic/bge-m3 등)은 서로 다른 키워드 사이에서도 기본 코사인 유사도가
            // 높게 나온다(실측 중앙값 ≈ 0.82). 0.6 컷은 수백 건의 노이즈를 만든다 — 실데이터 분포에
            // 맞춰 0.85로 상향(실측: 195키워드/3소스에서 브리지 ~69, 클러스터 ~6, 갭 ~19 = 약 90건).
            // 사용자는 설정 화면에서 다시 조절할 수 있다.
            bridge_sim_cut: 0.85,
            gap_sim_cut: 0.5,
            gap_min_freq: 3,
            cluster_sim_cut: 0.85,
            cluster_min_size: 3,
            drift_min_delta: 0.4,
            knn_k: 10,
            weak_signal_score: 0.5,
        }
    }
}

/// 발견 계산에 쓰는 병합 키워드 레코드 (source.rs의 병합 산출물)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRecord {
    pub source: SourceId,
    pub text: String,
    pub normalized_text: String,
    /// 실서비스 응답이 정수 아닌 부동소수 빈도(가중·감쇠 점수 등)를 보내는 경우가 있어 f64로 수용
    pub frequency: f64,
    pub avg_emotion_score: f32,
    pub first_seen: Option<NaiveDate>,
    pub last_seen: Option<NaiveDate>,
}

impl KeywordRecord {
    /// 벡터 키로 변환
    pub fn key(&self) -> VecKey {
        VecKey::new(self.source.clone(), self.normalized_text.clone())
    }
}

/// 발견 유형 (PRD §3.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryType {
    Bridge,
    Gap,
    Cluster,
    Drift,
}

/// 발견 근거 — 수치는 그대로 보존해 투명하게 제시 (PRD §6.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub semantic_sim: f32,
    pub temporal_overlap: f32,
    pub frequency_signal: f32,
    pub period_from: Option<NaiveDate>,
    pub period_to: Option<NaiveDate>,
    /// 유형별 부가 설명 (갭: 비어 있는 소스 등)
    pub note: Option<String>,
}

/// 발견 후보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub id: Uuid,
    pub dtype: DiscoveryType,
    pub members: Vec<KeywordRecord>,
    pub evidence: Evidence,
    pub score: f32,
    /// 약한 신호 여부 (근거가 약하면 true — UI에서 라벨 병기)
    pub weak_signal: bool,
}

impl Discovery {
    /// 접근성용 근거 완전 문장 생성 — 스크린리더가 "무엇이 왜 이어졌는지" 읽을 수 있게 (PRD §7)
    pub fn evidence_sentence(&self) -> String {
        let member_desc: Vec<String> = self
            .members
            .iter()
            .map(|m| format!("{}({}) {}회", m.text, m.source.label_ko(), m.frequency))
            .collect();
        let type_ko = match self.dtype {
            DiscoveryType::Bridge => "브리지(숨은 연결)",
            DiscoveryType::Gap => "갭(비어 있는 맥락)",
            DiscoveryType::Cluster => "이머전트 클러스터(이름 없는 주제)",
            DiscoveryType::Drift => "드리프트(생각의 이동)",
        };
        let weak = if self.weak_signal { " 근거가 약한 신호입니다." } else { "" };
        let note = self.evidence.note.as_deref().unwrap_or("");
        format!(
            "{} 유형 발견: {}. 의미 유사도 {:.2}, 기간 겹침 {:.2}, 빈도 신호 {:.2}, 종합 점수 {:.2}.{}{}",
            type_ko,
            member_desc.join(", "),
            self.evidence.semantic_sim,
            self.evidence.temporal_overlap,
            self.evidence.frequency_signal,
            self.score,
            if note.is_empty() { String::new() } else { format!(" {note}.") },
            weak
        )
    }
}

/// 기간 겹침 비율 계산 — 겹친 일수 / 짧은 구간 일수, 0..1 클램프. 날짜 없으면 0
pub fn temporal_overlap(
    a_from: Option<NaiveDate>,
    a_to: Option<NaiveDate>,
    b_from: Option<NaiveDate>,
    b_to: Option<NaiveDate>,
) -> f32 {
    let (Some(af), Some(at), Some(bf), Some(bt)) = (a_from, a_to, b_from, b_to) else {
        return 0.0;
    };
    let start = af.max(bf);
    let end = at.min(bt);
    if end < start {
        return 0.0;
    }
    let overlap = (end - start).num_days() + 1;
    let span_a = (at - af).num_days() + 1;
    let span_b = (bt - bf).num_days() + 1;
    let min_span = span_a.min(span_b).max(1);
    ((overlap as f32) / (min_span as f32)).clamp(0.0, 1.0)
}

/// 빈도 신호 — 양쪽 다 유의미한 양인지: sqrt(min/max), 0빈도는 0
pub fn frequency_signal(a: f64, b: f64) -> f32 {
    if a <= 0.0 || b <= 0.0 {
        return 0.0;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    ((lo / hi) as f32).sqrt()
}

/// 3축 융합 스코어 (PRD §3.4.5) — 결정적
pub fn fusion_score(sim: f32, t_overlap: f32, f_signal: f32, w: &FusionWeights) -> f32 {
    w.w_s * sim + w.w_t * t_overlap + w.w_f * f_signal
}

/// 발견 엔진 — 레코드 + 벡터 저장소를 입력으로 후보를 계산 (결정적)
pub struct DiscoveryEngine {
    pub config: DiscoveryConfig,
}

impl DiscoveryEngine {
    /// 기본 설정으로 생성
    pub fn new(config: DiscoveryConfig) -> Self {
        Self { config }
    }

    /// 브리지 탐지: 서로 다른 소스의 키워드 쌍이 의미 유사도로 강하게 이어짐
    pub fn detect_bridges(
        &self,
        records: &[KeywordRecord],
        store: &dyn VectorStore,
    ) -> Result<Vec<Discovery>> {
        let by_key: BTreeMap<VecKey, &KeywordRecord> =
            records.iter().map(|r| (r.key(), r)).collect();
        let mut seen_pairs: BTreeSet<(VecKey, VecKey)> = BTreeSet::new();
        let mut out = Vec::new();

        for rec in records {
            let key = rec.key();
            let Some(vec) = store.get(&key) else { continue };
            let neighbors = store.knn(&vec, self.config.knn_k, Some(&key))?;
            for (nkey, sim) in neighbors {
                if nkey.source == rec.source || sim < self.config.bridge_sim_cut {
                    continue;
                }
                let pair = if key <= nkey { (key.clone(), nkey.clone()) } else { (nkey.clone(), key.clone()) };
                if !seen_pairs.insert(pair) {
                    continue;
                }
                let Some(other) = by_key.get(&nkey) else { continue };
                let t = temporal_overlap(rec.first_seen, rec.last_seen, other.first_seen, other.last_seen);
                let f = frequency_signal(rec.frequency, other.frequency);
                let score = fusion_score(sim, t, f, &self.config.weights);
                out.push(Discovery {
                    id: Uuid::new_v4(),
                    dtype: DiscoveryType::Bridge,
                    members: vec![rec.clone(), (*other).clone()],
                    evidence: Evidence {
                        semantic_sim: sim,
                        temporal_overlap: t,
                        frequency_signal: f,
                        period_from: rec.first_seen.min(other.first_seen),
                        period_to: rec.last_seen.max(other.last_seen),
                        note: None,
                    },
                    score,
                    weak_signal: score < self.config.weak_signal_score,
                });
            }
        }
        // 점수 내림차순 (결정적: 동률은 멤버 텍스트 순)
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.members[0].normalized_text.cmp(&b.members[0].normalized_text))
        });
        Ok(out)
    }

    /// 갭 탐지: 한 소스에 강한 키워드가 다른 소스에 의미 이웃이 없음 ("아는데 안 쓴 생각")
    pub fn detect_gaps(
        &self,
        records: &[KeywordRecord],
        store: &dyn VectorStore,
    ) -> Result<Vec<Discovery>> {
        // 데이터에 실제 존재하는 소스 집합만 대상으로 (미연결 소스로 갭을 만들지 않음)
        let present_sources: BTreeSet<SourceId> =
            records.iter().map(|r| r.source.clone()).collect();
        let mut out = Vec::new();

        for rec in records {
            if rec.frequency < self.config.gap_min_freq as f64 {
                continue;
            }
            let key = rec.key();
            let Some(vec) = store.get(&key) else { continue };
            let neighbors = store.knn(&vec, store.len(), Some(&key))?;
            for target in &present_sources {
                if *target == rec.source {
                    continue;
                }
                // 상대 소스에서 가장 가까운 이웃의 유사도
                let best = neighbors
                    .iter()
                    .filter(|(k, _)| k.source == *target)
                    .map(|(_, s)| *s)
                    .fold(f32::MIN, f32::max);
                if best < self.config.gap_sim_cut {
                    let score = fusion_score(
                        1.0 - best.max(0.0), // 이웃이 멀수록 갭이 뚜렷
                        0.0,
                        frequency_signal(rec.frequency, rec.frequency),
                        &self.config.weights,
                    );
                    out.push(Discovery {
                        id: Uuid::new_v4(),
                        dtype: DiscoveryType::Gap,
                        members: vec![rec.clone()],
                        evidence: Evidence {
                            semantic_sim: best.max(0.0),
                            temporal_overlap: 0.0,
                            frequency_signal: 1.0,
                            period_from: rec.first_seen,
                            period_to: rec.last_seen,
                            note: Some(format!("{}에는 관련 맥락이 비어 있음", target.label_ko())),
                        },
                        score,
                        weak_signal: score < self.config.weak_signal_score,
                    });
                }
            }
        }
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.members[0].normalized_text.cmp(&b.members[0].normalized_text))
        });
        Ok(out)
    }

    /// 이머전트 클러스터: 소스를 가로지르는 의미 덩어리 (v0.1 그리디 — HDBSCAN은 동일 인터페이스로 후속)
    pub fn detect_clusters(
        &self,
        records: &[KeywordRecord],
        store: &dyn VectorStore,
    ) -> Result<Vec<Discovery>> {
        let by_key: BTreeMap<VecKey, &KeywordRecord> =
            records.iter().map(|r| (r.key(), r)).collect();
        // 빈도 내림차순 시드 순서(동률은 키 순) → 결정적
        let mut seeds: Vec<&KeywordRecord> = records.iter().collect();
        seeds.sort_by(|a, b| {
            b.frequency
                .partial_cmp(&a.frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.key().cmp(&b.key()))
        });

        let mut assigned: BTreeSet<VecKey> = BTreeSet::new();
        let mut out = Vec::new();

        for seed in seeds {
            let seed_key = seed.key();
            if assigned.contains(&seed_key) {
                continue;
            }
            let Some(seed_vec) = store.get(&seed_key) else { continue };
            let neighbors = store.knn(&seed_vec, store.len(), Some(&seed_key))?;

            let mut members: Vec<&KeywordRecord> = vec![seed];
            let mut sims: Vec<f32> = Vec::new();
            for (nkey, sim) in &neighbors {
                if sim < &self.config.cluster_sim_cut || assigned.contains(nkey) {
                    continue;
                }
                if let Some(rec) = by_key.get(nkey) {
                    members.push(rec);
                    sims.push(*sim);
                }
            }

            let sources: BTreeSet<&SourceId> = members.iter().map(|m| &m.source).collect();
            if members.len() >= self.config.cluster_min_size && sources.len() >= 2 {
                for m in &members {
                    assigned.insert(m.key());
                }
                let avg_sim = if sims.is_empty() { 0.0 } else { sims.iter().sum::<f32>() / sims.len() as f32 };
                // 기간: 멤버 전체의 합집합, 겹침: 멤버 쌍 평균 대신 시드-멤버 평균으로 근사
                let period_from = members.iter().filter_map(|m| m.first_seen).min();
                let period_to = members.iter().filter_map(|m| m.last_seen).max();
                let t_avg = members
                    .iter()
                    .skip(1)
                    .map(|m| temporal_overlap(seed.first_seen, seed.last_seen, m.first_seen, m.last_seen))
                    .sum::<f32>()
                    / (members.len().max(2) - 1) as f32;
                let f_avg = members
                    .iter()
                    .skip(1)
                    .map(|m| frequency_signal(seed.frequency, m.frequency))
                    .sum::<f32>()
                    / (members.len().max(2) - 1) as f32;
                let score = fusion_score(avg_sim, t_avg, f_avg, &self.config.weights);
                out.push(Discovery {
                    id: Uuid::new_v4(),
                    dtype: DiscoveryType::Cluster,
                    members: members.into_iter().cloned().collect(),
                    evidence: Evidence {
                        semantic_sim: avg_sim,
                        temporal_overlap: t_avg,
                        frequency_signal: f_avg,
                        period_from,
                        period_to,
                        note: Some(format!("{}개 소스에 걸친 의미 덩어리", sources.len())),
                    },
                    score,
                    weak_signal: score < self.config.weak_signal_score,
                });
            } else {
                assigned.insert(seed_key);
            }
        }
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.members[0].normalized_text.cmp(&b.members[0].normalized_text))
        });
        Ok(out)
    }

    /// 드리프트: 같은 키워드의 두 기간 스냅샷 간 감정 이동 (v0.1: 기간별 조회 결과 두 개를 입력)
    pub fn detect_drift(
        &self,
        early: &KeywordRecord,
        late: &KeywordRecord,
    ) -> Option<Discovery> {
        if early.normalized_text != late.normalized_text || early.source != late.source {
            return None;
        }
        let delta = late.avg_emotion_score - early.avg_emotion_score;
        if delta.abs() < self.config.drift_min_delta {
            return None;
        }
        let direction = if delta > 0.0 { "긍정 방향" } else { "부정 방향" };
        let score = fusion_score(
            delta.abs().clamp(0.0, 1.0),
            0.0,
            frequency_signal(early.frequency, late.frequency),
            &self.config.weights,
        );
        Some(Discovery {
            id: Uuid::new_v4(),
            dtype: DiscoveryType::Drift,
            members: vec![early.clone(), late.clone()],
            evidence: Evidence {
                semantic_sim: 0.0,
                temporal_overlap: 0.0,
                frequency_signal: frequency_signal(early.frequency, late.frequency),
                period_from: early.first_seen,
                period_to: late.last_seen,
                note: Some(format!("감정 점수 {:.2} → {:.2} ({direction} 이동)", early.avg_emotion_score, late.avg_emotion_score)),
            },
            score,
            weak_signal: score < self.config.weak_signal_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::{Embedder, HashEmbedder};
    use crate::vector::InMemoryVectorStore;

    // 테스트 레코드 헬퍼
    fn rec(source: SourceId, text: &str, freq: u64, from: &str, to: &str) -> KeywordRecord {
        KeywordRecord {
            source,
            text: text.into(),
            normalized_text: text.replace(' ', "").to_lowercase(),
            frequency: freq as f64,
            avg_emotion_score: 0.0,
            first_seen: NaiveDate::parse_from_str(from, "%Y-%m-%d").ok(),
            last_seen: NaiveDate::parse_from_str(to, "%Y-%m-%d").ok(),
        }
    }

    // 레코드 목록을 해시 임베더로 색인
    fn index(records: &[KeywordRecord]) -> InMemoryVectorStore {
        let e = HashEmbedder::new(128);
        let texts: Vec<String> = records.iter().map(|r| r.text.clone()).collect();
        let vecs = e.embed(&texts).unwrap();
        let mut s = InMemoryVectorStore::new();
        for (r, v) in records.iter().zip(vecs) {
            s.upsert(r.key(), v).unwrap();
        }
        s
    }

    // 기간 겹침·빈도 신호·융합 스코어 수치 검증
    #[test]
    fn fusion_math() {
        let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok();
        // 완전 포함: 짧은 구간 기준 1.0
        let t = temporal_overlap(d("2026-05-01"), d("2026-07-31"), d("2026-06-01"), d("2026-06-30"));
        assert!((t - 1.0).abs() < 1e-6);
        // 겹침 없음
        assert_eq!(temporal_overlap(d("2026-01-01"), d("2026-01-31"), d("2026-03-01"), d("2026-03-31")), 0.0);
        // 빈도 신호
        assert!((frequency_signal(4.0, 16.0) - 0.5).abs() < 1e-6);
        assert_eq!(frequency_signal(0.0, 10.0), 0.0);
        // 융합: 기본 가중치 0.6/0.2/0.2
        let w = FusionWeights::default();
        assert!((fusion_score(1.0, 1.0, 1.0, &w) - 1.0).abs() < 1e-6);
        assert!((fusion_score(0.8, 0.5, 0.5, &w) - (0.48 + 0.1 + 0.1)).abs() < 1e-6);
    }

    // 브리지: 다른 소스의 유사 표기 키워드가 이어지고, 무관 키워드는 이어지지 않음
    #[test]
    fn bridge_detection_cross_source_only() {
        let records = vec![
            rec(SourceId::TxtDiary, "observer effect", 10, "2026-06-01", "2026-06-30"),
            rec(SourceId::TxtBrain, "observer effects", 8, "2026-06-10", "2026-07-10"),
            rec(SourceId::TxtBrain, "김치찌개 레시피", 5, "2026-06-01", "2026-06-30"),
            // 같은 소스 내 유사 쌍은 브리지가 아님
            rec(SourceId::TxtDiary, "observer bias", 4, "2026-06-01", "2026-06-30"),
        ];
        let store = index(&records);
        let engine = DiscoveryEngine::new(DiscoveryConfig { bridge_sim_cut: 0.5, ..Default::default() });
        let bridges = engine.detect_bridges(&records, &store).unwrap();

        assert!(!bridges.is_empty());
        // 최상위 브리지는 observer effect(일기) ↔ observer effects(문서)
        let top = &bridges[0];
        assert_eq!(top.dtype, DiscoveryType::Bridge);
        let sources: BTreeSet<_> = top.members.iter().map(|m| m.source.clone()).collect();
        assert_eq!(sources.len(), 2, "브리지는 반드시 교차 소스");
        // 근거 문장이 완전한 한국어 문장으로 생성되는지
        let sentence = top.evidence_sentence();
        assert!(sentence.contains("의미 유사도"));
        assert!(sentence.contains("브리지"));
        // 김치찌개는 어떤 브리지에도 등장하지 않아야 함
        assert!(bridges.iter().all(|b| b.members.iter().all(|m| !m.text.contains("김치찌개"))));
    }

    // 갭: Brain에만 강한 키워드가 Diary에 의미 이웃이 없으면 갭
    #[test]
    fn gap_detection() {
        let records = vec![
            rec(SourceId::TxtBrain, "quantum decoherence", 9, "2026-06-01", "2026-06-30"),
            rec(SourceId::TxtDiary, "산책 일기", 6, "2026-06-01", "2026-06-30"),
        ];
        let store = index(&records);
        let engine = DiscoveryEngine::new(DiscoveryConfig::default());
        let gaps = engine.detect_gaps(&records, &store).unwrap();

        let decoherence_gap = gaps
            .iter()
            .find(|g| g.members[0].text == "quantum decoherence")
            .expect("decoherence 갭이 탐지되어야 함");
        assert_eq!(decoherence_gap.dtype, DiscoveryType::Gap);
        assert!(decoherence_gap.evidence.note.as_ref().unwrap().contains("일기"));
    }

    // 클러스터: 세 소스에 흩어진 유사 키워드가 하나의 덩어리로 묶임
    #[test]
    fn emergent_cluster_across_sources() {
        let records = vec![
            rec(SourceId::TxtDiary, "observer effect diary", 12, "2026-05-01", "2026-07-10"),
            rec(SourceId::TxtBrain, "observer effect paper", 8, "2026-05-15", "2026-07-01"),
            rec(SourceId::TxtAiMemory, "observer effect chat", 5, "2026-06-01", "2026-07-10"),
            rec(SourceId::TxtDiary, "김치찌개", 3, "2026-06-01", "2026-06-05"),
        ];
        let store = index(&records);
        let engine = DiscoveryEngine::new(DiscoveryConfig {
            cluster_sim_cut: 0.5,
            cluster_min_size: 3,
            ..Default::default()
        });
        let clusters = engine.detect_clusters(&records, &store).unwrap();

        assert_eq!(clusters.len(), 1, "observer 덩어리 1개만 나와야 함");
        let c = &clusters[0];
        assert_eq!(c.dtype, DiscoveryType::Cluster);
        assert!(c.members.len() >= 3);
        let sources: BTreeSet<_> = c.members.iter().map(|m| m.source.clone()).collect();
        assert!(sources.len() >= 2, "클러스터는 최소 2개 소스에 걸쳐야 함");
        assert!(c.members.iter().all(|m| m.text.contains("observer")));
    }

    // 드리프트: 감정 점수가 임계값 이상 이동하면 탐지, 미만이면 None
    #[test]
    fn drift_detection() {
        let mut early = rec(SourceId::TxtDiary, "몰입", 5, "2026-04-01", "2026-04-30");
        early.avg_emotion_score = -0.1;
        let mut late = rec(SourceId::TxtDiary, "몰입", 7, "2026-06-01", "2026-06-30");
        late.avg_emotion_score = 0.6;

        let engine = DiscoveryEngine::new(DiscoveryConfig::default());
        let drift = engine.detect_drift(&early, &late).expect("0.7 이동은 드리프트");
        assert_eq!(drift.dtype, DiscoveryType::Drift);
        assert!(drift.evidence.note.as_ref().unwrap().contains("긍정 방향"));

        let mut small = late.clone();
        small.avg_emotion_score = 0.1; // delta 0.2 < 0.4
        assert!(engine.detect_drift(&early, &small).is_none());
    }

    // 결정성: 같은 입력이면 같은 후보 집합(순서 포함, id 제외)
    #[test]
    fn deterministic_output() {
        let records = vec![
            rec(SourceId::TxtDiary, "observer effect", 10, "2026-06-01", "2026-06-30"),
            rec(SourceId::TxtBrain, "observer effects", 8, "2026-06-10", "2026-07-10"),
            rec(SourceId::TxtAiMemory, "observer effect log", 5, "2026-06-05", "2026-07-05"),
        ];
        let store = index(&records);
        let engine = DiscoveryEngine::new(DiscoveryConfig { bridge_sim_cut: 0.5, ..Default::default() });
        let a = engine.detect_bridges(&records, &store).unwrap();
        let b = engine.detect_bridges(&records, &store).unwrap();
        let sig = |v: &[Discovery]| -> Vec<(String, String, String)> {
            v.iter()
                .map(|d| {
                    (
                        d.members[0].normalized_text.clone(),
                        d.members[1].normalized_text.clone(),
                        format!("{:.4}", d.score),
                    )
                })
                .collect()
        };
        assert_eq!(sig(&a), sig(&b));
    }
}
