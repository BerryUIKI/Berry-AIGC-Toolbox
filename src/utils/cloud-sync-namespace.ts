import { invoke } from "@tauri-apps/api/core";
import type { CloudBackupConfig, CloudSyncNamespacePreview, CloudSyncOptions } from "../types";

type Invoke = <T>(command: string, args: Record<string, unknown>) => Promise<T>;

/** Start only after the saved current mapping has been explicitly confirmed. */
export async function startCloudSyncWithConfirmation(
  config: CloudBackupConfig,
  options: CloudSyncOptions,
  confirm: (preview: CloudSyncNamespacePreview) => boolean | Promise<boolean>,
  call: Invoke = invoke,
): Promise<boolean> {
  // Snapshot caller-owned objects before either asynchronous step.
  const capturedConfig = { ...config };
  const capturedOptions = { ...options, folder_ids: options.folder_ids?.slice() ?? options.folder_ids };
  const preview = await call<CloudSyncNamespacePreview>("cloud_sync_preview_namespace", {
    options: capturedOptions,
  });
  if (!await confirm(preview)) return false;
  await call<void>("cloud_sync_start", {
    config: capturedConfig,
    options: { ...capturedOptions, namespace_manifest_id: preview.manifest_id },
  });
  return true;
}
