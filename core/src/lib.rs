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
pub const APP_VERSION: &str = "0.2.5";

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
    (
        "0.2.1",
        "2026-07-14",
        "실제 3앱(TXTDiary/TXTBrain/TXTAIMemory) 연동 버그 수정 — 실배포 스키마(keyword/cooccurrences/항목별 \
         source) 수용, 소스 동기화 파싱 실패를 조용히 삼키던 버그 제거, 실측 포트로 기본값 교정, 토큰 없이 3소스를 \
         한 번에 받는 TXTSpace 허브 연결 옵션 추가. 실제 로컬 서비스 대상 라이브 검증 완료(190개 키워드 수신).",
    ),
    (
        "0.2.2",
        "2026-07-14",
        "3소스 개별 직접 연결 — TXTDiary/TXTBrain/TXTAIMemory에 허브를 거치지 않고 각각 직접 연결. TXTSpace가 이미 \
         발급받은 공유 페어링 토큰(Windows 자격 증명, 서비스명 TXTSpace)을 자동 재사용하므로 토큰 입력 없이 연결된다. \
         소스별 인증 헤더(TXTAIMemory=X-Pairing-Token, 나머지=Bearer)와 TXTAIMemory items 스키마(keyword/weight/ai_id) \
         수용, \"3개 앱에 지금 연결\" 원클릭 버튼 추가. 소스 앱은 전혀 수정하지 않고 TXTMyWorld만 확장.",
    ),
    (
        "0.2.3",
        "2026-07-14",
        "발견이 항상 0건이던 치명 버그 2건 수정 — (1) SqliteVecStore::get()이 항상 None을 반환하는 스텁이라 \
         실제 앱의 발견 엔진이 시드 벡터를 못 가져와 모든 발견을 조용히 skip했다(인메모리 저장소는 정상이라 유닛 \
         테스트가 못 잡음). 실제 벡터를 복원하도록 수정하고 SqliteVecStore E2E 회귀 테스트 추가. (2) 임베더가 \
         Ollama 가동만 확인하고 bge-m3 미설치 시에도 그걸로 시도해 임베딩이 전부 실패했다 — /api/tags로 실제 설치된 \
         임베딩 모델(nomic-embed-text 등)을 감지·선택하고 실패 시 해시 폴백. 의미 임베딩에 맞춰 발견 임계값 상향 \
         (0.6→0.85). 실측: 195키워드/3소스 → 발견 98건(브리지 69·갭 19·클러스터 10).",
    ),
    (
        "0.2.4",
        "2026-07-15",
        "TXTAIMemory가 스키마 v2.0(keywords 배열)을 items와 함께 보내기 시작한 것에 대응 — merge_keywords가 \
         keywords가 있으면 items(레거시)는 건너뛰도록 수정해 이중 집계를 방지. keywords 쪽이 실제 first_seen/last_seen을 \
         담고 있어 AIMemory 키워드의 시간축 발견(temporal_overlap) 정확도가 개선됨(TXT 패밀리 스키마 통합 Phase 2).",
    ),
    (
        "0.2.5",
        "2026-07-15",
        "X1 벡터 공유가 실제로는 항상 버려지고 있던 버그 수정 — sync_source가 소스 공유 벡터와의 공간 정합 \
         판정을 \"bge-m3/1024\"로 고정 비교하고 있었다(이 기기의 실제 선택 모델은 nomic-embed-text/768). \
         TXTDiary·TXTBrain·TXTAIMemory 3소스가 방금 /vectors를 제공하기 시작했는데도 이 하드코딩 때문에 \
         전부 \"불일치\"로 판정되어 조용히 버려지고 있었음. select_embedder()로 실제 로컬 모델을 조회해 \
         비교하도록 수정 — 이제 실제로 소스 공유 벡터를 받아 저장한다(전략 A).",
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
