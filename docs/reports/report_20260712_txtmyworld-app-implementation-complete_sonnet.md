# 완료보고서 — TXTMyWorld 데스크톱 앱 구현 (v0.1 MVP)

작성일: 2026-07-12 / 작성자: Claude Sonnet 5 / 요청자: InBlue (소장님)
이어받은 작업: Claude Code(코어 엔진 구현) → Claude Fable 5(기획 문서 세트) → **Claude Sonnet 5(앱 구현, 본 보고서)**

---

## 1. 작업 개요

인계받은 작업지시서(§10)의 "Sonnet이 이어서 할 것" 6개 항목을 전부 구현했다. Tauri 2 + React 19 데스크톱 앱이 실제로 빌드·렌더링되며, PRD가 정의한 S0~S6 전 화면과 IPC 백엔드가 갖춰졌다.

## 2. 구현 범위

### 2.1 Rust 백엔드 (`app/src-tauri`)

| 파일 | 역할 |
|-----|-----|
| `commands.rs` | 페어링·동기화·발견·카드CRUD·환류·설정 전체 IPC 커맨드(19개) |
| `pipeline.rs` | 동기화·발견 오케스트레이션(fetch→merge→embed→색인→발견→영속화), 데모 시드 |
| `secure.rs` | keyring(OS 보안 저장소) 토큰 저장, SHA-256 지문(sha2 크레이트) |
| `feedback_client.rs` | X2 환류 HTTP 브리지(트레이트 기반, 프로토콜 확정 전 임시 구현) |
| `embed_select.rs` | Ollama bge-m3 우선, 미가동 시 결정적 해시 폴백 |
| `dto.rs`, `state.rs` | 프론트엔드 직렬화 DTO, 전역 상태(Mutex\<Store\>) |

### 2.2 React 프론트엔드 (`app/src`)

S0 온보딩·페어링, S1 추천 피드, S2 발견 상세(카드에 내장), S3 주제 카드 생성 모달, S4 탐색(필터), S5 보관함(편집·환류·Markdown 내보내기), S6 설정(엔진 파라미터·연동 주소·접근성·버전/업데이트 히스토리).

접근성: 다크·고대비 기본, 최소 폰트 16px, 터치 타겟 50px, 색+텍스트 라벨 병행, 그래프 없이 리스트만으로 전 정보 접근 가능.

### 2.3 코어 확장 (`core/`) — RC 계약 불변

- `vector_sqlite.rs`: `VectorStore` 트레이트의 **sqlite-vec** 구현체(feature `sqlitevec`). PRD §3.4.3이 요구한 "규모 확장 시 ANN 승격 경로"의 실증.
- `store.rs`: UI 배선에 필요한 조회 메서드(list/get 계열) 추가. 기존 시그니처·스키마는 변경 없음.
- `models.rs`: `SourceId::as_str`/`parse_lenient` 헬퍼(가독성 개선).

## 3. 검증 결과

| 항목 | 결과 |
|-----|-----|
| `cargo test`(core, feature sqlitevec) | 33/33 통과 |
| `cargo test`(app) | 1/1 통과 |
| `cargo clippy --all-targets`(core, app) | 경고 0 |
| `npx tsc --noEmit` | 오류 0 |
| `npm run build`(vite) | 성공 |
| 실 렌더링 확인 | Vite dev 서버를 브라우저 프리뷰로 열어 온보딩→발견→탐색→보관함→설정 4탭 전환, 콘솔 에러 0건 확인 |

## 4. 개발 중 발견·수정한 버그 2건

1. **임베딩 매핑 인덱스 어긋남** (`pipeline.rs::run_discovery`) — 신규 임베딩이 필요한 키워드 목록(`pending_texts`)과 임베딩 결과를 다시 짝지을 때 전체 레코드 리스트와 `zip`했던 실수. 인덱스가 어긋나 엉뚱한 키워드에 벡터가 붙을 뻔했다. `pending_records`를 별도로 모아 수정.
2. **X1 벡터 무검증 수용** (`pipeline.rs::sync_source`) — 소스가 공유하는 벡터를 차원·모델 검증 없이 그대로 저장하면, sqlite-vec의 고정 차원 테이블이 깨질 수 있었다. `VectorSpace::is_compatible`로 정합을 확인한 뒤에만 저장하고, 불일치 시 조용히 건너뛰어 발견 단계에서 로컬 재임베딩(전략 B)으로 자연스럽게 폴백하도록 고쳤다.
3. **Settings 화면 무한 로딩** (`Settings.tsx`) — 브라우저 프리뷰로 검증하던 중 발견. `Promise.all`로 묶인 설정 조회 하나가 실패하면 화면 전체가 "불러오는 중…"에 영구히 갇히는 회복성 결함이었다. 세 호출을 개별 try/catch로 분리하고 실패 시 기본값으로 폴백하도록 수정 — 실제 배포 후 백엔드 일시 오류가 있어도 사용자가 갇히지 않는다.

## 5. 아직 남은 것 (코드 문제 아님 — 패밀리 차원 협의/외부 검증)

- **X1-a**: 공통 API 벡터 공유 확장(schema v1.1)을 마스터·3축 앱이 실제 구현해야 소스측 공유(전략 A)가 작동한다. 현재는 로컬 재임베딩(전략 B)으로 전 기능 정상 동작.
- **X2-a**: TXTAIMemory의 실제 MCP 수신 방식(stdio vs HTTP 게이트웨이) 미확정. 현재는 임시 HTTP 브리지.
- 실행 중인 TXT 패밀리 서버가 이 환경에 없어 페어링·동기화 E2E는 코드 경로 정적 검증까지만 진행.
- 2D 관계 뷰(그래프), HNSW 승격, 설치 패키징(msi/nsis)은 MoSCoW Should/후속 범위로 의도적으로 남김.

## 6. 참고

- 상세 인계 기록: `docs/handoff/handoff_20260712_txtmyworld-sprint0-1_claudecode.md` §11
- 저장소: https://github.com/kuroicode-beep/TXTMyWorld (커밋 `f2d9745`)
- 실행: `cd app && npm install && npm run tauri dev`
