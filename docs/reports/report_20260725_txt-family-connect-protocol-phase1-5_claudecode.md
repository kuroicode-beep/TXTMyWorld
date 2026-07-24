# 완료보고서 — TXT 패밀리 연결프로토콜 v2.0 Phase 1~5 구현

- 작업자: Claude Code (Fable 5)
- 작성일: 2026-07-25
- 대상: TXT 패밀리 6개 저장소 (TXTMyWorld 중심, 크로스 리포)
- 기준 문서: `C:\Downloads`의 4개 전략 문서(전체그림·앱별 지시문·통합 로드맵·통합 PRD) + 앞선 세션의 정정본

## 1. 배경

앞선 턴에서 사용자가 준 4개 전략 문서를 실측 대조해 정정한 뒤, "실제 업데이트 작업을 로드맵 전체(Phase 0~5) 범위로 끝까지 진행"하기로 결정. Phase 0(문서 개정·OpenAPI)은 순수 문서라 보류하고, **실제 코드가 필요한 Phase 1~5를 6개 저장소에 구현**했다.

## 2. Phase별 구현 내역

### Phase 1 — 인증 통일
- **TXTAIMemory** `control_api.py`: `POST /write`가 페어링 토큰 발급 이후 인증 필수(토큰 미발급 상태는 하위호환 통과). `Authorization: Bearer`·`X-Pairing-Token` 둘 다 수용. (v0.9.2)
- **TXTDrop** `memory_client.py`·설정 UI: AI 기억 캡처(Ctrl+Shift+M)에 토큰 필드 추가, `Bearer` 헤더 첨부, 401 시 명확한 로그. (v0.8.1)

### Phase 2 — 스키마 통합
- **TXTAIMemory** `keyword_api.py`: Keyword API가 family v2.0(`keywords` 배열, 실제 `first_seen`/`last_seen`/`cooccurrence`/`category`)로 응답. 구 소비자용 `items` 병행 제공. (v0.9.3)
- **TXTMyWorld** `core/source.rs`: `merge_keywords`가 `keywords`가 있으면 `items`를 건너뛰어 이중 집계 방지. (v0.2.4)
- **TXTSpace Hub** `adapters.rs`: AIMemory를 다른 두 소스와 동일 파싱 경로로 통일(전용 items 변환은 구버전 폴백으로만). **schema_version 상한 major 1→2** — 방치 시 v2.0 응답이 `update_required`로 막혀 AIMemory 연동이 통째로 끊길 회귀였음. (v0.1.5-dev)

### Phase 3 — X1 벡터 공유
- **TXTDiary**: 실제 v1.0 RC 앱이 `main`에 병합된 적이 없어 **먼저 브랜치 병합**(claude/outline-project-wiki 브랜치 → main). 이후 `/vectors`·`vector_capability` 추가(all-minilm/384, 온디맨드 재임베딩). (v1.0.1)
- **TXTBrain** `keyword_api_service.py`: `/vectors`·`vector_capability` 추가(nomic-embed-text, 하이브리드 검색 임베딩 재사용). (v1.9.3)
- **TXTAIMemory** `keyword_api.py`: `/health`(무인증)·`/vectors` 신설. 이 앱은 임베딩 파이프라인이 없었고 47531에 `/health`조차 없어 TXTMyWorld 헬스체크가 조용히 404 실패 중이던 것도 해소. (v0.9.4)
- **TXTMyWorld** `pipeline.rs`: `sync_source`의 X1 공간 정합 판정이 `bge-m3`로 고정 비교 → 이 기기 실제 모델(nomic-embed-text/768)과 항상 불일치해 **소스가 방금 제공 시작한 벡터를 전부 폐기**하던 버그. `select_embedder()` 실제 모델로 비교하도록 수정. (v0.2.5)

### Phase 4 — X2 환류 정식화
- **TXTAIMemory**: `WriteRequest.external_id` + 멱등 upsert(같은 발견 재환류 시 원장 중복 누적 방지). `raw_memories.external_id` 컬럼·유니크 인덱스, `source` 허용값에 `txtmyworld` 추가. (v0.9.5)
- **TXTMyWorld** `feedback_client.rs`: `DEFAULT_ENDPOINT`가 존재하지 않는 포트(8765)를 가리켜 **X2가 한 번도 성공한 적 없던** 것 수정 → 실제 control API(47530/write)로 교체, 페이로드를 WriteRequest 계약으로 변환, 페어링 토큰 첨부. (v0.2.6)

### Phase 5 — 디스커버리 레지스트리
- **TXTAIMemory·TXTDiary·TXTBrain·TXTSpace Hub**: 기동 시 `%LOCALAPPDATA%\SVIL\registry.json`에 자기 포트 기록(원자적 병합, 실패해도 기동 무영향). TXTAIMemory는 control/keyword 2개 키. (각 v0.9.6 / v1.0.2 / v1.9.4 / v0.1.6-dev)
- **TXTSpace Hub·TXTMyWorld**: 소스 포트를 레지스트리 우선 조회 후 하드코딩 폴백. (TXTMyWorld v0.2.7)

## 3. 검증 결과

- 전 저장소 테스트 통과: TXTMyWorld core 36/36 + app 6/6, TXTAIMemory 84/84, TXTBrain 56/56(단일 인스턴스 테스트 2건은 실앱 상주로 인한 사전 존재 실패라 제외), TXTDiary 15/15, TXTSpace Hub 5/5.
- 라이브 헬스체크(read-only): 47530/8811/47540 응답 확인. **단, 현재 실행 중인 앱은 전부 이번 세션 이전 빌드** — 새 기능은 각 앱 재시작 후에야 실동작한다(재시작은 사용자 판단 대기로 보류).

## 4. 발견한 실버그(신규 기능이 아니라 원래 깨져 있던 것)

1. **X2 환류가 단 한 번도 성공한 적 없음** — 잘못된 포트. (offline 폴백에 가려 에러로 표면화 안 됨)
2. **X1 벡터가 정상 수신 후 전량 폐기** — 하드코딩 bge-m3 비교.
3. **TXTAIMemory Keyword API에 /health 부재** — TXTMyWorld 헬스체크가 이 소스만 조용히 실패.
4. **TXTSpace Hub의 schema 상한** — AIMemory v2.0 응답이 막힐 회귀.
5. **TXTDiary main에 실제 RC 앱 미병합** — main엔 문서+임시 사이드카만 있었음.

## 5. 남은 것

- **Phase 0**: 마스터 문서 개정 + OpenAPI 스펙(순수 문서). 미착수.
- **실앱 재배포/재시작**: 새 기능 실동작 확인은 각 앱 재빌드·재시작 후 가능. TXTAIMemory는 MCP 연결 4개가 붙어 있어 재시작 시 다른 AI 세션 영향 주의.
- 릴리즈 빌드·바탕화면 바로가기: 재시작 보류에 따라 이번 체크포인트에서는 미실행.

## 6. 커밋

- TXTMyWorld: `07dee44`·`43885e6`·`ea6eb36`·`39a1985` (main, push 완료)
- TXTSpace: `8d96eb6`·`dd7381e` (main, push 완료)
- TXTAIMemory/TXTDrop/TXTDiary/TXTBrain: 각 저장소 커밋 완료(원격 반영 확인, ahead 0)
