# TXTMyWorld 통합 스펙 — X1 벡터 공유 확장 · X2 AIMemory 환류 (RC 계약)

문서 버전: v0.1 / 대상: TXTMyWorld RC(v1.0) 범위 계약 / 작성일: 2026-07-12 / 작성자: Claude Code (SVIL)

> 상위: TXTMyWorld v0.1 PRD(v0.3) §16 X1·X2. 본 문서는 두 **크로스-프로젝트 계약**의 상세 규격이다. TXTMyWorld는 소비자/생산자 중 **자기 몫**을 RC까지 구현하고, 상대 앱 몫은 패밀리 의존성으로 추적한다.
> 상위 규약: TXT 패밀리 마스터 §3(공통 Keyword/Context API), §4(AI 공통 규칙 — 본문 최소주의·동의 기반 외부 전송).

---

## 0. 계약 요약

| 계약 | 무엇 | TXTMyWorld 몫 | 상대 앱 몫(의존성) | RC 목표 |
|-----|-----|-----------|--------------|-------|
| **X1** | 공통 API에 임베딩 벡터 공유 확장(schema v1.1) | **소비자**: 벡터 수신·정합·재색인 + 스키마 정의 | **생산자**: 소스가 자기 본문으로 임베딩→벡터만 공유 | 소비자 정착, 준비된 소스부터 실연동 |
| **X2** | 주제 카드를 TXTAIMemory에 MCP 쓰기로 환류 | **생산자**: 통합 기억 등록 페이로드 생성·전송 | **수신자**: TXTAIMemory MCP 쓰기 엔드포인트 | 환류 정착·이력·멱등 |

**공통 원칙(마스터 상속):** 본문 미전송(벡터·메타데이터만), 동의 기반, schema_version 관리, 모르는 필드 무시(graceful), localhost/로컬 우선.

---

## 1. 용어

* **생산자(producer):** 벡터/데이터를 제공하는 쪽. X1은 3축 소스 앱, X2는 TXTMyWorld.
* **소비자(consumer):** 받는 쪽. X1은 TXTMyWorld, X2는 TXTAIMemory.
* **벡터 공간 정합:** 서로 다른 모델의 임베딩은 직접 비교 불가. `model`·`dim` 태깅으로 동일 공간만 비교하거나 재임베딩으로 정렬.

---

## 2. X1 — 공통 Keyword/Context API 벡터 공유 확장 (schema v1.1)

### 2.1 설계 원칙

* **하위 호환:** schema v1.0의 상위 집합. v1.0 소비자는 새 필드를 무시하면 그대로 동작.
* **옵트인·능력 광고:** 소스는 벡터 미지원일 수 있음. `/health`가 능력을 광고하고, 소비자는 없으면 로컬 임베딩(전략 B)으로 폴백.
* **본문 불출:** 공유하는 것은 **벡터와 그 출처 메타데이터뿐.** 원문·문장은 절대 나가지 않는다.
* **대용량 대응:** 벡터는 크므로 `/keywords` 인라인 포함은 옵트인이고, 대량 동기화는 전용 `/vectors` 엔드포인트 + 증분(`since`).

### 2.2 `/health` 확장

```json
{
  "schema_version": "1.1",
  "source": "txtdiary | txtbrain | txtaimemory",
  "app_version": "x.y.z",
  "vector_capability": {
    "supported": true,
    "model": "bge-m3",
    "dim": 1024,
    "normalized": true,
    "count": 1234,
    "updated_at": "ISO8601"
  }
}
```

* `supported=false`(또는 `vector_capability` 부재)이면 소비자는 그 소스에 대해 전략 B(로컬 임베딩)로 전환.

### 2.3 `/keywords` 확장 (인라인, 옵트인)

* 쿼리: 기존 `from/to/category/min_frequency/limit` + `include=embedding`.
* `include=embedding`이고 지원 시, 각 keyword에 `embedding` 추가:

```json
{
  "schema_version": "1.1",
  "source": "txtbrain",
  "generated_at": "ISO8601",
  "keywords": [
    {
      "text": "측정 문제",
      "normalized_text": "측정문제",
      "category": "topic",
      "frequency": 12,
      "avg_emotion_score": 0.1,
      "first_seen": "2026-05-01",
      "last_seen": "2026-07-10",
      "cooccurrence": [{"text": "관측자", "count": 5}],
      "embedding": {
        "model": "bge-m3",
        "dim": 1024,
        "normalized": true,
        "vector": [0.0123, -0.0456, "... (1024개)"]
      }
    }
  ]
}
```

### 2.4 `/vectors` 신설 (배치·증분 동기화)

| Endpoint | Method | 목적 |
|----------|--------|-----|
| `/vectors` | GET | 벡터 배치 조회. 쿼리: `since`(ISO8601, 증분), `limit`, `cursor` |

```json
{
  "schema_version": "1.1",
  "source": "txtbrain",
  "model": "bge-m3",
  "dim": 1024,
  "normalized": true,
  "generated_at": "ISO8601",
  "next_cursor": "opaque-or-null",
  "vectors": [
    {"normalized_text": "측정문제", "vector": [/* dim개 */], "updated_at": "ISO8601"}
  ]
}
```

* **증분:** `since`로 마지막 동기화 이후 변경분만. 소비자는 `updated_at`로 upsert.
* **인증/보안:** 기존 페어링 토큰 그대로. read-only(GET). 쓰기 메서드 없음.

### 2.5 소비자(TXTMyWorld) 동작 규칙

1. `/health`로 소스별 `vector_capability` 확인 → 지원 소스만 X1 경로, 나머지 전략 B.
2. `/vectors`(우선) 또는 `/keywords?include=embedding`으로 벡터 수신.
3. **공간 정합 판정:** `(model, dim, normalized)`가 **TXTMyWorld 기준 모델(bge-m3/1024/normalized)과 일치**하면 직접 사용.
   * 불일치 시 정책(설정): (a) 해당 소스만 로컬 재임베딩으로 정렬(권장 기본), (b) 별도 공간으로 격리해 소스 내 검색만.
4. `embeddings` 테이블에 `origin=shared`, `model/dim` 태깅 저장 → `vec_index` 재색인.
5. 실패/미지원은 조용히 전략 B 폴백. 어떤 소스가 벡터를 안 줘도 발견은 계속된다.

### 2.6 버전·오류

* `schema_version`이 소비자 상한보다 크면 오류 대신 "업데이트 필요" 안내(마스터 방어 원칙).
* 벡터 `dim`이 선언과 다르면 해당 레코드 스킵 + 경고 로그(앱 비중단).

### 2.7 RC 판정 기준(X1)

* 최소 1개 소스와 실제 `/vectors` 증분 동기화 성공, 공간 정합/재임베딩 폴백 검증, 미지원 소스 혼재 시에도 통합 발견 정상.

---

## 3. X2 — TXTAIMemory 환류 (MCP 쓰기 계약)

### 3.1 목적

TXTMyWorld가 생성·확정한 **주제 카드**를 TXTAIMemory에 **통합 기억(consolidated)**으로 등록해 생태계를 순환시킨다. "발견이 다시 AI의 기억이 된다."

### 3.2 채널

* TXTAIMemory MCP 서버의 **쓰기 인터페이스**를 사용(예: `memory_write`, 필요 시 `memory_consolidate`).
* 동의 기반: 사용자가 카드별로 "AI 기억에 남기기"를 켠 경우에만 호출.
* TXTAIMemory 미실행/미연결 시 기능 비활성(회색 처리 + 안내), 앱 정상 동작.

### 3.3 요청 페이로드 (payload_schema v1.0)

```json
{
  "payload_schema_version": "1.0",
  "origin": "txtmyworld",
  "ai_id": "txtmyworld",
  "memory_tier": "consolidated",
  "external_id": "topic_card:<uuid>",
  "title": "관측 문제: 나의 세 갈래",
  "summary": "일기의 '관측자', 문서의 '측정 문제', AI 대화의 'observer effect'가 한 주제로 수렴.",
  "discovery_type": "cluster",
  "member_keywords": [
    {"text": "관측자", "normalized_text": "관측자", "source": "txtdiary"},
    {"text": "측정 문제", "normalized_text": "측정문제", "source": "txtbrain"},
    {"text": "observer effect", "normalized_text": "observereffect", "source": "txtaimemory"}
  ],
  "evidence": {
    "semantic_sim": 0.82,
    "period": {"from": "2026-05-01", "to": "2026-07-10"},
    "frequency_signal": "rising"
  },
  "deeplinks": [
    "txtdiary://search?keyword=관측자",
    "txtbrain://search?keyword=측정문제",
    "txtaimemory://recall?keyword=observereffect"
  ],
  "created_at": "ISO8601"
}
```

* **본문 없음:** 소스 원문/문장은 전송하지 않는다. 키워드·근거 요약·딥링크만.
* **멱등:** `external_id`(= `topic_card:<uuid>`)로 중복 등록 방지. 같은 카드 재환류는 update.

### 3.4 응답

```json
{ "ok": true, "memory_id": "aim-...", "external_id": "topic_card:<uuid>", "status": "created | updated" }
```

### 3.5 생산자(TXTMyWorld) 동작 규칙

1. 카드 환류 토글 ON + 사용자 확인(전송 범위 고지) → 페이로드 생성.
2. MCP 쓰기 호출. 성공 시 `feedbacks` 테이블에 이력(`memory_id`, `sent_at`, `status`) 기록.
3. 실패(미연결/오류)는 재시도 큐 또는 사용자 안내. 자동 무한 재시도 금지.
4. 카드가 수정되면 같은 `external_id`로 update 재환류(사용자 재확인).

### 3.6 프라이버시·안전

* 전송 전 사용자에게 **정확한 전송 항목**(제목·키워드·근거 요약·딥링크, 본문 없음)을 고지하고 동의.
* 진단 금지: summary는 경향 서술이지 단정/진단이 아니다(마스터 §4).
* 환류 취소: 사용자가 카드를 삭제하면 대응 통합 기억 삭제 요청(`memory_forget`, 선택) 가능.

### 3.7 RC 판정 기준(X2)

* 동의 흐름 → MCP 쓰기 → 이력 기록 → 멱등 update까지 E2E 성공. 미연결 폴백/재시도 정상. 전송 본문 미포함 검증.

---

## 4. 패밀리 의존성 요약 (합의 필요 항목)

| ID | 항목 | 필요 합의 상대 | TXTMyWorld 폴백 |
|----|-----|-----------|-------------|
| X1-a | schema v1.1 확장(`/health` 능력, `/vectors`, `include=embedding`) 채택 | 마스터 + 3축 앱 | 전략 B(로컬 임베딩) |
| X1-b | 소스별 임베딩 모델 정합(가능하면 bge-m3 통일) | 3축 앱 | 소스별 재임베딩 정렬 |
| X2-a | TXTAIMemory MCP 쓰기 수신 스키마/툴명 확정 | TXTAIMemory | 환류 비활성 |

## 5. 변경 이력

* v0.1 (2026-07-12, Claude Code): X1(벡터 공유 schema v1.1)·X2(AIMemory MCP 환류) 계약 최초 정의. RC 판정 기준·패밀리 의존성 명시.
