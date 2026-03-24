import { emitTo } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { PreviewSession } from "../types/app";

export const PREVIEW_WINDOW_LABEL = "preview-window";
export const PREVIEW_WINDOW_QUERY = "?window=preview";
export const PREVIEW_SESSION_EVENT = "preview-session-updated";
const PREVIEW_SESSION_STORAGE_KEY = "l2dtoolbox.preview-session";

function serializeSession(session: PreviewSession): string {
  return JSON.stringify(session);
}

export function loadPreviewSession(): PreviewSession | null {
  const raw = window.localStorage.getItem(PREVIEW_SESSION_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as PreviewSession;
  } catch {
    window.localStorage.removeItem(PREVIEW_SESSION_STORAGE_KEY);
    return null;
  }
}

export function savePreviewSession(session: PreviewSession): void {
  window.localStorage.setItem(
    PREVIEW_SESSION_STORAGE_KEY,
    serializeSession(session),
  );
}

async function createPreviewWindow(): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel(PREVIEW_WINDOW_LABEL);
  if (existing) {
    return existing;
  }

  const previewWindow = new WebviewWindow(PREVIEW_WINDOW_LABEL, {
    title: "Live2D 预览",
    url: `/${PREVIEW_WINDOW_QUERY}`,
    width: 1220,
    height: 860,
    minWidth: 900,
    minHeight: 640,
    resizable: true,
    center: true,
  });

  return await new Promise<WebviewWindow>((resolve, reject) => {
    let settled = false;

    void previewWindow.once("tauri://created", () => {
      settled = true;
      resolve(previewWindow);
    });

    void previewWindow.once("tauri://error", (event) => {
      if (!settled) {
        reject(
          new Error(
            typeof event.payload === "string"
              ? event.payload
              : "预览窗口创建失败",
          ),
        );
      }
    });
  });
}

export async function openPreviewWindow(session: PreviewSession): Promise<void> {
  savePreviewSession(session);

  const previewWindow = await createPreviewWindow();
  await emitTo(PREVIEW_WINDOW_LABEL, PREVIEW_SESSION_EVENT, session);
  await previewWindow.show();
  await previewWindow.setFocus();
}
