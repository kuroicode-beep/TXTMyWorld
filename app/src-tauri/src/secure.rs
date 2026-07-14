// src-tauri/src/secure.rs — 소스 페어링 토큰의 OS 보안 저장소 왕복 (Windows Credential Manager 등)
// 원칙: 토큰 평문은 SQLite에 저장하지 않는다. DB에는 해시(지문)만 남기고, 실제 값은 keyring에만 둔다.

use keyring::Entry;
use sha2::{Digest, Sha256};

const SERVICE: &str = "com.svil.txtmyworld";
/// TXT 패밀리 공용 토큰 저장 서비스명 — TXTSpace UI·hub가 소스 페어링 토큰을 여기에 둔다
/// (TXTSpace hub adapters.rs의 KEYRING_SERVICE와 동일). 이미 페어링된 소스 토큰을 재사용하기 위함.
const FAMILY_SERVICE: &str = "TXTSpace";

/// 소스 페어링 토큰을 OS 보안 저장소에 저장한다
pub fn set_token(source: &str, token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, source).map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())
}

/// TXTMyWorld 자신이 저장한 토큰만 조회한다 (없으면 None)
pub fn get_token(source: &str) -> Option<String> {
    Entry::new(SERVICE, source).ok()?.get_password().ok()
}

/// TXT 패밀리 공용(TXTSpace) 저장소의 소스 토큰을 조회한다.
/// TXTSpace가 이미 페어링해 둔 토큰을 그대로 재사용할 수 있다 — 소스 앱은 토큰이 유효하면
/// 누가 제시하든 받아주므로(단순 해시 검증), TXTSpace와 TXTMyWorld가 같은 토큰을 동시에 써도 된다.
pub fn get_family_token(source: &str) -> Option<String> {
    Entry::new(FAMILY_SERVICE, source).ok()?.get_password().ok()
}

/// 소스 연결에 쓸 토큰을 해석한다: TXTMyWorld 자체 페어링 우선, 없으면 TXTSpace 공유 토큰 폴백.
/// 반환: (토큰, 공유토큰인지 여부).
pub fn resolve_token(source: &str) -> Option<(String, bool)> {
    if let Some(t) = get_token(source) {
        return Some((t, false));
    }
    get_family_token(source).map(|t| (t, true))
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
