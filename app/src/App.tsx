// src/App.tsx — 앱 셸: 탭 내비게이션, 온보딩 게이트, 접근성 상태 적용, 토스트.

import { useEffect, useState } from "react";
import "./theme.css";
import { AccessibilityDto, api } from "./api";
import { Onboarding } from "./screens/Onboarding";
import { Feed } from "./screens/Feed";
import { Explore } from "./screens/Explore";
import { Archive } from "./screens/Archive";
import { Settings } from "./screens/Settings";

type Tab = "onboarding" | "feed" | "explore" | "archive" | "settings";

const ONBOARDED_KEY = "txtmyworld_onboarded";

const TABS: { id: Exclude<Tab, "onboarding">; label: string }[] = [
  { id: "feed", label: "발견" },
  { id: "explore", label: "탐색" },
  { id: "archive", label: "보관함" },
  { id: "settings", label: "설정" },
];

export default function App() {
  const [tab, setTab] = useState<Tab>(() => (localStorage.getItem(ONBOARDED_KEY) ? "feed" : "onboarding"));
  const [version, setVersion] = useState("0.1.0");
  const [toast, setToast] = useState<string | null>(null);
  const [accessibility, setAccessibility] = useState<AccessibilityDto>({
    high_contrast: true,
    font_scale: 1,
    reduce_motion: false,
  });

  useEffect(() => {
    api.getAppInfo().then((info) => setVersion(info.version)).catch(() => {});
    api
      .getSettings()
      .then((s) => setAccessibility(s.accessibility))
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 4500);
    return () => clearTimeout(t);
  }, [toast]);

  function handleOnboardingDone() {
    localStorage.setItem(ONBOARDED_KEY, "1");
    setTab("feed");
  }

  const shellClass = ["app-shell", accessibility.high_contrast ? "high-contrast" : "", accessibility.reduce_motion ? "reduce-motion" : ""]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={shellClass} style={{ fontSize: `${16 * accessibility.font_scale}px` }}>
      <header className="app-header">
        <h1 className="app-title">TXTMyWorld</h1>
        <span className="app-version">v{version}</span>
        {tab !== "onboarding" && (
          <nav className="tabs" aria-label="주요 화면 전환">
            {TABS.map((t) => (
              <button key={t.id} aria-current={tab === t.id ? "page" : undefined} onClick={() => setTab(t.id)}>
                {t.label}
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
        {tab === "settings" && (
          <Settings onToast={setToast} accessibility={accessibility} onAccessibilityChange={setAccessibility} />
        )}
      </main>

      {toast && (
        <div className="toast" role="status" aria-live="polite">
          {toast}
        </div>
      )}
    </div>
  );
}
