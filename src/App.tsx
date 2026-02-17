import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "./stores/useSettingsStore";
import { useToastStore } from "./stores/useToastStore";
import i18n from "./i18n";
import { useRecordingPipeline } from "./hooks/useRecordingPipeline";
import { SetupWizard } from "./features/setup/SetupWizard";
import { SettingsPage } from "./pages/settings/SettingsPage";
import { FloatingWidget } from "./windows/floating/FloatingWidget";
import { PreviewWindow } from "./windows/preview/PreviewWindow";
import { ThemeToggle } from "./components/ThemeToggle";
import { ToastContainer } from "./components/ui";

export default function App() {
  const setupCompleted = useSettingsStore((s) => s.setupCompleted);
  const themeMode = useSettingsStore((s) => s.themeMode);
  const uiLanguage = useSettingsStore((s) => s.uiLanguage);
  const rightClickDevtools = useSettingsStore((s) => s.rightClickDevtools);
  const { toasts, dismissToast } = useToastStore();
  const [windowLabel, setWindowLabel] = useState<string>("main");

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", themeMode);
  }, [themeMode]);

  useEffect(() => {
    if (i18n.language !== uiLanguage) {
      void i18n.changeLanguage(uiLanguage);
    }
  }, [uiLanguage]);

  useEffect(() => {
    document.documentElement.setAttribute("data-window", windowLabel);
    document.body.setAttribute("data-window", windowLabel);
  }, [windowLabel]);

  useEffect(() => {
    try {
      const label = getCurrentWindow().label;
      setWindowLabel(label);
    } catch {
      // Fallback to main if Tauri API is not available
    }
  }, []);

  useEffect(() => {
    if (!rightClickDevtools) return;
    const handler = (event: MouseEvent) => {
      event.preventDefault();
      try {
        void invoke("open_devtools", { windowLabel });
      } catch {
        // ignore
      }
    };
    window.addEventListener("contextmenu", handler);
    return () => window.removeEventListener("contextmenu", handler);
  }, [rightClickDevtools]);

  // Handle recording pipeline events (only in main window)
  useRecordingPipeline();

  if (windowLabel === "floating") {
    return <FloatingWidget />;
  }

  if (windowLabel === "preview") {
    return <PreviewWindow />;
  }

  return (
    <div className="h-screen bg-bg-primary">
      <div className="absolute right-4 top-4 z-50">
        <ThemeToggle />
      </div>
      {setupCompleted ? <SettingsPage /> : <SetupWizard />}
      <ToastContainer toasts={toasts} onDismiss={dismissToast} />
    </div>
  );
}
