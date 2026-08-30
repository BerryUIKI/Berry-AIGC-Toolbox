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
  is_favorite?: boolean;
  is_nsfw?: boolean;
}

export interface Album {
  id: number;
  name: string;
  description?: string | null;
  created_at: string;
}

export interface Tag {
  id: number;
  name: string;
  color?: string | null;
  created_at: string;
}

export interface PromptStat {
  text: string;
  count: number;
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
  is_favorite?: boolean | null;
  is_nsfw?: boolean | null;
  album_id?: number | null;
  tag_id?: number | null;
  folder_id?: number | null;
  sort?: FileSortField | null;
  direction?: SortDirection | null;
  limit?: number | null;
  offset?: number | null;
}

export type NavTarget =
  | { type: "all" }
  | { type: "favorites" }
  | { type: "nsfw" }
  | { type: "folder"; folder: Folder }
  | { type: "album"; album: Album }
  | { type: "tag"; tag: Tag };

export interface PromptKeywordStat {
  keyword: string;
  count: number;
  avg_rating?: number | null;
}

export interface PromptStats {
  total_analyzed: number;
  top_positive_words: PromptKeywordStat[];
  top_negative_words: PromptKeywordStat[];
  top_models: PromptKeywordStat[];
  top_samplers: PromptKeywordStat[];
}

export interface CheckpointModelStat {
  model_name: string;
  model_hash?: string | null;
  count: number;
}

export interface ModelCacheEntry {
  hash: string;
  name: string;
  title?: string | null;
  sha256?: string | null;
}

export interface DatabaseStats {
  file_count: number;
  folder_count: number;
  album_count: number;
  tag_count: number;
  model_cache_count: number;
  db_size_bytes: number;
  page_size: number;
  page_count: number;
  freelist_count: number;
}


