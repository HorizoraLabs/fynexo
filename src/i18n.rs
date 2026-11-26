//! Centralized multi-language locale normalization and metadata.
//!
//! Replaces ad-hoc `!lang.eq_ignore_ascii_case("en")` checks so adding third or
//! fourth languages (e.g. `zh-TW`, `ja`, `ko`) never misroutes non-English
//! locales into Simplified Chinese.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleMeta {
    pub code: &'static str,
    pub native_name: &'static str,
    pub english_name: &'static str,
}

/// Ordered list of officially shipped UI locales.
pub const SUPPORTED_LOCALES: &[LocaleMeta] = &[
    LocaleMeta {
        code: "zh",
        native_name: "简体中文",
        english_name: "Simplified Chinese",
    },
    LocaleMeta {
        code: "en",
        native_name: "English",
        english_name: "English",
    },
];

/// Normalize an arbitrary locale tag (`"zh"`, `"zh-CN"`, `"zh_SG"`, `"en-US"`,
/// `"ja"`, `""`) to a canonical supported locale code.
///
/// Empty input defaults to `"zh"` (matching historical Fynexo defaults).
/// Unrecognized non-Chinese tags fall back to `"en"`.
pub fn normalize_locale(raw: &str) -> &'static str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "zh";
    }
    let norm = trimmed.to_ascii_lowercase().replace('_', "-");
    for meta in SUPPORTED_LOCALES {
        if norm == meta.code {
            return meta.code;
        }
    }
    if norm == "zh" || norm.starts_with("zh-") {
        return "zh";
    }
    if norm == "en" || norm.starts_with("en-") {
        return "en";
    }
    "en"
}

/// True only when the normalized locale belongs to the Chinese family (`zh*`).
#[inline]
pub fn is_zh_locale(raw: &str) -> bool {
    normalize_locale(raw).starts_with("zh")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_normalization_handles_variants_and_fallbacks() {
        assert_eq!(normalize_locale(""), "zh");
        assert_eq!(normalize_locale("zh"), "zh");
        assert_eq!(normalize_locale("zh-CN"), "zh");
        assert_eq!(normalize_locale("zh_SG"), "zh");
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("en-US"), "en");
        assert_eq!(normalize_locale("en_GB"), "en");
        // Non-English, non-Chinese locales must fall back to English, never Chinese.
        assert_eq!(normalize_locale("ja"), "en");
        assert_eq!(normalize_locale("ko-KR"), "en");
        assert_eq!(normalize_locale("fr"), "en");

        assert!(is_zh_locale("zh"));
        assert!(is_zh_locale("zh-CN"));
        assert!(is_zh_locale(""));
        assert!(!is_zh_locale("en"));
        assert!(!is_zh_locale("en-US"));
        assert!(!is_zh_locale("ja"));
    }
}
