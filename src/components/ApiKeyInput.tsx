import { useState, useEffect } from "react";
import { Input, Button } from "./ui";
import { saveApiKey, hasApiKey, deleteApiKey } from "../lib/tauri";
import { debugUiEvent } from "../lib/debug";

interface ApiKeyInputProps {
  provider: string;
  getKeyUrl?: string;
}

export function ApiKeyInput({ provider, getKeyUrl }: ApiKeyInputProps) {
  const [key, setKey] = useState("");
  const [saved, setSaved] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    hasApiKey(provider)
      .then((exists) => {
        setSaved(exists);
        void debugUiEvent("apikey/check", { provider, exists });
      })
      .catch(() => {
        setSaved(false);
        void debugUiEvent("apikey/check_error", { provider });
      });
  }, [provider]);

  const handleSave = async () => {
    if (!key.trim()) return;
    setLoading(true);
    void debugUiEvent("apikey/save_attempt", { provider, key });
    try {
      await saveApiKey(provider, key.trim());
      setSaved(true);
      setKey("");
      void debugUiEvent("apikey/save_success", { provider });
    } catch {
      // Error handled by parent
      void debugUiEvent("apikey/save_error", { provider });
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    setLoading(true);
    void debugUiEvent("apikey/delete_attempt", { provider });
    try {
      await deleteApiKey(provider);
      setSaved(false);
      void debugUiEvent("apikey/delete_success", { provider });
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    setKey("");
    void debugUiEvent("apikey/input_cancel", { provider });
  };

  if (saved) {
    return (
      <div className="flex flex-col gap-1.5">
        <label className="text-sm font-medium text-text-secondary">API Key</label>
        <div className="flex items-center gap-2">
          <div className="flex-1 rounded-lg border border-border bg-bg-input px-3 py-2 font-mono text-sm text-text-muted">
            ••••••••••••
          </div>
          <Button variant="ghost" size="sm" onClick={handleDelete} disabled={loading}>
            Remove
          </Button>
        </div>
      </div>
    );
  }

  return (
    <Input
      label="API Key"
      type="password"
      placeholder="Enter API key..."
      value={key}
      onChange={(e) => setKey(e.target.value)}
      onKeyDown={(e) => e.key === "Enter" && handleSave()}
      rightElement={
        <div className="flex items-center gap-2">
          {key.trim() && (
            <>
              <Button variant="ghost" size="sm" onClick={handleCancel} disabled={loading}>
                Cancel
              </Button>
              <Button variant="primary" size="sm" onClick={handleSave} disabled={loading}>
                Save
              </Button>
            </>
          )}
          {getKeyUrl && (
            <a
              href={getKeyUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-accent hover:underline whitespace-nowrap"
            >
              Get API Key
            </a>
          )}
        </div>
      }
    />
  );
}
