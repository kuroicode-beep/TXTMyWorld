// src/screens/Feed.tsx — S1 홈: 추천 피드. 하이브리드 발견의 "자동 제시" 축.

import { useEffect, useState } from "react";
import { api, DiscoveryDto } from "../api";
import { DiscoveryCard } from "../components/DiscoveryCard";
import { AdoptModal } from "../components/AdoptModal";
import { useI18n } from "../lib/i18n";

interface Props {
  onToast: (msg: string) => void;
}

export function Feed({ onToast }: Props) {
  const { t } = useI18n();
  const [discoveries, setDiscoveries] = useState<DiscoveryDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [running, setRunning] = useState(false);
  const [adoptTarget, setAdoptTarget] = useState<DiscoveryDto | null>(null);
  const [summary, setSummary] = useState<string | null>(null);

  async function load() {
    setLoading(true);
    try {
      const list = await api.listDiscoveries("new");
      setDiscoveries(list);
    } catch (e) {
      onToast(t("feed.loadFail", { e: String(e) }));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleRunDiscovery() {
    setRunning(true);
    try {
      const result = await api.runDiscovery();
      setSummary(
        t("feed.summary", {
          total: result.total_keywords,
          embedded: result.embedded_count,
          bridges: result.bridges,
          gaps: result.gaps,
          clusters: result.clusters,
        })
      );
      await load();
    } catch (e) {
      onToast(t("feed.runFail", { e: String(e) }));
    } finally {
      setRunning(false);
    }
  }

  async function handleDismiss(id: string) {
    await api.dismissDiscovery(id);
    setDiscoveries((prev) => prev.filter((d) => d.id !== id));
  }

  async function handleOpenDeeplink(url: string) {
    try {
      await api.openDeeplink(url);
    } catch (e) {
      onToast(t("common.deeplinkFail", { e: String(e) }));
    }
  }

  return (
    <section aria-labelledby="feed-title">
      <div className="toolbar">
        <h2 id="feed-title" style={{ margin: 0 }}>
          {t("feed.title")}
        </h2>
        <button className="primary" onClick={handleRunDiscovery} disabled={running}>
          {running ? t("feed.running") : t("feed.rerun")}
        </button>
      </div>

      {summary && <p className="field-hint">{summary}</p>}

      {loading && <p>{t("common.loading")}</p>}

      {!loading && discoveries.length === 0 && (
        <div className="empty-state">
          <p>{t("feed.empty")}</p>
          <p className="field-hint">{t("feed.emptyHint")}</p>
        </div>
      )}

      <ul className="card-list" aria-label={t("feed.listAria")}>
        {discoveries.map((d) => (
          <DiscoveryCard
            key={d.id}
            discovery={d}
            onAdopt={setAdoptTarget}
            onDismiss={handleDismiss}
            onOpenDeeplink={handleOpenDeeplink}
          />
        ))}
      </ul>

      {adoptTarget && (
        <AdoptModal
          discovery={adoptTarget}
          onClose={() => setAdoptTarget(null)}
          onAdopted={(_cardId, feedbackSent) => {
            onToast(feedbackSent ? t("common.adoptedWithFeedback") : t("common.adopted"));
            const adoptedId = adoptTarget.id;
            setAdoptTarget(null);
            setDiscoveries((prev) => prev.filter((d) => d.id !== adoptedId));
          }}
        />
      )}
    </section>
  );
}
