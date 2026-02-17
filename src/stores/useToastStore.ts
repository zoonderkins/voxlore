import { create } from "zustand";
import type { ToastData } from "../components/ui/Toast";

interface ToastState {
  toasts: ToastData[];
  addToast: (message: string, type: ToastData["type"]) => void;
  dismissToast: (id: string) => void;
}

let toastId = 0;

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  addToast: (message, type) => {
    const id = `toast-${++toastId}`;
    set((state) => ({
      toasts: [...state.toasts, { id, message, type }],
    }));
  },
  dismissToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }));
  },
}));
