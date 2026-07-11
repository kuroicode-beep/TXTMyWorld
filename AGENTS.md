# TXTMyWorld — 에이전트 지침 (Cursor / Codex / 범용)

이 문서는 모든 AI 에이전트를 위한 범용 지침이다. 상세 내용은 [`CLAUDE.md`](CLAUDE.md)와 동일하며, 여기서는 핵심만 요약한다.

## 프로젝트 한 줄 정의

TXTMyWorld = **TXT 패밀리의 최종 레이어(가칭)**. 세 맥락(개인 기억·문서 지식·AI 대화)을 가로질러 연관을 찾고 새 맥락을 **생성**하는 "잇기·만들기" 앱. TXTSpace(보기)의 상위.

- 상태: 착수 전. 전용 PRD 미작성.
- 전체 파악 시작점: `docs/context/README_txt-series-overview_20260712.md`
- 상위 규약: `docs/architecture/masterspec_20260709_txt-family-master_yumi.md`

## 반드시 지킬 것

1. **공통 규격 상속**: Keyword/Context API schema v1.0을 통합 조회로 소비. localhost 전용·read-only·본문 미포함·`source`/`ai_id`/`schema_version`. 상세 CLAUDE.md §2.
2. **버전 규칙**: 세미버 `0.1.0` 시작. 루트 `VERSION`이 단일 소스. 상세 `VERSIONING.md`.
3. **히스토리 메뉴**: UI가 생기면 설정 화면에 버전별 업데이트 내역(최신순)을 앱 내에서 노출.
4. **접근성**: 다크·고대비·최소 폰트 16px·터치 타겟 50px·색상만으로 상태 구분 금지.
5. **문서 이중 저장**: 완료보고서 등은 `docs/` + `G:\내 드라이브\SVIL Vault\03_PRJ\TXTMyWorld\` 동시 저장. 파일명 `카테고리_YYYYMMDD_내용_작업자.md`, 공백 금지, UTF-8.
6. **코드 규칙**: 파일 경로 주석, 함수 상단 한 줄 주석, DRY, 에러 핸들링, 민감정보 노출 금지.

자세한 내용은 [`CLAUDE.md`](CLAUDE.md)를 참고할 것.
