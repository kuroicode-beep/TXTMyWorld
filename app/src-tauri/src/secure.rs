// src-tauri/src/secure.rs — 소스 페어링 토큰의 OS 보안 저장소 왕복 (Windows Credential Manager 등)
// 원칙: 토큰 평문은 SQLite에 저장하지 않는다. DB에는 해시(지문)만 남기고, 실제 값은 keyring에만 둔다.

use keyring::Entry;
use sha2::{Digest, Sha256};

const SERVICE: &str = "com.svil.txtmyworld";

/// 소스 페어링 토큰을 OS 보안 저장소에 저장한다
pub fn set_token(source: &str, token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, source).map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())
}

/// 저장된 토큰을 조회한다 (없으면 None)
pub fn get_token(source: &str) -> Option<String> {
    Entry::new(SERVICE, source).ok()?.get_password().ok()
}

/// 토큰을 삭제한다 (페어링 해제). 이미 없는 항목이어도 오류로 보지 않는다.
pub fn delete_token(source: &str) -> Result<(), String> {
    if let Ok(entry) = Entry::new(SERVICE, source) {
        let _ = entry.delete_credential();
    }
    Ok(())
}

/// DB에 남길 지문(해시) — 원문 유추 불가, "토큰이 설정되어 있는지" 표시용
pub fn fingerprint(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic_and_distinct() {
        let a = fingerprint("token-abc");
        let b = fingerprint("token-abc");
        let c = fingerprint("token-xyz");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64, "SHA-256 hex는 64자");
    }
}
