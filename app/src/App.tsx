// src/App.tsx — 앱 셸: 탭 내비게이션(+ 브라우저 히스토리 연동), 온보딩 게이트, 토스트.

import { useCallback, useEffect, useRef, useState } from "react";
import "./theme.css";
import { api } from "./api";
import { useI18n } from "./lib/i18n";
import { Onboarding } from "./screens/Onboarding";
import { Feed } from "./screens/Feed";
import { Explore } from "./screens/Explore";
import { Archive } from "./screens/Archive";
import { Settings } from "./screens/Settings";

type Tab = "onboarding" | "feed" | "explore" | "archive" | "settings";

const ONBOARDED_KEY = "txtmyworld_onboarded";

export default function App() {
  const { t } = useI18n();
  const [tab, setTab] = useState<Tab>(() => (localStorage.getItem(ONBOARDED_KEY) ? "feed" : "onboarding"));
  const [version, setVersion] = useState("0.1.0");
  const [toast, setToast] = useState<string | null>(null);
  const tabRef = useRef(tab);
  tabRef.current = tab;

  const TABS: { id: Exclude<Tab, "onboarding">; label: string }[] = [
    { id: "feed", label: t("nav.feed") },
    { id: "explore", label: t("nav.explore") },
    { id: "archive", label: t("nav.archive") },
    { id: "settings", label: t("nav.settings") },
  ];

  useEffect(() => {
    api.getAppInfo().then((info) => setVersion(info.version)).catch(() => {});
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 4500);
    return () => clearTimeout(timer);
  }, [toast]);

  // 뒤로/앞으로 내비게이션: history state에 현재 탭을 실어 popstate로 복원한다.
  useEffect(() => {
    history.replaceState({ tab: tabRef.current }, "");
    const onPopState = (ev: PopStateEvent) => {
      const nextTab = (ev.state?.tab as Tab | undefined) ?? "feed";
      setTab(nextTab);
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  const navigateTab = useCallback((next: Tab) => {
    if (next === tabRef.current) return;
    history.pushState({ tab: next }, "");
    setTab(next);
  }, []);

  // Alt+←/→ 및 마우스 뒤로/앞으로 버튼으로 히스토리 이동 (SVIL 표준 §4.1). Backspace는 사용하지 않는다.
  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if (ev.altKey && ev.key === "ArrowLeft") {
        ev.preventDefault();
        history.back();
      } else if (ev.altKey && ev.key === "ArrowRight") {
        ev.preventDefault();
        history.forward();
      }
    };
    const onMouse = (ev: MouseEvent) => {
      if (ev.button === 3) {
        ev.preventDefault();
        history.back();
      } else if (ev.button === 4) {
        ev.preventDefault();
        history.forward();
      }
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("mouseup", onMouse);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mouseup", onMouse);
    };
  }, []);

  function handleOnboardingDone() {
    localStorage.setItem(ONBOARDED_KEY, "1");
    navigateTab("feed");
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1 className="app-title">{t("app.title")}</h1>
        <span className="app-version">v{version}</span>
        {tab !== "onboarding" && (
          <nav className="tabs" aria-label={t("nav.aria")}>
            {TABS.map((tb) => (
              <button key={tb.id} aria-current={tab === tb.id ? "page" : undefined} onClick={() => navigateTab(tb.id)}>
                {tb.label}
              </button>
            ))}
          </nav>
        )}
      </header>

      <main className="app-body">
        {tab === "onboarding" && <Onboarding onToast={setToast} onDone={handleOnboardingDone} />}
        {tab === "feed" && <Feed onToast={setToast} />}
        {tab === "explore" && <Explore onToast={setToast} />}
        {tab === "archive" && <Archive onToast={setToast} />}
        {tab === "settings" && <Settings onToast={setToast} />}
      </main>

      {toast && (
        <div className="toast" role="status" aria-live="polite">
          {toast}
        </div>
      )}
    </div>
  );
}
