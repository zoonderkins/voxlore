use ferrous_opencc::{config::BuiltinConfig, OpenCC};

/// Convert Simplified Chinese text to Traditional Chinese.
/// Vosk models typically output Simplified; this converts for zh-TW users.
pub fn simplified_to_traditional(text: &str) -> String {
    match OpenCC::from_config(BuiltinConfig::S2t) {
        Ok(cc) => cc.convert(text),
        Err(_) => text.to_string(), // Fallback: return original
    }
}

/// Check if a language code indicates Traditional Chinese.
pub fn needs_s2t_conversion(language: &str) -> bool {
    let lang = language.to_lowercase();
    lang == "zh-tw" || lang == "zh_tw" || lang == "zh-hant"
}
