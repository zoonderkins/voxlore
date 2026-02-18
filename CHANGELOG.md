# Changelog

## [0.1.4] - 2026-02-18

### Added

- **Language-aware prompt tuning (EN/JA)** — Added dedicated enhancement prompts for English and Japanese, including disfluency cleanup rules (e.g. `um/uh/you know`, `えー/あの/その/なんか`) while preserving meaning.

### Changed

- **STT prompt switching by language** — OpenRouter Audio and OpenAI Whisper now use explicit `zh-TW` / `ja` / `en` transcription prompt variants.
- **Speech language label clarity** — Provider speech-language option `zh` now displays as `繁體中文（台灣）` (and aligned equivalents in `en` / `zh-CN` / `ja` locales).
- **macOS paste encoding robustness** — Clipboard read/write for auto-paste now enforces UTF-8 locale to avoid Traditional Chinese mojibake in direct-insert flow.
- **Version bump and release artifacts** — Updated app and website version display and DMG target to `v0.1.4`.

## [0.1.3] - 2026-02-18

### Added

- **Japanese UI language support** — Added app locale file `ja` and wired it into i18n resources and language selectors.
- **Website Japanese language option** — Added `ja` option and full Japanese marketing copy in `website/index.html`.

### Changed

- **Settings translation completeness (zh-TW / zh-CN / en)** — Replaced remaining hardcoded English labels in Voice Provider / Enhancement sections with i18n keys.
- **Version bump and release artifacts** — Updated app/website version display and DMG download target to `v0.1.3`.

## [0.1.2] - 2026-02-18

### Added

- **Taiwan lexicon normalization for enhancement** — Added built-in Taiwan slang/youth wording dictionary (`qq/哭哭`, `Y2K`, `Z世代`, `I人/E人`, etc.) with prompt hints and post-processing normalization.
- **macOS file logger** — Added persistent runtime log output at `~/Library/Logs/Voxlore/run.log` while preserving terminal output.

### Changed

- **Enhancement prompt quality** — Improved zh-TW/zh-CN prompt behavior, mixed-language handling, and Taiwanese writing conventions alignment.
- **Settings health-check cadence** — Voice/Enhancement provider auto health-check now runs once on open and then every 1 hour (previously every 20 seconds).
- **API key localization completeness** — Added missing `settings.apiKey.*` translation keys in `en`, `zh-TW`, `zh-CN` to prevent raw i18n key text in UI.
- **Website messaging refresh** — Updated `website/index.html` with intent-first positioning, v0.1.2 download link, and refreshed multilingual copy.

## [0.1.0] - 2026-02-18

### Changed

- **Branding update to Voxlore** — Product naming and bundle identifier updated to `Voxlore` / `app.voxlore`.
- **Default STT provider/model update** — Voice STT now defaults to `openrouter` with model `google/gemini-3-flash-preview`.
- **Provider selector layout update** — Provider buttons now use a stable grid layout to avoid uneven wrapping in narrow windows.
- **UI language behavior update** — `uiLanguage` now actively applies via `i18n.changeLanguage(...)` instead of only persisting settings.
- **macOS crash on Option+Space** — Fixed floating widget crash caused by AppKit window operations from `tokio-runtime-worker`. `show_floating_widget` / `hide_floating_widget` now execute on the main thread via `app.run_on_main_thread(...)`.
- **Floating processing widget overflow** — Fixed processing text overflow/truncation by using a compact single-line subtitle and strict truncation behavior.
- **OpenRouter model empty-state inconsistency** — Existing settings with `openrouter` + empty model now backfill to `google/gemini-3-flash-preview` on rehydrate.
- **Setup language labeling ambiguity** — Setup now distinguishes between `UI Language` and `Speech Language (STT)`, preventing interpretation mismatch.
- **Localization completeness for settings panel** — Added missing translation keys for Recording / Timeout / Debug / DevTools labels and descriptions across `en`, `zh-TW`, `zh-CN`.
- **Microphone permission dialog not appearing** — Added `NSMicrophoneUsageDescription` to Info.plist via Tauri `infoPlist` config. Without this key, macOS silently refuses to show the permission dialog. Also fixed `request_microphone()` to call `stream.play()` (cpal `build_input_stream` alone doesn't start audio capture, so macOS never intercepts the access). Added fallback to open System Settings if stream setup fails.
- **Download progress bar flickering** — Progress bar no longer bounces between 40-60% during model downloads. Added Rust-side throttle (emit at most every 100ms or 1% change) and replaced CSS transition with direct width updates to prevent animation conflicts.
- **macOS permission prompts not appearing** — "Grant Access" buttons in the Permissions setup step now trigger real macOS system dialogs for Microphone (`AVCaptureDevice`) and Accessibility (`AXIsProcessTrusted`) permissions instead of only toggling frontend state. When microphone is denied, opens System Settings for manual grant.
- **Floating window not appearing** — Toggle in Settings now correctly calls `show_floating_widget` / `hide_floating_widget` Tauri commands. Added `useEffect` to bridge the Zustand setting to the Tauri window lifecycle. Floating window now has transparent background via `cocoa` crate (`NSWindow.setBackgroundColor_(clearColor)`).
- **Manual connection test button (Voice + Enhancement)** — Added `測試連線` action beside provider health status. Users can now verify endpoint/key/model on demand and get immediate success/failure toast feedback.
- **Cloud request observability upgrade** — Added request-level debug logs for health checks, STT, and enhancement (`request_id`, upstream request id, HTTP status, latency ms), while avoiding sensitive key output.
