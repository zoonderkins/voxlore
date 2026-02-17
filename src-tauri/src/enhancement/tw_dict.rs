/// 台灣常見口語 / 梗圖 / 世代語彙對照字典。
/// 支援 emoji 與一般文字正規化。
#[derive(Clone, Copy)]
pub struct TwLexiconEntry {
    pub aliases: &'static [&'static str],
    pub replacement: &'static str,
}

const TW_LEXICON: &[TwLexiconEntry] = &[
    TwLexiconEntry {
        aliases: &["qq", "QQ", "哭哭"],
        replacement: "😭",
    },
    TwLexiconEntry {
        aliases: &["笑死", "笑爛", "XD", "xD"],
        replacement: "😂",
    },
    TwLexiconEntry {
        aliases: &["傻眼", "無言"],
        replacement: "🙄",
    },
    TwLexiconEntry {
        aliases: &["火大", "氣死"],
        replacement: "😤",
    },
    TwLexiconEntry {
        aliases: &["愛心"],
        replacement: "❤️",
    },
    TwLexiconEntry {
        aliases: &["Y2K", "y2k"],
        replacement: "千禧年復古風格",
    },
    TwLexiconEntry {
        aliases: &["Z世代", "Gen Z", "gen z"],
        replacement: "Z世代",
    },
    TwLexiconEntry {
        aliases: &["I人", "i人"],
        replacement: "偏內向人格",
    },
    TwLexiconEntry {
        aliases: &["E人", "e人"],
        replacement: "偏外向人格",
    },
    TwLexiconEntry {
        aliases: &["破防"],
        replacement: "情緒被戳中",
    },
    TwLexiconEntry {
        aliases: &["不EY", "不ey"],
        replacement: "不意外",
    },
    TwLexiconEntry {
        aliases: &["母湯", "母湯喔"],
        replacement: "不行",
    },
    TwLexiconEntry {
        aliases: &["踹共"],
        replacement: "出來講",
    },
    TwLexiconEntry {
        aliases: &["住海邊"],
        replacement: "管太多",
    },
    TwLexiconEntry {
        aliases: &["最頂"],
        replacement: "最強",
    },
];

fn is_zh_language(language: &str) -> bool {
    language.to_ascii_lowercase().starts_with("zh")
}

/// 根據輸入內容挑選提示詞，避免把整份字典塞進 prompt。
pub fn collect_relevant_hints(text: &str, language: &str) -> Vec<String> {
    if !is_zh_language(language) {
        return Vec::new();
    }

    let mut hints = Vec::new();
    for entry in TW_LEXICON {
        if entry.aliases.iter().any(|alias| text.contains(alias)) {
            hints.push(format!(
                "{} -> {}",
                entry.aliases.join("/"),
                entry.replacement
            ));
        }
    }
    hints
}

/// 在增強結果上套用字典替換，確保常見口語可穩定正規化。
pub fn apply_tw_lexicon_dict(text: &str, language: &str) -> String {
    if !is_zh_language(language) {
        return text.to_string();
    }

    let mut output = text.to_string();
    for entry in TW_LEXICON {
        for alias in entry.aliases {
            output = output.replace(alias, entry.replacement);
        }
    }
    output
}
