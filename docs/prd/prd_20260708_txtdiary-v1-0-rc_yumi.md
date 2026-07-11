# TXTDiary v1.0 RC 제품 요구사항 정의서(PRD)

하루의 시작과 마무리. 나의 마음을 들여다보다.

문서 버전: v3.1 / 대상 제품 버전: TXTDiary v1.0 RC / 작성일: 2026-07-08 / 작성자: SVIL 유미

> 출처: SVIL Outline 위키 (2026-07-12 로컬 복제). TXT 패밀리 맥락화 3축의 첫 번째 축이자 **공통 Keyword API schema v1.0의 원본 정의처** — TXTMyWorld가 상속하는 규격의 근거 문서. 참조용.

변경 이력:

* v2.0 (2026-06-27, 루미): MindBoard v1.0 RC PRD 최초 작성
* v3.0 (2026-07-08, 유미): 제품 분리 결정 반영. MindBoard → **TXTDiary**로 개명(MindBoard 브랜드 완전 폐기, TXT 패밀리로 통일). 생각 은하수(키워드 우주 맵)는 별도 앱 **TXTSpace**로 분리. Ghost 블로그 연동과 TXTBrain 직접 연동은 v1.1 이후로 이동. 저사양 폴백, 오프라인 라이선스, 정신건강 안전장치, KPI 측정 방식 보완.
* v3.1 (2026-07-08, 유미): TXTSpace 데이터 전달 방식을 **로컬 read-only Keyword API(실시간)**로 확정. TXTSpace는 TXTDiary RC 완료 후 순차 개발로 확정.

문서 목적: 사람이 읽기 쉬우면서도 Cursor, Claude Code, Codex 같은 AI 개발 도구가 바로 이해하고 작업 단위로 분해할 수 있도록 제품 철학, 범위, 기능, 데이터, AI, 검색, 아키텍처, 테스트 기준을 구조화한다.

---

## 0. 핵심 결정사항 요약

| 항목  | 결정  |
|-----|-----|
| 제품 방향 | 개인 기록 중심의 로컬 우선 AI 다이어리. v1.0 RC는 "기록 → AI 분석 → 재노출(전광판/검색)" 핵심 루프의 완성도에 집중한다. |
| 제품 분리 | 키워드 우주 맵(구 생각 은하수)은 별도 앱 **TXTSpace**로 분리한다. TXTDiary는 시각화 대신 **키워드 데이터 제공자** 역할을 맡는다. TXTSpace는 TXTDiary RC 완료 후 순차 개발한다. |
| TXTSpace 연동 | **로컬 read-only Keyword API**(localhost 전용, 실시간)로 키워드 데이터를 제공한다. 수동 JSON export는 보조 수단이다. |
| AI 모델 | 로컬 무료: qwen3:8b 기본, gemma4:12b 고품질 모드. DeepSeek API는 사용자가 각자 API Key를 조달한다. |
| 수익 모델 | Basic 영구 무료 + Premium 영구 소장. 광고 제거, 테마/폰트/이모지, 클라우드 백업 등 프리미엄 해금. |
| 클라우드 | Railway 백엔드. 시리얼 키를 인증 토큰으로 검증하고 백업 권한을 판단한다. |
| 검색  | 단순 키워드 검색(FTS5), AI 자연어 검색, 벡터 검색을 모두 포함한다. |
| 생태계 | TXTDiary=개인 기록, TXTSpace=키워드 우주 맵/연결 허브, TXTBrain=문서 자료, Profile Engine=성장 흐름 분석. |
| 플랫폼 | v1.0 RC는 Windows 중심. 향후 Mac, iOS, Android 확장 예정. |
| 블로그 연동 | v1.0 RC에서 제외. Ghost 연동은 v1.1 예정. |
| 초기 KPI | 다운로드 100회, 활성 사용자 20명. |

## 1. 제품 비전과 철학

### 1.1 제품 정의

TXTDiary는 사용자의 일상 기록, 감정, 깨달음, 계획, 감사, 생각을 로컬에 안전하게 저장하고, AI 분석과 바탕화면 전광판을 통해 사용자가 자신의 생각을 자연스럽게 다시 마주하도록 돕는 독립 구동형 오픈소스 다이어리 앱이다.

TXTDiary는 단순한 일기장이 아니라 SVIL Personal Intelligence Ecosystem의 첫 번째 축이다. 사용자는 TXTDiary에 생각을 기록하고, TXTSpace에서 키워드의 우주를 탐험하며, TXTBrain에서 문서를 구조화하고, 향후 Profile Engine을 통해 자신의 생각의 흐름과 성장 패턴을 발견한다.

### 1.2 생태계 슬로건

생각을 기록하고, 문서를 구조화하고, 나의 생각의 흐름을 눈치채고, 지식과 지식, 생각과 생각을 연결해 나를 성장시키는 앱.

### 1.3 핵심 가치

* Local-first: 개인 기록은 우선 사용자 기기에 저장한다.
* Token is Time: 토큰은 시간이고 시간은 금이다. 불필요한 API 호출과 모델 실행을 줄인다.
* Open Source Transparency: 광고, 후원, 시리얼, 클라우드 백업 정책을 투명하게 운영한다.
* Accessible by Design: 저시력자와 키보드 사용자도 핵심 흐름을 사용할 수 있도록 설계한다.
* Focused RC: v1.0 RC는 "기록-분석-재노출" 핵심 루프를 완결성 있게 검증한다. 시각화와 외부 연동은 TXTSpace와 후속 버전에 위임한다.

### 1.4 제품이 해결하는 문제

| 문제  | TXTDiary의 해결 방향 |
|-----|-----------------|
| 기록이 흩어져 다시 보지 않음 | 전광판, 회고, 검색으로 기록을 재노출한다. |
| 일기를 써도 성장 흐름이 보이지 않음 | 키워드, 감정, 기간별 패턴을 구조화해 저장하고, TXTSpace/Profile Engine 분석의 기반 데이터를 만든다. |
| AI 앱이 비용과 리소스를 과도하게 사용함 | 로컬 모델 우선, 캐시, 수동 API Key, 모델 생명주기 관리로 비용을 통제한다. |
| 복잡한 다이어리 앱은 쓰기 부담스러움 | 4섹션 고정 구조와 간단한 AI 보조로 쓰기 마찰을 줄인다. |

## 2. SVIL Personal Intelligence Ecosystem

### 2.1 네 앱의 역할

| 앱   | 중심 데이터 | 핵심 질문 | 역할  |
|-----|--------|-------|-----|
| TXTDiary | 일기, 감정, 계획, 감사, 한 줄 생각 | 나는 무엇을 경험하고 느꼈는가? | 개인 기록 중심 앱 (본 PRD) |
| TXTSpace | 키워드, 연결, 감정 색상, 시간 흐름 | 내 생각들은 어떻게 이어져 있는가? | 키워드 우주 맵/연결 허브 앱 (별도 PRD) |
| TXTBrain | PDF, Markdown, 책, 논문, 웹자료, 메모 | 이 생각과 연결되는 지식은 무엇인가? | 문서 자료 중심 앱 |
| Profile Engine | 장기 키워드, 감정 추이, 관심 변화 | 나는 어떤 방향으로 변하고 있는가? | 개인 프로파일링/성장 분석 앱 (향후) |

### 2.2 생태계 흐름

1. TXTDiary에서 사용자가 생각을 기록한다.
2. AI가 기록에서 키워드, 감정, 요약, 한 줄 생각을 추출한다.
3. TXTDiary 검색 화면에서 키워드 기반으로 과거 기록을 연결한다.
4. TXTSpace가 TXTDiary의 키워드 데이터를 읽어 우주 맵으로 시각화한다. (TXTSpace 앱)
5. TXTSpace가 향후 TXTBrain 문서 키워드까지 통합해 생각과 지식을 연결한다.
6. 장기적으로 Profile Engine이 반복 키워드, 관심 변화, 감정 흐름을 분석한다.

### 2.3 TXTSpace 연동 (Keyword API)

TXTDiary는 v1.0 RC에서 시각화를 직접 구현하지 않는 대신, TXTSpace가 실시간으로 소비할 수 있는 **로컬 read-only Keyword API**를 제공한다. (확정: 2026-07-08)

| 구성  | v1.0 RC 구현 | 향후 확장 |
|-----|------------|-------|
| Keyword API | localhost 전용 read-only HTTP API. 키워드/감정/날짜/빈도/동시 출현 통계를 실시간 제공 | 변경 이벤트 push(SSE), 임베딩 벡터 공유 |
| 보조 export | 수동 JSON export (TXTDiary 미실행 시 TXTSpace 폴백용, 백업/이동용) | —     |
| 딥링크 수신 | `txtdiary://search?keyword={keyword}` URI 스킴 등록. TXTSpace에서 노드 클릭 시 TXTDiary 검색 결과 페이지가 열린다 | 기간/감정 파라미터 확장 |
| 프라이버시 | 일기 **본문은 API 응답과 export에 절대 포함하지 않는다**. 키워드/메타데이터만 제공 | 사용자 선택형 요약 포함 옵션 |

* API 응답 스키마는 TXTSpace PRD와 공동 소유하며, 버전 필드(schema_version)를 포함한다.
* 사용자는 설정에서 Keyword API를 켜고 끌 수 있어야 한다. 기본값은 OFF이며, TXTSpace 최초 연결 시 사용자 동의 후 활성화한다.
* API는 127.0.0.1에만 바인딩하며, 최초 연결 시 발급되는 페어링 토큰으로 인증한다. (§8.3 참조)

### 2.4 Profile Engine을 위한 예약 요구사항

* TXTDiary는 감정점수, 키워드, 작성일, 섹션별 본문, AI 요약, 한 줄 생각을 구조화해 저장해야 한다.
* 모든 분석 결과는 나중에 외부 앱이 읽을 수 있도록 JSON export 또는 로컬 API로 내보낼 수 있어야 한다.
* 사용자는 프로파일링 대상 데이터를 직접 선택하거나 제외할 수 있어야 한다.
* Profile Engine은 v1.0 RC 범위가 아니며, 본 PRD에서는 연동 준비만 포함한다.
* 개인 프로파일링은 민감 정보로 취급하며 사용자의 명시적 동의, 로컬 우선 처리, 삭제 가능성을 기본 원칙으로 한다.

## 3. 제품 범위와 릴리즈 전략

### 3.1 v1.0 RC 원칙

v1.0 RC는 "기록 → AI 분석 → 재노출" 핵심 루프의 완결성을 검증하는 버전이다. 시각화(TXTSpace 분리), 블로그 연동(v1.1), 외부 문서 연결(TXTSpace 경유)은 범위에서 제외해 1인 개발 체제에서 완주 가능한 규모를 유지한다. 보안, 데이터 손실 방지, 앱 종료 시 AI 리소스 반환은 RC에서도 필수 품질 기준이다.

### 3.2 범위

| 구분  | 포함  | 제외/이동 |
|-----|-----|-------|
| 일기  | 4섹션 작성, 저장, 수정, 삭제(soft delete), 검색 | 복잡한 템플릿 마켓 |
| AI  | 요약, 감정, 키워드, 명언, 명상 스크립트, 자연어 검색, 운세, 뉴스 | 서버 부담형 무료 AI 제공 |
| 전광판 | 바탕화면 텍스트 흐름, 설정, Alt+클릭 | 고급 애니메이션/3D 효과 |
| 키워드 시각화 | **제외 → TXTSpace 앱** (Keyword API만 제공) | —     |
| 검색  | FTS5 키워드 검색, AI 자연어 검색, 벡터 검색 | 대규모 분산 검색 |
| 클라우드 | Railway + 시리얼 키 토큰 검증 + 암호화 백업 | 다중 클라우드 제공자 |
| 블로그 | **제외 → v1.1** (Ghost 연동) | WordPress/Tistory/Velog |
| TXTBrain | **제외 → TXTSpace 경유로 v2.0** | 직접 URI 연동 |
| 플랫폼 | Windows RC | Mac/iOS/Android 네이티브 최적화 |

### 3.3 버전 로드맵

| 버전  | 목표  | 주요 기능 |
|-----|-----|-------|
| v0.1 Prototype | 핵심 UI와 저장 검증 | 일기 CRUD, SQLite, 기본 전광판 |
| v0.5 Alpha | AI/검색/키워드 흐름 연결 | 로컬 LLM, 키워드 추출, FTS5, 검색 결과 페이지 |
| v0.8 Beta | 수익/백업/접근성 검증 | 시리얼 키, 광고, 테마, Railway 백업, WCAG 체크 |
| v1.0 RC | 핵심 루프 완성 | 운세, 뉴스, 명상, TTS, 벡터 검색, **Keyword API(read-only)** |
| v1.1 | 안정화 + 확장 1차 | 성능 개선, 백업 복원 안정화, **Ghost 블로그 연동** |
| v1.5 | TXTSpace 연동 강화 | API 변경 이벤트 push(SSE), 딥링크 파라미터 확장 |
| v2.0 | 생태계 통합 | TXTSpace 경유 TXTBrain 연결, Profile Engine 연동, 멀티플랫폼 |

### 3.4 MoSCoW 우선순위

| 우선순위 | 항목  |
|------|-----|
| Must | 일기 CRUD, SQLite 암호화 저장, 키워드 검색(FTS5), AI 요약/키워드/감정, 전광판, Ollama 사이드카 생명주기, DeepSeek API Key 입력, 시리얼 키 검증, 기본 백업/복원, 접근성 기본, **Keyword API(read-only, localhost)** |
| Should | 벡터 검색, AI 자연어 검색, 명상/TTS, 운세, 뉴스 큐레이션, txtdiary:// 딥링크 수신 |
| Could | 수동 JSON export(폴백/백업용), 고급 테마, 확장 이모지, 커버 이미지 자동 매핑, 월간 회고 리포트 |
| Won't for RC | 키워드 우주 맵(TXTSpace), Ghost/블로그, TXTBrain 직접 연동, 모바일 앱, Profile Engine, 팀/공유 기능 |

## 4. 타겟 사용자와 사용자 시나리오

### 4.1 주요 페르소나

| 페르소나 | 특징  | 원하는 가치 |
|------|-----|--------|
| 성찰형 기록자 | 매일 감정과 생각을 기록하고 싶은 사용자 | 나의 변화와 반복 패턴을 알고 싶다. |
| 전문가/창작자 | 업무 중 핵심 가치와 아이디어를 자주 되새기는 사용자 | 기록이 창작과 의사결정으로 이어지길 원한다. |
| 테크니컬 유저 | 로컬 데이터, 오픈소스, 리소스 최적화에 민감한 사용자 | 데이터 주권과 성능 통제를 원한다. |
| 저시력 사용자 | 고대비, 큰 글자, 키보드 조작이 중요한 사용자 | 방해 없이 접근 가능한 기록 환경을 원한다. |

### 4.2 핵심 사용자 여정

1. 아침에 앱을 열고 오늘의 계획과 한 줄 마음을 기록한다.
2. AI가 오늘의 기록에서 키워드와 명언을 추천한다.
3. 전광판에 오늘의 한 줄 생각과 명언이 흐른다.
4. 하루를 마무리하며 한 일, 감상, 감사를 기록한다.
5. AI가 감정, 키워드, 요약, 명상 스크립트를 생성한다.
6. 사용자는 명상 TTS를 듣거나 검색에서 관련 기록을 다시 본다.
7. TXTSpace를 설치했다면, 우주 맵에서 노드를 클릭해 TXTDiary 검색으로 돌아온다.
8. 주간/월간 회고에서 반복되는 생각과 성장 흐름을 확인한다.

## 5. 핵심 기능 상세 명세

### 5.1 온보딩

| 단계  | 입력/동작 | 결과  |
|-----|-------|-----|
| 개인화 정보 | 이름, 생년월일시, 표시 이름, 언어 | 운세/개인화 설정에 저장 |
| 시스템 사양 진단 | OS, RAM, GPU/VRAM, Ollama 설치 여부 | 추천 AI 모드 표시 |
| AI 모드 선택 | qwen3:8b / gemma4:12b / DeepSeek API / **AI 없이 사용** | 기본 AI Provider 저장 |
| 데이터 보안 설정 | 로컬 암호화 비밀번호 또는 OS 보안 저장소 사용 | SQLite 암호화 키 준비 |
| 전광판 튜토리얼 | Alt+클릭 설정, 속도/투명도 안내 | 사용자가 방해 없이 전광판 제어 가능 |
| 접근성 설정 | 고대비, 글자 크기, 자간, 행간, 키보드 모드 | 초기 UI 접근성 프로필 저장 |

#### 5.1.1 저사양 폴백 정책 (신설)

| 진단 결과 | 안내 및 폴백 |
|-------|---------|
| VRAM 12GB 이상 | gemma4:12b 포함 전체 로컬 모드 추천 |
| VRAM 8GB 내외 | qwen3:8b 기본 모드 추천 |
| VRAM 부족/GPU 없음 | CPU 추론 속도 저하 경고 + DeepSeek API 모드 또는 "AI 없이 사용" 안내 |
| Ollama 미설치 | 설치 가이드 링크 + "AI 없이 사용" 즉시 시작 옵션 |

* **"AI 없이 사용" 모드는 일기/검색(FTS5)/전광판이 완전히 동작해야 한다.** AI는 있으면 좋은 층위이지 필수 전제가 아니다.
* 사양 진단 실패 시에도 온보딩이 중단되면 안 된다.

### 5.2 지능형 일기 시스템

일기는 고정 4섹션 구조를 기본으로 한다. 고정 구조는 작성 부담을 낮추고, AI 분석과 검색 품질을 높이며, 기간별 비교를 쉽게 만든다.

| 섹션  | 목적  | 예시  |
|-----|-----|-----|
| 계획  | 하루의 방향을 기록 | 오늘은 PRD 정리와 산책을 한다. |
| 한 일 | 실제 수행한 일을 기록 | TXTDiary 검색 구조를 정리했다. |
| 감상  | 감정과 깨달음을 기록 | 생각과 지식이 연결되는 흐름이 보였다. |
| 감사  | 감사한 일/사람/상황을 기록 | 오늘도 아이디어가 이어져서 감사하다. |

* 일기에는 제목이 없어도 저장 가능해야 한다. 제목이 비어 있으면 날짜 + 대표 키워드로 자동 제목을 생성한다.
* 저장 시 작성일, 수정일, 로컬 타임존, 섹션별 글자 수, AI 분석 상태를 기록한다.
* 서적, 음악, 영화 제목으로 보이는 표현은 후보 엔티티로 저장하고, API 매핑은 실패해도 일기 저장을 막지 않는다.
* 나의 한 줄 생각은 전광판 데이터셋으로 전송할 수 있다.
* 삭제는 즉시 영구 삭제가 아니라 휴지통/soft delete를 기본으로 한다.

### 5.3 AI 분석 기능

| 기능  | Provider | RC 출력 | 캐시 정책 |
|-----|----------|-------|-------|
| 요약  | 로컬 우선    | 3문장 요약, 한 줄 요약 | 일기 수정 전까지 재사용 |
| 감정 분석 | 로컬 우선    | 감정 라벨, 감정 점수 -1.0~1.0 | 일기 버전 기준 재사용 |
| 키워드 추출 | 로컬 우선    | 대표 키워드 3~10개, 중요도 | 일기 버전 기준 재사용 |
| 명언 추천 | 로컬 또는 DeepSeek | 오늘의 명언 1개, 이유 1문장 | 동일 키워드 7일 캐시 |
| 명상 스크립트 | DeepSeek 또는 로컬 | 1~3분 분량 스크립트 | 동일 일기 기준 재사용 |
| 운세  | DeepSeek | 주/월/년 운세 요약 | 일/주/월 단위 캐시 |
| 뉴스 큐레이션 | DeepSeek + 외부 소스 | HTML 뉴스 페이지 링크 목록 | 6시간 캐시 |

### 5.4 힐링 워크플로우

1. 사용자가 일기를 저장한다.
2. AI가 감정 선과 대표 키워드를 분석한다.
3. 사용자가 명상 생성 버튼을 누른다.
4. AI가 1:1 맞춤형 명상 스크립트를 생성한다.
5. TTS 엔진이 차분한 톤으로 읽어준다.
6. 오디오 플레이어 UI에서 재생/정지/속도/볼륨을 조절한다.
7. 생성된 스크립트와 오디오 메타데이터는 일기와 연결해 저장한다.

#### 5.4.1 정서 안전장치 (신설)

* 감정 분석 결과는 진단이 아니며, UI에 이를 명시한다.
* 부정 감정 점수가 강하게 반복되는 패턴(예: 최근 14일 중 10일 이상 emotion_score ≤ -0.6)이 감지되면, 판단하거나 진단하지 않는 문구로 정신건강 지원 자원 안내 카드를 **선택적으로** 노출한다.
* 안내 카드는 사용자가 끌 수 있으며, 안내 여부/설정은 로컬에만 저장하고 외부로 전송하지 않는다.
* 관련 문구는 지역화(한국: 보건복지부 상담전화 등)를 고려해 locale별 리소스로 관리한다.

### 5.5 바탕화면 전광판

| 항목  | 요구사항 |
|-----|------|
| 윈도우 속성 | Windows에서는 WS_EX_TRANSPARENT(클릭 무시), WS_EX_TOOLWINDOW(Alt+Tab 제외)를 적용한다. |
| 진입 방식 | 전광판 영역에서 Alt+클릭 시 설정 화면으로 진입한다. |
| 표시 데이터 | 나의 한 줄 생각, 오늘의 명언, 감사 문장, AI 추출 대표 키워드, 회고 문장 |
| 제어  | 속도, 갱신 주기, 투명도, 폰트 크기, 위치, 모니터 선택, 일시정지 |
| 접근성 | 고대비 모드, 큰 글자, 그림자/외곽선 옵션, 애니메이션 최소화 옵션 |
| 성능  | 일반 상태 CPU 1% 이하, GPU 부하 최소화, 애니메이션 FPS 제한 가능 |

### 5.6 검색 시스템

검색은 TXTDiary의 핵심 기능이다. TXTSpace에서 노드를 클릭하면 딥링크로 동일한 검색 결과 페이지가 열리므로, 이 페이지는 두 앱이 공유하는 도착점이다.

#### 5.6.1 검색 종류

| 검색 유형 | 설명  | RC 구현 |
|-------|-----|-------|
| 단순 키워드 검색 | 사용자가 입력한 단어와 일치하는 일기/키워드/태그 검색 | SQLite FTS5 |
| AI 자연어 검색 | 의미 기반 질문을 AI가 검색 의도로 해석 | 질문 → 키워드/기간/감정 조건 추출 → 검색 |
| 벡터 검색 | 문장/키워드 임베딩 기반 유사 기록 검색 | 로컬 임베딩 저장 + cosine similarity |
| 딥링크 검색 | TXTSpace 노드 클릭으로 검색 실행 | `txtdiary://search?keyword=` 수신 |

#### 5.6.2 검색 결과 페이지 구성

| 영역  | 내용  |
|-----|-----|
| 검색 헤더 | 검색어, 검색 유형, 기간 필터, 정렬 옵션 |
| 요약 카드 | AI가 검색 결과를 3~5문장으로 요약 |
| 일기 결과 | 관련 일기 목록, 감정 점수, 대표 키워드, 한 줄 요약 |
| 연관 키워드 | 동시 출현 키워드, 유사 키워드, 기간별 변화 |
| 생각 흐름 | 같은 키워드가 시간에 따라 어떻게 변했는지 간단 타임라인 표시 |
| 액션  | 전광판에 추가, 회고에 포함, 키워드 고정 |

#### 5.6.3 자연어 검색 예시

| 사용자 질문 | AI 해석 | 검색 조건 |
|--------|-------|-------|
| 작년에 가장 우울했던 날 | 기간=작년, 감정=부정 높은 기록 | date range + emotion_score ascending |
| 회사 때문에 스트레스 받은 날 | 키워드=회사/업무/스트레스, 감정=부정 | FTS + keyword + emotion |
| 양자역학에 대해 생각했던 기록 | 키워드=양자역학, 유사어=관측자/불확정성 | FTS + vector |
| 요즘 반복되는 고민 | 최근 30일, 빈도 높은 부정 키워드 | keyword frequency + emotion |
| 감사에 대해 쓴 날 | section=감사, keyword=감사 관련 | section filter |

## 6. 수익 모델 및 라이선스

| 구분  | Basic | Premium |
|-----|-------|---------|
| 이용 기간 | 영구 무료 | 영구 소장   |
| 개인화 | 기본 테마/폰트/이모지 | 전체 테마, 감성 서체, 확장 이모지 |
| 광고  | 하단 컴팩트 광고 | 광고 완전 제거 |
| 백업  | 로컬 백업/복원 | 시리얼 키 기반 Railway 클라우드 백업 |
| AI  | 로컬 무료, DeepSeek API Key 직접 입력 | 우선 업데이트/고급 템플릿. 서버 AI 비용은 제공하지 않음 |

### 6.1 시리얼 키 로직

1. 사용자가 후원/결제 플랫폼에서 후원한다.
2. Railway 백엔드가 웹훅을 수신한다.
3. 서버가 사용자 이메일, 결제 id, 발급 시각, 상품 정보를 기반으로 시리얼 키를 생성한다.
4. 시리얼 키를 사용자 이메일로 발송한다.
5. 앱에서 시리얼 키를 입력하면 로컬 검증을 먼저 수행한다.
6. 클라우드 백업 기능 사용 시 Railway 서버에 시리얼 키를 토큰처럼 제출해 유효성을 확인한다.
7. 검증 성공 시 Premium 기능이 해금되고 로컬 라이선스 캐시에 저장된다.

### 6.2 오프라인 라이선스 정책 (신설)

* 한 번 검증에 성공한 시리얼 키는 로컬 라이선스 캐시에 저장되며, **오프라인 상태에서도 Premium 기능(클라우드 백업 제외)이 유지된다.**
* 서버 재검증은 클라우드 백업 요청 시에만 필수이며, 그 외 기능의 주기적 강제 재검증은 하지 않는다.
* 라이선스 캐시가 손상된 경우 시리얼 키 재입력으로 복구 가능하다.

## 7. AI 아키텍처와 프롬프트 정책

### 7.1 Provider 정책

| Provider | 용도  | 비용 정책 | 비고  |
|----------|-----|-------|-----|
| Ollama qwen3:8b | 기본 요약/키워드/감정/검색 해석 | 무료/로컬 | 기본 모드 |
| Ollama gemma4:12b | 고품질 추론/회고/명상 | 무료/로컬 | 일반 이상 사양 권장 |
| DeepSeek API | 운세, 명상, 고난도 추론, 뉴스 큐레이션 | 사용자 API Key 직접 입력 | 앱 서버 비용 없음 |
| 없음 (AI-off) | 일기/검색/전광판만 사용 | 무료    | 저사양/프라이버시 우선 사용자 |
| Fallback Rule | 로컬 실패 시 사용자 선택으로 DeepSeek 전환 | 자동 과금 방지 | 사용자 동의 필요 |

### 7.2 모델 생명주기

* 앱은 Tauri Sidecar 패턴으로 Ollama 프로세스를 관리한다.
* AI 작업이 끝난 뒤 idle timeout이 지나면 모델을 unload한다.
* 앱 종료 시 Ollama 모델을 즉시 unload하고 RAM/GPU VRAM을 반환한다.
* 백그라운드 전광판만 실행 중인 상태에서는 AI 모델을 유지하지 않는다.
* 사용자는 설정에서 모델 자동 실행, 수동 실행, 종료 시 unload 정책을 선택할 수 있다.

### 7.3 AI 출력 JSON 스키마

```
{
  "summary": "string",
  "one_line_thought": "string",
  "emotion": {
    "label": "positive|neutral|negative|mixed",
    "score": 0.0,
    "confidence": 0.0
  },
  "keywords": [
    {"text": "string", "weight": 0.0, "category": "topic|emotion|person|work|creative|other"}
  ],
  "quote": {
    "text": "string",
    "author": "string",
    "reason": "string"
  },
  "warnings": []
}
```

### 7.4 프롬프트 원칙

* AI는 사용자를 진단하거나 단정하지 않는다. 기록 기반의 경향과 가능성만 제안한다.
* 감정 분석 결과는 의료/심리 진단이 아니다.
* 프롬프트는 반드시 입력, 작업, 출력 JSON Schema, 금지사항을 포함한다.
* 민감한 일기 내용은 사용자가 DeepSeek API 사용을 허용한 경우에만 외부 API로 전송한다.
* 동일한 일기 버전의 동일 분석은 캐시를 우선한다.

## 8. 데이터 설계

### 8.1 SQLite 주요 테이블

| 테이블 | 목적  | 주요 필드 |
|-----|-----|-------|
| diaries | 일기 본문 | id, date, title, plan, did, reflection, gratitude, created_at, updated_at, deleted_at |
| diary_ai_results | AI 분석 결과 | id, diary_id, provider, model, summary, one_line_thought, emotion_label, emotion_score, json, version_hash |
| keywords | 키워드 사전 | id, text, normalized_text, type, created_at |
| diary_keywords | 일기-키워드 연결 | diary_id, keyword_id, weight, source |
| embeddings | 임베딩 저장 | id, target_type, target_id, provider, model, vector_blob, dimension, created_at |
| display_items | 전광판 데이터 | id, text, source_type, source_id, priority, enabled, start_at, end_at |
| search_history | 검색 기록 | id, query, query_type, result_count, created_at |
| backups | 백업 이력 | id, backup_type, path_or_remote_id, status, created_at |
| licenses | 라이선스 상태 | id, serial_hash, status, premium_until, last_verified_at |
| settings | 앱 설정 | key, value_json, updated_at |
| space_api | Keyword API 상태 | id, enabled, port, pairing_token_hash, last_connected_at |

### 8.2 인덱스

* diaries.date, diaries.updated_at 인덱스
* keywords.normalized_text unique index
* diary_keywords.keyword_id index
* FTS5 virtual table: diary_fts(title, plan, did, reflection, gratitude, summary, keywords)
* embedding target index: target_type, target_id
* search_history.created_at index

### 8.3 Keyword API 명세 (TXTSpace 공유, 신설)

로컬 read-only HTTP API. 127.0.0.1에만 바인딩하며 외부 네트워크에 절대 노출하지 않는다.

| Endpoint | Method | 목적  |
|----------|--------|-----|
| /health  | GET    | API 상태, schema_version, 앱 버전 확인 |
| /keywords | GET    | 키워드 목록. 쿼리 파라미터: from, to, category, min_frequency, limit |
| /keywords/{normalized_text}/cooccurrence | GET    | 해당 키워드의 동시 출현 통계 |

**인증/보안:**

* 최초 연결 시 TXTDiary가 페어링 토큰을 발급하고 사용자에게 승인 다이얼로그를 표시한다.
* 이후 요청은 Authorization 헤더의 페어링 토큰으로 검증한다. 토큰은 해시로만 저장한다.
* 쓰기 계열 메서드(POST/PUT/DELETE)는 존재하지 않는다. (read-only 원칙)

**/keywords 응답 스키마 (schema v1.0):**

```
{
  "schema_version": "1.0",
  "source": "txtdiary",
  "generated_at": "ISO8601",
  "date_range": {"from": "date", "to": "date"},
  "keywords": [
    {
      "text": "string",
      "normalized_text": "string",
      "category": "topic|emotion|person|work|creative|other",
      "frequency": 0,
      "avg_emotion_score": 0.0,
      "first_seen": "date",
      "last_seen": "date",
      "cooccurrence": [{"text": "string", "count": 0}]
    }
  ]
}
```

* 일기 본문/요약은 API 응답에 절대 포함하지 않는다. (프라이버시 원칙)
* 수동 JSON export(Could)는 동일한 스키마의 파일 스냅샷이다.
* schema_version은 TXTSpace PRD와 동기화하여 관리한다.

### 8.4 데이터 보안

* SQLite 암호화는 SQLCipher 또는 동등 수준의 암호화 레이어를 사용한다.
* 암호화 키는 Windows DPAPI, macOS Keychain, iOS/Android secure storage에 저장한다.
* DeepSeek API Key, Railway backup token은 평문 저장 금지.
* 백업 파일은 앱에서 암호화한 뒤 업로드한다. 서버는 평문 일기 내용을 볼 수 없어야 한다.
* 사용자는 export, delete, local-only 모드를 제어할 수 있어야 한다.

## 9. 시스템 아키텍처

### 9.1 기술 스택

| 영역  | 기술  | 비고  |
|-----|-----|-----|
| 프런트엔드 | Tauri + React 또는 Vue | 가벼운 런타임, 네이티브 API 제어 |
| 백엔드/네이티브 | Rust commands | SQLite, 파일, OS API, sidecar 제어 |
| DB  | SQLite + FTS5 + 암호화 | 로컬 영속성 |
| 로컬 AI | Ollama qwen3:8b / gemma4:12b | sidecar/lifecycle 관리 |
| 클라우드 AI | DeepSeek API | 사용자 API Key 입력 |
| 클라우드 | Railway | 시리얼 키, 백업 토큰 검증, 암호화 백업 저장 |
| TTS | OS TTS 또는 내장 TTS adapter | RC에서는 단순 재생 우선 |
| Keyword API | Rust 경량 HTTP 서버 (axum 등) | 127.0.0.1 전용, 페어링 토큰 인증 |
| i18n | locales/*.json | 한국어/영어 기본 |

### 9.2 모듈 구조

```
src/
  app/
  features/
    diary/
    ai/
    search/
    billboard/
    meditation/
    fortune/
    news/
    backup/
    license/
    space_api/
    accessibility/
  shared/
    db/
    models/
    settings/
    i18n/
    ui/
src-tauri/
  src/
    commands/
    db/
    sidecar/
    license/
    backup/
    os_window/
    deep_link/
    security/
```

* v2.0 대비 변경: `galaxy/`, `ghost/`, `txtbrain_bridge/` 모듈 제거. `space_api/`(Keyword API 서버), `deep_link/`(txtdiary:// 수신) 신설.

### 9.3 Railway 백엔드

| Endpoint | Method | 목적  |
|----------|--------|-----|
| /health  | GET    | 서버 상태 확인 |
| /webhooks/payment | POST   | 후원/결제 웹훅 수신 |
| /license/issue | POST   | 시리얼 키 발급. 내부/관리용 |
| /license/verify | POST   | 시리얼 키 유효성 검증 |
| /backup/upload | POST   | 암호화 백업 업로드 |
| /backup/list | GET    | 사용자 백업 목록 |
| /backup/download/{id} | GET    | 암호화 백업 다운로드 |
| /backup/delete/{id} | DELETE | 백업 삭제 |

## 10. 성능 및 리소스 목표

| 항목  | 목표  |
|-----|-----|
| 앱 실행 | 일반 환경 2초 내 메인 화면 표시 |
| 일기 저장 | 300ms 이내 로컬 저장. AI 분석은 비동기 |
| 키워드 검색 | 일기 1만 건 기준 100ms~300ms 내 1차 결과 |
| AI 자연어 검색 | 로컬 모델 사용 시 진행 상태 표시, 결과 캐시 |
| 전광판 | CPU 1% 이하 목표, 애니메이션 FPS 제한 가능 |
| Idle 상태 | AI 모델 미사용 시 VRAM 점유 없음이 원칙 |
| 앱 종료 | Ollama 모델 unload 및 sidecar 종료 확인 |
| 백업  | 실패해도 로컬 데이터 손상 없음 |
| 메모리 | AI 미사용 메인 앱 RAM 150MB~300MB 목표 |
| Keyword API | /keywords 응답 일기 1만 건 기준 500ms 이내 (집계 캐시 활용) |

## 11. 접근성 요구사항

* 고대비 다크 테마를 설정 깊은 곳이 아니라 초기 설정과 상단 접근성 메뉴에 제공한다.
* 모든 버튼, 이미지, 카드, 입력창에 접근 가능한 이름 또는 aria-label을 제공한다.
* 키보드만으로 일기 작성, 저장, 검색, 설정, 전광판 제어가 가능해야 한다.
* 포커스 위치는 항상 시각적으로 명확해야 한다.
* 폰트 크기, 자간, 행간, 문단 간격을 사용자가 조절할 수 있어야 한다.
* 최소 폰트 16px, 터치/클릭 타겟 50px 기준을 따른다. (SVIL 접근성 표준)
* NVDA, Windows Narrator 기준 기본 화면을 테스트한다.
* 400% 확대에서도 주요 작성/검색/설정 흐름이 깨지지 않아야 한다.
* 움직이는 전광판에는 애니메이션 줄이기 옵션을 제공한다.

## 12. 백업, 복원, 오프라인 정책

### 12.1 백업

| 기능  | Basic | Premium |
|-----|-------|---------|
| 수동 로컬 백업 | 지원    | 지원      |
| 자동 로컬 백업 | 지원    | 지원      |
| Railway 클라우드 백업 | 미지원   | 지원      |
| 백업 암호화 | 지원    | 지원      |
| 복원 미리보기 | 지원    | 지원      |
| 백업 버전 목록 | 로컬 파일 기준 | 클라우드 목록 포함 |

### 12.2 오프라인 동작

| 기능  | 오프라인 가능 여부 |
|-----|------------|
| 일기 작성/수정/삭제 | 가능         |
| 키워드 검색/FTS5 | 가능         |
| 벡터 검색 | 임베딩이 존재하면 가능 |
| 로컬 AI 분석 | Ollama 설치 및 모델 존재 시 가능 |
| DeepSeek 기능 | 불가         |
| 운세/뉴스 최신화 | 불가. 캐시 표시 가능 |
| 전광판 | 가능         |
| Premium 기능(라이선스 캐시 기반) | 가능 (클라우드 백업 제외) |
| 클라우드 백업 | 불가. 온라인 시 재시도 |
| Keyword API (localhost) | 가능 (인터넷 불필요) |

## 13. KPI와 성공 기준

측정 원칙: 모든 사용 지표는 **옵트인 익명 텔레메트리**로만 수집한다. 옵트인하지 않은 사용자의 데이터는 어떤 형태로도 서버에 전송하지 않으며, 이 경우 릴리즈 다운로드 수와 자발적 사용자 리포트(GitHub Discussions/설문)로 보완 측정한다.

| 지표  | v1.0 RC 목표 | 측정 방법 |
|-----|------------|-------|
| 다운로드 | 100회       | 릴리즈 다운로드 수 |
| 활성 사용자 | 20명        | 옵트인 텔레메트리 + 사용자 리포트 |
| 일기 작성률 | 설치 사용자 중 60%가 1회 이상 작성 | 옵트인 로컬 익명 통계 |
| 재방문 | D7 30% 이상 목표 | 옵트인 익명 지표 |
| 검색 사용률 | 작성자 중 40% 이상 검색 사용 | 옵트인 로컬 이벤트 카운트 |
| AI 사용률 | 작성자 중 50% 이상 AI 분석 사용 | 옵트인 로컬 이벤트 카운트 |
| 크래시 | 주요 흐름 크래시 0건 목표 | 로그/이슈 |
| 데이터 손실 | 0건         | 이슈/복구 로그 |

## 14. 테스트 및 Acceptance Criteria

### 14.1 핵심 수락 기준

| 기능  | Given | When | Then |
|-----|-------|------|------|
| 일기 저장 | 사용자가 4섹션 중 하나 이상 입력했다 | 저장 버튼을 누른다 | SQLite에 저장되고 목록에 표시된다 |
| AI 분석 | 일기가 저장되어 있다 | AI 분석을 실행한다 | 요약, 감정, 키워드가 JSON으로 저장된다 |
| AI-off 모드 | AI Provider가 없음으로 설정됐다 | 일기 작성/검색/전광판을 사용한다 | 모든 흐름이 오류 없이 동작한다 |
| 전광판 | 전광판 표시 항목이 있다 | 전광판을 켠다 | 작업 방해 없이 텍스트가 흐른다 |
| 모델 종료 | AI 모델이 실행 중이다 | 앱을 종료한다 | sidecar가 종료되고 모델 unload가 호출된다 |
| 키워드 검색 | 일기에 양자역학 키워드가 있다 | 양자역학을 검색한다 | 관련 일기가 검색 결과에 표시된다 |
| AI 자연어 검색 | 여러 일기가 있다 | 작년에 우울했던 날을 검색한다 | 기간/감정 조건으로 관련 결과가 표시된다 |
| 딥링크 | 앱이 설치되어 있다 | txtdiary://search?keyword=양자역학 URI가 호출된다 | 검색 결과 페이지가 keyword=양자역학으로 열린다 |
| Keyword API | API가 활성화되고 페어링이 완료됐다 | GET /keywords를 호출한다 | 스키마 v1.0 JSON이 반환되고 본문이 포함되지 않는다 |
| API 인증 | 페어링 토큰 없이 접근한다 | GET /keywords를 호출한다 | 401이 반환되고 데이터가 노출되지 않는다 |
| API read-only | API가 활성화되어 있다 | POST/PUT/DELETE를 호출한다 | 405가 반환되고 데이터가 변경되지 않는다 |
| 시리얼 검증 | 사용자가 시리얼 키를 입력했다 | 검증 버튼을 누른다 | Premium 상태가 로컬에 저장된다 |
| 오프라인 Premium | 시리얼 검증이 완료된 상태에서 오프라인이다 | 앱을 실행한다 | 클라우드 백업 제외 Premium 기능이 유지된다 |
| 클라우드 백업 | Premium 검증이 완료됐다 | 백업을 실행한다 | 암호화된 백업 파일이 Railway에 업로드된다 |
| 접근성 | 마우스를 사용하지 않는다 | Tab/Enter/Esc로 조작한다 | 주요 흐름을 완료할 수 있다 |
| 정서 안전장치 | 부정 감정 반복 패턴이 감지됐다 | 메인 화면에 진입한다 | 진단 없는 지원 안내 카드가 1회 노출되고, 끌 수 있다 |

### 14.2 테스트 범위

* Unit: DB repository, keyword parser, license validator, cache policy, API response builder, deep link parser
* Integration: diary save → AI analysis → FTS index → search result / API 응답 → 스키마 검증
* E2E: onboarding → diary → AI → billboard → search → deep link 수신 → Keyword API 페어링/조회
* Performance: app start, search latency, billboard CPU, model unload, API 응답 시간
* Security: API Key encryption, backup encryption, serial token handling, Keyword API 본문 미포함/localhost 바인딩/토큰 인증 검증
* Accessibility: keyboard navigation, screen reader labels, high contrast mode
* Recovery: backup restore, failed AI call, failed cloud upload, corrupted cache, corrupted license cache

## 15. 리스크와 대응

| 리스크 | 영향  | 대응  |
|-----|-----|-----|
| 기능 범위 과다 | RC 일정 지연 | 은하수/Ghost/TXTBrain을 범위에서 제거 완료. 추가 요구는 v1.1+ parking lot으로 |
| 로컬 LLM 리소스 과다 | 사용자 불만 | 저사양 폴백 정책(5.1.1), AI-off 모드, 모델 unload |
| DeepSeek 비용 오해 | 지원 부담 | 사용자 API Key 직접 조달 정책을 온보딩에 명확히 표시 |
| 개인 기록 민감성 | 신뢰 저하 | 로컬 우선, 암호화, 외부 전송 전 동의, API 본문 제외 |
| Railway 백업 비용 | 운영 비용 증가 | Premium 전용, 용량 제한, 압축/중복 제거 |
| 검색 품질 낮음 | 핵심 가치 저하 | FTS5 + AI query rewrite + 벡터 검색의 3중 구조 |
| TXTSpace 스키마 불일치 | 생태계 연동 실패 | API 스키마 공동 소유 + schema_version 관리 |
| 접근성 미흡 | 핵심 사용자 배제 | 초기부터 WCAG 체크리스트와 테스트 포함 |

## 16. 개발 스프린트 제안

| Sprint | 목표  | 산출물 |
|--------|-----|-----|
| Sprint 0 | 프로젝트 초기화 | Tauri 앱, DB 연결, 기본 UI, 문서 구조 |
| Sprint 1 | 일기 MVP | 4섹션 CRUD, SQLite, 설정, soft delete |
| Sprint 2 | 검색 기반 | FTS5, 검색 결과 페이지, 딥링크 수신 |
| Sprint 3 | AI 로컬 | Ollama sidecar, 모델 설정, 요약/키워드/감정, AI-off 모드 |
| Sprint 4 | 전광판 | 윈도우 속성, Alt+클릭, 표시 큐, 접근성 옵션 |
| Sprint 5 | DeepSeek 기능 | API Key 저장, 운세, 명상, 뉴스, 자연어 검색 |
| Sprint 6 | 벡터 검색 + Keyword API | 임베딩 생성/저장, 유사 일기 검색, Keyword API(read-only) + 페어링 |
| Sprint 7 | 라이선스/백업 | Railway 시리얼 검증, Premium 해금, 오프라인 캐시, 암호화 백업 |
| Sprint 8 | 접근성/성능/RC 패키징 | WCAG 점검, 모델 unload, 정서 안전장치, 설치 패키지, 릴리즈 노트 |

* v2.0 대비: 스프린트 10개 → 8개. 은하수(구 Sprint 5), Ghost/TXTBrain(구 Sprint 9) 제거.

## 17. AI 개발도구용 구현 지시 요약

```
Project: TXTDiary v1.0 RC (formerly MindBoard)
Goal: Implement a local-first AI diary app with diary CRUD, AI analysis, desktop billboard, three-tier search, Railway license/backup, and a local read-only Keyword API for the companion app TXTSpace.

Hard Requirements:
- Local SQLite encrypted storage.
- Diary sections: plan, did, reflection, gratitude.
- Search must support FTS5 keyword search, AI natural language search, and vector search.
- Register txtdiary://search?keyword= URI scheme; deep link opens the search result page.
- Keyword API: read-only HTTP server bound to 127.0.0.1 only, pairing-token auth, GET /health + GET /keywords returning schema v1.0 JSON (keywords, frequency, emotion, cooccurrence). Responses MUST NOT contain diary body text. No write methods exist. Default OFF; user consent dialog on first pairing.
- Local AI models: qwen3:8b and gemma4:12b via Ollama. App must be fully usable in AI-off mode.
- DeepSeek API is user-supplied; never charge server-side AI by default.
- On app exit, unload Ollama model and release RAM/VRAM.
- Railway backend verifies serial key as token and supports encrypted premium backup.
- Verified license persists offline via local cache (cloud backup excluded).
- No blog integration in v1.0 RC (Ghost planned for v1.1). No direct TXTBrain integration (via TXTSpace in v2.0).
- Accessibility: high contrast, keyboard navigation, screen reader labels, scalable text, min font 16px, touch target 50px.

Definition of Done:
- Core loop (write -> analyze -> re-surface via billboard/search) works end-to-end.
- No known diary data loss bug.
- Main app runs fully without internet and without AI.
- AI failures do not block diary writing.
- Keyword API validates against schema v1.0, contains no body text, rejects unauthenticated and write requests.
- Release package includes README, privacy note, license/backup policy, and known limitations.
```

## 18. 부록: 용어 정의

| 용어  | 정의  |
|-----|-----|
| TXTDiary | 개인 기록 중심 앱. 생각, 감정, 감사, 계획을 기록한다. (구 MindBoard) |
| TXTSpace | 키워드 우주 맵/마인드맵 앱. TXTDiary·TXTBrain의 키워드를 시각화하고 연결한다. |
| TXTBrain | 문서 자료 중심 앱. PDF/Markdown/논문/자료를 구조화한다. |
| Profile Engine | 향후 개발될 개인 프로파일링 앱. 장기 사고 흐름과 성장 패턴을 분석한다. |
| Keyword API | TXTDiary가 TXTSpace에 제공하는 로컬 read-only 키워드 데이터 API(127.0.0.1 전용). 본문 미포함. |
| 검색 결과 페이지 | 키워드 검색, AI 검색, TXTSpace 딥링크가 모두 도착하는 공통 화면. |
| 전광판 | 바탕화면 위에 생각/명언/감사 문장을 흐르게 표시하는 overlay UI. |
| Local-first | 데이터의 기본 저장과 주요 기능이 사용자 기기에서 동작하는 원칙. |
| RC  | Release Candidate. 핵심 루프가 완결성 있게 동작하는 출시 후보 버전. |

## 19. 미해결 결정사항

**착수 전 확정 필수 (blocking):**

* Windows 전광판 구현에서 Tauri window API와 Win32 확장 적용 방식 확정. (Sprint 4 전)
* SQLite 암호화 구현체(SQLCipher 등) 최종 확정. (Sprint 1 전)
* v1.0 RC 설치 패키징 방식(msi/nsis/portable) 확정. (Sprint 8 전이지만 서명/배포 준비 때문에 조기 결정 권장)

**확정 완료 (2026-07-08):**

* ~~Keyword 데이터 전달 방식~~ → **로컬 read-only API로 확정.** 수동 JSON export는 Could(폴백/백업용).
* ~~TXTSpace 개발 시점~~ → **TXTDiary RC 완료 후 순차 개발로 확정.**
* ~~MindBoard 브랜드~~ → **완전 폐기, TXT 패밀리(TXTDiary/TXTSpace/TXTBrain)로 통일 확정.**

**개발 중 확정 가능 (non-blocking):**

* 시리얼 키 발급에 사용할 실제 결제/후원 플랫폼 확정.
* 임베딩 모델과 vector 저장 형식 최종 확정.
* 옵트인 텔레메트리 수집 항목과 수신 엔드포인트 설계.
* 정서 안전장치 안내 문구/지역별 자원 목록 확정.

---

## 부록 A. TXT 패밀리 통합 준비 (2026-07-09 추가, 유미)

> 본 부록은 TXTDiary v1.0 RC 구현을 **변경하지 않는다.** TXT 패밀리 마스터(공통 프로토콜)와 TXTAIMemory·TXTSpace가 정의된 뒤, TXTDiary가 패밀리 통합 관점에서 어디에 서 있는지 정리하고 후속 버전의 준비사항만 명시한다. 상위 문서: **TXT 패밀리 마스터 — 목적과 공통 프로토콜**.

### A.1 패밀리 내 위치 재확인

TXTDiary는 맥락화 3축 중 **개인 기억·감정·경험**을 담당하는 첫 번째 축이자, **공통 Keyword API schema v1.0의 원본 정의처**다. TXTBrain(지식)·TXTAIMemory(AI 대화)는 이 규격을 상속하고, TXTSpace는 세 소스를 모두 소비한다. 즉 TXTDiary의 §8.3 Keyword API가 사실상 패밀리 표준의 근거 문서다.

### A.2 마스터 프로토콜과의 정합성 (이미 충족)

RC 구현이 이미 마스터 §3 공통 프로토콜을 충족한다. 별도 작업이 필요 없는 항목:

* localhost 전용(127.0.0.1), read-only(GET만, 쓰기 405), 페어링 토큰 인증, 본문 미포함, schema_version 포함, 기본 OFF — 모두 구현됨.
* 딥링크 `txtdiary://search?keyword=` 등록 — 구현됨.
* 응답 스키마(keywords/frequency/emotion/cooccurrence) — 마스터 §3.2와 동일.

결론: TXTDiary는 **통합 대응을 위해 지금 당장 고칠 것이 없다.** 아래는 후속 버전(v1.5~v2.0)에서 다룰 준비사항이다.

### A.3 후속 버전 통합 준비사항

| 항목  | 현재(RC) | 통합 목표 | 대상 버전 |
|-----|--------|-------|-------|
| `source` 필드 값 | "txtdiary" 고정 반환 | 변경 없음(이미 정합) | —     |
| 확장 필드 정책 | 코어 필드만 반환 | 소비 측이 모르는 필드 무시 원칙에 맞춰, 향후 필드 추가 시 schema_version 마이너 증가 | v1.5  |
| SSE push | polling 전제 | 변경 이벤트 push(SSE)로 TXTSpace 실시간 갱신 지원 | v1.5  |
| TXTBrain 직접 연결 | 없음(TXTSpace 경유) | 유지 — TXTDiary는 TXTBrain과 직접 연동하지 않는다. 연결의 발견은 TXTSpace/TXTMyWorld 몫 | 원칙 고정 |
| TXTAIMemory 관계 | 없음     | **직접 연동 없음.** 두 앱은 형제일 뿐, 교차는 TXTSpace 지도에서만. TXTDiary는 일기, TXTAIMemory는 AI 대화로 도메인이 분리됨 | 원칙 고정 |
| TXTDrop 관계 | 없음     | TXTDrop→TXTBrain(문서)·TXTDrop→TXTAIMemory(AI 맥락) 경로가 주력. TXTDiary는 TXTDrop 캡처 대상이 아님(일기는 직접 작성) | 원칙 고정 |

### A.4 경계 원칙 (혼동 방지)

통합 논의에서 TXTDiary의 범위가 번지지 않도록 못 박는다.

* TXTDiary는 **개인 기억**만 소유한다. 문서는 TXTBrain/SAC, AI 대화는 TXTAIMemory 소관.
* TXTDiary는 다른 앱의 데이터를 **읽지 않는다.** 오직 자기 키워드를 read-only로 **제공**할 뿐이다.
* 소스 간 연결·통합 뷰·새 연결 생성은 TXTDiary가 하지 않는다. 그건 TXTSpace(보기)와 TXTMyWorld(잇기·만들기)의 일이다.
* 이 원칙 덕분에 TXTDiary RC는 패밀리가 커져도 **그대로 안정적**이다. 통합의 복잡성은 상위 레이어가 흡수한다.

### A.5 변경 이력 (부록)

* 부록 A v1.0 (2026-07-09, 유미): TXT 패밀리 통합 준비 관점 추가. RC 구현 무변경, 마스터 프로토콜 정합성 확인, 후속 버전 준비사항 및 경계 원칙 명시.
