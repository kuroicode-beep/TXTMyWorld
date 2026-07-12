// src/screens/Settings.tsx — S6 설정·접근성. 엔진 파라미터, 클라우드 동의 설정, 접근성, 버전/업데이트 히스토리.

import { useEffect, useState } from "react";
import { AccessibilityDto, AppInfo, EmbedderStatus, SettingsDto, api } from "../api";
import { SourcePairingPanel } from "../components/SourcePairingPanel";

interface Props {
  onToast: (msg: string) => void;
  accessibility: AccessibilityDto;
  onAccessibilityChange: (a: AccessibilityDto) => void;
}

export function Settings({ onToast, accessibility, onAccessibilityChange }: Props) {
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
    accessibility,
  };

  async function loadAll() {
    // 각 호출을 개별 처리 — 하나가 실패해도 나머지는 정상 표시되고, 설정 화면이 무한 로딩에 갇히지 않는다.
    try {
      setSettings(await api.getSettings());
    } catch (e) {
      onToast(`설정을 불러오지 못했습니다: ${e}`);
      setSettings(fallbackSettings);
    }
    try {
      setAppInfo(await api.getAppInfo());
    } catch {
      // 버전 정보 실패는 무해하게 무시 — 화면 하단에 "…"로 표시됨
    }
    try {
      setEmbedder(await api.getEmbedderStatus());
    } catch {
      // 임베더 상태 실패도 무해하게 무시
    }
  }

  useEffect(() => {
    loadAll();
  }, []);

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    try {
      const next: SettingsDto = { ...settings, accessibility };
      await api.setSettings(next);
      onToast("설정을 저장했습니다.");
    } catch (e) {
      onToast(`저장 실패: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  async function handleSeedDemo() {
    const n = await api.seedDemoData();
    onToast(`데모 키워드 ${n}개를 추가했습니다.`);
  }

  if (!settings) {
    return <p>불러오는 중…</p>;
  }

  return (
    <section aria-labelledby="settings-title" className="stack">
      <h2 id="settings-title">설정</h2>

      <SourcePairingPanel onToast={onToast} />

      <div className="panel stack">
        <h3>발견 엔진</h3>
        <p className="field-hint">
          현재 임베딩 모델: <strong>{embedder?.model ?? "확인 중"}</strong>{" "}
          {embedder && !embedder.is_real_model && "(Ollama 미연결 — 로컬 테스트용 폴백 사용 중)"} · 차원 {embedder?.dim ?? "-"}
        </p>
        <div className="row">
          <div>
            <label htmlFor="w-s">의미 유사도 가중치 (w_s): {settings.discovery_config.weights.w_s.toFixed(2)}</label>
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
            <label htmlFor="w-t">기간 겹침 가중치 (w_t): {settings.discovery_config.weights.w_t.toFixed(2)}</label>
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
            <label htmlFor="w-f">빈도 신호 가중치 (w_f): {settings.discovery_config.weights.w_f.toFixed(2)}</label>
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
            <label htmlFor="bridge-cut">브리지 최소 유사도: {settings.discovery_config.bridge_sim_cut.toFixed(2)}</label>
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
            <label htmlFor="cluster-min">클러스터 최소 멤버 수: {settings.discovery_config.cluster_min_size}</label>
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
          <button onClick={handleSeedDemo}>데모 데이터 추가</button>
        </div>
      </div>

      <div className="panel stack">
        <h3>연동</h3>
        <div>
          <label htmlFor="ollama-url">Ollama 주소 (로컬 bge-m3 임베딩)</label>
          <input
            id="ollama-url"
            type="url"
            value={settings.ollama_base_url}
            onChange={(e) => setSettings({ ...settings, ollama_base_url: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="aimemory-endpoint">TXTAIMemory 환류 엔드포인트 (X2)</label>
          <input
            id="aimemory-endpoint"
            type="url"
            value={settings.aimemory_endpoint}
            onChange={(e) => setSettings({ ...settings, aimemory_endpoint: e.target.value })}
          />
          <p className="field-hint">비워두지 않아도 됩니다 — 미연결 시 환류는 실패로 기록되고 앱은 정상 동작합니다.</p>
        </div>
      </div>

      <div className="panel stack">
        <h3>접근성</h3>
        <label style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <input
            type="checkbox"
            style={{ width: 24, height: 24 }}
            checked={accessibility.high_contrast}
            onChange={(e) => onAccessibilityChange({ ...accessibility, high_contrast: e.target.checked })}
          />
          고대비 모드
        </label>
        <label style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <input
            type="checkbox"
            style={{ width: 24, height: 24 }}
            checked={accessibility.reduce_motion}
            onChange={(e) => onAccessibilityChange({ ...accessibility, reduce_motion: e.target.checked })}
          />
          애니메이션 줄이기
        </label>
        <div>
          <label htmlFor="font-scale">글자 크기 배율: {accessibility.font_scale.toFixed(2)}x</label>
          <input
            id="font-scale"
            type="range"
            min={1}
            max={2}
            step={0.1}
            value={accessibility.font_scale}
            onChange={(e) => onAccessibilityChange({ ...accessibility, font_scale: Number(e.target.value) })}
          />
        </div>
      </div>

      <div className="row">
        <button className="primary" onClick={handleSave} disabled={saving}>
          {saving ? "저장 중…" : "설정 저장"}
        </button>
      </div>

      <div className="panel stack">
        <h3>정보</h3>
        <p>버전 v{appInfo?.version ?? "…"}</p>
        <h4>업데이트 히스토리</h4>
        <ul>
          {(appInfo?.history ?? [])
            .slice()
            .reverse()
            .map((h) => (
              <li key={h.version}>
                <strong>v{h.version}</strong> ({h.date}) — {h.summary}
              </li>
            ))}
        </ul>
      </div>
    </section>
  );
}
