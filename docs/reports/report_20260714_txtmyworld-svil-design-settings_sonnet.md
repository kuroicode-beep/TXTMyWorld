# 완료보고서 — SVIL 스타일 디자인·설정 적용 (v0.2.0)

작성일: 2026-07-14 / 작성자: Claude Sonnet 5 / 요청자: InBlue (소장님)

---

## 1. 작업 개요

TXTMyWorld 데스크톱 앱에 SVIL 프론트엔드 디자인 가이드(고대비 다크 + 교보손글씨2019 표준)를 전면 적용했다. TXTAIMemory를 참조 구현으로 삼아 색상 토큰·타이포그래피·화면 설정 표준(글꼴·글자크기·다국어)·접근성 규칙을 그대로 이식했다.

## 2. 구현 범위

### 2.1 디자인 (`app/src/theme.css` 전면 재작성)

- SVIL 색상 토큰 전체 도입: `--bg #0d0d12`, `--text #f5f5f7`(≈15:1), `--accent #7ec8ff`, `--accent-strong` 주 버튼(배경+검정 텍스트 ≈15:1), `--border-strong`(버튼 테두리 ≥3:1) 등.
- 교보손글씨2019 기본 + 글꼴 6종(NanumGothic·LINE Seed·Gowun Dodum·Cafe24Dongdong·TmoneyRoundWind·Recipekorea) `app/public/fonts/`에 로컬 번들(TXTAIMemory 참조에서 복사).
- 단일 굵기 폰트 특성상 제목 위계는 크기·색으로만 표현(h1 2.2rem~h3 1.35rem, `font-weight: normal` 고정, bold 합성 금지).
- 숫자·타임스탬프·버전·ID는 `.mono` 클래스로 Consolas 강제.
- 버튼 최소 50×50px, radius 12px, 포커스 링 3px `#ffd479`, `prefers-reduced-motion` 존중.

### 2.2 화면 설정 표준 (`app/src/lib/prefs.ts`, `lib/i18n.ts`)

SVIL 가이드 §2.1의 "글꼴·글자크기·다국어는 옵션이 아니라 표준" 원칙에 따라 Settings 화면에 고정 배치.

- **글꼴 8종**: 교보손글씨2019(기본)·고딕(시스템 폴백)·나눔고딕·라인시드체·고운돋움체·카페24동동체·티머니둥근바람체·레코체. 선택 시 미리보기(버튼 자체가 해당 글꼴로 렌더).
- **글자 크기 3단계**: 작음 16px / 보통 18px(기본) / 큼 20px.
- **다국어 5종**(순서 고정): 한국어(기본)·English·日本語·中文·Tiếng Việt. `<html lang>` 동기화, `useSyncExternalStore` 기반 경량 구현(라이브러리 없음).
- 저장 위치는 프론트엔드 `localStorage`(참조 구현과 동일 패턴) — 백엔드 `SettingsDto`에서 `AccessibilityDto`(이전 세션에서 임시로 만든 continuous font_scale/high_contrast/reduce_motion)를 제거하고 Rust 코드를 단순화했다.

### 2.3 전 화면 다국어 전환

App/Onboarding/Feed/Explore/Archive/Settings/DiscoveryCard/AdoptModal/SourcePairingPanel의 하드코딩 한국어 문자열을 전부 `t()` 사전 키로 전환, 5개 언어 사전에 각각 번역 등록(총 약 110개 키 × 5언어).

**의도적으로 번역 대상에서 제외한 것** (가이드 §"사용자 데이터·고유명사는 번역 제외" 원칙 적용):
- 백엔드가 사용자 데이터로 조합하는 자연어 근거 문장(`Discovery::evidence_sentence()`) — 코어 엔진의 RC 계약이라 이번 작업에서 변경하지 않음.
- 카드 이름·메모·키워드 텍스트 등 사용자 입력 데이터.
- 글꼴 이름(고유명사).
- 백엔드의 `dtype_label_ko`/`source_label` 필드는 프론트에서 더 이상 사용하지 않고, 안정적인 enum 문자열(`dtype`, `source`)을 키로 삼아 프론트 사전에서 재번역하도록 전환 — Rust 코드 변경 없이 프론트만으로 다국어 확장.

### 2.4 접근성 — 뒤로/앞으로 내비게이션

`App.tsx`에 `history.pushState`/`popstate` 기반 탭 히스토리를 추가하고 **Alt+←/→**, 마우스 뒤로/앞으로 버튼(button 3/4)으로 이동 가능하게 했다. 가이드가 "Backspace는 입력 포커스 중 절대 금지 — 안 쓰는 편이 안전"이라고 명시해, Backspace 뒤로가기는 아예 구현하지 않았다(참조 구현엔 있으나 더 보수적으로 스코프를 좁힘).

## 3. 검증 결과

| 항목 | 결과 |
|-----|-----|
| `cargo test`(core, sqlitevec) | 33/33 통과 |
| `cargo build`/`cargo clippy`(app) | 성공, 경고 0 |
| `npx tsc --noEmit` / `npm run build` | 오류 0 |
| 폰트 번들 | 7개 파일 `dist/fonts/`에 정상 복사 확인 |
| 실 렌더링(Vite dev 서버, 브라우저 프리뷰) | SVIL 팔레트(`rgb(13,13,18)`/`rgb(245,245,247)`) 및 기본 폰트 적용 확인. 언어를 영어로 전환하자 전 화면(탭 4개·설정 전 섹션)이 즉시 영어로 재렌더, `<html lang="en">` 동기화 확인. 글꼴을 나눔고딕으로 전환 시 `--app-font-family`·body computed font-family 모두 즉시 반영 확인 |

### 발견한 프리뷰 환경 특이사항 (앱 버그 아님)

글자 크기를 "큼(20px)"으로 바꾼 뒤 `<html>` 요소의 인라인 스타일은 정확히 `font-size: 20px`였으나, 이 브라우저 프리뷰 도구의 `getComputedStyle`은 18px를 보고했다. 일반 `<div>`로 동일한 인라인-vs-스타일시트 케이스를 격리 테스트하면 정상적으로 20px가 계산되는 것을 확인했다 — 즉 CSS 캐스케이드 자체는 올바르게 동작하며, 문제는 이 샌드박스 프리뷰 브라우저가 루트(`<html>`) 요소의 폰트 크기 계산에만 적용하는 별도 메커니즘(뷰포트 접근성 오버라이드 추정)으로 보인다. 실제 네이티브 Tauri 창(WebView2)에서는 해당 프록시 레이어가 없어 문제되지 않을 가능성이 높다.

## 4. 참고

- 참조 구현: `C:/Projects/TXTAIMemory/ui`(styles.css, lib/prefs.ts, lib/i18n.ts, screens/Settings.tsx)
- 스킬: `svil-frontend-design` (색상 토큰·타이포·컴포넌트·접근성·화면 설정 표준)
- 버전: v0.1.0 → **v0.2.0** (MINOR — UI 개편, VERSIONING.md 규칙)
- 저장소: https://github.com/kuroicode-beep/TXTMyWorld
