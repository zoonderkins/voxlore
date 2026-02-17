import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Card, Button, ProgressBar } from "./ui";
import {
  listVoskModels,
  listDownloadedVoskModels,
  downloadVoskModel,
  loadVoskModel,
  getVoskStatus,
  type VoskModel,
  type VoskModelStatus,
  type DownloadProgress,
} from "../lib/tauri";

export function ModelManager() {
  const [models, setModels] = useState<VoskModel[]>([]);
  const [downloaded, setDownloaded] = useState<string[]>([]);
  const [status, setStatus] = useState<VoskModelStatus | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("model:download-progress", (event) => {
      setProgress(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function loadData() {
    try {
      const [modelList, downloadedList, voskStatus] = await Promise.all([
        listVoskModels(),
        listDownloadedVoskModels(),
        getVoskStatus(),
      ]);
      setModels(modelList);
      setDownloaded(downloadedList);
      setStatus(voskStatus);
    } catch {
      // Models may not be available in dev mode
    }
  }

  async function handleDownload(modelId: string) {
    setDownloading(modelId);
    setError(null);
    setProgress(null);
    try {
      await downloadVoskModel(modelId);
      setDownloaded((prev) => [...prev, modelId]);
    } catch (e) {
      setError(String(e));
    } finally {
      setDownloading(null);
      setProgress(null);
    }
  }

  async function handleLoad(modelId: string) {
    setError(null);
    try {
      const newStatus = await loadVoskModel(modelId);
      setStatus(newStatus);
    } catch (e) {
      setError(String(e));
    }
  }

  const formatSize = (mb: number) => (mb >= 1000 ? `${(mb / 1000).toFixed(1)} GB` : `${mb} MB`);

  return (
    <div className="flex flex-col gap-3">
      <span className="text-sm font-medium text-text-secondary">Vosk Models</span>

      {error && (
        <div className="rounded-lg bg-error/10 px-3 py-2 text-xs text-error">{error}</div>
      )}

      {downloading && progress && (
        <Card padding="sm">
          <ProgressBar
            value={progress.percent}
            label={progress.stage === "extracting" ? "Extracting..." : "Downloading..."}
            showPercent
          />
        </Card>
      )}

      <div className="flex flex-col gap-2">
        {models.map((model) => {
          const isDownloaded = downloaded.includes(model.id);
          const isLoaded = status?.modelId === model.id && status?.loaded;
          const isDownloading = downloading === model.id;

          return (
            <Card key={model.id} padding="sm" className="flex items-center justify-between">
              <div className="flex flex-col gap-0.5">
                <span className="text-sm font-medium text-text-primary">{model.name}</span>
                <span className="text-xs text-text-muted">
                  {model.description} ({formatSize(model.size_mb)})
                </span>
              </div>
              <div className="flex items-center gap-2">
                {isLoaded ? (
                  <span className="text-xs font-medium text-success">Active</span>
                ) : isDownloaded ? (
                  <Button variant="secondary" size="sm" onClick={() => handleLoad(model.id)}>
                    Load
                  </Button>
                ) : (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => handleDownload(model.id)}
                    disabled={isDownloading}
                  >
                    {isDownloading ? "..." : "Download"}
                  </Button>
                )}
              </div>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
