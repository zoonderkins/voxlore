import { create } from "zustand";

export interface AppState {
  isReady: boolean;
  appVersion: string;

  // Actions
  setReady: (ready: boolean) => void;
  setAppVersion: (version: string) => void;
}

export const useAppStore = create<AppState>()((set) => ({
  isReady: false,
  appVersion: "0.1.0",

  setReady: (ready) => set({ isReady: ready }),
  setAppVersion: (version) => set({ appVersion: version }),
}));
