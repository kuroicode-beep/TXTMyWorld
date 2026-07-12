// src/screens/Explore.tsx — S4 탐색: 사용자가 직접 유형·소스·검색어로 필터링하는 리스트 뷰.
// (2D 관계 뷰는 MoSCoW "Should"로 v0.1 범위 밖 — 리스트 뷰가 항상 완전한 정보를 제공하므로
// 접근성 Must 요구사항인 "리스트/그래프 동등성"은 리스트만으로도 충족된다.)

import { useEffect, useMemo, useState } from "react";
import { api, DiscoveryDto, DiscoveryType } from "../api";
import { DiscoveryCard } from "../components/DiscoveryCard";
import { AdoptModal } from "../components/AdoptModal";

interface Props {
  onToast: (msg: string) => void;
}

const TYPE_OPTIONS: { value: DiscoveryType | "all"; label: string }[] = [
  { value: "all", label: "전체" },
  { value: "bridge", label: "브리지" },
  { value: "gap", label: "갭" },
  { value: "cluster", label: "이머전트 클러스터" },
  { value: "drift", label: "드리프트" },
];

export function Explore({ onToast }: Props) {
  const [all, setAll] = useState<DiscoveryDto[]>([]);
  const [typeFilter, setTypeFilter] = useState<DiscoveryType | "all">("all");
  const [query, setQuery] = useState("");
  const [minScore, setMinScore] = useState(0);
  const [includeWeak, setIncludeWeak] = useState(true);
  const [adoptTarget, setAdoptTarget] = useState<DiscoveryDto | null>(null);
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    try {
      setAll(await api.listDiscoveries());
    } catch (e) {
      onToast(`목록을 불러오지 못했습니다: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return all.filter((d) => {
      if (typeFilter !== "all" && d.dtype !== typeFilter) return false;
      if (d.score < minScore) return false;
      if (!includeWeak && d.weak_signal) return false;
      if (q && !d.members.some((m) => m.text.toLowerCase().includes(q))) return false;
      return true;
    });
  }, [all, typeFilter, query, minScore, includeWeak]);

  async function handleDismiss(id: string) {
    await api.dismissDiscovery(id);
    setAll((prev) => prev.filter((d) => d.id !== id));
  }

  async function handleOpenDeeplink(url: string) {
    try {
      await api.openDeeplink(url);
    } catch (e) {
      onToast(`연결된 앱을 열 수 없습니다: ${e}`);
    }
  }

  return (
    <section aria-labelledby="explore-title">
      <h2 id="explore-title">탐색</h2>

      <div className="panel">
        <div className="row">
          <div>
            <label htmlFor="filter-type">발견 유형</label>
            <select id="filter-type" value={typeFilter} onChange={(e) => setTypeFilter(e.target.value as any)}>
              {TYPE_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="filter-query">키워드 검색</label>
            <input id="filter-query" type="text" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="예: 관측자" />
          </div>
          <div>
            <label htmlFor="filter-score">최소 점수: {minScore.toFixed(2)}</label>
            <input
              id="filter-score"
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={minScore}
              onChange={(e) => setMinScore(Number(e.target.value))}
            />
          </div>
          <label style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 22 }}>
            <input type="checkbox" checked={includeWeak} onChange={(e) => setIncludeWeak(e.target.checked)} style={{ width: 22, height: 22 }} />
            약한 신호 포함
          </label>
        </div>
      </div>

      {loading && <p>불러오는 중…</p>}

      <p className="field-hint" role="status">
        {filtered.length}건 표시 중 (전체 {all.length}건)
      </p>

      {!loading && filtered.length === 0 && <div className="empty-state">조건에 맞는 발견이 없습니다.</div>}

      <ul className="card-list" aria-label="탐색 결과 목록">
        {filtered.map((d) => (
          <DiscoveryCard key={d.id} discovery={d} onAdopt={setAdoptTarget} onDismiss={handleDismiss} onOpenDeeplink={handleOpenDeeplink} />
        ))}
      </ul>

      {adoptTarget && (
        <AdoptModal
          discovery={adoptTarget}
          onClose={() => setAdoptTarget(null)}
          onAdopted={(_cardId, feedbackSent) => {
            onToast(feedbackSent ? "주제 카드가 저장되고 AI 기억에도 전송됐습니다." : "주제 카드가 저장됐습니다.");
            setAdoptTarget(null);
          }}
        />
      )}
    </section>
  );
}
