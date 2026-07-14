// src/lib/prefs.ts — 화면 설정(글꼴·글자 크기) 영속화 + 부트스트랩 (SVIL 표준 §2.1)
// CSS 변수 + localStorage. 폰트 8종은 전부 로컬 번들(오프라인 동작), "고딕"만 시스템 폴백.
import { useEffect, useState } from "react";

export type FontScale = "S" | "M" | "L";

// SVIL 표준: 작음 16px / 보통 18px(기본) / 큼 20px — 본문 기준.
const SCALE_PX: Record<FontScale, string> = {
  S: "16px",
  M: "18px",
  L: "20px",
};

export type AppFontFamily =
  | "kyobo-handwriting-2019"
  | "gothic"
  | "nanum-gothic"
  | "line-seed"
  | "gowun-dodum"
  | "cafe24-dongdong"
  | "tmoney-round"
  | "recipe-korea";

// label은 폰트 고유명(번역하지 않음). isDefault 항목엔 UI에서 "(기본)" 태그를 붙인다.
export const FONT_FAMILY_OPTIONS: Array<{
  id: AppFontFamily;
  label: string;
  css: string;
  isDefault?: boolean;
}> = [
  {
    id: "kyobo-handwriting-2019",
    label: "교보손글씨2019",
    css: "'KyoboHandwriting2019', 'Pretendard', 'Malgun Gothic', sans-serif",
    isDefault: true,
  },
  { id: "gothic", label: "고딕", css: "'Pretendard', 'Malgun Gothic', sans-serif" },
  { id: "nanum-gothic", label: "나눔고딕", css: "'NanumGothic', 'Malgun Gothic', sans-serif" },
  { id: "line-seed", label: "라인시드체", css: "'LINE Seed Sans KR', 'Malgun Gothic', sans-serif" },
  { id: "gowun-dodum", label: "고운돋움체", css: "'Gowun Dodum', 'Malgun Gothic', sans-serif" },
  { id: "cafe24-dongdong", label: "카페24동동체", css: "'Cafe24Dongdong', 'Malgun Gothic', sans-serif" },
  { id: "tmoney-round", label: "티머니둥근바람체", css: "'TmoneyRoundWind', 'Malgun Gothic', sans-serif" },
  { id: "recipe-korea", label: "레코체", css: "'Recipekorea', 'Malgun Gothic', sans-serif" },
];

const LS_SCALE = "txtmyworld_font_scale";
const LS_FONT = "txtmyworld_font_family";

function isFontScale(v: string | null): v is FontScale {
  return v === "S" || v === "M" || v === "L";
}

function isFontFamily(v: string | null): v is AppFontFamily {
  return FONT_FAMILY_OPTIONS.some((o) => o.id === v);
}

function fontCss(id: AppFontFamily): string {
  return FONT_FAMILY_OPTIONS.find((o) => o.id === id)?.css ?? FONT_FAMILY_OPTIONS[0].css;
}

function applyScale(scale: FontScale) {
  document.documentElement.style.fontSize = SCALE_PX[scale];
}

function applyFont(id: AppFontFamily) {
  document.documentElement.style.setProperty("--app-font-family", fontCss(id));
}

/** 앱 부트 시 저장된 화면 설정을 document에 적용 (렌더 전에 호출 — 깜빡임 방지) */
export function bootstrapPrefs() {
  const scale = localStorage.getItem(LS_SCALE);
  applyScale(isFontScale(scale) ? scale : "M");
  const font = localStorage.getItem(LS_FONT);
  applyFont(isFontFamily(font) ? font : "kyobo-handwriting-2019");
}

export function usePrefs() {
  const [scale, setScale] = useState<FontScale>(() => {
    const s = localStorage.getItem(LS_SCALE);
    return isFontScale(s) ? s : "M";
  });
  const [fontFamily, setFontFamily] = useState<AppFontFamily>(() => {
    const f = localStorage.getItem(LS_FONT);
    return isFontFamily(f) ? f : "kyobo-handwriting-2019";
  });

  useEffect(() => {
    applyScale(scale);
    localStorage.setItem(LS_SCALE, scale);
  }, [scale]);

  useEffect(() => {
    applyFont(fontFamily);
    localStorage.setItem(LS_FONT, fontFamily);
  }, [fontFamily]);

  return { scale, setScale, fontFamily, setFontFamily };
}
