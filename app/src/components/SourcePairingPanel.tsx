// src/components/SourcePairingPanel.tsx — 소스 페어링 UI. Onboarding(S0)과 Settings(S6)가 공유.

import { useEffect, useState } from "react";
import { api, SOURCE_IDS, SourceIdStr, SourceStatusDto } from "../api";
import { useI18n } from "../lib/i18n";

const DEFAULT_PORTS: Record<SourceIdStr, string> = {
  txtdiary: "http://127.0.0.1:4001",
  txtbrain: "http://127.0.0.1:4002",
  txtaimemory: "http://127.0.0.1:4003",
};

interface Props {
  onToast: (msg: string) => void;
  onChanged?: () => void;
}

export function SourcePairingPanel({ onToast, onChanged }: Props) {
  const { t } = useI18n();
  const [sources, setSources] = useState<SourceStatusDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [form, setForm] = useState<{ source: SourceIdStr; baseUrl: string; token: string }>({
    source: "txtdiary",
    baseUrl: DEFAULT_PORTS.txtdiary,
    token: "",
  });

  async function load() {
    setLoading(true);
    try {
      setSources(await api.listSources());
    } catch (e) {
      onToast(t("source.listFail", { e: String(e) }));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handlePair(e: React.FormEvent) {
    e.preventDefault();
    try {
      await api.pairSource(form.source, form.baseUrl, form.token);
      onToast(t("source.savedToast", { label: t(`source.name.${form.source}`) }));
      setForm((f) => ({ ...f, token: "" }));
      await load();
      onChanged?.();
    } catch (e) {
      onToast(t("source.pairFail", { e: String(e) }));
    }
  }

  async function handleUnpair(source: string) {
    await api.unpairSource(source);
    await load();
    onChanged?.();
  }

  async function handleSync(source: string, baseUrl: string) {
    const result = await api.syncSource(source, baseUrl);
    if (result.status === "ok") {
      onToast(t("source.syncOkToast", { source, kw: result.keyword_count, vec: result.vector_count }));
    } else {
      onToast(t("source.syncFailToast", { source, msg: result.message ?? result.status }));
    }
    await load();
    onChanged?.();
  }

  async function handleSyncAll() {
    const results = await api.syncAll();
    const ok = results.filter((r) => r.status === "ok").length;
    onToast(t("source.syncAllToast", { ok, total: results.length }));
    await load();
    onChanged?.();
  }

  return (
    <div className="stack">
      <form className="panel stack" onSubmit={handlePair} aria-label={t("source.title")}>
        <h3>{t("source.title")}</h3>
        <div className="row">
          <div>
            <label htmlFor="pair-source">{t("source.selectLabel")}</label>
            <select
              id="pair-source"
              value={form.source}
              onChange={(e) => {
                const source = e.target.value as SourceIdStr;
                setForm((f) => ({ ...f, source, baseUrl: DEFAULT_PORTS[source] }));
              }}
            >
              {SOURCE_IDS.map((s) => (
                <option key={s} value={s}>
                  {t(`source.name.${s}`)}
                </option>
              ))}
            </select>
          </div>
          <div style={{ flex: 1 }}>
            <label htmlFor="pair-url">{t("source.urlLabel")}</label>
            <input id="pair-url" type="url" value={form.baseUrl} onChange={(e) => setForm((f) => ({ ...f, baseUrl: e.target.value }))} required />
          </div>
        </div>
        <div>
          <label htmlFor="pair-token">{t("source.tokenLabel")}</label>
          <input id="pair-token" type="password" value={form.token} onChange={(e) => setForm((f) => ({ ...f, token: e.target.value }))} />
          <p className="field-hint">{t("source.tokenHint")}</p>
        </div>
        <div className="row">
          <button type="submit" className="primary">
            {t("source.save")}
          </button>
        </div>
      </form>

      <div className="panel">
        <div className="toolbar">
          <h3 style={{ margin: 0 }}>{t("source.connectedTitle")}</h3>
          <button onClick={handleSyncAll} disabled={sources.length === 0}>
            {t("source.syncAll")}
          </button>
        </div>
        {loading && <p>{t("common.loading")}</p>}
        {!loading && sources.length === 0 && <p className="field-hint">{t("source.empty")}</p>}
        <ul className="card-list">
          {sources.map((s) => (
            <li key={s.source} className="card-item">
              <div className="row" style={{ justifyContent: "space-between" }}>
                <strong>{t(`source.name.${(s.source as SourceIdStr) ?? "txtdiary"}`)}</strong>
                <span className={`badge ${s.paired ? "status-ok" : ""}`}>{s.paired ? t("source.paired") : t("source.unpaired")}</span>
              </div>
              <p className="field-hint mono">
                {s.base_url} · {t("source.lastSync", { v: s.last_synced_at ? new Date(s.last_synced_at).toLocaleString() : t("source.never") })}
              </p>
              <div className="row">
                <button onClick={() => handleSync(s.source, s.base_url)}>{t("source.syncNow")}</button>
                <button className="danger" onClick={() => handleUnpair(s.source)}>
                  {t("source.unpair")}
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
