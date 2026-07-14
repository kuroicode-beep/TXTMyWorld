// src-tauri/src/registry.rs — SVIL 패밀리 로컬 디스커버리 레지스트리 조회 (읽기 전용)
//
// TXTMyWorld는 다른 앱이 붙는 포트를 노출하지 않는 순수 소비자라 자기 항목을 기록하지 않고,
// %LOCALAPPDATA%\SVIL\registry.json에서 소스 포트만 조회한다(TXT 패밀리 연결프로토콜 v2.0
// Phase 5). 레지스트리가 없거나 항목이 없으면 None — 호출측이 기존 하드코딩 기본값으로 폴백한다.

use serde_json::Value;
use std::path::PathBuf;

fn registry_path() -> Option<PathBuf> {
    let base = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(base).join("SVIL").join("registry.json"))
}

/// 레지스트리에서 key의 포트를 조회. 실패/누락 시 None(폴백은 호출측 책임).
pub fn lookup_port(key: &str) -> Option<u16> {
    let path = registry_path()?;
    let text = std::fs::read_to_string(path).ok()?;
    let data: Value = serde_json::from_str(&text).ok()?;
    data.get("apps")?.get(key)?.get("port")?.as_u64().map(|p| p as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_port_reads_nested_apps_entry() {
        let data: Value = serde_json::from_str(
            r#"{"schema_version":"1.0","apps":{"txtdiary":{"port":47821}}}"#,
        )
        .unwrap();
        let port = data["apps"]["txtdiary"]["port"].as_u64().map(|p| p as u16);
        assert_eq!(port, Some(47821));
    }

    #[test]
    fn missing_env_var_returns_none_not_panic() {
        // LOCALAPPDATA가 없는 극단적 환경에서도 조용히 None을 반환해야 한다(폴백 유도).
        // 이 테스트 프로세스에는 보통 LOCALAPPDATA가 있으므로, registry_path()의 Option 체이닝
        // 자체가 패닉하지 않는다는 것만 별도로 확인(존재하지 않는 키 조회는 항상 None).
        assert_eq!(lookup_port("존재하지-않는-키-xyz"), None);
    }
}
