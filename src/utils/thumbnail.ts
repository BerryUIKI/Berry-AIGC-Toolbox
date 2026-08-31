import { invoke } from "@tauri-apps/api/core";
import { assetUrl } from "./image";
import type { ImageFile } from "../types";

export interface ThumbnailCacheStats {
  total_bytes: number;
  file_count: number;
  cache_dir: string;
}

const THUMBNAIL_SETTING_KEY = "berry_thumbnail_max_edge";
const DEFAULT_MAX_EDGE = 384; // 64 * 6, perfect balanced resolution for 130px~360px grid zoom
const MAX_MEMORY_CACHE_ENTRIES = 3000;

class LruThumbnailCache {
  private cache = new Map<number, string>();
  private maxSize: number;

  constructor(maxSize = MAX_MEMORY_CACHE_ENTRIES) {
    this.maxSize = maxSize;
  }

  get(key: number): string | undefined {
    const val = this.cache.get(key);
    if (val !== undefined) {
      this.cache.delete(key);
      this.cache.set(key, val);
    }
    return val;
  }

  set(key: number, value: string): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey);
      }
    }
    this.cache.set(key, value);
  }

  clear(): void {
    this.cache.clear();
  }

  get size(): number {
    return this.cache.size;
  }
}

// In-memory runtime LRU map of file_id -> cached thumbnail asset url
const memoryCache = new LruThumbnailCache(3000);

// Active in-flight promises to deduplicate concurrent requests for the same file
const inFlightRequests = new Map<number, Promise<string>>();

/**
 * Get the user-configured max edge resolution from localStorage.
 */
export function getThumbnailMaxEdge(): number {
  try {
    const val = localStorage.getItem(THUMBNAIL_SETTING_KEY);
    if (val) {
      const parsed = parseInt(val, 10);
      if (parsed >= 128 && parsed <= 1024) return parsed;
    }
  } catch {
    // Ignore localStorage access errors
  }
  return DEFAULT_MAX_EDGE;
}

/**
 * Save user-configured thumbnail resolution.
 */
export function setThumbnailMaxEdge(maxEdge: number): void {
  try {
    localStorage.setItem(THUMBNAIL_SETTING_KEY, String(maxEdge));
    // Clear in-memory cache so images request new resolution
    memoryCache.clear();
  } catch {
    // Ignore errors
  }
}

/**
 * Check if thumbnail URL is already available in memory cache synchronously.
 */
export function getThumbnailUrlSync(file: ImageFile): string | null {
  const fileId = file.id ?? 0;
  if (!fileId) return null;
  return memoryCache.get(fileId) ?? null;
}

/**
 * Get or asynchronously generate thumbnail URL for a given image file.
 */
export async function getThumbnailUrl(
  file: ImageFile,
  maxEdge: number = getThumbnailMaxEdge(),
): Promise<string> {
  const fileId = file.id ?? 0;
  if (!fileId) return assetUrl(file.path);

  // Check memory cache first
  const cached = memoryCache.get(fileId);
  if (cached) return cached;

  // Deduplicate in-flight requests
  if (inFlightRequests.has(fileId)) {
    return inFlightRequests.get(fileId)!;
  }

  const promise = (async () => {
    try {
      const diskPath = await invoke<string>("get_or_create_thumbnail", {
        fileId,
        filePath: file.path,
        modifiedAt: file.modified_at,
        maxEdge,
      });
      const url = assetUrl(diskPath);
      memoryCache.set(fileId, url);
      return url;
    } catch {
      // Fallback to original image if downsampling fails (e.g. video)
      const fallbackUrl = assetUrl(file.path);
      memoryCache.set(fileId, fallbackUrl);
      return fallbackUrl;
    } finally {
      inFlightRequests.delete(fileId);
    }
  })();

  inFlightRequests.set(fileId, promise);
  return promise;
}

/**
 * Batch generate thumbnails in background for a list of files.
 */
export async function requestBatchThumbnails(
  files: ImageFile[],
  maxEdge: number = getThumbnailMaxEdge(),
): Promise<number> {
  if (!files || files.length === 0) return 0;

  const items = files
    .filter((f) => f.id != null)
    .map((f) => ({
      file_id: f.id!,
      file_path: f.path,
      modified_at: f.modified_at,
    }));

  if (items.length === 0) return 0;

  try {
    return await invoke<number>("batch_generate_thumbnails", {
      items,
      maxEdge,
    });
  } catch {
    return 0;
  }
}

/**
 * Fetch thumbnail cache statistics from disk.
 */
export async function getThumbnailCacheStats(): Promise<ThumbnailCacheStats> {
  return await invoke<ThumbnailCacheStats>("get_thumbnail_cache_stats");
}

/**
 * Clear all thumbnail cache files from disk and memory.
 */
export async function clearThumbnailCache(): Promise<number> {
  memoryCache.clear();
  inFlightRequests.clear();
  return await invoke<number>("clear_thumbnail_cache");
}
