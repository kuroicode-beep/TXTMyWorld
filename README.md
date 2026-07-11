# TXTMyWorld

> TXT 패밀리의 **연결·생성 레이어** (가칭). 세 종류의 맥락(개인 기억·문서 지식·AI 대화)을 가로질러 연관을 찾고, 새로운 맥락을 만들어내는 개인 지능 생태계의 최종 앱.

현재 버전: `v0.1.0` · 상태: **착수 전 (컨텍스트 정리 단계)**

## TXT 패밀리 안에서의 위치

```
[맥락화 3축]                     [시각화]        [연결·생성]
TXTDiary  (개인 기억) ─┐
TXTBrain  (문서 지식) ─┼─▶ TXTSpace (보기) ─▶ TXTMyWorld (잇기·만들기)
TXTAIMemory (AI 대화) ─┘
```

- **TXTSpace = 보기.** 세 소스를 하나의 지도로 시각화한다.
- **TXTMyWorld = 잇기·만들기.** 그 지도 위에서 새 연결을 발견하고 생성한다. (본 프로젝트)

## 문서

프로젝트 전반과 TXT 패밀리 시리즈 컨텍스트는 `docs/`에 정리되어 있다.

- 시작점: [`docs/context/README_txt-series-overview_20260712.md`](docs/context/README_txt-series-overview_20260712.md)
- 상위 규약: [`docs/architecture/masterspec_20260709_txt-family-master_yumi.md`](docs/architecture/masterspec_20260709_txt-family-master_yumi.md)
- 형제 PRD: [`docs/prd/`](docs/prd/) (TXTSpace, TXTDiary)

프로젝트 규칙·버전 정책은 [`CLAUDE.md`](CLAUDE.md) / [`AGENTS.md`](AGENTS.md) / [`VERSIONING.md`](VERSIONING.md) 참조.

## PRD

- [`docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md`](docs/prd/prd_20260712_txtmyworld-v0-1_claudecode.md) — TXTMyWorld v0.1 PRD 초안
  - 핵심: 세 소스 키워드를 **기간·빈도·벡터**로 조합해 새 주제를 발견·생성
  - 방향: 하이브리드 발견 · 주제 카드+환류 · 로컬 우선+동의형 클라우드 · 독립 앱

## 다음 할 일

- [x] TXTMyWorld 전용 PRD 초안 (패밀리 마스터 §2.1·§3 기반)
- [ ] 앱 이름 확정 (현재 가칭 — PRD §16 D0)
- [ ] 벡터 소스 확정: 로컬 자체 임베딩 vs 공통 API 벡터 확장 (PRD §16 D1)
- [ ] 상위 3축 + TXTSpace 규격 준비 상태 점검
