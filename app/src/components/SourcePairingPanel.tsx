// src/components/SourcePairingPanel.tsx — 소스 페어링 UI. Onboarding(S0)과 Settings(S6)가 공유.

import { useEffect, useState } from "react";
import { api, SOURCE_IDS, SOURCE_LABELS, SourceIdStr, SourceStatusDto } from "../api";

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
      onToast(`소스 목록을 불러오지 못했습니다: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handlePair(e: React.FormEvent) {
    e.preventDefault();
    try {
      await api.pairSource(form.source, form.baseUrl, form.token);
      onToast(`${SOURCE_LABELS[form.source]} 연결을 저장했습니다.`);
      setForm((f) => ({ ...f, token: "" }));
      await load();
      onChanged?.();
    } catch (e) {
      onToast(`연결 실패: ${e}`);
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
      onToast(`${source}: 키워드 ${result.keyword_count}개, 벡터 ${result.vector_count}개 동기화 완료`);
    } else {
      onToast(`${source} 동기화 실패: ${result.message ?? result.status}`);
    }
    await load();
    onChanged?.();
  }

  async function handleSyncAll() {
    const results = await api.syncAll();
    const ok = results.filter((r) => r.status === "ok").length;
    onToast(`${ok}/${results.length}개 소스 동기화 완료`);
    await load();
    onChanged?.();
  }

  return (
    <div className="stack">
      <form className="panel stack" onSubmit={handlePair} aria-label="새 소스 연결">
        <h3>소스 연결</h3>
        <div className="row">
          <div>
            <label htmlFor="pair-source">소스</label>
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
                  {SOURCE_LABELS[s]}
                </option>
              ))}
            </select>
          </div>
          <div style={{ flex: 1 }}>
            <label htmlFor="pair-url">주소 (localhost 전용)</label>
            <input id="pair-url" type="url" value={form.baseUrl} onChange={(e) => setForm((f) => ({ ...f, baseUrl: e.target.value }))} required />
          </div>
        </div>
        <div>
          <label htmlFor="pair-token">페어링 토큰 (소스 앱 승인 화면에서 발급)</label>
          <input id="pair-token" type="password" value={form.token} onChange={(e) => setForm((f) => ({ ...f, token: e.target.value }))} />
          <p className="field-hint">토큰은 이 기기의 OS 보안 저장소에만 저장됩니다. 비워두면 인증 없는 로컬 소스로 취급합니다.</p>
        </div>
        <div className="row">
          <button type="submit" className="primary">
            연결 저장
          </button>
        </div>
      </form>

      <div className="panel">
        <div className="toolbar">
          <h3 style={{ margin: 0 }}>연결된 소스</h3>
          <button onClick={handleSyncAll} disabled={sources.length === 0}>
            전체 동기화
          </button>
        </div>
        {loading && <p>불러오는 중…</p>}
        {!loading && sources.length === 0 && <p className="field-hint">아직 연결된 소스가 없습니다. Diary 하나만 연결해도 시작할 수 있습니다.</p>}
        <ul className="card-list">
          {sources.map((s) => (
            <li key={s.source} className="card-item">
              <div className="row" style={{ justifyContent: "space-between" }}>
                <strong>{SOURCE_LABELS[(s.source as SourceIdStr) ?? "txtdiary"] ?? s.source}</strong>
                <span className={`badge ${s.paired ? "status-ok" : ""}`}>{s.paired ? "페어링됨" : "미페어링"}</span>
              </div>
              <p className="field-hint">
                {s.base_url} · 마지막 동기화: {s.last_synced_at ? new Date(s.last_synced_at).toLocaleString("ko-KR") : "없음"}
              </p>
              <div className="row">
                <button onClick={() => handleSync(s.source, s.base_url)}>지금 동기화</button>
                <button className="danger" onClick={() => handleUnpair(s.source)}>
                  연결 해제
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
