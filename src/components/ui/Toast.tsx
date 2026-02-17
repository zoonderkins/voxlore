import { useEffect } from "react";

export interface ToastData {
  id: string;
  message: string;
  type: "success" | "error" | "info";
}

interface ToastProps {
  toast: ToastData;
  onDismiss: (id: string) => void;
}

function Toast({ toast, onDismiss }: ToastProps) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(toast.id), 4000);
    return () => clearTimeout(timer);
  }, [toast.id, onDismiss]);

  const bgColor = {
    success: "bg-success/15 border-success/30 text-success",
    error: "bg-error/15 border-error/30 text-error",
    info: "bg-accent/15 border-accent/30 text-accent",
  }[toast.type];

  return (
    <div
      className={`animate-slide-in rounded-lg border px-4 py-2.5 text-sm shadow-lg ${bgColor}`}
      onClick={() => onDismiss(toast.id)}
    >
      {toast.message}
    </div>
  );
}

interface ToastContainerProps {
  toasts: ToastData[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {toasts.map((t) => (
        <Toast key={t.id} toast={t} onDismiss={onDismiss} />
      ))}
    </div>
  );
}
