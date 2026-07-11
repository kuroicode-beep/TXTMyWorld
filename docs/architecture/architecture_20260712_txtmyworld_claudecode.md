# TXTMyWorld 아키텍처

문서 버전: v0.1 / 대상: TXTMyWorld v0.1 → RC(v1.0) / 작성일: 2026-07-12 / 작성자: Claude Code (SVIL)

> 상위: TXTMyWorld PRD(v0.3), 통합 스펙(X1/X2), TXT 패밀리 마스터 §2·§3. 본 문서는 시스템 구조·모듈·데이터 흐름·벡터 서브시스템을 정의한다. 스택은 패밀리 관례(Tauri + React, Windows 우선)를 상속한다.

---

## 1. 아키텍처 개요

TXTMyWorld는 **로컬 우선 데스크톱 앱**이다. 세 소스를 read-only로 통합 조회하고, 로컬 벡터 엔진으로 의미 연결을 발견하며, 사용자가 채택한 주제 카드만 자체 소유한다. 외부 전송(클라우드 합성·환류)은 전부 동의 기반.

```
┌──────────────────────────── TXTMyWorld (Tauri) ────────────────────────────┐
│  Frontend (React)                                                          │
│   추천 피드 · 탐색 캔버스 · 관계뷰/리스트뷰 · 주제카드 · 설정/접근성        │
│        │  Tauri IPC (commands/events)                                       │
│  Core (Rust)                                                               │
│   ┌────────────┐ ┌───────────────┐ ┌──────────────┐ ┌──────────────────┐   │
│   │ Source     │ │ Vector Engine │ │ Discovery    │ │ Topic / Feedback │   │
│   │ Client     │→│ (embed/index/ │→│ Engine       │→│ Store            │   │
│   │ (X1 소비)  │ │  KNN/cluster) │ │ (3축 융합)   │ │ (카드·환류)      │   │
│   └─────┬──────┘ └──────┬────────┘ └──────┬───────┘ └────────┬─────────┘   │
│         │               │ SQLite(+sqlite-vec)                 │ MCP(X2)     │
│         │               ▼                                     ▼             │
│         │        [로컬 DB: cache/embeddings/vec_index/         (동의형)     │
│         │         clusters/discoveries/topic_cards/feedbacks]  → TXTAIMemory│
│  ┌──────┴───────┐                        ┌───────────────┐                  │
│  │ Embedding    │  (로컬 임베딩 전략 B)  │ Synth (선택)  │ (동의형)         │
│  │ Runtime      │←──────────────────────│ DeepSeek API  │→ 라벨/설명       │
│  │ (bge-m3)     │                        └───────────────┘                  │
│  └──────────────┘                                                           │
└───────────┬─────────────────────────────────────────────────────────────────┘
            │ HTTP 127.0.0.1 (페어링 토큰, read-only)  + X1 /vectors(schema v1.1)
   ┌────────┴───────────────┬────────────────────────┐
   ▼                        ▼                         ▼
 TXTDiary                TXTBrain                 TXTAIMemory
 (source=txtdiary)      (source=txtbrain)        (source=txtaimemory)
```

## 2. 기술 스택

| 영역 | 기술 | 비고 |
|-----|-----|-----|
| 프레임워크 | Tauri + React | 패밀리 통일, TXTSpace와 컴포넌트 재사용 |
| 코어 | Rust | 통합 조회·벡터·발견·저장·MCP 클라이언트 |
| DB | SQLite + **sqlite-vec(vec0)** | 로컬 영속 + 벡터 KNN. 대규모 시 HNSW |
| 임베딩 | **bge-m3**(1024-dim, 다국어) via Ollama/llama.cpp | 로컬. L2 정규화 후 코사인 |
| 소스 연동 | HTTP 클라이언트 ×3 (127.0.0.1) | 공통 스키마 v1.0/v1.1, 페어링 토큰 |
| 환류(X2) | MCP 클라이언트(쓰기) | TXTAIMemory, 동의형 |
| 합성(선택) | DeepSeek API 클라이언트 | 라벨/설명, 동의형, 사용자 Key |
| 딥링크 | OS shell open (txtdiary:// 등) | 소스 미설치 시 안내 |

## 3. 모듈 구조

```
src/                                # React
  app/
  features/
    feed/            # 추천 피드
    explore/         # 탐색 캔버스(필터·조합)
    graph/           # 2D 관계 뷰
    list/            # 리스트/근거 뷰(접근성 동등)
    topic_card/      # 주제 카드 생성·편집·환류 토글
    settings/        # 소스 페어링·접근성·엔진 설정
  shared/ { ui/, models/, i18n/, a11y/ }

src-tauri/src/                      # Rust
  commands/          # IPC 진입점
  source_client/     # X1 소비: /health,/keywords,/vectors, 페어링, 폴백
  embedding/         # bge-m3 런타임, 배치·증분 임베딩(전략 B)
  vector/            # sqlite-vec 색인, KNN, 공간 정합, 재색인
  discovery/         # 브리지/갭/클러스터/드리프트, 3축 융합 스코어
  store/             # cache/embeddings/vec_index/clusters/discoveries/topic_cards/feedbacks
  feedback/          # X2 MCP 쓰기, 멱등, 이력
  synth/             # DeepSeek 라벨/설명(동의형)
  security/          # 토큰 보안 저장, 동의 게이트
```

## 4. 데이터 흐름 (발견 → 생성 → 환류)

1. **페어링/조회:** `source_client`가 소스별 `/health` 확인 → 페어링 → `/keywords`(+가능시 `/vectors`) 조회 → `keyword_cache` upsert.
2. **임베딩:** X1 벡터가 있으면 정합 판정 후 사용, 없으면 `embedding`이 로컬 bge-m3로 생성(증분). → `embeddings` + `vec_index`.
3. **발견:** `discovery`가 KNN·클러스터(의미)를 뼈대로 기간·빈도 신호를 융합해 `discoveries` 후보 생성(결정적·근거 포함).
4. **제시:** `feed`(자동) + `explore`(수동). `graph`/`list` 동등 표시.
5. **생성:** 사용자가 채택·명명 → `topic_cards` 저장(근거 스냅샷 보존).
6. **환류(동의):** `feedback`이 X2 MCP 쓰기로 TXTAIMemory에 통합 기억 등록 → `feedbacks` 이력.
7. **합성(선택·동의):** `synth`가 로컬/DeepSeek 라벨·설명 제공.

## 5. 벡터 서브시스템 (핵심, RC 지향)

* **이중 임베딩:** 전략 A(X1 소스측 공유 수신) 우선, 없으면 전략 B(로컬). 상세 통합 스펙 §2.5.
* **공간 정합:** `(model,dim,normalized)` 태깅. 기준(bge-m3/1024/normalized)과 불일치 시 재임베딩 정렬 or 격리.
* **인덱스:** sqlite-vec KNN → 규모↑ 시 HNSW. 증분 upsert, 백그라운드 재색인.
* **연산:** 교차소스 매칭 / HDBSCAN 의미 클러스터 / 의미 갭 / 의미 질의 / 드리프트.
* **3축 융합:** `score = w_s·semantic_sim + w_t·temporal_overlap + w_f·frequency_signal`. 낮은 의미 유사 후보 컷 → 오탐↓.

## 6. 신뢰 경계 & 보안

* **읽기 전용(소스):** 소스로 쓰기 요청을 절대 보내지 않는다(X1은 GET only).
* **소유 데이터:** `topic_cards`만 원본. 나머지는 재생성 가능 파생.
* **토큰:** 소스 페어링 토큰·DeepSeek Key는 OS 보안 저장소. 평문 저장 금지.
* **동의 게이트:** 외부로 나가는 모든 경로(X2 환류, DeepSeek 합성)는 `security`의 동의 확인 통과 필수. 전송 항목 사전 고지.
* **본문 불출:** 어떤 경로로도 소스 원문·문장을 내보내지 않는다(벡터·메타·라벨만).

## 7. 성능·확장 (RC 목표)

| 항목 | 목표 |
|-----|-----|
| 통합 조회→피드 | 3소스 합산 3,000 키워드 2초 내 1차 후보 |
| 임베딩/재색인 | 변경 500개 백그라운드, UI 비블로킹 |
| KNN 질의 | 수만 벡터에서 체감 지연 없음(sqlite-vec→HNSW 승격) |
| 관계 뷰 | 노드 100개 60fps, 저사양 30fps |
| 메모리 | 앱 RAM 300MB 이하 목표(임베딩 캐시 별도) |

## 8. 장애·폴백

| 상황 | 대응 |
|-----|-----|
| 소스 미실행 | 그 소스만 오프라인 배지, 캐시 사용, 나머지로 발견 계속 |
| X1 미지원 소스 | 전략 B(로컬 임베딩) 폴백 |
| 벡터 dim 불일치 | 레코드 스킵 + 경고, 앱 비중단 |
| TXTAIMemory 미연결 | 환류 비활성 안내, 재시도 큐 |
| DeepSeek 실패/미동의 | 로컬 라벨로 폴백 |

## 9. 변경 이력

* v0.1 (2026-07-12, Claude Code): 최초 작성. 모듈 구조·데이터 흐름·벡터 서브시스템·신뢰 경계·폴백 정의. X1/X2 연동 반영.
