import { useState, useEffect, useRef } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function FloatingWidget() {
  const [status, setStatus] = useState<"recording" | "processing">("recording");
  const [statusMessage, setStatusMessage] = useState("Listening...");
  const [progressDots, setProgressDots] = useState("");
  const [duration, setDuration] = useState(0);
  const [audioLevel, setAudioLevel] = useState(0);
  const intervalRef = useRef<ReturnType<typeof setInterval>>(undefined);

  // Timer
  useEffect(() => {
    intervalRef.current = setInterval(() => {
      setDuration((d) => d + 1);
    }, 1000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  // Listen for recording status changes
  useEffect(() => {
    const unlisten = listen<{ status: string; message?: string }>("recording:status", (event) => {
      const { status: newStatus, message } = event.payload;
      if (message) {
        setStatusMessage(message);
      }
      if (newStatus === "processing") {
        setStatus("processing");
        if (!message) {
          setStatusMessage("Processing...");
        }
        if (intervalRef.current) clearInterval(intervalRef.current);
      }
      if (newStatus === "recording") {
        setStatusMessage("Listening...");
      }
      if (newStatus === "done" || newStatus === "error") {
        getCurrentWindow().close().catch(() => {});
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    if (status !== "processing") {
      setProgressDots("");
      return;
    }
    const timer = setInterval(() => {
      setProgressDots((prev) => (prev.length >= 3 ? "" : `${prev}.`));
    }, 400);
    return () => clearInterval(timer);
  }, [status]);

  // Listen for audio level updates (0.0 - 1.0)
  useEffect(() => {
    const unlisten = listen<{ level: number }>("recording:audio-level", (event) => {
      setAudioLevel(event.payload.level);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleStop = async () => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    await emit("recording:stop", {});
    getCurrentWindow().close().catch(() => {});
  };

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  const processingSubtitle = `Cloud processing${progressDots}`;

  return (
    <div
      className="flex h-screen w-screen items-center justify-center"
      style={{ background: "transparent" }}
      data-tauri-drag-region
    >
      <div className="flex items-center gap-3 rounded-full bg-[#1a1a2e]/95 px-4 py-2.5 shadow-2xl backdrop-blur-sm border border-[#2a2a45]/50">
        {/* Waveform bars — driven by audio level */}
        <div className="flex items-center gap-[3px]">
          {[0, 1, 2, 3, 4].map((i) => (
            <WaveBar key={i} index={i} level={audioLevel} active={status === "recording"} />
          ))}
        </div>

        {/* Status text */}
        <div className="flex min-w-0 flex-1 flex-col leading-tight">
          <span className="text-xs font-medium text-white">
            {status === "recording" ? "Recording" : "Processing..."}
          </span>
          <span className="truncate text-[10px] text-[#9ca3af]" title={statusMessage}>
            {status === "recording" ? formatTime(duration) : processingSubtitle}
          </span>
        </div>

        {/* Stop button */}
        <button
          onClick={handleStop}
          className="ml-1 flex h-6 w-6 items-center justify-center rounded-full bg-red-500/80 hover:bg-red-500 transition-colors cursor-pointer"
          title="Stop recording (Option+Space)"
        >
          <div className="h-2 w-2 rounded-[1px] bg-white" />
        </button>
      </div>
    </div>
  );
}

/** Wave bar driven by actual audio level, not CSS animation. */
function WaveBar({ index, level, active }: { index: number; level: number; active: boolean }) {
  const MIN_HEIGHT = 4;
  const MAX_HEIGHT = 24;
  // Each bar has a slightly different response curve for visual variety
  const multipliers = [0.6, 1.0, 0.8, 0.95, 0.7];

  const height = active
    ? MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * level * (multipliers[index] ?? 1)
    : MIN_HEIGHT;

  return (
    <div
      className="w-[3px] rounded-full bg-white/80"
      style={{
        height: `${Math.max(MIN_HEIGHT, height)}px`,
        transition: "height 80ms ease-out",
      }}
    />
  );
}
