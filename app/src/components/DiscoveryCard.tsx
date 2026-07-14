// src/components/DiscoveryCard.tsx — 발견 후보 1건 표시 (S1/S4 공용). 리스트 뷰만으로도
// 그래프 없이 완전한 정보(구성·근거·딥링크)를 제공한다 — PRD §7 리스트/설명 뷰 동등성 원칙.
// evidence_sentence는 백엔드가 사용자 데이터로 조합하는 자연어 문장이라 번역 대상에서 제외한다.

import { DiscoveryDto } from "../api";
import { useI18n } from "../lib/i18n";

interface Props {
  discovery: DiscoveryDto;
  onAdopt: (d: DiscoveryDto) => void;
  onDismiss: (id: string) => void;
  onOpenDeeplink: (url: string) => void;
}

export function DiscoveryCard({ discovery, onAdopt, onDismiss, onOpenDeeplink }: Props) {
  const { t } = useI18n();
  return (
    <li className="card-item">
      <div className="row" style={{ justifyContent: "space-between" }}>
        <div className="row">
          <span className={`badge type-${discovery.dtype}`}>{t(`discovery.type.${discovery.dtype}`)}</span>
          {discovery.weak_signal && <span className="badge status-weak">{t("discovery.weakSignal")}</span>}
        </div>
        <span className="field-hint mono">{t("discovery.scoreLabel", { v: discovery.score.toFixed(2) })}</span>
      </div>

      <h3>{discovery.members.map((m) => m.text).join(" · ")}</h3>

      <p className="evidence-text">{discovery.evidence_sentence}</p>

      <div className="deeplink-list" role="list" aria-label={t("discovery.deeplinkListAria")}>
        {discovery.members.map((m, i) => (
          <button
            key={i}
            onClick={() => onOpenDeeplink(m.deeplink)}
            aria-label={t("discovery.deeplinkAria", { source: t(`source.label.${m.source}`), text: m.text })}
          >
            {t(`source.label.${m.source}`)}: {m.text}
          </button>
        ))}
      </div>

      <div className="row" style={{ marginTop: 12 }}>
        <button className="primary" onClick={() => onAdopt(discovery)}>
          {t("discovery.adopt")}
        </button>
        <button onClick={() => onDismiss(discovery.id)}>{t("discovery.dismiss")}</button>
      </div>
    </li>
  );
}
