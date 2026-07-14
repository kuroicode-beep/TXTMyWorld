// src/screens/Explore.tsx — S4 탐색: 사용자가 직접 유형·소스·검색어로 필터링하는 리스트 뷰.
// (2D 관계 뷰는 MoSCoW "Should"로 v0.1 범위 밖 — 리스트 뷰가 항상 완전한 정보를 제공하므로
// 접근성 Must 요구사항인 "리스트/그래프 동등성"은 리스트만으로도 충족된다.)

import { useEffect, useMemo, useState } from "react";
import { api, DiscoveryDto, DiscoveryType } from "../api";
import { DiscoveryCard } from "../components/DiscoveryCard";
import { AdoptModal } from "../components/AdoptModal";
import { useI18n } from "../lib/i18n";

interface Props {
  onToast: (msg: string) => void;
}

const TYPES: DiscoveryType[] = ["bridge", "gap", "cluster", "drift"];

export function Explore({ onToast }: Props) {
  const { t } = useI18n();
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
      onToast(t("explore.loadFail", { e: String(e) }));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
      onToast(t("common.deeplinkFail", { e: String(e) }));
    }
  }

  return (
    <section aria-labelledby="explore-title">
      <h2 id="explore-title">{t("explore.title")}</h2>

      <div className="panel">
        <div className="row">
          <div>
            <label htmlFor="filter-type">{t("explore.typeLabel")}</label>
            <select id="filter-type" value={typeFilter} onChange={(e) => setTypeFilter(e.target.value as DiscoveryType | "all")}>
              <option value="all">{t("explore.type.all")}</option>
              {TYPES.map((tp) => (
                <option key={tp} value={tp}>
                  {t(`discovery.type.${tp}`)}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="filter-query">{t("explore.queryLabel")}</label>
            <input
              id="filter-query"
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("explore.queryPlaceholder")}
            />
          </div>
          <div>
            <label htmlFor="filter-score">{t("explore.scoreLabel", { v: minScore.toFixed(2) })}</label>
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
            {t("explore.includeWeak")}
          </label>
        </div>
      </div>

      {loading && <p>{t("common.loading")}</p>}

      <p className="field-hint" role="status">
        {t("explore.resultCount", { shown: filtered.length, total: all.length })}
      </p>

      {!loading && filtered.length === 0 && <div className="empty-state">{t("explore.empty")}</div>}

      <ul className="card-list" aria-label={t("explore.listAria")}>
        {filtered.map((d) => (
          <DiscoveryCard key={d.id} discovery={d} onAdopt={setAdoptTarget} onDismiss={handleDismiss} onOpenDeeplink={handleOpenDeeplink} />
        ))}
      </ul>

      {adoptTarget && (
        <AdoptModal
          discovery={adoptTarget}
          onClose={() => setAdoptTarget(null)}
          onAdopted={(_cardId, feedbackSent) => {
            onToast(feedbackSent ? t("common.adoptedWithFeedback") : t("common.adopted"));
            setAdoptTarget(null);
          }}
        />
      )}
    </section>
  );
}
