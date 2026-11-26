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

fn try_embed_windows_icon() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let ico = Path::new("assets/fynexo.ico");
    println!("cargo:rerun-if-changed={}", ico.display());
    if !ico.is_file() {
        return;
    }
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };
    let Ok(abs_ico) = fs::canonicalize(ico) else {
        return;
    };
    let mut ico_str = abs_ico.to_string_lossy().replace('\\', "/");
    if let Some(s) = ico_str.strip_prefix("//?/") {
        ico_str = s.to_string();
    }
    let rc_path = Path::new(&out_dir).join("fynexo_icon.rc");
    let res_path = Path::new(&out_dir).join("fynexo_icon.res");
    let rc_content = format!("1 ICON \"{}\"\n", ico_str);
    if fs::write(&rc_path, rc_content).is_err() {
        return;
    }
    let ok = std::process::Command::new("rc.exe")
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_path)
        .arg(&rc_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok && res_path.is_file() {
        println!("cargo:rustc-link-arg-bin=fynexo={}", res_path.display());
    }
}

/// Lossless CSS minifier: strips `/* ... */` comments outside strings, trims indentation,
/// drops empty lines, and collapses redundant horizontal spaces while preserving `calc(...)`.
fn minify_css(src: &str) -> String {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut no_comments = String::with_capacity(len);
    let mut i = 0;
    let mut quote: Option<u8> = None;

    while i < len {
        let b = bytes[i];
        if let Some(q) = quote {
            let ch = src[i..].chars().next().unwrap();
            no_comments.push(ch);
            if b == b'\\' && i + 1 < len {
                i += 1;
                let next_ch = src[i..].chars().next().unwrap();
                no_comments.push(next_ch);
                i += next_ch.len_utf8();
                continue;
            } else if b == q {
                quote = None;
            }
            i += ch.len_utf8();
            continue;
        }
        if b == b'"' || b == b'\'' {
            quote = Some(b);
            no_comments.push(b as char);
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(len);
            continue;
        }
        let ch = src[i..].chars().next().unwrap();
        no_comments.push(ch);
        i += ch.len_utf8();
    }

    let mut out = String::with_capacity(no_comments.len());
    for raw_line in no_comments.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut prev_space = false;
        let mut q: Option<char> = None;
        let mut esc = false;
        for ch in trimmed.chars() {
            if let Some(qc) = q {
                out.push(ch);
                if esc {
                    esc = false;
                } else if ch == '\\' {
                    esc = true;
                } else if ch == qc {
                    q = None;
                }
                continue;
            }
            if ch == '"' || ch == '\'' {
                q = Some(ch);
                prev_space = false;
                out.push(ch);
            } else if ch == ' ' || ch == '\t' {
                if !prev_space {
                    out.push(' ');
                    prev_space = true;
                }
            } else {
                prev_space = false;
                out.push(ch);
            }
        }
        out.push('\n');
    }
    out
}

/// Provably safe line-preserving JS minifier:
/// - Tracks strings (`'...'`, `"..."`), nested template literals (`` `...${...}...` ``),
///   and multiline `/* ... */` block comments so literal contents are never altered.
/// - Outside template literals, trims leading/trailing indentation and drops blank lines
///   and pure whole-line `//` or `/* ... */` comment lines.
/// - Preserves line breaks between statements so ASI and inline regex/division tokens
///   are 100% immune to corruption.
fn minify_js_safe(src: &str) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Code,
        SingleQuote,
        DoubleQuote,
        BlockComment,
        Template,
    }

    let mut out = String::with_capacity(src.len());
    let mut mode = Mode::Code;
    let mut tpl_brace_stack: Vec<usize> = Vec::new();

    for raw_line in src.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();

        if mode == Mode::Code {
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("/*")
                && trimmed.ends_with("*/")
                && trimmed.len() >= 4
                && !trimmed[2..trimmed.len() - 2].contains("*/")
            {
                continue;
            }
        }

        let start_mode = mode;
        let line_to_scan = if start_mode == Mode::Code { trimmed } else { line };
        let chars: Vec<char> = line_to_scan.chars().collect();
        let mut i = 0;
        let mut emit_line = String::with_capacity(line_to_scan.len());

        while i < chars.len() {
            let c = chars[i];
            match mode {
                Mode::Code => {
                    if c == '\'' {
                        mode = Mode::SingleQuote;
                        emit_line.push(c);
                    } else if c == '"' {
                        mode = Mode::DoubleQuote;
                        emit_line.push(c);
                    } else if c == '`' {
                        mode = Mode::Template;
                        emit_line.push(c);
                    } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' && i == 0 {
                        mode = Mode::BlockComment;
                        i += 2;
                        continue;
                    } else if c == '{' {
                        if let Some(top) = tpl_brace_stack.last_mut() {
                            *top += 1;
                        }
                        emit_line.push(c);
                    } else if c == '}' {
                        if let Some(top) = tpl_brace_stack.last_mut() {
                            if *top == 0 {
                                tpl_brace_stack.pop();
                                mode = Mode::Template;
                            } else {
                                *top -= 1;
                            }
                        }
                        emit_line.push(c);
                    } else {
                        emit_line.push(c);
                    }
                    i += 1;
                }
                Mode::SingleQuote => {
                    emit_line.push(c);
                    if c == '\\' && i + 1 < chars.len() {
                        i += 1;
                        emit_line.push(chars[i]);
                    } else if c == '\'' {
                        mode = Mode::Code;
                    }
                    i += 1;
                }
                Mode::DoubleQuote => {
                    emit_line.push(c);
                    if c == '\\' && i + 1 < chars.len() {
                        i += 1;
                        emit_line.push(chars[i]);
                    } else if c == '"' {
                        mode = Mode::Code;
                    }
                    i += 1;
                }
                Mode::BlockComment => {
                    if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        mode = Mode::Code;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Mode::Template => {
                    emit_line.push(c);
                    if c == '\\' && i + 1 < chars.len() {
                        i += 1;
                        emit_line.push(chars[i]);
                    } else if c == '`' {
                        mode = Mode::Code;
                    } else if c == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                        i += 1;
                        emit_line.push('{');
                        tpl_brace_stack.push(0);
                        mode = Mode::Code;
                    }
                    i += 1;
                }
            }
        }

        // JS single/double quoted strings cannot span lines unless ending with `\`;
        // resetting here prevents regex literals containing quotes (e.g. /'/g) from leaking mode.
        if (mode == Mode::SingleQuote || mode == Mode::DoubleQuote) && !emit_line.ends_with('\\') {
            mode = Mode::Code;
        }

        if start_mode == Mode::Template {
            out.push_str(&emit_line);
            out.push('\n');
        } else {
            let final_trim = emit_line.trim();
            if !final_trim.is_empty() {
                out.push_str(final_trim);
                out.push('\n');
            }
        }
    }
    out
}

fn generate_minified_hud_assets() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };
    let out_path = Path::new(&out_dir);

    let js_files = [
        ("src/hud/runtime.js", "hud_runtime.min.js"),
        ("src/hud/content_agent.js", "hud_content_agent.min.js"),
        ("src/hud/icons.js", "hud_icons.min.js"),
        ("src/hud/i18n.js", "hud_i18n.min.js"),
        ("src/hud/scriptforge.js", "hud_scriptforge.min.js"),
        ("src/hud/extensions.js", "hud_extensions.min.js"),
    ];
    for (src_rel, out_name) in js_files {
        println!("cargo:rerun-if-changed={}", src_rel);
        let raw = fs::read_to_string(src_rel)
            .unwrap_or_else(|e| panic!("failed reading {}: {}", src_rel, e));
        let minified = minify_js_safe(&raw);
        fs::write(out_path.join(out_name), minified)
            .unwrap_or_else(|e| panic!("failed writing {}: {}", out_name, e));
    }

    let css_files = [
        ("src/hud/tokens.css", "hud_tokens.min.css"),
        ("src/hud/styles.css", "hud_styles.min.css"),
        ("src/hud/content_styles.css", "hud_content_styles.min.css"),
    ];
    for (src_rel, out_name) in css_files {
        println!("cargo:rerun-if-changed={}", src_rel);
        let raw = fs::read_to_string(src_rel)
            .unwrap_or_else(|e| panic!("failed reading {}: {}", src_rel, e));
        let minified = minify_css(&raw);
        fs::write(out_path.join(out_name), minified)
            .unwrap_or_else(|e| panic!("failed writing {}: {}", out_name, e));
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for key in ENV_KEYS {
        println!("cargo:rerun-if-env-changed={}", key);
    }
    load_env_file(Path::new(".env"));
    load_env_file(Path::new(".env.local"));
    try_embed_windows_icon();
    generate_minified_hud_assets();
}
