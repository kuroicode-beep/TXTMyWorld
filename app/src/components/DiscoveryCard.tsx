// src/components/DiscoveryCard.tsx — 발견 후보 1건 표시 (S1/S4 공용). 리스트 뷰만으로도
// 그래프 없이 완전한 정보(구성·근거·딥링크)를 제공한다 — PRD §7 리스트/설명 뷰 동등성 원칙.

import { DiscoveryDto } from "../api";

interface Props {
  discovery: DiscoveryDto;
  onAdopt: (d: DiscoveryDto) => void;
  onDismiss: (id: string) => void;
  onOpenDeeplink: (url: string) => void;
}

export function DiscoveryCard({ discovery, onAdopt, onDismiss, onOpenDeeplink }: Props) {
  return (
    <li className="card-item">
      <div className="row" style={{ justifyContent: "space-between" }}>
        <div className="row">
          <span className={`badge type-${discovery.dtype}`}>{discovery.dtype_label_ko}</span>
          {discovery.weak_signal && <span className="badge status-weak">약한 신호</span>}
        </div>
        <span className="field-hint">종합 점수 {discovery.score.toFixed(2)}</span>
      </div>

      <h3>{discovery.members.map((m) => m.text).join(" · ")}</h3>

      <p className="evidence-text">{discovery.evidence_sentence}</p>

      <div className="deeplink-list" role="list" aria-label="관련 소스로 이동">
        {discovery.members.map((m, i) => (
          <button key={i} onClick={() => onOpenDeeplink(m.deeplink)} aria-label={`${m.source_label}에서 '${m.text}' 열기`}>
            {m.source_label}: {m.text}
          </button>
        ))}
      </div>

      <div className="row" style={{ marginTop: 12 }}>
        <button className="primary" onClick={() => onAdopt(discovery)}>
          주제 카드로 만들기
        </button>
        <button onClick={() => onDismiss(discovery.id)}>관심 없음</button>
      </div>
    </li>
  );
}
