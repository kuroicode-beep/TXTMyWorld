// src/components/AdoptModal.tsx — S3 주제 카드 생성 모달. 채택·명명 + (선택) 즉시 환류 동의.

import { useState } from "react";
import { DiscoveryDto, api } from "../api";
import { useI18n } from "../lib/i18n";

interface Props {
  discovery: DiscoveryDto;
  onClose: () => void;
  onAdopted: (cardId: string, feedbackSent: boolean) => void;
}

export function AdoptModal({ discovery, onClose, onAdopted }: Props) {
  const { t } = useI18n();
  const suggested = discovery.members.map((m) => m.text).join(" · ");
  const [name, setName] = useState(suggested);
  const [note, setNote] = useState("");
  const [sendToMemory, setSendToMemory] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) {
      setError(t("adopt.nameRequired"));
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
        <h2 id="adopt-modal-title">{t("adopt.title")}</h2>
        <form className="stack" onSubmit={handleSubmit}>
          <div>
            <label htmlFor="card-name">{t("adopt.nameLabel")}</label>
            <input id="card-name" type="text" value={name} onChange={(e) => setName(e.target.value)} required />
            <p className="field-hint">{t("adopt.suggestedHint", { v: suggested })}</p>
          </div>

          <div>
            <label htmlFor="card-note">{t("adopt.noteLabel")}</label>
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
              {t("adopt.memoryCheckbox")}
            </label>
            <p className="field-hint">{t("adopt.memoryHint")}</p>
          </div>

          {error && (
            <p role="alert" style={{ color: "var(--danger)" }}>
              {error}
            </p>
          )}

          <div className="row" style={{ justifyContent: "flex-end" }}>
            <button type="button" onClick={onClose} disabled={busy}>
              {t("adopt.cancel")}
            </button>
            <button type="submit" className="primary" disabled={busy}>
              {busy ? t("adopt.saving") : t("adopt.save")}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
