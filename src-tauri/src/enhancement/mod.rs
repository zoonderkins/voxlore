pub mod ollama;
pub mod openai_compat;
pub mod tw_dict;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Enhancement mode determines what the LLM does with the text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnhancementMode {
    FixGrammar,
    AddPunctuation,
    AdjustTone,
    Custom,
}

/// Configuration for text enhancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementConfig {
    pub mode: EnhancementMode,
    pub language: String,
    pub model: String,
    pub custom_prompt: Option<String>,
    pub source_has_mixed_script: bool,
    pub tw_lexicon_hints: Vec<String>,
}

/// Trait for LLM-based text enhancement engines.
#[allow(async_fn_in_trait)]
pub trait EnhancementEngine: Send + Sync {
    async fn enhance(&self, text: &str, config: &EnhancementConfig) -> Result<String, AppError>;
    #[allow(dead_code)]
    fn provider_name(&self) -> &str;
}

/// Build the system prompt for enhancement based on mode.
pub fn build_enhancement_prompt(config: &EnhancementConfig) -> String {
    let lang = config.language.to_lowercase();
    let is_zh_tw = lang == "zh-tw" || lang == "zh";
    let is_zh_cn = lang == "zh-cn";

    match config.mode {
        EnhancementMode::FixGrammar => {
            if is_zh_tw {
                let mut prompt =
                    "請將以下語音轉文字內容修正為「臺灣繁體中文」，依語氣停頓補上自然標點（，。！？）；\
                 修正常見同音字與錯字，但不要改變原意、不要擴寫。\
                 用詞與字形請優先遵循教育部《重編國語辭典修訂本》與《異體字字典》，\
                 標點請優先遵循教育部《重訂標點符號手冊》，\
                 若涉及台語常用詞可參照教育部《臺灣台語常用詞辭典》。\
                 只回傳修正後文字。"
                        .to_string();
                if config.source_has_mixed_script {
                    prompt.push_str(
                        "若原文含中英混說，英文品牌名、產品名、API 名稱、程式碼片段請保留原文，不翻譯、不改大小寫。",
                    );
                }
                if !config.tw_lexicon_hints.is_empty() {
                    prompt.push_str("若語句符合以下台灣常見口語對照，請優先使用對應詞彙：");
                    prompt.push_str(&config.tw_lexicon_hints.join("；"));
                    prompt.push('。');
                }
                prompt
            } else if is_zh_cn {
                let mut prompt =
                    "请将以下语音转文字内容修正为「简体中文」，按语气停顿补上自然标点（，。！？）；\
                 修正常见同音字与错字，但不要改变原意、不要扩写。\
                 只返回修正后的文本。"
                        .to_string();
                if config.source_has_mixed_script {
                    prompt.push_str(
                        "若原文含中英混说，英文品牌名、产品名、API 名称、代码片段请保留原文，不翻译、不改大小写。",
                    );
                }
                prompt
            } else {
                let mut prompt = format!(
                    "Fix grammar and spelling errors in the following {} text. \
                     Return only the corrected text, nothing else.",
                    config.language
                );
                if config.source_has_mixed_script {
                    prompt.push_str(
                        " If the input mixes languages, preserve brand names, product names, API names, and code snippets exactly as written.",
                    );
                }
                prompt
            }
        }
        EnhancementMode::AddPunctuation => format!(
            "Add proper punctuation to the following {} text from speech recognition. \
             Return only the punctuated text, nothing else.",
            config.language
        ),
        EnhancementMode::AdjustTone => format!(
            "Adjust the tone of the following {} text to be more professional and polished. \
             Return only the adjusted text, nothing else.",
            config.language
        ),
        EnhancementMode::Custom => config
            .custom_prompt
            .clone()
            .unwrap_or_else(|| "Improve the following text. Return only the improved text.".into()),
    }
}
