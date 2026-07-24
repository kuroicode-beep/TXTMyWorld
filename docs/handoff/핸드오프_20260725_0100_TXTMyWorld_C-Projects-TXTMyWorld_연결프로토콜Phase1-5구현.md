# 핸드오프_20260725_0100_TXTMyWorld_C-Projects-TXTMyWorld_연결프로토콜Phase1-5구현

## 대상
- 프로젝트: TXTMyWorld (+ 크로스 리포 5개: TXTAIMemory·TXTDrop·TXTDiary·TXTBrain·TXTSpace)
- 작업 폴더: C:\Projects\TXTMyWorld
- 세션 시각: 2026-07-25 01:00 (KST)

## 세션 요약
`C:\Downloads`의 4개 전략 문서(전체그림·앱별지시문·통합로드맵·통합PRD)를 실측 대조·정정한 뒤, TXT 패밀리 연결프로토콜 v2.0 로드맵의 **실제 코드가 필요한 Phase 1~5를 6개 저장소에 구현**했다. Phase 0(마스터 문서 개정·OpenAPI)은 순수 문서라 보류.

## 완료된 작업
- **Phase 1 인증 통일**: TXTAIMemory `control_api.py` `/write` 토큰 필수화(Bearer/X-Pairing-Token 병행, 토큰 미발급 시 하위호환 통과). TXTDrop `memory_client.py`+설정 UI에 토큰 필드·헤더.
- **Phase 2 스키마 통합**: TXTAIMemory `keyword_api.py`를 family v2.0(keywords 배열, 실제 first_seen/last_seen/cooccurrence)로. TXTMyWorld `core/source.rs` merge 이중집계 방지. TXTSpace Hub `adapters.rs` AIMemory 동일 파싱 경로 통일 + schema 상한 major 1→2(회귀 수정).
- **Phase 3 X1 벡터**: Diary/Brain/AIMemory 3소스 모두 `/vectors`+`vector_capability`. TXTDiary는 실 RC 앱이 main 미병합 상태라 브랜치(claude/outline-project-wiki-e372a7) 먼저 병합. TXTMyWorld `pipeline.rs` X1 정합 판정 하드코딩 bge-m3 버그 수정(select_embedder 실모델 비교).
- **Phase 4 X2**: TXTAIMemory `WriteRequest.external_id`+멱등 upsert(raw_memories.external_id 컬럼·유니크 인덱스). TXTMyWorld `feedback_client.rs` DEFAULT_ENDPOINT를 죽은 포트 8765→실제 47530/write로 교체, WriteRequest 계약 변환, 토큰 첨부.
- **Phase 5 레지스트리**: AIMemory·Diary·Brain·Hub가 `%LOCALAPPDATA%\SVIL\registry.json`에 자기 포트 기록. Hub `config.rs`·TXTMyWorld `pipeline.rs`가 하드코딩 대신 레지스트리 우선 조회.
- **검증**: 전 저장소 테스트 통과(MyWorld core 36 + app 6, AIMemory 84, Brain 56, Diary 15, Hub 5).
- **커밋/푸시**: TXTMyWorld `07dee44 43885e6 ea6eb36 39a1985`(push✓), TXTSpace `8d96eb6 dd7381e`(push✓), 나머지 4개 저장소 커밋 완료(원격 ahead 0).
- **버전**: MyWorld v0.2.7 · AIMemory v0.9.6 · Drop v0.8.1 · Diary v1.0.2 · Brain v1.9.4 · Space v0.1.6-dev.
- 완료보고서: `docs/reports/report_20260725_txt-family-connect-protocol-phase1-5_claudecode.md` (Vault·Outline 위키 동기화 완료).

## 진행 중 / 미완료 작업
- **Phase 0 미착수**: "TXT 패밀리 마스터" 문서(Outline id 0acded6a) 개정 + OpenAPI 스펙 작성. 순수 문서 작업.
- **실앱 재배포·실동작 E2E**: 현재 실행 중인 앱들(TXTBrain v1.9.2·TXTSpace-hub 0.1.0·TXTAIMemory 구버전)은 이번 세션 이전 빌드라 새 기능이 실동작하지 않는다. 각 앱 재빌드·재시작 후에야 X1/X2/레지스트리/인증이 실제로 작동. **재시작은 사용자 판단 대기로 보류**(TXTAIMemory는 MCP 연결 4개가 붙어 있어 재시작 시 다른 AI 세션 영향 주의).
- 릴리즈 빌드·바탕화면 바로가기: 재시작 보류에 따라 이번 체크포인트에서 미실행.

## 주요 결정사항 / 규칙
- 로드맵 Phase 1~5 전 범위를 6개 저장소에 실제 구현(사용자 지시: "로드맵 전체 진행").
- 각 저장소에서 소스 앱은 최소 침습 — 기존 컨벤션(TXTBrain=CHANGELOG, 나머지=VERSION_HISTORY) 준수.
- 레지스트리 조회 실패 시 항상 기존 하드코딩 폴백(하위호환, 하드 의존 아님).
- TXTAIMemory는 레지스트리에 control/keyword 2개 키 별도 등록. 소비 측은 목적에 맞는 키로 조회.

## 참고 정보
- 완료보고서(본문 원본): C:\Projects\TXTMyWorld\docs\reports\report_20260725_txt-family-connect-protocol-phase1-5_claudecode.md
- Outline TXTMyWorld 위키: /doc/txtmyworld-DDTZamg5Zi (id 849e1a16-1d75-46ab-a369-0a714732997e, revision 7)
- 정정한 4개 전략 문서: C:\Downloads\TXT_패밀리_*.md (실측 정정 블록 포함)
- 실측 포트: Diary 47821 / Brain 8811 / AIMemory control 47530·keyword 47531 / Space Hub 47540

## 다음 세션 시작 시 할 일
1. (사용자 확정 시) 각 앱 재빌드·재시작 → X1 벡터 수신·X2 환류·레지스트리·인증 실동작 E2E 검증. TXTAIMemory MCP 연결 영향 먼저 확인.
2. Phase 0: 마스터 문서 개정 + OpenAPI 스펙 작성.
3. 실동작 검증에서 발견되는 문제 개선(이번 세션 "기능구현확인 및 고도화" 루프의 원래 목표).
