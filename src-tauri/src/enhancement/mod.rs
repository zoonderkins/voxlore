pub mod ollama;
pub mod openai_compat;

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

    match config.mode {
        EnhancementMode::FixGrammar => {
            if is_zh_tw {
                "請將以下語音轉文字內容修正為「臺灣繁體中文」，依語氣停頓補上自然標點（，。！？）；\
                 修正常見同音字與錯字，但不要改變原意、不要擴寫。\
                 只回傳修正後文字。"
                    .to_string()
            } else {
                format!(
                    "Fix grammar and spelling errors in the following {} text. \
                     Return only the corrected text, nothing else.",
                    config.language
                )
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
