// src/api.ts — Rust IPC 커맨드에 대한 타입 래퍼. Tauri invoke() 호출을 이 파일로 모아
// 화면 컴포넌트가 문자열 커맨드명을 직접 다루지 않게 한다.

import { invoke } from "@tauri-apps/api/core";

export interface VersionEntry {
  version: string;
  date: string;
  summary: string;
}

export interface AppInfo {
  version: string;
  history: VersionEntry[];
}

export interface KeywordDto {
  source: string;
  source_label: string;
  text: string;
  normalized_text: string;
  frequency: number;
  avg_emotion_score: number;
  deeplink: string;
}

export type DiscoveryType = "bridge" | "gap" | "cluster" | "drift";

export interface DiscoveryDto {
  id: string;
  dtype: DiscoveryType;
  dtype_label_ko: string;
  members: KeywordDto[];
  semantic_sim: number;
  temporal_overlap: number;
  frequency_signal: number;
  note: string | null;
  score: number;
  weak_signal: boolean;
  evidence_sentence: string;
}

export interface TopicCardDto {
  id: string;
  name: string;
  label_source: "user" | "ai";
  discovery_id: string;
  members: KeywordDto[];
  note: string | null;
  status: "draft" | "confirmed" | "archived";
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  deeplinks: string[];
  external_id: string;
}

export interface SourceStatusDto {
  source: string;
  base_url: string;
  paired: boolean;
  last_synced_at: string | null;
  online: boolean;
  vector_capable: boolean;
  message: string | null;
}

export interface FeedbackRecordDto {
  target: string;
  payload_summary: string;
  status: string;
  memory_id: string | null;
  sent_at: string;
}

export interface SyncResultDto {
  source: string;
  status: "ok" | "update_required" | "offline";
  keyword_count: number;
  vector_count: number;
  message: string | null;
}

export interface DiscoveryRunSummaryDto {
  total_keywords: number;
  embedded_count: number;
  bridges: number;
  gaps: number;
  clusters: number;
  discoveries: DiscoveryDto[];
}

export interface FusionWeights {
  w_s: number;
  w_t: number;
  w_f: number;
}

export interface DiscoveryConfig {
  weights: FusionWeights;
  bridge_sim_cut: number;
  gap_sim_cut: number;
  gap_min_freq: number;
  cluster_sim_cut: number;
  cluster_min_size: number;
  drift_min_delta: number;
  knn_k: number;
  weak_signal_score: number;
}

// 글꼴·글자 크기·언어는 SVIL 표준에 따라 프론트엔드 전용(localStorage) 관심사다 — lib/prefs.ts, lib/i18n.ts 참조.
export interface SettingsDto {
  discovery_config: DiscoveryConfig;
  ollama_base_url: string;
  aimemory_endpoint: string;
}

export interface EmbedderStatus {
  model: string;
  is_real_model: boolean;
  dim: number;
}

// txtspace-hub는 실제 소스가 아니라 3소스(txtdiary/txtbrain/txtaimemory)를 토큰 없이
// 한 번에 통합 제공하는 로컬 허브(TXTSpace-hub, 기본 포트 47540) — 개별 소스마다 각 앱에서
// 토큰을 발급받아야 하는 번거로움 없이 한 번의 연결로 세 소스를 전부 받아온다.
export const SOURCE_IDS = ["txtdiary", "txtbrain", "txtaimemory", "txtspace-hub"] as const;
export type SourceIdStr = (typeof SOURCE_IDS)[number];

export const api = {
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  openDeeplink: (url: string) => invoke<void>("open_deeplink", { url }),

  pairSource: (source: string, baseUrl: string, token: string) =>
    invoke<void>("pair_source", { source, baseUrl, token }),
  unpairSource: (source: string) => invoke<void>("unpair_source", { source }),
  listSources: () => invoke<SourceStatusDto[]>("list_sources"),
  checkSourceHealth: (source: string, baseUrl: string) =>
    invoke<SourceStatusDto>("check_source_health", { source, baseUrl }),

  syncSource: (source: string, baseUrl: string) => invoke<SyncResultDto>("sync_source", { source, baseUrl }),
  syncAll: () => invoke<SyncResultDto[]>("sync_all"),
  connectAllSources: () => invoke<SyncResultDto[]>("connect_all_sources"),
  seedDemoData: () => invoke<number>("seed_demo_data"),

  runDiscovery: () => invoke<DiscoveryRunSummaryDto>("run_discovery"),
  listDiscoveries: (status?: string) => invoke<DiscoveryDto[]>("list_discoveries", { status: status ?? null }),
  dismissDiscovery: (id: string) => invoke<void>("dismiss_discovery", { id }),

  adoptDiscovery: (discoveryId: string, name: string, labelSource: "user" | "ai") =>
    invoke<TopicCardDto>("adopt_discovery", { discoveryId, name, labelSource }),
  listTopicCards: (includeDeleted = false) => invoke<TopicCardDto[]>("list_topic_cards", { includeDeleted }),
  updateTopicCard: (id: string, patch: { name?: string; note?: string; status?: string }) =>
    invoke<TopicCardDto>("update_topic_card", {
      id,
      name: patch.name ?? null,
      note: patch.note ?? null,
      status: patch.status ?? null,
    }),
  deleteTopicCard: (id: string) => invoke<void>("delete_topic_card", { id }),

  sendFeedback: (cardId: string) => invoke<FeedbackRecordDto>("send_feedback", { cardId }),
  getFeedbackHistory: (cardId: string) => invoke<FeedbackRecordDto[]>("get_feedback_history", { cardId }),

  getSettings: () => invoke<SettingsDto>("get_settings"),
  setSettings: (settings: SettingsDto) => invoke<void>("set_settings", { settings }),
  getEmbedderStatus: () => invoke<EmbedderStatus>("get_embedder_status"),
};
