import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../../components/ui";

export function PreviewWindow() {
  const [text, setText] = useState("");
  const [applying, setApplying] = useState(false);
  const [targetBundleId, setTargetBundleId] = useState<string | null>(null);

  // Pull the preview text from Rust state once mounted
  useEffect(() => {
    invoke<string | null>("get_preview_text").then((t) => {
      if (t) setText(t);
    });
    invoke<string | null>("get_preview_target_bundle_id").then((id) => {
      setTargetBundleId(id);
    });
  }, []);

  const handleApply = async () => {
    setApplying(true);
    try {
      await invoke("apply_preview_text", { text });
    } catch {
      // Window will close via Rust side
    }
  };

  const handleCancel = async () => {
    await invoke("close_preview_window");
  };

  return (
    <div className="flex h-screen flex-col bg-bg-primary p-4">
      {/* Header */}
      <div className="mb-3">
        <h2 className="text-sm font-semibold text-text-primary">Preview Transcription</h2>
        <p className="text-xs text-text-muted">Edit the text below, then Apply to insert.</p>
        <p className="mt-1 text-xs text-text-muted">
          Target Window: {targetBundleId ?? "Unknown (will use smart fallback)"}
        </p>
      </div>

      {/* Editable text area */}
      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="flex-1 resize-none rounded-lg border border-border bg-bg-input p-3 text-sm text-text-primary placeholder-text-muted focus:border-border-focus focus:outline-none"
        placeholder="Transcribed text will appear here..."
        autoFocus
      />

      {/* Action buttons */}
      <div className="mt-3 flex items-center justify-between">
        <Button variant="ghost" size="sm" onClick={handleCancel}>
          Cancel
        </Button>
        <Button
          variant="primary"
          size="sm"
          onClick={handleApply}
          disabled={!text.trim() || applying}
        >
          {applying ? "Inserting..." : "Apply"}
        </Button>
      </div>
    </div>
  );
}
