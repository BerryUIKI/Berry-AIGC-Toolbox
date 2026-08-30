import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Strip the Windows `\\?\` verbatim prefix, if present.
 */
export function normalizePath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

/**
 * Convert a local file system path to a URL that the Tauri webview can load
 * via the `asset:` custom protocol.
 */
export function assetUrl(path: string): string {
  if (!path) return "";
  const clean = normalizePath(path);
  return convertFileSrc(clean);
}

/**
 * Extract filename from a path (handling both POSIX and Windows separators).
 */
export function getFileName(path: string): string {
  const clean = normalizePath(path);
  return clean.split(/[\\/]/).pop() ?? clean;
}

/**
 * Format byte count to human-readable size.
 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Format unix timestamp (in seconds) to local date-time string.
 */
export function formatDateTime(unixSeconds: number): string {
  if (!unixSeconds) return "—";
  return new Date(unixSeconds * 1000).toLocaleString();
}
