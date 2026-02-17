import { create } from "zustand";

export type RecordingStatus = "idle" | "recording" | "processing" | "previewing" | "inserting" | "error";

export interface RecordingState {
  status: RecordingStatus;
  duration: number;
  partialText: string;
  finalText: string;
  enhancedText: string;
  error: string | null;
  /** True when text was only copied to clipboard (no auto-paste). */
  clipboardOnly: boolean;

  // Actions
  setStatus: (status: RecordingStatus) => void;
  setDuration: (duration: number) => void;
  setPartialText: (text: string) => void;
  setFinalText: (text: string) => void;
  setEnhancedText: (text: string) => void;
  setError: (error: string | null) => void;
  setClipboardOnly: (value: boolean) => void;
  reset: () => void;
}

const initialState = {
  status: "idle" as RecordingStatus,
  duration: 0,
  partialText: "",
  finalText: "",
  enhancedText: "",
  error: null,
  clipboardOnly: false,
};

export const useRecordingStore = create<RecordingState>()((set) => ({
  ...initialState,

  setStatus: (status) => set({ status }),
  setDuration: (duration) => set({ duration }),
  setPartialText: (text) => set({ partialText: text }),
  setFinalText: (text) => set({ finalText: text }),
  setEnhancedText: (text) => set({ enhancedText: text }),
  setError: (error) => set({ error, status: error ? "error" : "idle" }),
  setClipboardOnly: (value) => set({ clipboardOnly: value }),
  reset: () => set(initialState),
}));
