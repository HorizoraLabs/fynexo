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
