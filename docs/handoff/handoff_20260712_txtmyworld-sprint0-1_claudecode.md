# TXTMyWorld 작업지시서 — Sprint 0~1 (착수)

문서 버전: v0.2 / 작성일: 2026-07-12 (갱신) / 작성자: Claude Code (SVIL) / 대상: Cursor·Sonnet(구현)·Codex(QA)

> **[구현 현황 v0.2]** 코어 엔진이 `core/`(Rust 크레이트 `txtmyworld-core`)로 **구현 완료** — 테스트 26/26 통과, clippy 클린. 아래 표의 ✅ 항목은 코어 레벨에서 끝났고, 남은 것은 Tauri 앱 셸·UI·실연동이다. §10 인계 노트 참조.

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

## 9. 변경 이력

* v0.1 (2026-07-12, Claude Code): 최초 작성. Sprint 0~1 실행 단위, X1 소비·벡터 엔진·발견·생성·X2 환류·QA 체크리스트.
* v0.2 (2026-07-12, Claude Code): 코어 엔진(`core/`) 구현 완료 반영 — 구현 현황·인계 노트(§10) 추가.
