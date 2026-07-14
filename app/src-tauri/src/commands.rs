// src-tauri/src/commands.rs — Tauri IPC 커맨드: 프론트엔드가 invoke()로 호출하는 전체 표면
// 각 커맨드는 얇은 어댑터다 — 실제 로직은 core(txtmyworld-core)와 pipeline.rs에 있다.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

use txtmyworld_core::discovery::DiscoveryConfig;
use txtmyworld_core::feedback::build_feedback_payload_with_type;
use txtmyworld_core::topic::{CardStatus, LabelSource, TopicCard};

use crate::dto::{
    app_info, AppInfoDto, DiscoveryDto, DiscoveryRunSummaryDto, FeedbackRecordDto, SourceStatusDto, SyncResultDto,
    TopicCardDto,
};
use crate::feedback_client::{FeedbackTransport, HttpFeedbackTransport, DEFAULT_ENDPOINT};
use crate::state::AppState;
use crate::{embed_select, pipeline, secure};

const SETTING_DISCOVERY_CONFIG: &str = "discovery_config";
const SETTING_OLLAMA_URL: &str = "ollama_base_url";
const SETTING_AIMEMORY_ENDPOINT: &str = "aimemory_endpoint";

fn default_ollama_url() -> String {
    "http://127.0.0.1:11434".to_string()
}

// ---------------------------------------------------------------------------
// 앱 정보
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_app_info() -> AppInfoDto {
    app_info()
}

/// 소스 딥링크(txtdiary:// 등)를 OS 기본 핸들러로 연다. 대상 앱 미설치 시 오류를 그대로 프론트로 반환한다.
#[tauri::command]
pub fn open_deeplink(app: AppHandle, url: String) -> Result<(), String> {
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 소스 페어링 (§1: X1 소비)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn pair_source(state: State<AppState>, source: String, base_url: String, token: String) -> Result<(), String> {
    if !base_url.starts_with("http://127.0.0.1") && !base_url.starts_with("http://localhost") {
        return Err("소스는 localhost(127.0.0.1)에만 연결할 수 있습니다.".into());
    }
    if !token.is_empty() {
        secure::set_token(&source, &token)?;
    }
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let fp = if token.is_empty() { None } else { Some(secure::fingerprint(&token)) };
    store.upsert_source(&source, &base_url, fp.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unpair_source(state: State<AppState>, source: String) -> Result<(), String> {
    secure::delete_token(&source)?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.delete_source(&source).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sources(state: State<AppState>) -> Result<Vec<SourceStatusDto>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let rows = store.list_sources().map_err(|e| e.to_string())?;
    Ok(rows.iter().map(SourceStatusDto::from).collect())
}

/// 소스 연결 상태를 실시간으로 점검한다 (/health 호출, 페어링 여부와 별개)
#[tauri::command]
pub fn check_source_health(base_url: String, source: String) -> SourceStatusDto {
    use txtmyworld_core::source::{fetch_health, AuthHeader, SourceConfig, SourceFetch};
    let token = secure::resolve_token(&source).map(|(t, _)| t);
    let auth = if source == "txtaimemory" { AuthHeader::XPairingToken } else { AuthHeader::Bearer };
    let cfg = SourceConfig::with_header(base_url.clone(), token.clone(), auth);
    match fetch_health(&cfg) {
        Ok(SourceFetch::Ok(h)) => SourceStatusDto {
            source,
            base_url,
            paired: token.is_some(),
            last_synced_at: None,
            online: true,
            vector_capable: h.vector_capability.map(|c| c.supported).unwrap_or(false),
            message: None,
        },
        Ok(SourceFetch::UpdateRequired) => SourceStatusDto {
            source,
            base_url,
            paired: token.is_some(),
            last_synced_at: None,
            online: false,
            vector_capable: false,
            message: Some("업데이트 필요".into()),
        },
        _ => SourceStatusDto {
            source,
            base_url,
            paired: token.is_some(),
            last_synced_at: None,
            online: false,
            vector_capable: false,
            message: Some("오프라인".into()),
        },
    }
}

// ---------------------------------------------------------------------------
// 동기화
// ---------------------------------------------------------------------------

fn resolve_ollama_url(store: &txtmyworld_core::store::Store) -> String {
    store
        .get_setting(SETTING_OLLAMA_URL)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(default_ollama_url)
}

#[tauri::command]
pub fn sync_source(state: State<AppState>, source: String, base_url: String) -> Result<SyncResultDto, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let ollama_url = resolve_ollama_url(&store);
    Ok(pipeline::sync_source(&store, &source, &base_url, &ollama_url))
}

#[tauri::command]
pub fn sync_all(state: State<AppState>) -> Result<Vec<SyncResultDto>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let ollama_url = resolve_ollama_url(&store);
    Ok(pipeline::sync_all(&store, &ollama_url))
}

/// 데모 데이터 시드 (실 소스 없이 발견 흐름을 체험) — 명시적 라벨링된 데모 전용 액션
#[tauri::command]
pub fn seed_demo_data(state: State<AppState>) -> Result<usize, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    Ok(pipeline::seed_demo_data(&store))
}

/// 3소스(TXTDiary/TXTBrain/TXTAIMemory)를 각 실측 포트로 등록하고 한 번에 동기화한다.
/// TXTSpace가 이미 저장해 둔 공유 토큰을 자동으로 재사용하므로, 대개 별도 토큰 입력 없이 바로 연결된다.
#[tauri::command]
pub fn connect_all_sources(state: State<AppState>) -> Result<Vec<SyncResultDto>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let ollama_url = resolve_ollama_url(&store);
    Ok(pipeline::connect_all_direct(&store, &ollama_url))
}

// ---------------------------------------------------------------------------
// 발견 엔진
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn run_discovery(state: State<AppState>) -> Result<DiscoveryRunSummaryDto, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let config = store
        .get_setting(SETTING_DISCOVERY_CONFIG)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value::<DiscoveryConfig>(v).ok())
        .unwrap_or_default();
    let ollama_url = resolve_ollama_url(&store);
    Ok(pipeline::run_discovery(&store, config, &ollama_url))
}

#[tauri::command]
pub fn list_discoveries(state: State<AppState>, status: Option<String>) -> Result<Vec<DiscoveryDto>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let items = store.list_discoveries(status.as_deref()).map_err(|e| e.to_string())?;
    Ok(items.iter().map(DiscoveryDto::from).collect())
}

#[tauri::command]
pub fn dismiss_discovery(state: State<AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.set_discovery_status(&uuid, "dismissed").map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// 주제 카드 (생성·보관함)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn adopt_discovery(
    state: State<AppState>,
    discovery_id: String,
    name: String,
    label_source: String,
) -> Result<TopicCardDto, String> {
    let uuid = Uuid::parse_str(&discovery_id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let discovery = store
        .get_discovery(&uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "발견 후보를 찾을 수 없습니다".to_string())?;

    let label = if label_source == "ai" { LabelSource::Ai } else { LabelSource::User };
    let card = TopicCard::adopt(&discovery, name, label);
    store.upsert_topic_card(&card).map_err(|e| e.to_string())?;
    store.set_discovery_status(&uuid, "adopted").map_err(|e| e.to_string())?;
    Ok(TopicCardDto::from(&card))
}

#[tauri::command]
pub fn list_topic_cards(state: State<AppState>, include_deleted: bool) -> Result<Vec<TopicCardDto>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let cards = store.list_topic_cards(include_deleted).map_err(|e| e.to_string())?;
    Ok(cards.iter().map(TopicCardDto::from).collect())
}

#[tauri::command]
pub fn update_topic_card(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    note: Option<String>,
    status: Option<String>,
) -> Result<TopicCardDto, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut card = store
        .get_topic_card(&uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "카드를 찾을 수 없습니다".to_string())?;

    if let Some(n) = name {
        card.name = n;
    }
    if let Some(note_val) = note {
        card.note = Some(note_val);
    }
    if let Some(s) = status {
        card.status = match s.as_str() {
            "confirmed" => CardStatus::Confirmed,
            "archived" => CardStatus::Archived,
            _ => CardStatus::Draft,
        };
    }
    card.updated_at = chrono::Utc::now();
    store.upsert_topic_card(&card).map_err(|e| e.to_string())?;
    Ok(TopicCardDto::from(&card))
}

#[tauri::command]
pub fn delete_topic_card(state: State<AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut card = store
        .get_topic_card(&uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "카드를 찾을 수 없습니다".to_string())?;
    card.soft_delete();
    store.upsert_topic_card(&card).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// X2 환류 (동의형)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn send_feedback(state: State<AppState>, card_id: String) -> Result<crate::dto::FeedbackRecordDto, String> {
    let uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let card = store
        .get_topic_card(&uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "카드를 찾을 수 없습니다".to_string())?;

    let dtype = store
        .get_discovery_type(&card.discovery_id)
        .map_err(|e| e.to_string())?
        .unwrap_or(txtmyworld_core::discovery::DiscoveryType::Cluster);
    let payload = build_feedback_payload_with_type(&card, &dtype);
    let summary = payload.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let endpoint = store
        .get_setting(SETTING_AIMEMORY_ENDPOINT)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
    let token = secure::resolve_token("txtaimemory").map(|(t, _shared)| t);
    let transport = HttpFeedbackTransport { endpoint, token };

    match transport.send(&payload) {
        Ok(ack) => {
            let status = if ack.ok { "created" } else { "failed" };
            store
                .insert_feedback(&uuid, &summary, status, ack.memory_id.as_deref())
                .map_err(|e| e.to_string())?;
        }
        Err(_) => {
            store.insert_feedback(&uuid, &summary, "offline", None).map_err(|e| e.to_string())?;
        }
    }

    let history = store.list_feedbacks(&uuid).map_err(|e| e.to_string())?;
    history
        .first()
        .map(FeedbackRecordDto::from)
        .ok_or_else(|| "환류 이력 기록 실패".to_string())
}

#[tauri::command]
pub fn get_feedback_history(state: State<AppState>, card_id: String) -> Result<Vec<FeedbackRecordDto>, String> {
    let uuid = Uuid::parse_str(&card_id).map_err(|e| e.to_string())?;
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let history = store.list_feedbacks(&uuid).map_err(|e| e.to_string())?;
    Ok(history.iter().map(FeedbackRecordDto::from).collect())
}

// ---------------------------------------------------------------------------
// 설정
// ---------------------------------------------------------------------------

// 화면 설정(글꼴·글자 크기·언어)은 SVIL 표준에 따라 프론트엔드 전용 관심사다 — 브라우저
// localStorage에 저장하고 CSS 변수로 즉시 적용한다(app/src/lib/prefs.ts, lib/i18n.ts).
// 백엔드 SettingsDto는 앱 기능 설정(발견 엔진·연동 주소)만 다룬다.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SettingsDto {
    pub discovery_config: DiscoveryConfig,
    pub ollama_base_url: String,
    pub aimemory_endpoint: String,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<SettingsDto, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let discovery_config = store
        .get_setting(SETTING_DISCOVERY_CONFIG)
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let ollama_base_url = store
        .get_setting(SETTING_OLLAMA_URL)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(default_ollama_url);
    let aimemory_endpoint = store
        .get_setting(SETTING_AIMEMORY_ENDPOINT)
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
    Ok(SettingsDto { discovery_config, ollama_base_url, aimemory_endpoint })
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, settings: SettingsDto) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .set_setting(SETTING_DISCOVERY_CONFIG, &serde_json::to_value(&settings.discovery_config).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    store
        .set_setting(SETTING_OLLAMA_URL, &serde_json::json!(settings.ollama_base_url))
        .map_err(|e| e.to_string())?;
    store
        .set_setting(SETTING_AIMEMORY_ENDPOINT, &serde_json::json!(settings.aimemory_endpoint))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 현재 임베더가 실제 bge-m3인지, 로컬 해시 폴백인지 (설정 화면 표시용)
#[tauri::command]
pub fn get_embedder_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let ollama_url = resolve_ollama_url(&store);
    let selected = embed_select::select_embedder(&ollama_url);
    Ok(serde_json::json!({
        "model": selected.model_name,
        "is_real_model": selected.is_real_model,
        "dim": selected.dim,
    }))
}
