# TXT 패밀리 시리즈 개요 & TXTMyWorld 위치

작성일: 2026-07-12 / 작성자: Claude Code / 성격: 프로젝트 부트스트랩 컨텍스트

> 이 문서는 TXTMyWorld 프로젝트를 처음 여는 세션이 TXT 패밀리 전체를 빠르게 파악하도록 만든 요약이다. 원본 위키 문서들은 `docs/prd/`, `docs/architecture/`, `docs/context/`에 로컬 복제되어 있다.

---

## 1. TXT 패밀리란

세 종류의 맥락(개인 기억·문서 지식·AI 대화)을 각각 **맥락화**하고 → 하나의 지도로 **보고(TXTSpace)** → 새로운 연결을 **발견·생성(TXTMyWorld)** 하는 개인 지능 생태계.

핵심 원리는 단순 저장이 아니라 **재정리(consolidation)**. 사람이 자는 동안 뇌가 경험을 묶고·요약하고·버리고·새로 연결하듯, 흩어진 기록/지식/AI 작업 맥락을 살아있는 기억으로 만든다.

## 2. 구성 (계층별)

| 계층  | 앱   | 맥락화 대상 | 상태  |
|-----|-----|--------|-----|
| 맥락화 | TXTDiary | 개인 기억·감정·경험 | v1.0 RC 구현 완료 |
| 맥락화 | TXTBrain | 문서·지식 자료 | 개발 예정 |
| 맥락화 | TXTAIMemory | AI 대화·실작업 | PRD 작성됨 |
| 시각화 | TXTSpace | 3축의 통합 지도 (보기) | RC 후 순차 개발 |
| **연결** | **TXTMyWorld (본 프로젝트, 가칭)** | 3축의 연관성·새 연결 (잇기·만들기) | **최종 목표 · 착수 전** |

보조 도구: **TXTDrop**(빠른 텍스트 캡처 입구), **SAC API**(문서 원본 저장, 별개 시스템).

개발 순서(의존성): TXTDiary → TXTAIMemory → TXTBrain → TXTSpace → **TXTMyWorld**.

## 3. TXTMyWorld의 역할 (마스터 문서 기준)

TXTMyWorld는 패밀리의 **최종 레이어**다. TXTSpace가 "지도를 그려 보여주기(보기)"라면, TXTMyWorld는 그 지도 위에서 **"여기서 무엇이 새로 태어나는가"를 만드는(잇기·만들기)** 층이다.

* **역할:** 세 소스(Diary/Brain/AIMemory)를 가로질러 연관을 찾고, 새 맥락을 생성한다.
* **데이터:** 공통 Keyword/Context API(schema v1.0)로 세 소스를 **통합 조회**한다. 원본을 소유하지 않는다.
* **경계:** TXTSpace는 read-only 표시까지만. 새 연결의 **생성·저장**은 TXTMyWorld의 몫.
* **미확정:** 이름(가칭 TXTMyWorld)이 아직 확정되지 않았다 — 열린 결정 사항.

## 4. 공통 프로토콜 (TXTMyWorld가 상속·소비)

맥락화 3앱이 모두 동일하게 노출하는 **로컬 read-only Keyword/Context API**:

* **localhost 전용**(127.0.0.1 바인딩), **read-only**(GET만, 쓰기 405), **페어링 토큰 인증**
* **본문 미포함**: 원문(일기/문서/대화 전문)은 절대 API로 나가지 않음 — 키워드·빈도·감정·동시출현 메타데이터만
* **schema_version** 필수, 소스는 `source` 필드(txtdiary/txtbrain/txtaimemory)로 구분, `ai_id` 확장 필드로 AI별 세부 구분
* 공통 엔드포인트: `/health`, `/keywords`, `/keywords/{normalized_text}/cooccurrence`
* 딥링크: `txtdiary://search`, `txtbrain://search`, `txtaimemory://recall`

응답 스키마(schema v1.0)와 상세는 `docs/architecture/masterspec_20260709_txt-family-master_yumi.md` §3 참조. 원본 정의처는 TXTDiary PRD §8.3.

## 5. AI 공통 규칙 (TXTMyWorld 설계 시 준수)

* **재정리 우선**: 원장(raw) → 통합(consolidated) → 망각(decay) 루프를 기본 사고틀로.
* **본문 최소주의**: 상위 레이어로 가는 데이터는 메타데이터 우선.
* **AI 구분 태깅**: `ai_id`로 구분 가능 + 태그 없이 통합 조회도 가능.
* **망각은 기능**: 통합 끝난 원장·오래된 기억은 삭제 가능.
* **진단 금지**: 감정·상태 분석은 진단이 아니라 경향 제안.
* **동의 기반 외부 전송**: 민감 원문은 명시적 허용 시에만 외부로.

## 6. 문서 상태 & 다음 할 일

* **TXTMyWorld v0.1 PRD 초안 작성됨** (2026-07-12): `docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md`. 4대 방향 확정 — 하이브리드 발견 · 주제 카드+환류 · 로컬 우선+동의형 클라우드 · 독립 앱.
* 참조: SVIL Outline 위키에 TXTSpace·TXTDiary PRD와 패밀리 마스터 존재. TXTMyWorld는 위키에 아직 미등록(로컬 초안 상태).
* 열린 결정(PRD §16): **앱 이름 확정(D0)**, 벡터 소스(로컬 임베딩 vs 공통 API 벡터 확장, D1), 로컬 임베딩 모델(D2).
* 다음 단계 후보: (1) 이름 확정, (2) PRD를 Outline 위키에 등록, (3) 상위 3축·TXTSpace 규격 준비 상태 점검.

## 7. 로컬 복제된 참조 문서

| 파일  | 내용  |
|-----|-----|
| `docs/architecture/masterspec_20260709_txt-family-master_yumi.md` | TXT 패밀리 마스터 — 목적·구성·공통 프로토콜·AI 규칙 (상위 문서) |
| `docs/prd/prd_20260709_txtspace-v0-1_yumi.md` | TXTSpace v0.1 PRD — 바로 아래 레이어(보기) |
| `docs/prd/prd_20260708_txtdiary-v1-0-rc_yumi.md` | TXTDiary v1.0 RC PRD — Keyword API schema v1.0 원본 정의처 |
| `docs/context/context_20260709_txt-family-connected_yumi.md` | 패밀리 설계가 하나로 묶인 날의 기록 (배경 맥락) |
