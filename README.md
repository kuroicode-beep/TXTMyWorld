# TXTMyWorld

> TXT 패밀리의 **연결·생성 레이어**. 세 종류의 맥락(개인 기억·문서 지식·AI 대화)을 **기간·빈도·벡터(의미)**로 조합해 연관을 찾고, 새로운 맥락을 만들어내는 개인 지능 생태계의 최종 앱. **벡터·의미 검색이 핵심 축**이다.

현재 버전: `v0.2.3` · 상태: **3소스 직접 연결 + 발견 파이프라인 실동작 (실데이터 98건 발견 검증)**

## TXT 패밀리 안에서의 위치

```
[맥락화 3축]                     [시각화]        [연결·생성]
TXTDiary  (개인 기억) ─┐
TXTBrain  (문서 지식) ─┼─▶ TXTSpace (보기) ─▶ TXTMyWorld (잇기·만들기)
TXTAIMemory (AI 대화) ─┘
```

- **TXTSpace = 보기.** 세 소스를 하나의 지도로 시각화한다.
- **TXTMyWorld = 잇기·만들기.** 그 지도 위에서 새 연결을 발견하고 생성한다. (본 프로젝트)

## 문서

프로젝트 전반과 TXT 패밀리 시리즈 컨텍스트는 `docs/`에 정리되어 있다.

- 시작점: [`docs/context/README_txt-series-overview_20260712.md`](docs/context/README_txt-series-overview_20260712.md)
- 상위 규약: [`docs/architecture/masterspec_20260709_txt-family-master_yumi.md`](docs/architecture/masterspec_20260709_txt-family-master_yumi.md)
- 형제 PRD: [`docs/prd/`](docs/prd/) (TXTSpace, TXTDiary)

프로젝트 규칙·버전 정책은 [`CLAUDE.md`](CLAUDE.md) / [`AGENTS.md`](AGENTS.md) / [`VERSIONING.md`](VERSIONING.md) 참조.

## PRD

- [`docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md`](docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md) — TXTMyWorld v0.1 PRD 초안 (문서 v0.2)
  - 핵심: 세 소스 키워드를 **기간·빈도·벡터(의미)**로 조합해 새 주제를 발견·생성
  - **벡터·의미 검색이 핵심 축** — RC까지 최종에 가깝게 설계 (PRD §3.4)
  - 방향: 하이브리드 발견 · 주제 카드+환류 · 로컬 우선+동의형 클라우드 · 독립 앱

## 코어 엔진 (`core/`)

Rust 크레이트 `txtmyworld-core` — 주요 로직 구현 완료 (테스트 33/33, `sqlitevec` feature 포함, clippy 클린).

- `models` 공통 스키마 v1.0/v1.1 파싱·방어 · `source` 통합 조회·병합·폴백(X1 소비)
- `embedding` bge-m3(Ollama)·전략 A/B 선택 · `vector` KNN 트레이트·공간 정합 · `vector_sqlite` **sqlite-vec** 구현체(feature `sqlitevec`)
- `discovery` 3축 융합(브리지/갭/클러스터/드리프트)+근거 문장 · `topic` 주제 카드
- `feedback` X2 환류 페이로드(멱등·본문 미포함) · `store` SQLite 저장소(전체 조회 API 포함)

```
cd core && cargo test --features sqlitevec
```

## 데스크톱 앱 (`app/`)

Tauri 2 + React 19 + TypeScript. `txtmyworld-core`를 path 의존성으로 연결한 실동작 앱 셸.

- **Rust(`app/src-tauri`)**: IPC 커맨드(`commands.rs`), 동기화·발견 오케스트레이션(`pipeline.rs`), 임베더 선택(`embed_select.rs`), X2 HTTP 브리지(`feedback_client.rs`), 페어링 토큰 OS 보안 저장소(`secure.rs`, keyring + SHA-256 지문).
- **React(`app/src`)**: S0(온보딩·페어링) · S1(피드) · S2(발견 상세, 카드에 내장) · S3(카드 생성 모달) · S4(탐색) · S5(보관함) · S6(설정: 화면·엔진 파라미터·연동·버전/업데이트 히스토리). 리스트 뷰로 전 정보 접근 가능(그래프 뷰는 v0.1 범위 밖, MoSCoW Should).
- **데모 모드**: 실 소스 없이도 "데모 데이터로 체험하기"로 발견 루프 전체를 확인 가능.

### SVIL 디자인·설정 표준 (v0.2.0)

`docs`의 SVIL 프론트엔드 가이드를 전면 적용했다.

- **디자인**: 고대비 다크 팔레트(`app/src/theme.css` 토큰), 교보손글씨2019 기본 폰트, 단일 굵기 폰트라 제목 위계는 크기·색으로만 표현(bold 합성 금지), 숫자·타임스탬프·버전은 Consolas 모노(`.mono`).
- **화면 설정 표준**(`app/src/lib/prefs.ts`, `lib/i18n.ts`, Settings §화면): 글꼴 8종(전부 로컬 번들 `app/public/fonts/`) · 글자 크기 3단계(16/18/20px) · 다국어 5종(ko/en/ja/zh/vi, 전 화면 사전 키 적용). 백엔드가 아니라 프론트 `localStorage`에 저장(참조 구현 TXTAIMemory와 동일 패턴).
- **접근성**: 최소 터치 타겟 50px, 포커스 링 상시, `prefers-reduced-motion` 존중, **Alt+←/→** 및 마우스 뒤로/앞으로 버튼으로 탭 히스토리 내비게이션(`App.tsx`, `history.pushState`/`popstate`).
- **범위 경계**: 백엔드가 생성하는 근거 문장(`evidence_sentence`)과 사용자 데이터(카드 이름·메모·키워드 텍스트)는 가이드의 "사용자 데이터는 번역 제외" 원칙에 따라 다국어 대상에서 제외.

```
cd app && npm install
npm run tauri dev      # 개발 모드 (네이티브 창)
npm run build           # 프론트엔드 타입체크 + 빌드 검증
cd src-tauri && cargo test && cargo clippy --all-targets
```

## 다음 할 일

- [x] TXTMyWorld 전용 PRD 초안 (패밀리 마스터 §2.1·§3 기반)
- [x] 앱 이름 확정 → **TXTMyWorld** (PRD §16 D0)
- [x] 벡터 소스 확정 → 이중 전략(소스측 공유 + 로컬 임베딩), 기본 모델 bge-m3 (PRD §3.4, §16 D1/D2)
- [x] 코어 엔진 구현 (스키마·벡터·발견·카드·환류·저장소, sqlite-vec 포함)
- [x] Tauri 데스크톱 앱 셸 + S0~S6 전 화면 + OS 보안 저장소
- [x] SVIL 디자인·설정 표준 적용 (고대비 다크·교보손글씨2019·글꼴 8종·글자크기 3단계·다국어 5종·Alt+←/→ 내비게이션)
- [x] 상위 3축(TXTDiary/TXTBrain/TXTAIMemory) 실서버 페어링·동기화 실검증 — 실배포 스키마 편차 수정, TXTSpace-hub 통합 옵션으로 195개 실제 키워드 수신 확인 (통합 스펙 §6)
- [x] **3소스 개별 직접 연결** — TXTSpace 공유 페어링 토큰(keyring `TXTSpace`) 재사용으로 소스 앱 무수정 직접 연결. 소스별 헤더·items 스키마 수용, "3개 앱에 지금 연결" 원클릭. 라이브 검증(12/100/78 키워드) (통합 스펙 §7)
- [ ] 패밀리 협의: 공통 API 벡터 공유 확장(schema v1.1) 소스측 생산자 구현, TXTAIMemory MCP 쓰기 실제 수신 방식 확정 (PRD §16 X1-a/X2-a)
- [ ] 릴리즈 패키징(msi/nsis) 및 배포 준비
