import { useState, useEffect, useCallback } from "react";
import { Card, Button } from "../../../components/ui";
import {
  checkPermissions,
  requestMicrophonePermission,
  requestAccessibilityPermission,
} from "../../../lib/tauri";

interface PermissionItem {
  name: string;
  key: "microphone" | "accessibility";
  description: string;
  status: string; // "granted" | "denied" | "not_determined" | "unknown"
  requesting: boolean;
}

export function PermissionsStep() {
  const [permissions, setPermissions] = useState<PermissionItem[]>([
    {
      name: "Microphone",
      key: "microphone",
      description: "Required for voice recording",
      status: "unknown",
      requesting: false,
    },
    {
      name: "Accessibility",
      key: "accessibility",
      description: "Required for text insertion",
      status: "unknown",
      requesting: false,
    },
  ]);

  const refreshPermissions = useCallback(async () => {
    try {
      const result = await checkPermissions();
      setPermissions((prev) =>
        prev.map((p) => ({
          ...p,
          status: result[p.key] ?? "unknown",
          requesting: false,
        })),
      );
    } catch {
      // Tauri API may not be available in dev mode
    }
  }, []);

  useEffect(() => {
    refreshPermissions();
  }, [refreshPermissions]);

  // Poll for accessibility changes (user may toggle it in System Settings)
  useEffect(() => {
    const interval = setInterval(refreshPermissions, 2000);
    return () => clearInterval(interval);
  }, [refreshPermissions]);

  async function handleRequest(key: "microphone" | "accessibility") {
    setPermissions((prev) =>
      prev.map((p) => (p.key === key ? { ...p, requesting: true } : p)),
    );

    try {
      if (key === "microphone") {
        await requestMicrophonePermission();
      } else {
        await requestAccessibilityPermission();
      }
      // Re-check after request
      await refreshPermissions();
    } catch {
      setPermissions((prev) =>
        prev.map((p) => (p.key === key ? { ...p, requesting: false } : p)),
      );
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold text-text-primary">Permissions</h2>
        <p className="mt-1 text-text-secondary">
          Voxlore needs microphone access to record and accessibility access to type text.
        </p>
      </div>

      <Card padding="none" className="divide-y divide-border">
        {permissions.map((perm) => (
          <div key={perm.key} className="flex items-center justify-between p-4">
            <div className="flex flex-col gap-0.5">
              <span className="text-sm font-semibold text-text-primary">{perm.name}</span>
              <span className="text-xs text-text-muted">{perm.description}</span>
            </div>
            {perm.status === "granted" ? (
              <div className="flex h-6 w-6 items-center justify-center rounded-full bg-success/20 text-success">
                ✓
              </div>
            ) : (
              <Button
                variant="primary"
                size="sm"
                disabled={perm.requesting}
                onClick={() => handleRequest(perm.key)}
              >
                {perm.requesting
                  ? "Requesting..."
                  : perm.status === "denied"
                    ? "Open Settings"
                    : "Grant Access"}
              </Button>
            )}
          </div>
        ))}
      </Card>

      {permissions.some((p) => p.status === "denied") && (
        <p className="text-xs text-text-muted">
          If a permission was previously denied, click "Open Settings" to grant it manually in
          System Settings.
        </p>
      )}
    </div>
  );
}
