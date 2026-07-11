# TXTMyWorld — 프로젝트 지침 (Claude Code)

## 1. 프로젝트 개요

TXTMyWorld는 **TXT 패밀리의 최종 레이어** — 세 종류의 맥락(개인 기억·문서 지식·AI 대화)의 키워드를 **기간·빈도·벡터(의미)**로 조합해 연관을 찾고 **새 맥락을 생성**하는 "잇기·만들기" 앱이다. TXTSpace가 "보기(지도)"라면, TXTMyWorld는 그 지도 위에서 새 연결을 만든다. **벡터·의미 검색이 핵심 축**(RC까지 최종에 가깝게 설계).

- 이름: **TXTMyWorld 확정** (2026-07-12, 가칭 아님).
- 상태: 착수 전 (컨텍스트 정리 단계). v0.1 PRD 초안 작성됨: `docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md`.
- 전체 파악 시작점: `docs/context/README_txt-series-overview_20260712.md`
- 상위 규약: `docs/architecture/masterspec_20260709_txt-family-master_yumi.md`

## 2. 설계 시 반드시 상속하는 공통 규격

TXT 패밀리 공통 **Keyword/Context API (schema v1.0)** 를 통합 조회로 소비한다.

- localhost 전용(127.0.0.1), read-only(GET만, 쓰기 405), 페어링 토큰 인증
- **본문 미포함** — 원문은 절대 API로 나가지 않고 메타데이터(키워드·빈도·감정·동시출현)만
- `source` 필드로 소스 구분, `ai_id` 확장 필드 지원, `schema_version` 필수
- AI 공통 규칙: 재정리 우선(원장→통합→망각), 진단 금지, 동의 기반 외부 전송

## 3. 버전 규칙

표준 `MAJOR.MINOR.PATCH` 세미버, `0.1.0` 시작. 루트 `VERSION` 파일이 단일 소스. 상세는 `VERSIONING.md`.

- MAJOR=호환 깨짐/데이터 구조 변경, MINOR=기능 추가/UI 개편, PATCH=버그 수정
- 코드에서 `APP_VERSION` 상수 + `VERSION_HISTORY` 리스트로 관리, 기능 추가 시마다 갱신
- 버전 표시: 로고 옆(없으면 창 제목 + 화면 상단 모서리)에 `vX.Y.Z` 상시 표시

## 4. 히스토리 메뉴 요구사항 (UI 앱 필수)

UI가 생기면 설정(또는 정보) 화면에 **"업데이트 히스토리"** 메뉴를 두고 버전별 날짜·변경 요약(최신순, 버전당 2~4줄)을 앱 안에서 바로 보여준다. CHANGELOG.md(git 로그성)와는 별개.

## 5. 접근성 기준 (모든 산출물 최우선)

어두운 배경 / 고대비 텍스트 / 최소 폰트 16px / 터치 타겟 50px 이상 / 색상만으로 상태 구분 금지(텍스트 라벨 병행) / 다크테마 기본 / 대비 약한 회색 텍스트 지양.

## 6. 문서 이중 저장 규칙

완료보고서·사용자요청문서 등은 **두 곳에 동시 저장**한다.

1. 로컬: `C:\Projects\TXTMyWorld\docs\` (성격에 맞는 하위 폴더)
2. Vault: `G:\내 드라이브\SVIL Vault\03_PRJ\TXTMyWorld\`

- 파일명 규칙: `카테고리_YYYYMMDD_내용_작업자.md`, 공백 금지(언더스코어), UTF-8
- 두 위치가 항상 같은 상태가 되도록 동기화한다.

## 7. docs/ 구조

```
docs/
  prd/           # PRD, 스펙 (TXTMyWorld PRD·통합 스펙, TXTSpace/TXTDiary 참조 PRD)
  architecture/  # 아키텍처 문서 (TXTMyWorld 아키텍처, TXT 패밀리 마스터)
  roadmap/       # 로드맵 (버전 마일스톤)
  context/       # 시리즈 개요·배경 맥락
  storyboard/    # 스토리보드
  handoff/       # 작업지시서
  reports/       # 완료보고서
```

## 8. 코드 작성 규칙

파일 경로 주석, 함수/메서드 상단 한 줄 주석, DRY, 에러 핸들링 포함, 민감정보(API 키·토큰·개인정보) 노출 금지.
