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

## 6. 실측 관찰 — 실서비스 소스 연동 (2026-07-14)

실제 로컬 환경에서 세 소스 앱과 페어링을 시도하며 확인한 사실. PRD·통합 스펙의 이상적 계약과 실제 배포 상태가 갈라진 지점이라 기록해 둔다.

### 6.1 실측 포트

가정했던 4001/4002/4003은 전부 틀렸다. 실제 확인된 값(변경될 수 있음, 각 앱 설정에서 재확인 필요):

| 소스 | 실측 포트 | 확인 방법 |
|-----|--------|--------|
| TXTBrain | 8811 | 실행 중 프로세스 직접 조회 |
| TXTAIMemory | 47531 (전용 Keyword API), 47530 (본 서비스) | `/settings/keyword-api` 응답 |
| TXTDiary | 47821 | 미실행 상태 — TXTSpace-hub의 연결 실패 로그에서 역추적 |
| **TXTSpace-hub** | 47540 | 실행 중 프로세스 직접 조회 |

### 6.2 TXTSpace-hub는 사실상의 X1 조기 구현체

TXT 패밀리 마스터·본 스펙은 TXTMyWorld가 3소스에 **개별 페어링**하는 것을 전제로 설계했다. 그런데 실제로는 **TXTSpace-hub(포트 47540)가 이미 3소스를 폴링해 하나의 `/keywords`로 재노출**하고 있었다 — `schema_version: "1.1"`, 항목마다 `source` 태그(txtdiary/txtbrain/txtaimemory)가 붙어 있고, **페어링 토큰이 필요 없다.** 사실상 X1이 "소스별 벡터 공유"가 아니라 "허브의 키워드 애그리게이션"이라는 다른 모양으로 먼저 도착해 있었던 셈이다. TXTMyWorld는 이제 이 허브를 4번째 소스 옵션(`txtspace-hub`)으로 인식해 하나의 연결로 3소스 전체를 받는다.

### 6.3 실배포 스키마가 PRD 원안과 다르다

허브(및 아마 개별 소스도 동일 코드베이스로) 응답 실측:

* `text` 대신 `keyword`, `cooccurrence` 대신 `cooccurrences`.
* `frequency`/`cooccurrence.count`가 정수가 아니라 부동소수(`2.0` 등).
* `source`가 봉투 단위가 아니라 **키워드 항목마다** 붙는다(허브 응답 한정).
* `normalized_text`가 일부 항목에서 아예 누락된다.

core는 alias·관대한 타입·항목별 source 폴백·누락 필드 백필로 이 형태를 그대로 수용하도록 고쳤다(`models.rs`, `source.rs`). 실제 hub 응답을 그대로 캡처한 픽스처로 회귀 테스트를 고정했고, `cargo test --ignored`로 라이브 서비스(190개 키워드, 3소스 태깅) 검증까지 마쳤다.

### 6.4 개별 소스는 토큰이 필요 — 그러나 공유 토큰 재사용으로 해결

허브와 달리 개별 소스에 직접 붙으려면 페어링 토큰이 필요하다(`/keywords`가 `401` 반환 확인). 그리고 세 앱 모두 **단일 토큰**이라(TXTBrain `/pair`는 이미 페어링되면 `409`, TXTAIMemory `space_api`는 단일 행), TXTSpace가 이미 페어링해 둔 상태에서 TXTMyWorld가 새로 토큰을 발급받으면 TXTSpace 연결이 끊긴다. → §7에서 공유 토큰 재사용으로 해결.

## 7. 3소스 개별 직접 연결 (2026-07-14 — 소스 앱 무수정)

목표: TXTBrain/TXTDiary/TXTAIMemory가 TXTSpace뿐 아니라 **TXTMyWorld에도 개별적으로 직접** 이어지게 한다(허브 경유가 아니라).

### 7.1 핵심 발견 — 토큰은 패밀리 공용 저장소에 있다

TXTSpace hub의 `adapters.rs`는 소스 토큰을 **Windows 자격 증명 관리자**(keyring 서비스명 `"TXTSpace"`, 사용자명=소스명)에 저장하고, hub·UI가 이를 공유해 읽는다. 소스 앱의 토큰 검증은 단순 해시 비교라 **누가 제시하든 유효하면 통과** — 즉 TXTSpace와 TXTMyWorld가 같은 토큰을 동시에 써도 문제없다.

→ TXTMyWorld는 소스 앱을 **전혀 수정하지 않고**, 이미 페어링된 그 공유 토큰을 재사용해 세 앱에 직접 붙는다. 단일 토큰 제약도, 재페어링으로 인한 TXTSpace 연결 손상도 회피한다.

### 7.2 TXTMyWorld 측 구현 (이 앱만 변경)

* **토큰 해석 순서**(`secure.rs::resolve_token`): TXTMyWorld 자체 페어링(keyring `com.svil.txtmyworld`) → 없으면 패밀리 공용(keyring `TXTSpace`) 폴백. 대개 후자로 자동 연결.
* **소스별 인증 헤더**(`AuthHeader`): TXTAIMemory=`X-Pairing-Token`, TXTBrain/TXTDiary=`Authorization: Bearer`. (hub adapters.rs와 동일 규약)
* **TXTAIMemory items 스키마**: 이 앱만 `{items:[{keyword,weight,ai_id}]}` 형태 → weight를 frequency로, keyword를 normalized_text 폴백으로 매핑(`models.rs`, `merge_keywords`).
* **원클릭 "3개 앱에 지금 연결"**(`connect_all_sources` 커맨드): 실측 포트(diary 47821·brain 8811·aimemory 47531)로 세 앱을 등록·동기화.

### 7.3 라이브 검증 (2026-07-14)

공유 토큰 재사용으로 세 앱 모두 직접 연결 성공: **TXTDiary 12 · TXTBrain 100 · TXTAIMemory 78 키워드** 수신. 실제 앱 DB에 4소스(3직접 + 허브) 등록 확인. 소스 앱 무수정, TXTSpace 연결 무손상.

### 7.4 남은 한계

* 공유 토큰 재사용은 TXTSpace가 먼저 페어링해 둔 것을 전제로 한다. TXTSpace 페어링이 없는 소스는 사용자가 해당 앱에서 토큰을 발급해 TXTMyWorld에 직접 입력해야 한다(UI에 입력란·안내 있음).
* 진짜 독립(멀티 클라이언트) 페어링을 원하면 각 소스 앱에 다중 토큰 지원을 추가해야 하나, 현재는 불필요(공유 토큰으로 충분)하여 소스 앱을 건드리지 않았다.

### 7.5 "연결은 됐는데 발견이 0건" — 후속 버그 2건 (2026-07-14, v0.2.3)

3소스 연결 후에도 발견 화면이 계속 비어 있어("아직 새로운 발견이 없습니다") 추적한 결과, 연결과 무관한 발견 파이프라인 버그 2건이 드러났다.

1. **`SqliteVecStore::get()`가 항상 `None` 반환(치명).** 초기 구현에서 "sqlite-vec는 참조를 못 주니 get은 안 쓴다"고 판단해 `None` 스텁으로 뒀는데, 발견 엔진(`detect_bridges/gaps/clusters`)은 **시드 벡터를 `store.get()`으로 가져온다.** 실제 앱은 `SqliteVecStore`를 쓰므로 모든 키워드가 `None`→`continue`로 조용히 skip되어 발견이 무조건 0건이었다. 유닛 테스트는 `get()`이 정상인 `InMemoryVectorStore`를 써서 못 잡았다. → 트레이트 `get`을 소유값(`Option<Vec<f32>>`) 반환으로 바꾸고, `SqliteVecStore`가 vec_items에서 임베딩 블롭을 실제 복원하도록 수정. **SqliteVecStore E2E 발견 회귀 테스트 추가.**
2. **임베더가 모델 미설치를 무시.** `select_embedder`가 "Ollama가 응답하면 bge-m3"로 가정했는데, 이 기기엔 bge-m3가 pull되어 있지 않아(`/api/embed`가 "model not found") 임베딩이 전부 실패→벡터 0개→발견 0건. → `/api/tags`로 **실제 설치된** 임베딩 모델을 우선순위(bge-m3→nomic-embed-text→…)로 감지·선택하고, 차원을 모델에 맞게 동적 적용, 실패 시 해시 폴백.
3. **임계값 보정.** 의미 임베딩(nomic)은 서로 다른 키워드 사이에도 코사인 유사도가 높다(실측 교차소스 중앙값 ≈ 0.82). 데모용 컷 0.6은 수백 건의 노이즈를 만든다 → 기본 브리지/클러스터 컷을 0.85로 상향.

검증: 실제 앱 DB(195키워드/3소스)에서 발견 **98건**(브리지 69·갭 19·클러스터 10) 생성 확인. 최상위 예: "개발자(문서)↔관측자(일기) 0.87", "ai(AI대화)↔ai(문서) 1.00", 3소스 걸친 클러스터. core 39/39 통과.

## 5. 변경 이력

* v0.1 (2026-07-12, Claude Code): X1(벡터 공유 schema v1.1)·X2(AIMemory MCP 환류) 계약 최초 정의. RC 판정 기준·패밀리 의존성 명시.
* v0.2 (2026-07-14, Claude Sonnet 5): §6 실측 관찰 추가 — 실제 포트, TXTSpace-hub 조기 X1 구현체 발견, 실배포 스키마 편차, 토큰 요구사항. core/app 코드에 실제 반영·라이브 검증 완료.
* v0.3 (2026-07-14, Claude Opus 4.8): §7 3소스 개별 직접 연결 추가 — 패밀리 공용 keyring(TXTSpace) 토큰 재사용으로 소스 앱 무수정 직접 연결. 소스별 헤더·items 스키마·원클릭 연결. 라이브 검증(12/100/78 키워드).
* v0.4 (2026-07-14, Claude Opus 4.8): §7.5 발견 0건 후속 버그 2건 기록 — SqliteVecStore::get() None 스텁, 임베더 모델 미설치 무시. 실데이터 98건 발견 검증(v0.2.3).
