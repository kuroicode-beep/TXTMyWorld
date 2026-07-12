// src/screens/Archive.tsx — S5 주제 카드 보관함: 목록·편집·환류 토글·삭제(soft)·내보내기.

import { useEffect, useState } from "react";
import { api, FeedbackRecordDto, TopicCardDto } from "../api";

interface Props {
  onToast: (msg: string) => void;
}

function statusLabel(s: TopicCardDto["status"]) {
  return s === "draft" ? "초안" : s === "confirmed" ? "확정" : "보관";
}

function cardToMarkdown(card: TopicCardDto): string {
  const lines = [
    `# ${card.name}`,
    "",
    `- 상태: ${statusLabel(card.status)}`,
    `- 생성일: ${new Date(card.created_at).toLocaleString("ko-KR")}`,
    `- 구성 키워드: ${card.members.map((m) => `${m.text}(${m.source_label})`).join(", ")}`,
    card.note ? `- 메모: ${card.note}` : "",
    "",
    "## 소스 링크",
    ...card.deeplinks.map((d) => `- ${d}`),
  ];
  return lines.filter((l) => l !== "").join("\n");
}

export function Archive({ onToast }: Props) {
  const [cards, setCards] = useState<TopicCardDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [history, setHistory] = useState<Record<string, FeedbackRecordDto[]>>({});
  const [editing, setEditing] = useState<Record<string, { name: string; note: string }>>({});

  async function load() {
    setLoading(true);
    try {
      setCards(await api.listTopicCards(false));
    } catch (e) {
      onToast(`보관함을 불러오지 못했습니다: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function toggleExpand(card: TopicCardDto) {
    if (expandedId === card.id) {
      setExpandedId(null);
      return;
    }
    setExpandedId(card.id);
    setEditing((prev) => ({ ...prev, [card.id]: prev[card.id] ?? { name: card.name, note: card.note ?? "" } }));
    try {
      const h = await api.getFeedbackHistory(card.id);
      setHistory((prev) => ({ ...prev, [card.id]: h }));
    } catch {
      // 이력 조회 실패는 무해하게 무시 — 카드 본체는 이미 표시됨
    }
  }

  async function saveEdit(card: TopicCardDto) {
    const edit = editing[card.id];
    if (!edit) return;
    try {
      const updated = await api.updateTopicCard(card.id, { name: edit.name, note: edit.note });
      setCards((prev) => prev.map((c) => (c.id === card.id ? updated : c)));
      onToast("카드를 저장했습니다.");
    } catch (e) {
      onToast(`저장 실패: ${e}`);
    }
  }

  async function setStatus(card: TopicCardDto, status: string) {
    const updated = await api.updateTopicCard(card.id, { status });
    setCards((prev) => prev.map((c) => (c.id === card.id ? updated : c)));
  }

  async function handleDelete(card: TopicCardDto) {
    await api.deleteTopicCard(card.id);
    setCards((prev) => prev.filter((c) => c.id !== card.id));
    onToast(`"${card.name}" 카드를 삭제했습니다.`);
  }

  async function handleSendFeedback(card: TopicCardDto) {
    try {
      const rec = await api.sendFeedback(card.id);
      setHistory((prev) => ({ ...prev, [card.id]: [rec, ...(prev[card.id] ?? [])] }));
      onToast(rec.status === "created" || rec.status === "updated" ? "AI 기억에 전송했습니다." : "전송에 실패했습니다 (TXTAIMemory 미연결일 수 있음).");
    } catch (e) {
      onToast(`환류 실패: ${e}`);
    }
  }

  function handleExport(card: TopicCardDto) {
    const md = cardToMarkdown(card);
    const blob = new Blob([md], { type: "text/markdown;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${card.name.replace(/[\\/:*?"<>|]/g, "_")}.md`;
    a.click();
    URL.revokeObjectURL(url);
  }

  async function handleOpenDeeplink(url: string) {
    try {
      await api.openDeeplink(url);
    } catch (e) {
      onToast(`연결된 앱을 열 수 없습니다: ${e}`);
    }
  }

  return (
    <section aria-labelledby="archive-title">
      <h2 id="archive-title">보관함</h2>
      {loading && <p>불러오는 중…</p>}
      {!loading && cards.length === 0 && <div className="empty-state">아직 저장한 주제 카드가 없습니다.</div>}

      <ul className="card-list" aria-label="주제 카드 목록">
        {cards.map((card) => {
          const isOpen = expandedId === card.id;
          const edit = editing[card.id] ?? { name: card.name, note: card.note ?? "" };
          const feedbacks = history[card.id] ?? [];
          const lastFeedback = feedbacks[0];
          const sentOk = lastFeedback && (lastFeedback.status === "created" || lastFeedback.status === "updated");

          return (
            <li key={card.id} className="card-item">
              <div className="row" style={{ justifyContent: "space-between" }}>
                <button
                  aria-expanded={isOpen}
                  onClick={() => toggleExpand(card)}
                  style={{ textAlign: "left", flex: 1, background: "transparent", border: "none" }}
                >
                  <h3 style={{ margin: 0 }}>{card.name}</h3>
                </button>
                <span className="badge">{statusLabel(card.status)}</span>
                <span className={`badge ${sentOk ? "status-ok" : ""}`}>{sentOk ? "환류됨" : "환류 안 함"}</span>
              </div>

              <p className="field-hint">구성: {card.members.map((m) => `${m.text}(${m.source_label})`).join(", ")}</p>

              {isOpen && (
                <div className="stack" style={{ marginTop: 12 }}>
                  <div>
                    <label htmlFor={`name-${card.id}`}>이름</label>
                    <input
                      id={`name-${card.id}`}
                      type="text"
                      value={edit.name}
                      onChange={(e) => setEditing((p) => ({ ...p, [card.id]: { ...edit, name: e.target.value } }))}
                    />
                  </div>
                  <div>
                    <label htmlFor={`note-${card.id}`}>메모</label>
                    <textarea
                      id={`note-${card.id}`}
                      rows={3}
                      value={edit.note}
                      onChange={(e) => setEditing((p) => ({ ...p, [card.id]: { ...edit, note: e.target.value } }))}
                    />
                  </div>

                  <div className="deeplink-list">
                    {card.deeplinks.map((url, i) => (
                      <button key={i} onClick={() => handleOpenDeeplink(url)}>
                        원본 열기 #{i + 1}
                      </button>
                    ))}
                  </div>

                  {feedbacks.length > 0 && (
                    <div>
                      <h4>환류 이력</h4>
                      <ul>
                        {feedbacks.map((f, i) => (
                          <li key={i}>
                            {new Date(f.sent_at).toLocaleString("ko-KR")} — {f.status}
                            {f.memory_id ? ` (memory_id: ${f.memory_id})` : ""}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}

                  <div className="row">
                    <button className="primary" onClick={() => saveEdit(card)}>
                      변경사항 저장
                    </button>
                    <button onClick={() => handleSendFeedback(card)}>AI 기억에 (재)전송</button>
                    <button onClick={() => handleExport(card)}>Markdown 내보내기</button>
                    <select value={card.status} onChange={(e) => setStatus(card, e.target.value)} aria-label="카드 상태 변경">
                      <option value="draft">초안</option>
                      <option value="confirmed">확정</option>
                      <option value="archived">보관</option>
                    </select>
                    <button className="danger" onClick={() => handleDelete(card)}>
                      삭제
                    </button>
                  </div>
                </div>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
