// src/screens/Settings.tsx — S6 설정. SVIL 표준 §2.1: 화면(언어·글자크기·글꼴) + 발견 엔진·연동·버전/히스토리.

import { useEffect, useState } from "react";
import { AppInfo, EmbedderStatus, SettingsDto, api } from "../api";
import { SourcePairingPanel } from "../components/SourcePairingPanel";
import { LANG_OPTIONS, useI18n } from "../lib/i18n";
import { FONT_FAMILY_OPTIONS, FontScale, usePrefs } from "../lib/prefs";

interface Props {
  onToast: (msg: string) => void;
}

const SIZES: FontScale[] = ["S", "M", "L"];

export function Settings({ onToast }: Props) {
  const { t, lang, setLang } = useI18n();
  const { scale, setScale, fontFamily, setFontFamily } = usePrefs();
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [embedder, setEmbedder] = useState<EmbedderStatus | null>(null);
  const [saving, setSaving] = useState(false);

  const fallbackSettings: SettingsDto = {
    discovery_config: {
      weights: { w_s: 0.6, w_t: 0.2, w_f: 0.2 },
      bridge_sim_cut: 0.6,
      gap_sim_cut: 0.5,
      gap_min_freq: 3,
      cluster_sim_cut: 0.7,
      cluster_min_size: 3,
      drift_min_delta: 0.4,
      knn_k: 10,
      weak_signal_score: 0.5,
    },
    ollama_base_url: "http://127.0.0.1:11434",
    aimemory_endpoint: "http://127.0.0.1:8765/mcp/tools/memory_write",
  };

  async function loadAll() {
    // 각 호출을 개별 처리 — 하나가 실패해도 나머지는 정상 표시되고, 설정 화면이 무한 로딩에 갇히지 않는다.
    try {
      setSettings(await api.getSettings());
    } catch (e) {
      onToast(t("settings.loadFail", { e: String(e) }));
      setSettings(fallbackSettings);
    }
    try {
      setAppInfo(await api.getAppInfo());
    } catch {
      // 버전 정보 실패는 무해하게 무시
    }
    try {
      setEmbedder(await api.getEmbedderStatus());
    } catch {
      // 임베더 상태 실패도 무해하게 무시
    }
  }

  useEffect(() => {
    loadAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    try {
      await api.setSettings(settings);
      onToast(t("settings.saveOk"));
    } catch (e) {
      onToast(t("settings.saveFail", { e: String(e) }));
    } finally {
      setSaving(false);
    }
  }

  async function handleSeedDemo() {
    const n = await api.seedDemoData();
    onToast(t("settings.demoAdded", { n }));
  }

  if (!settings) {
    return <p>{t("common.loading")}</p>;
  }

  return (
    <section aria-labelledby="settings-title" className="stack">
      <h2 id="settings-title">{t("settings.title")}</h2>

      {/* 화면 — SVIL 표준: 언어·글자 크기·글꼴 (설정 메뉴에 항상 표시) */}
      <div className="panel stack">
        <h3>{t("settings.display")}</h3>

        <div className="toolbar">
          <label htmlFor="lang-select" style={{ minWidth: 140 }}>
            {t("settings.language")}
          </label>
          <select id="lang-select" value={lang} onChange={(e) => setLang(e.target.value as typeof lang)}>
            {LANG_OPTIONS.map((o) => (
              <option key={o.id} value={o.id}>
                {o.label}
              </option>
            ))}
          </select>
        </div>

        <div className="toolbar" role="group" aria-label={t("settings.fontSize")}>
          <span style={{ minWidth: 140 }}>{t("settings.fontSize")}</span>
          {SIZES.map((s) => (
            <button key={s} className={scale === s ? "primary" : ""} aria-pressed={scale === s} onClick={() => setScale(s)}>
              {t(`settings.size.${s}`)}
            </button>
          ))}
        </div>

        <div className="toolbar" role="group" aria-label={t("settings.font")}>
          <span style={{ minWidth: 140, alignSelf: "flex-start", paddingTop: 12 }}>{t("settings.font")}</span>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 10, flex: 1 }}>
            {FONT_FAMILY_OPTIONS.map((o) => (
              <button
                key={o.id}
                className={fontFamily === o.id ? "primary" : ""}
                aria-pressed={fontFamily === o.id}
                style={{ fontFamily: o.css }}
                onClick={() => setFontFamily(o.id)}
              >
                {o.label}
                {o.isDefault ? ` ${t("settings.fontDefaultTag")}` : ""}
              </button>
            ))}
          </div>
        </div>
      </div>

      <SourcePairingPanel onToast={onToast} />

      <div className="panel stack">
        <h3>{t("settings.engineTitle")}</h3>
        <p className="field-hint">
          {t("settings.embedderModel", { model: embedder?.model ?? "…" })}{" "}
          {embedder && !embedder.is_real_model && t("settings.embedderFallback")} · {t("settings.embedderDim", { dim: embedder?.dim ?? "-" })}
        </p>
        <div className="row">
          <div>
            <label htmlFor="w-s">{t("settings.weightSemantic", { v: settings.discovery_config.weights.w_s.toFixed(2) })}</label>
            <input
              id="w-s"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.discovery_config.weights.w_s}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  discovery_config: { ...settings.discovery_config, weights: { ...settings.discovery_config.weights, w_s: Number(e.target.value) } },
                })
              }
            />
          </div>
          <div>
            <label htmlFor="w-t">{t("settings.weightTemporal", { v: settings.discovery_config.weights.w_t.toFixed(2) })}</label>
            <input
              id="w-t"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.discovery_config.weights.w_t}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  discovery_config: { ...settings.discovery_config, weights: { ...settings.discovery_config.weights, w_t: Number(e.target.value) } },
                })
              }
            />
          </div>
          <div>
            <label htmlFor="w-f">{t("settings.weightFrequency", { v: settings.discovery_config.weights.w_f.toFixed(2) })}</label>
            <input
              id="w-f"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.discovery_config.weights.w_f}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  discovery_config: { ...settings.discovery_config, weights: { ...settings.discovery_config.weights, w_f: Number(e.target.value) } },
                })
              }
            />
          </div>
        </div>
        <div className="row">
          <div>
            <label htmlFor="bridge-cut">{t("settings.bridgeCut", { v: settings.discovery_config.bridge_sim_cut.toFixed(2) })}</label>
            <input
              id="bridge-cut"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.discovery_config.bridge_sim_cut}
              onChange={(e) => setSettings({ ...settings, discovery_config: { ...settings.discovery_config, bridge_sim_cut: Number(e.target.value) } })}
            />
          </div>
          <div>
            <label htmlFor="cluster-min">{t("settings.clusterMin", { v: settings.discovery_config.cluster_min_size })}</label>
            <input
              id="cluster-min"
              type="number"
              min={2}
              max={10}
              value={settings.discovery_config.cluster_min_size}
              onChange={(e) => setSettings({ ...settings, discovery_config: { ...settings.discovery_config, cluster_min_size: Number(e.target.value) } })}
            />
          </div>
        </div>
        <div className="row">
          <button onClick={handleSeedDemo}>{t("settings.addDemoData")}</button>
        </div>
      </div>

      <div className="panel stack">
        <h3>{t("settings.integrationTitle")}</h3>
        <div>
          <label htmlFor="ollama-url">{t("settings.ollamaUrl")}</label>
          <input id="ollama-url" type="url" value={settings.ollama_base_url} onChange={(e) => setSettings({ ...settings, ollama_base_url: e.target.value })} />
        </div>
        <div>
          <label htmlFor="aimemory-endpoint">{t("settings.aimemoryEndpoint")}</label>
          <input
            id="aimemory-endpoint"
            type="url"
            value={settings.aimemory_endpoint}
            onChange={(e) => setSettings({ ...settings, aimemory_endpoint: e.target.value })}
          />
          <p className="field-hint">{t("settings.aimemoryHint")}</p>
        </div>
      </div>

      <div className="row">
        <button className="primary" onClick={handleSave} disabled={saving}>
          {saving ? t("settings.savingSettings") : t("settings.saveSettings")}
        </button>
      </div>

      <div className="panel stack">
        <h3>{t("settings.infoTitle")}</h3>
        <p className="mono">{t("settings.versionLabel", { v: appInfo?.version ?? "…" })}</p>
        <h4>{t("settings.historyTitle")}</h4>
        <ul>
          {(appInfo?.history ?? [])
            .slice()
            .reverse()
            .map((h) => (
              <li key={h.version}>
                <strong className="mono">v{h.version}</strong> <span className="mono">({h.date})</span> — {h.summary}
              </li>
            ))}
        </ul>
      </div>
    </section>
  );
}
