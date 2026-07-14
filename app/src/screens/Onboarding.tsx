// src/screens/Onboarding.tsx — S0 온보딩·소스 페어링. 소스 없이도 데모 데이터로 즉시 체험 가능.

import { useState } from "react";
import { api } from "../api";
import { SourcePairingPanel } from "../components/SourcePairingPanel";
import { useI18n } from "../lib/i18n";

interface Props {
  onToast: (msg: string) => void;
  onDone: () => void;
}

export function Onboarding({ onToast, onDone }: Props) {
  const { t } = useI18n();
  const [seeding, setSeeding] = useState(false);

  async function handleSeedDemo() {
    setSeeding(true);
    try {
      const n = await api.seedDemoData();
      onToast(t("onboarding.seedSuccess", { n }));
      onDone();
    } catch (e) {
      onToast(t("onboarding.seedFail", { e: String(e) }));
    } finally {
      setSeeding(false);
    }
  }

  return (
    <section aria-labelledby="onboarding-title" className="stack">
      <div className="panel">
        <h2 id="onboarding-title">{t("onboarding.title")}</h2>
        <p>{t("onboarding.desc")}</p>
        <div className="row">
          <button className="primary" onClick={handleSeedDemo} disabled={seeding}>
            {seeding ? t("onboarding.seeding") : t("onboarding.seedDemo")}
          </button>
          <button onClick={onDone}>{t("onboarding.skip")}</button>
        </div>
        <p className="field-hint">{t("onboarding.demoHint")}</p>
      </div>

      <SourcePairingPanel onToast={onToast} onChanged={onDone} />
    </section>
  );
}
