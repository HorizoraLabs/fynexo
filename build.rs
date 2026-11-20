use std::fs;
use std::path::Path;

const ENV_KEYS: [&str; 4] = [
    "DATABASE_URL",
    "DATABASE_ANON_KEY",
    "DATABASE_SCHEMA",
    "DATABASE_TABLE",
];

fn is_postgres_uri(val: &str) -> bool {
    let lower = val.to_ascii_lowercase();
    lower.starts_with("postgres://") || lower.starts_with("postgresql://")
}

fn decode_base64url(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    for b in input.trim().trim_end_matches('=').bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => return None,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1u32 << bits) - 1;
        }
    }
    Some(out)
}

fn is_service_role_jwt(token: &str) -> bool {
    let mut parts = token.trim().split('.');
    let Some(_hdr) = parts.next() else {
        return false;
    };
    let Some(payload_b64) = parts.next() else {
        return false;
    };
    let Some(decoded) = decode_base64url(payload_b64) else {
        return false;
    };
    let compact: String = String::from_utf8_lossy(&decoded)
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    compact.contains("\"role\":\"service_role\"")
}

fn load_env_file(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((k, v)) = trimmed.split_once('=') else {
            continue;
        };
        let key = k.trim();
        if !ENV_KEYS.contains(&key) {
            continue;
        }
        if std::env::var_os(key).is_some() {
            continue;
        }
        let val = v.trim().trim_matches(|c| c == '"' || c == '\'');
        if key == "DATABASE_URL" && is_postgres_uri(val) {
            println!("cargo:warning=DATABASE_URL in {} is a Postgres connection URI; skipping compile-time injection (use your Supabase HTTPS Project URL https://<ref>.supabase.co instead).", path.display());
            continue;
        }
        if key == "DATABASE_ANON_KEY" && is_service_role_jwt(val) {
            println!("cargo:warning=DATABASE_ANON_KEY in {} is a privileged 'service_role' secret key; skipping compile-time injection (use the public 'anon' key instead).", path.display());
            continue;
        }
        println!("cargo:rustc-env={}={}", key, val);
    }
}

fn main() {
    for key in ENV_KEYS {
        println!("cargo:rerun-if-env-changed={}", key);
    }
    load_env_file(Path::new(".env"));
    load_env_file(Path::new(".env.local"));
}
