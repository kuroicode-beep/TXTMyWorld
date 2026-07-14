// src-tauri/src/lib.rs — Tauri 앱 진입점: 상태 초기화(앱 데이터 디렉터리의 SQLite)와 IPC 커맨드 등록.
// 코어(txtmyworld-core)의 RC 데이터 모델·트레이트는 변경하지 않고, 이 레이어에서만 배선한다.

mod commands;
mod dto;
mod embed_select;
mod feedback_client;
mod pipeline;
mod registry;
mod secure;
mod state;

use tauri::Manager;
use txtmyworld_core::store::Store;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("앱 데이터 디렉터리를 확인할 수 없습니다");
            std::fs::create_dir_all(&data_dir).expect("앱 데이터 디렉터리 생성 실패");
            let db_path = data_dir.join("txtmyworld.sqlite");
            let store = Store::open(db_path.to_str().expect("DB 경로 인코딩 오류"))
                .expect("SQLite 저장소 초기화 실패");
            app.manage(AppState::new(store));

            // 창 제목에 버전 상시 표시 (CLAUDE.md §3 규칙)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&format!("TXTMyWorld v{}", txtmyworld_core::APP_VERSION));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::open_deeplink,
            commands::pair_source,
            commands::unpair_source,
            commands::list_sources,
            commands::check_source_health,
            commands::sync_source,
            commands::sync_all,
            commands::seed_demo_data,
            commands::connect_all_sources,
            commands::run_discovery,
            commands::list_discoveries,
            commands::dismiss_discovery,
            commands::adopt_discovery,
            commands::list_topic_cards,
            commands::update_topic_card,
            commands::delete_topic_card,
            commands::send_feedback,
            commands::get_feedback_history,
            commands::get_settings,
            commands::set_settings,
            commands::get_embedder_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
