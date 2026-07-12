# TXTMyWorld 작업지시서 — Sprint 0~1 (착수)

문서 버전: v0.3 / 작성일: 2026-07-12 (갱신) / 작성자: Claude Code → Claude Sonnet 5 (SVIL) / 대상: Codex(QA)·후속 세션

> **[구현 현황 v0.3]** 코어 엔진(`core/`) + Tauri 데스크톱 앱(`app/`) **모두 구현 완료** — core 테스트 33/33, app 테스트 통과, `cargo clippy` 경고 0, 프론트엔드 `tsc`+`vite build` 클린, 실제 렌더링 확인(Vite dev 서버로 4개 탭 전환·에러 없음 검증). §11 Sonnet 완료 보고 참조. 남은 것은 패밀리 차원 협의(X1-a/X2-a)와 실서버 연동 검증뿐이다.

> 상위: PRD(v0.3), 통합 스펙(X1/X2), 아키텍처, 로드맵, 스토리보드. 본 문서는 **착수 스프린트의 실행 단위**다. 스택 Tauri + React + Rust, Windows 우선. 코드 규칙: 파일 경로 주석, 함수 상단 한 줄 주석, DRY, 에러 핸들링, 민감정보 노출 금지(CLAUDE.md §8).

---

## 0. 공통 준비 (Sprint 0)

| # | 작업 | 산출물 | 완료 기준 |
|---|-----|-----|--------|
| 0.1 | Tauri + React 앱 스캐폴딩 | 실행되는 빈 셸, `APP_VERSION=0.1.0` | 창 제목·상단에 `v0.1.0` 표시 |
| 0.2 | SQLite + sqlite-vec 연결 | DB 초기화, 마이그레이션 러너 | vec0 가상 테이블 생성 확인 |
| 0.3 | 모듈 골격 | `source_client/embedding/vector/discovery/store/feedback/synth/security` | 아키텍처 §3 구조와 일치 |
| 0.4 | 설정·접근성 셸 | 다크·고대비·글자크기, 업데이트 히스토리 메뉴 | 키보드로 설정 도달, `VERSION_HISTORY` 렌더 |
| 0.5 | i18n(ko/en) 골격 | locales 로드 | 라벨 하드코딩 없음 |

## 1. 소스 통합 조회 — X1 소비 (Sprint 1)

| # | 작업 | 완료 기준 |
|---|-----|--------|
| 1.1 | 소스 페어링(승인 다이얼로그 수신, 토큰 OS 보안 저장) | 3소스 각각 페어링, 토큰 평문 저장 없음 |
| 1.2 | `/health` 파서(schema_version, vector_capability) | v1.0/v1.1 모두 파싱, 미지원 감지 |
| 1.3 | `/keywords` 통합 조회·병합(`source`별) | 3소스 병합, keyword_cache upsert |
| 1.4 | `/vectors` 증분 동기화(`since`,`cursor`) | 지원 소스 벡터 upsert(origin=shared) |
| 1.5 | 폴백 격리(소스 미실행/미지원) | 한 소스 꺼져도 나머지 정상, 오프라인 배지 |
| 1.6 | 스키마 방어(상위 버전·손상) | 오류 대신 안내, 앱 비중단 |

**참조:** 통합 스펙 §2. **주의:** 소스에 쓰기 요청 금지(GET only). 본문 필드 있으면 무시.

## 2. 벡터 엔진 (Sprint 1, 핵심)

| # | 작업 | 완료 기준 |
|---|-----|--------|
| 2.1 | bge-m3 로컬 임베딩 런타임(배치·증분) | 키워드 텍스트→1024-dim, L2 정규화 |
| 2.2 | embeddings 저장(source/origin/model/dim 태깅) | 전략 A/B 혼재 태깅 |
| 2.3 | sqlite-vec 색인 + KNN | top-K 최근접 질의 동작 |
| 2.4 | 공간 정합(모델/차원 일치 판정, 불일치 재임베딩) | 이종 모델 격리 or 정렬 |
| 2.5 | 교차소스 의미 매칭 | 다른 표기 같은 개념을 소스 간 연결 |
| 2.6 | 기본 의미 클러스터 + 의미 기반 질의 | 클러스터 후보 추출, 자연어 질의→이웃 |

**주의:** 벡터는 로컬 저장·외부 미전송. 데이터 모델은 RC 형태로(축소는 규모/튜닝만) — 나중에 갈아엎지 않게.

## 3. 발견 엔진 (Sprint 1)

| # | 작업 | 완료 기준 |
|---|-----|--------|
| 3.1 | 3축 융합 스코어 `w_s·sim + w_t·overlap + w_f·freq` | 결정적·재현 가능, 낮은 유사도 컷 |
| 3.2 | 발견 유형(브리지/갭/클러스터/드리프트) | discoveries에 type·evidence 저장 |
| 3.3 | 근거 산출(수치+문장) | 리스트/스크린리더용 완전 문장 생성 |

## 4. 발견→생성 UI (Sprint 1)

| # | 작업 | 완료 기준 |
|---|-----|--------|
| 4.1 | 추천 피드(S1) | 후보 카드 목록, 유형 배지(색+라벨) |
| 4.2 | 발견 상세·근거(S2) | 근거 표·문장, 키워드 딥링크 |
| 4.3 | 주제 카드 생성·저장(S3) | 채택·명명·근거 스냅샷, topic_cards CRUD |
| 4.4 | 리스트 뷰 동등성 | 그래프 없이도 전부 조작 가능 |
| 4.5 | 키보드·고대비·400% 확대 | 주요 흐름 무손상 |

## 5. 환류 — X2 (Sprint 1 기본)

| # | 작업 | 완료 기준 |
|---|-----|--------|
| 5.1 | 환류 토글 + 전송 항목 고지 동의 | 본문 미포함 항목만 표시 |
| 5.2 | MCP 쓰기 페이로드 생성(payload_schema v1.0, external_id 멱등) | 통합 스펙 §3.3 형식 |
| 5.3 | 미연결 폴백·이력 기록(feedbacks) | 비활성 안내, 성공 시 memory_id 저장 |

**참조:** 통합 스펙 §3. **주의:** 자동 무한 재시도 금지, 동의 없이 전송 금지.

## 6. QA 체크리스트 (Codex)

- [ ] 소스 쓰기 요청 0건(네트워크 캡처로 GET only 검증).
- [ ] 본문 텍스트가 어떤 외부 경로(X2/DeepSeek)에도 실리지 않음.
- [ ] 벡터 dim 불일치·상위 schema_version에서 앱 비중단.
- [ ] 한 소스만 켜도 발견 동작(폴백 격리).
- [ ] 인터넷·클라우드 없이 전 기능 동작.
- [ ] 키보드만으로 피드→상세→카드 생성→환류 토글 완주.
- [ ] X2 멱등: 같은 카드 재환류가 update로 처리.

## 7. 착수 순서 권장

`0.1→0.2→0.3` → `1.1~1.3` → `2.1~2.4`(벡터 코어) → `3.1~3.3` → `4.1~4.5` → `1.4/2.5/2.6`(의미 심화) → `5.1~5.3`(환류).

## 8. 열린 값(개발 중 확정)

- 융합 가중치 w_s/w_t/w_f 기본값, 유사도 컷, 클러스터 최소 크기.
- bge-m3 배포 형식(Ollama pull vs 번들).
- 추천 피드 계산 주기·재색인 트리거.

## 10. 인계 노트 — 코어 엔진 구현 완료 (2026-07-12, Claude Code → Sonnet)

**위치:** `core/` — Rust 라이브러리 크레이트 `txtmyworld-core`. Tauri `src-tauri`가 path 의존으로 사용하면 된다. `cargo test` 26/26 통과, `cargo clippy` 경고 0.

**코어에서 구현 완료 (✅):**

| 지시 항목 | 모듈 | 내용 |
|--------|-----|-----|
| 1.2 /health 파서 ✅ | `models.rs`, `source.rs` | schema v1.0/v1.1, `vector_capability` 감지, 방어(상위 major→UpdateRequired) |
| 1.3 /keywords 파싱·병합 ✅ | `source.rs` | 소스별 별도 항목·결정적 순서 병합(`merge_keywords`), HTTP GET 클라이언트(read-only) |
| 1.4 /vectors 파싱 ✅ | `source.rs` | X1 배치·증분(`since`/`cursor`), dim 불일치 레코드 스킵(비중단) |
| 1.5/1.6 폴백·방어 ✅ | `source.rs` | `SourceFetch::{Ok,UpdateRequired,Offline}` 소스 단위 격리 |
| 2.1 임베딩 런타임 ✅ | `embedding.rs` | `Embedder` 트레이트 + `OllamaEmbedder`(bge-m3, /api/embed) + `HashEmbedder`(테스트용) |
| 2.2 임베딩 저장 ✅ | `store.rs` | embeddings 테이블(source/origin/model/dim 태깅, BLOB 왕복) |
| 2.3 KNN ✅ | `vector.rs` | `VectorStore` 트레이트 + 인메모리 브루트포스(결정적) — sqlite-vec은 동일 트레이트로 교체 |
| 2.4 공간 정합 ✅ | `vector.rs`, `embedding.rs` | `VectorSpace::is_compatible` + `choose_strategy`(UseShared/RealignLocally/LocalOnly) |
| 2.5/2.6 의미 연산 ✅ | `discovery.rs` | 교차소스 매칭(브리지), 그리디 의미 클러스터, KNN 질의 기반 |
| 3.1~3.3 발견 엔진 ✅ | `discovery.rs` | 융합 스코어(w 0.6/0.2/0.2), 브리지/갭/클러스터/드리프트, 약한 신호 라벨, **근거 한국어 완전 문장**(`evidence_sentence`) |
| 5.2 X2 페이로드 ✅ | `feedback.rs`, `topic.rs` | payload_schema v1.0, `external_id` 멱등, 본문 미포함 검증 테스트 |
| 저장소 ✅ | `store.rs` | PRD §8 전 테이블 DDL + 캐시/임베딩/발견/카드(soft delete)/환류 이력/설정 CRUD |

**Sonnet이 이어서 할 것 (미구현):**

1. **0.1 Tauri+React 앱 셸** — `src-tauri` 생성, `txtmyworld-core` path 의존, IPC commands로 코어 노출.
2. **0.4/4.x UI** — 피드(S1)·상세(S2)·카드 생성(S3)·탐색(S4)·보관함(S5)·설정(S6), 리스트 뷰 동등성(코어의 `evidence_sentence` 활용), 키보드·고대비.
3. **sqlite-vec 통합** — `vector.rs`의 `VectorStore` 트레이트 구현체 추가(현 인메모리는 v0.1 규모에 충분).
4. **1.1 페어링 UI + OS 보안 저장소** — 토큰은 keyring 등으로, 코어 `SourceConfig.pairing_token`에 주입.
5. **5.1/5.3 환류 실연동** — MCP 클라이언트 호출부(코어는 페이로드·이력까지 제공), 동의 다이얼로그.
6. **동기화 오케스트레이션** — fetch→merge→embed(증분)→index→discover 파이프라인을 백그라운드 작업으로.

**설계 계약(변경 금지):** 코어 데이터 모델·트레이트는 RC 형태다(PRD §3.4). `VectorStore`/`Embedder` 트레이트, `Discovery`/`TopicCard`/Evidence 구조, X2 페이로드 스키마를 바꾸지 말고 구현체만 추가할 것.

## 11. Sonnet 구현 완료 보고 (2026-07-12, Claude Sonnet 5)

§10 인계 노트의 "Sonnet이 이어서 할 것" 6개 항목을 전부 구현했다. 위치는 `app/`(Tauri 2 + React 19 + TS).

| 인계 항목 | 구현 위치 | 비고 |
|---------|---------|-----|
| 1. Tauri+React 앱 셸 | `app/src-tauri`, `app/src` | `txtmyworld-core`를 path 의존(feature `sqlitevec`)으로 연결. 창 제목에 `TXTMyWorld vX.Y.Z` 상시 표시(0.1 §0.1 기준 충족) |
| 2. UI(S0~S6) | `app/src/screens/*`, `app/src/components/*` | Onboarding(S0)·Feed(S1)·발견상세는 DiscoveryCard에 내장(S2)·AdoptModal(S3)·Explore(S4)·Archive(S5)·Settings(S6, 버전+업데이트 히스토리 포함). 다크·고대비 기본, 최소 폰트 16px, 터치 타겟 50px, 색+텍스트 라벨 병행, 리스트 뷰만으로 전 정보 접근(그래프 뷰는 미구현 — MoSCoW Should라 v0.1 범위 밖으로 명시적 스코프 아웃) |
| 3. sqlite-vec 통합 | `core/src/vector_sqlite.rs` (feature `sqlitevec`) | `VectorStore` 트레이트 구현체. `vec0` 가상 테이블 + KNN(`MATCH ... AND k = ?`), L2→코사인 환산. 테스트 2건 포함. `pipeline.rs`의 `run_discovery`가 실제로 이 구현체를 사용(인메모리, 발견 실행마다 조립) |
| 4. 페어링 UI + OS 보안 저장소 | `app/src-tauri/src/secure.rs`, `commands.rs::pair_source` | `keyring` 크레이트(Windows Credential Manager 등). 토큰 원문은 keyring에만, SQLite에는 SHA-256 지문만 저장(코드에 자체 구현 대신 `sha2` 크레이트 사용 — 해시를 손수 구현했다가 검증 부담이 커서 표준 크레이트로 교체) |
| 5. 환류 실연동 | `app/src-tauri/src/feedback_client.rs`, `commands.rs::send_feedback` | `FeedbackTransport` 트레이트 + `HttpFeedbackTransport`(로컬 HTTP POST). **X2-a(AIMemory 실제 수신 프로토콜) 미확정이라 최선의 임시 구현** — MCP stdio 클라이언트가 아니라 설정 가능한 HTTP 엔드포인트로 payload_schema v1.0을 전송한다. 트레이트로 분리해 두었으니 실제 프로토콜 확정 시 구현체만 교체하면 됨. 미연결 시 이력에 `offline`으로 기록, 앱 비중단 |
| 6. 동기화 오케스트레이션 | `app/src-tauri/src/pipeline.rs` | `sync_source`/`sync_all`(fetch→merge→upsert, X1 벡터는 공간 정합 확인 후에만 수용) + `run_discovery`(증분 임베딩→sqlite-vec 색인→3유형 발견→영속화). `seed_demo_data`로 실 소스 없이도 전체 루프 체험 가능(명시적 라벨링, PRD 예시 키워드 재사용) |

**개발 중 발견해 고친 버그 2건 (설계 리뷰 가치):**

1. `run_discovery`에서 신규 임베딩 대상(`pending_texts`/`pending_keys`)을 계산한 뒤, 벡터를 다시 레코드에 매핑할 때 전체 `records` 리스트와 `zip`했던 실수 — 인덱스가 어긋나 엉뚱한 키워드에 임베딩이 붙을 뻔했다. `pending_records`를 별도로 모아 짝을 맞춰 수정.
2. `sync_source`가 X1 벡터를 검증 없이 그대로 저장했던 부분 — 소스가 로컬 기준과 다른 임베딩 모델/차원을 보내면 `sqlite-vec` 고정 차원 테이블이 깨질 수 있었다. `VectorSpace::is_compatible`로 정합 확인 후에만 저장하고, 불일치 시 조용히 건너뛰어 발견 단계의 로컬 재임베딩(전략 B)으로 자연스럽게 폴백하게 했다.

**검증 방법:** `cargo test`(core 33/33, app 1/1) · `cargo clippy --all-targets`(core/app 모두 경고 0) · `npx tsc --noEmit` · `npm run build`(vite) · Vite dev 서버(`http://localhost:1420`)를 브라우저 프리뷰로 열어 온보딩→발견→탐색→보관함→설정 4개 탭 전환과 콘솔 에러 0건 확인. 백엔드 IPC가 없는 순수 브라우저 환경이라 `invoke()` 호출은 실패하지만, 모든 화면이 크래시 없이 우아하게 폴백(토스트 안내)하는 것도 함께 확인했다 — 이 과정에서 Settings 화면이 API 실패 시 "불러오는 중…"에 무한히 갇히는 회복성 버그를 찾아 고쳤다(개별 요청 catch + 기본값 폴백).

**아직 남은 것 (패밀리 차원 협의 필요, 코드 문제 아님):**

* **X1-a** — 공통 API 벡터 공유 확장(schema v1.1)을 마스터·3축 앱이 실제로 구현해야 소스측 공유(전략 A)가 살아난다. 지금은 로컬 재임베딩(전략 B)으로 전 기능 동작.
* **X2-a** — TXTAIMemory의 실제 MCP 수신 방식(stdio JSON-RPC vs 로컬 HTTP 게이트웨이) 확정. 지금은 임시 HTTP 브리지.
* 실서버(TXTDiary 등) 대상 페어링·동기화 E2E 검증 — 이 환경엔 실행 중인 TXT 패밀리 서버가 없어 코드 경로만 정적으로 검증했다.
* 2D 관계 뷰(그래프), HNSW 승격, msi/nsis 패키징 — MoSCoW Should/후속 범위로 의도적으로 남김.

## 9. 변경 이력

* v0.1 (2026-07-12, Claude Code): 최초 작성. Sprint 0~1 실행 단위, X1 소비·벡터 엔진·발견·생성·X2 환류·QA 체크리스트.
* v0.2 (2026-07-12, Claude Code): 코어 엔진(`core/`) 구현 완료 반영 — 구현 현황·인계 노트(§10) 추가.
* v0.3 (2026-07-12, Claude Sonnet 5): Tauri 앱(`app/`) 구현 완료 반영 — §11 Sonnet 완료 보고 추가(파일 위치·발견한 버그 2건·검증 방법·남은 패밀리 협의 항목).
