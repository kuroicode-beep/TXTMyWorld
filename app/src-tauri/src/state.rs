// src-tauri/src/state.rs — Tauri 전역 상태: 앱 SQLite 저장소 핸들
// 벡터 인덱스는 상태로 들고 있지 않는다 — run_discovery 호출마다 persisted embeddings로부터
// 새로 조립한다(§pipeline.rs). 이렇게 하면 Send/Sync 경계가 Mutex<Store> 하나로 단순해진다.

use std::sync::Mutex;

use txtmyworld_core::store::Store;

pub struct AppState {
    pub store: Mutex<Store>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        Self { store: Mutex::new(store) }
    }
}
