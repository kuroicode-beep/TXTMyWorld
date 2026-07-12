// src/components/AdoptModal.tsx — S3 주제 카드 생성 모달. 채택·명명 + (선택) 즉시 환류 동의.

import { useState } from "react";
import { DiscoveryDto, api } from "../api";

interface Props {
  discovery: DiscoveryDto;
  onClose: () => void;
  onAdopted: (cardId: string, feedbackSent: boolean) => void;
}

export function AdoptModal({ discovery, onClose, onAdopted }: Props) {
  const suggested = discovery.members.map((m) => m.text).join(" · ");
  const [name, setName] = useState(suggested);
  const [note, setNote] = useState("");
  const [sendToMemory, setSendToMemory] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) {
      setError("이름을 입력해 주세요.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const card = await api.adoptDiscovery(discovery.id, name.trim(), "user");
      if (note.trim()) {
        await api.updateTopicCard(card.id, { note: note.trim() });
      }
      let feedbackSent = false;
      if (sendToMemory) {
        try {
          await api.sendFeedback(card.id);
          feedbackSent = true;
        } catch {
          feedbackSent = false;
        }
      }
      onAdopted(card.id, feedbackSent);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="adopt-modal-title"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id="adopt-modal-title">주제 카드 만들기</h2>
        <form className="stack" onSubmit={handleSubmit}>
          <div>
            <label htmlFor="card-name">이름</label>
            <input id="card-name" type="text" value={name} onChange={(e) => setName(e.target.value)} required />
            <p className="field-hint">제안: {suggested}</p>
          </div>

          <div>
            <label htmlFor="card-note">메모 (선택)</label>
            <textarea id="card-note" rows={3} value={note} onChange={(e) => setNote(e.target.value)} />
          </div>

          <div className="panel" style={{ padding: 12 }}>
            <label style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 4 }}>
              <input
                type="checkbox"
                checked={sendToMemory}
                onChange={(e) => setSendToMemory(e.target.checked)}
                style={{ width: 24, height: 24 }}
              />
              이 주제를 AI 기억(TXTAIMemory)에 남기기
            </label>
            <p className="field-hint">
              전송 항목: 제목·구성 키워드·근거 요약·소스 딥링크만 보냅니다. 일기·문서·대화 본문은 절대 포함되지 않습니다.
              TXTAIMemory가 연결되어 있지 않으면 전송은 실패로 기록되고 나중에 다시 시도할 수 있습니다.
            </p>
          </div>

          {error && (
            <p role="alert" style={{ color: "var(--negative)" }}>
              {error}
            </p>
          )}

          <div className="row" style={{ justifyContent: "flex-end" }}>
            <button type="button" onClick={onClose} disabled={busy}>
              취소
            </button>
            <button type="submit" className="primary" disabled={busy}>
              {busy ? "저장 중…" : "저장"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
