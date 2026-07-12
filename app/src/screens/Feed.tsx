// src/screens/Feed.tsx — S1 홈: 추천 피드. 하이브리드 발견의 "자동 제시" 축.

import { useEffect, useState } from "react";
import { api, DiscoveryDto } from "../api";
import { DiscoveryCard } from "../components/DiscoveryCard";
import { AdoptModal } from "../components/AdoptModal";

interface Props {
  onToast: (msg: string) => void;
}

export function Feed({ onToast }: Props) {
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
      onToast(`발견 목록을 불러오지 못했습니다: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleRunDiscovery() {
    setRunning(true);
    try {
      const result = await api.runDiscovery();
      setSummary(
        `키워드 ${result.total_keywords}개 중 ${result.embedded_count}개 새로 임베딩 · 브리지 ${result.bridges} · 갭 ${result.gaps} · 클러스터 ${result.clusters}`
      );
      await load();
    } catch (e) {
      onToast(`발견 실행 실패: ${e}`);
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
      onToast(`연결된 앱을 열 수 없습니다: ${e}`);
    }
  }

  return (
    <section aria-labelledby="feed-title">
      <div className="toolbar">
        <h2 id="feed-title" style={{ margin: 0 }}>
          이번에 새로 이어진 것
        </h2>
        <button className="primary" onClick={handleRunDiscovery} disabled={running}>
          {running ? "발견 실행 중…" : "발견 다시 실행"}
        </button>
      </div>

      {summary && <p className="field-hint">{summary}</p>}

      {loading && <p>불러오는 중…</p>}

      {!loading && discoveries.length === 0 && (
        <div className="empty-state">
          <p>아직 새로운 발견이 없습니다.</p>
          <p className="field-hint">
            소스를 동기화하거나 데모 데이터로 체험해 보세요(설정 화면), 그다음 "발견 다시 실행"을 눌러 보세요.
          </p>
        </div>
      )}

      <ul className="card-list" aria-label="새로 발견된 주제 후보 목록">
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
            onToast(feedbackSent ? "주제 카드가 저장되고 AI 기억에도 전송됐습니다." : "주제 카드가 저장됐습니다.");
            const adoptedId = adoptTarget.id;
            setAdoptTarget(null);
            setDiscoveries((prev) => prev.filter((d) => d.id !== adoptedId));
          }}
        />
      )}
    </section>
  );
}
