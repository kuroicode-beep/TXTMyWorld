// core/src/lib.rs — TXTMyWorld 코어 엔진 진입점: 모듈 선언·버전 상수·에러 타입
//
// 구조 (아키텍처 문서 §3 대응):
//   models    — 공통 Keyword/Context API 스키마 v1.0/v1.1 타입 + 스키마 방어
//   source    — 소스 클라이언트: 파싱·병합·폴백 격리·HTTP(X1 소비)
//   embedding — Embedder 트레이트 + Ollama(bge-m3) + 결정적 테스트 임베더 + 전략 A/B 선택
//   vector    — VectorStore 트레이트 + 인메모리 KNN + 코사인/정규화 + 공간 정합
//   discovery — 3축 융합 스코어(브리지/갭/클러스터/드리프트) + 근거 문장(접근성)
//   topic     — 주제 카드 (유일한 자체 소유 원본 데이터)
//   feedback  — X2 TXTAIMemory MCP 환류 페이로드 (payload_schema v1.0, 멱등)
//   store     — SQLite 저장소 (PRD §8 테이블; vec_index는 sqlite-vec 통합 시 확장)

pub mod discovery;
pub mod embedding;
pub mod feedback;
pub mod models;
pub mod source;
pub mod store;
pub mod topic;
pub mod vector;
#[cfg(feature = "sqlitevec")]
pub mod vector_sqlite;

/// 앱 버전 단일 소스(루트 VERSION 파일과 동기 유지)
pub const APP_VERSION: &str = "0.2.0";

/// 버전 히스토리 (버전, 날짜, 요약) — UI 설정의 "업데이트 히스토리" 메뉴가 이 목록을 렌더링한다.
/// 최신 버전이 배열 끝에 오도록 유지한다(화면에서는 최신순으로 뒤집어 표시).
pub const VERSION_HISTORY: &[(&str, &str, &str)] = &[
    (
        "0.1.0",
        "2026-07-12",
        "프로젝트 초기화. 코어 엔진(스키마 v1.0/v1.1 파싱, sqlite-vec KNN, 3축 융합 발견, 주제 카드, X2 페이로드, SQLite 저장소)과 \
         Tauri 데스크톱 앱(소스 페어링·동기화·발견·보관함·설정 화면, OS 보안 저장소 토큰, 접근성 테마) 구현.",
    ),
    (
        "0.2.0",
        "2026-07-14",
        "SVIL 표준 디자인 전면 적용 — 고대비 다크 팔레트, 교보손글씨2019 기본 + 글꼴 8종, 화면(언어 5종·글자크기 3단계·\
         글꼴) 설정 메뉴, 전 화면 다국어(ko/en/ja/zh/vi), Alt+←/→ 뒤로/앞으로 내비게이션. 임시 접근성 토글은 SVIL \
         표준으로 대체.",
    ),
];

/// 코어 공통 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("스키마 버전이 지원 범위를 초과: {0} (업데이트 필요)")]
    SchemaUpdateRequired(String),
    #[error("스키마 파싱 실패: {0}")]
    SchemaInvalid(String),
    #[error("벡터 차원 불일치: 기대 {expected}, 실제 {actual}")]
    DimMismatch { expected: usize, actual: usize },
    #[error("저장소 오류: {0}")]
    Store(#[from] rusqlite::Error),
    #[error("HTTP 오류: {0}")]
    Http(String),
    #[error("직렬화 오류: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
