// src/screens/Onboarding.tsx — S0 온보딩·소스 페어링. 소스 없이도 데모 데이터로 즉시 체험 가능.

import { useState } from "react";
import { api } from "../api";
import { SourcePairingPanel } from "../components/SourcePairingPanel";

interface Props {
  onToast: (msg: string) => void;
  onDone: () => void;
}

export function Onboarding({ onToast, onDone }: Props) {
  const [seeding, setSeeding] = useState(false);

  async function handleSeedDemo() {
    setSeeding(true);
    try {
      const n = await api.seedDemoData();
      onToast(`데모 키워드 ${n}개를 넣었습니다. "발견" 탭에서 "발견 다시 실행"을 눌러보세요.`);
      onDone();
    } catch (e) {
      onToast(`데모 데이터 추가 실패: ${e}`);
    } finally {
      setSeeding(false);
    }
  }

  return (
    <section aria-labelledby="onboarding-title" className="stack">
      <div className="panel">
        <h2 id="onboarding-title">TXTMyWorld에 오신 것을 환영합니다</h2>
        <p>
          TXTDiary(일기)·TXTBrain(문서)·TXTAIMemory(AI 대화)에서 나오는 키워드를 기간·빈도·벡터(의미)로 조합해,
          아직 이름 없는 새 주제를 찾아드립니다.
        </p>
        <div className="row">
          <button className="primary" onClick={handleSeedDemo} disabled={seeding}>
            {seeding ? "준비 중…" : "데모 데이터로 바로 체험하기"}
          </button>
          <button onClick={onDone}>소스 연결만 하고 넘어가기</button>
        </div>
        <p className="field-hint">데모 데이터는 실제 기록이 아닌 예시 키워드입니다. 언제든 설정에서 다시 켤 수 있습니다.</p>
      </div>

      <SourcePairingPanel onToast={onToast} onChanged={onDone} />
    </section>
  );
}
