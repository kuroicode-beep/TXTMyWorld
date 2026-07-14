import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { bootstrapPrefs } from "./lib/prefs";
import { bootstrapLang } from "./lib/i18n";

// 저장된 화면 설정(글꼴·크기·언어)을 렌더 전에 적용해 깜빡임을 막는다
bootstrapPrefs();
bootstrapLang();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
