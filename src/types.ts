// TypeScript mirrors of the serde types returned by the Tauri commands.
// Field names stay snake_case, matching Rust's serde defaults.

export interface AppInfo {
  app_version: string;
  schema_version: number;
  database_path: string;
}

export interface Folder {
  id: number;
  path: string;
  added_at: string;
}

export type Container = "png" | "jpg" | "webp" | "mp4" | "txt";

export interface ExtractedMetadata {
  format: string;
  parameters: string | null;
  raw: string | null;
  prompt: string | null;
  negative_prompt: string | null;
  width: number | null;
  height: number | null;
  seed: string | null;
  steps: number | null;
  cfg_scale: number | null;
  sampler: string | null;
  model_name: string | null;
  model_hash: string | null;
}

export type FileSortField =
  | "modified_at"
  | "path"
  | "size_bytes"
  | "rating"
  | "aesthetic_score";

export type SortDirection = "asc" | "desc";

export interface LibraryCounts {
  total: number;
  folders: Record<number, number>;
}

export interface ImageFile {
  id: number | null;
  folder_id: number;
  path: string;
  size_bytes: number;
  modified_at: number;
  container: Container;
  metadata: ExtractedMetadata | null;
  rating?: number | null;
  aesthetic_score?: number | null;
}


export interface ScanProgress {
  folder_id: number;
  scanned: number;
  found: number;
  current: string | null;
}

export interface ScanStats {
  folder_id: number;
  found: number;
  added: number;
  updated: number;
  unchanged: number;
  removed: number;
  failed: number;
  duration_ms: number;
}

export interface SearchCriteria {
  text?: string | null;
  prompt?: string | null;
  negative_prompt?: string | null;
  model_name?: string | null;
  model_hash?: string | null;
  sampler?: string | null;
  min_steps?: number | null;
  max_steps?: number | null;
  min_cfg?: number | null;
  max_cfg?: number | null;
  min_rating?: number | null;
  max_rating?: number | null;
  min_aesthetic?: number | null;
  max_aesthetic?: number | null;
  folder_id?: number | null;
  sort?: FileSortField | null;
  direction?: SortDirection | null;
  limit?: number | null;
  offset?: number | null;
}

