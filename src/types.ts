// TypeScript mirrors of the serde types returned by the Tauri commands.
// Field names stay snake_case, matching Rust's serde defaults.

export interface AppInfo {
  app_version: string;
  schema_version: number;
  database_path: string;
}

export type FolderType = "link" | "managed" | "pipeline";
export type IngestAction = "copy" | "move";

export interface Folder {
  id: number;
  path: string;
  added_at: string;
  folder_type?: FolderType;
  source_path?: string | null;
  ingest_action?: IngestAction | null;
  grace_period_hours?: number | null;
  auto_harvest?: boolean;
}

export type Container = "png" | "jpg" | "webp" | "mp4" | "webm" | "txt";

export interface ExtractedMetadata {
  /** null for technical image facts without a generation platform. */
  format: string | null;
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
  duration_seconds?: number | null;
  fps?: number | null;
  video_codec?: string | null;
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
  favorites?: number;
  nsfw?: number;
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
  similarity_score?: number | null;
  stack_id?: string | null;
  stack_order?: number;
}

export interface PageCursor {
  sort_value: string;
  id: number;
}

export interface CursorFilePage {
  items: ImageFile[];
  total: number;
  next_cursor?: PageCursor | null;
  prev_cursor?: PageCursor | null;
  has_more: boolean;
}

export interface StorageRoot {
  root_uuid: string;
  display_name: string;
  root_type: string;
  created_at: number;
  updated_at: number;
}

export interface NormalizedPath {
  root_uuid: string;
  relative_path: string;
}

export interface ChangeLogEntry {
  id: number;
  event_type: string;
  entity_id: number;
  secondary_id?: string | null;
  client_id: string;
  payload?: string | null;
  created_at: number;
}

export interface MutationResult {
  success: boolean;
  current_version: number;
  rows_affected: number;
  conflict_detected: boolean;
}

export interface ChangeLogSyncQuery {
  after_id: number;
  exclude_client_id?: string | null;
  limit: number;
}

export interface DatabasePingResult {
  success: boolean;
  latency_ms: number;
  backend: string;
  message: string;
}

export interface MigrationOptions {
  target_dialect: string;
  target_root_uuid: string;
  destination: string;
}

export interface MigrationSummary {
  success: boolean;
  target_dialect: string;
  total_files: number;
  total_albums: number;
  total_tags: number;
  total_tag_associations: number;
  output_path?: string | null;
  error_message?: string | null;
  duration_ms: number;
}

export type ExportFormat = "original" | "webp" | "jpeg" | "png" | "avif";

export type TransformFormat = "original" | "webp" | "jpeg" | "png" | "avif";
export type TransformMetadataPolicy = "keep_supported" | "strip_ai" | "strip_all";
export type TransformCollisionPolicy = "rename" | "skip";

export interface TransformSpec {
  format: TransformFormat;
  quality?: number | null;
  max_edge?: number | null;
  scale_percent?: number | null;
  align_multiple?: number | null;
  target_size_kb?: number | null;
  metadata_policy: TransformMetadataPolicy;
  collision_policy: TransformCollisionPolicy;
}

export interface TransformPreset {
  id: string;
  name: string;
  spec: TransformSpec;
  original_disposition?: OriginalDisposition;
}

export type OriginalDisposition = "keep" | "archive" | "trash";

export interface LibraryTransformRequest {
  file_ids: number[];
  spec: TransformSpec;
  original_disposition: OriginalDisposition;
}

export type TransformItemStatus = "succeeded" | "failed" | "skipped" | "canceled";

export interface TransformItemReceipt {
  source_id_or_path: string;
  output_id_or_path?: string | null;
  status: TransformItemStatus;
  error_code?: string | null;
  original_action?: string | null;
}

export interface TransformJobReceipt {
  job_id: string;
  phase: string;
  total: number;
  succeeded: number;
  failed: number;
  skipped: number;
  canceled: number;
  items: TransformItemReceipt[];
}

export interface TransformProgressEvent {
  current: number;
  total: number;
  current_path: string;
}

export interface ExportEstimateResult {
  original_bytes: number;
  estimated_bytes: number;
  original_width: number;
  original_height: number;
  output_width: number;
  output_height: number;
  format: string;
  savings_percent: number;
}

export type MetadataPrivacyMode =
  | "keep_all"
  | "strip_prompt_only"
  | "strip_all_ai_metadata"
  | "strip_all";

export type ExportSidecar = "none" | "text_prompt" | "json_metadata";

export interface ExportOptions {
  file_ids: number[];
  format: ExportFormat;
  quality: number;
  privacy: MetadataPrivacyMode;
  sidecar: ExportSidecar;
  filename_template: string;
  destination_path: string;
  as_zip: boolean;
  max_edge?: number | null;
  export_html_showcase?: boolean;
  html_title?: string;
}

export interface ExportProgressEvent {
  current: number;
  total: number;
  current_filename: string;
}

export interface ExportSummary {
  success: boolean;
  total_exported: number;
  total_failed: number;
  total_bytes_written: number;
  duration_ms: number;
  output_path: string;
  errors: string[];
}

export type CloudStorageProvider = "local_path" | "webdav" | "s3";

export interface CloudBackupConfig {
  provider: CloudStorageProvider;
  local_path?: string | null;
  webdav_endpoint?: string | null;
  webdav_username?: string | null;
  webdav_password?: string | null;
  s3_endpoint?: string | null;
  s3_bucket?: string | null;
  s3_region?: string | null;
  s3_access_key?: string | null;
  s3_secret_key?: string | null;
  s3_prefix?: string | null;
  auto_backup_enabled?: boolean;
  auto_backup_interval_days?: number;
}

export interface CloudSnapshotMeta {
  snapshot_id: string;
  filename: string;
  size_bytes: number;
  created_at: number;
  file_count: number;
  folder_count: number;
  tag_count: number;
  album_count: number;
  description?: string | null;
  berry_version: string;
}

export interface CloudPingResult {
  success: boolean;
  latency_ms: number;
  message: string;
}

export interface CloudBackupResult {
  success: boolean;
  snapshot?: CloudSnapshotMeta | null;
  duration_ms: number;
  error?: string | null;
}

export interface CloudRestoreResult {
  success: boolean;
  restored_files_count: number;
  duration_ms: number;
  error?: string | null;
}

export type CloudSyncDirection = 'upload_to_remote' | 'download_from_remote';
export type CloudSyncStrategy = 'fast_fingerprint' | 'sha256_checksum';
export type CloudSyncPhase = 'idle' | 'scanning' | 'syncing' | 'completed' | 'cancelled' | 'failed';

export interface CloudSyncOptions {
  direction?: CloudSyncDirection;
  strategy?: CloudSyncStrategy;
  concurrency?: number;
  bandwidth_limit_kbs?: number | null;
  dry_run?: boolean;
  remote_prefix?: string;
  folder_ids?: number[] | null;
  namespace_manifest_id?: string | null;
}

export interface CloudSyncRootMapping {
  folder_id: number;
  source_path: string;
  root_uuid: string;
  legacy_prefix: string;
  new_prefix: string;
  legacy_ambiguous: boolean;
}

export interface CloudSyncNamespaceManifest {
  version: number;
  remote_prefix: string;
  roots: CloudSyncRootMapping[];
}

export interface CloudSyncNamespacePreview {
  manifest_id: string;
  manifest_path: string;
  manifest: CloudSyncNamespaceManifest;
}

export interface CloudSyncProgress {
  phase: CloudSyncPhase;
  total_files: number;
  completed_files: number;
  skipped_files: number;
  failed_files: number;
  total_bytes: number;
  transferred_bytes: number;
  current_file?: string | null;
  speed_bytes_per_sec: number;
  eta_seconds?: number | null;
  error?: string | null;
}

export interface CloudSyncResult {
  success: boolean;
  total_files: number;
  synced_files: number;
  skipped_files: number;
  failed_files: number;
  transferred_bytes: number;
  duration_ms: number;
  dry_run: boolean;
  errors: string[];
}

export interface FilePage {
  items: ImageFile[];
  total: number;
  offset: number;
  has_more: boolean;
}

export interface PipelineDetectedPath {
  tool_name: string;
  path: string;
  category: string;
}

export interface CleanupQueueItem {
  id: number;
  source_file_path: string;
  target_image_id: number;
  scheduled_delete_at: number;
  created_at: number;
  status: "pending" | "deleted" | "cancelled" | "failed";
}

export interface StackSummary {
  stack_id: string;
  count: number;
  hero_image_id: number | null;
}

export interface AutoStackResult {
  created_stacks: number;
  stacked_images: number;
  eligible_images: number;
  skipped_without_prompt: number;
}

export interface SimilarityMatch {
  file_id: number;
  score: number;
}

export interface SimilarFileItem {
  file: ImageFile;
  score: number;
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
  discovering: boolean;
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

export interface LibraryFilesChanged {
  folder_id: number;
  stats: ScanStats;
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
  folder_path?: string | null;
  recursive?: boolean | null;
  stack_id?: string | null;
  media_type?: string | null;
  min_duration?: number | null;
  max_duration?: number | null;
  min_fps?: number | null;
  max_fps?: number | null;
  sort?: FileSortField | null;
  direction?: SortDirection | null;
  limit?: number | null;
  offset?: number | null;
  cursor?: PageCursor | null;
}

export interface SubdirectoryEntry {
  name: string;
  path: string;
  has_children: boolean;
  file_count: number;
}

export type NavTarget =
  | { type: "all" }
  | { type: "favorites" }
  | { type: "nsfw" }
  | { type: "folder"; folder: Folder; subfolderPath?: string | null; recursive?: boolean }
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

export interface TagPrediction {
  name: string;
  category: "General" | "Character" | "Rating" | string | number;
  confidence: number;
}

export interface TaggerConfig {
  general_threshold: number;
  character_threshold: number;
  include_rating: boolean;
  max_tags: number;
  write_to_prompt?: boolean;
  append_prompt?: boolean;
  allow_override_existing_prompt?: boolean;
}

export interface TaggerModelSummary {
  name: string;
  dir_path: string;
  model_path: string;
  tags_path: string;
  is_loaded: boolean;
}

export interface BatchTagResult {
  processed_files: number;
  tags_added: number;
}

export interface BatchTagProgress {
  current: number;
  total: number;
  percent: number;
  current_file: string;
  processed_files: number;
  failed_files: number;
  tags_added: number;
  is_complete: boolean;
  is_canceled: boolean;
}

export interface TaggerDownloadProgress {
  phase: string;
  current_file: string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  speed_bytes_per_sec: number;
  error?: string | null;
}

export interface ClipModelSummary {
  name: string;
  dir_path: string;
  visual_path: string;
  textual_path: string;
  tokenizer_path: string;
  is_loaded: boolean;
}

export interface ClipIndexStatus {
  model_id: string;
  indexed_images: number;
  total_images: number;
  is_loaded: boolean;
}

export interface ClipBatchIndexResult {
  indexed_count: number;
  remaining_count: number;
  total_count: number;
  failed_count?: number;
}

export interface ClipModelInfo {
  model_id: string;
  name: string;
  folder_path: string;
  visual_model_path: string;
  textual_model_path: string;
  tokenizer_path: string;
  image_size: number;
  embedding_dim: number;
}

export interface LoraModel {
  id: number;
  name: string;
  hash?: string | null;
  trigger_words: string[];
  preview_url?: string | null;
  description?: string | null;
  weight_default: number;
  created_at: string;
  updated_at: string;
}

export interface DetectedLora {
  name: string;
  weight: number;
  hash?: string | null;
  model?: LoraModel | null;
}

export interface DiscoveredSource {
  source_id: string;
  identifier: string;
  root_path: string;
  database_path: string;
  config_path?: string | null;
  file_count: number;
  database_size_bytes: number;
  total_size_bytes: number;
  schema_version: number;
  is_locked: boolean;
}

export interface MigratedArtifact {
  category: string;
  source_path: string;
  destination_path: string;
  status: string;
  size_bytes: number;
}

export interface MigrationReceipt {
  receipt_id: string;
  source_id: string;
  source_identifier: string;
  source_root: string;
  source_schema_version: number;
  destination_root: string;
  destination_db: string;
  created_at: number;
  artifacts: MigratedArtifact[];
  integrity_hash: string;
  cleanup_status: string;
}

export interface MigrationError {
  code: string;
  message_key: string;
  retryable: boolean;
  context?: string | null;
}

export interface LegacyMigrationStatus {
  stage: string;
  discovered_sources: DiscoveredSource[];
  destination_exists: boolean;
  active_receipt?: MigrationReceipt | null;
  available_actions: string[];
}

export interface LegacyMigrationPreview {
  plan_id: string;
  source: DiscoveredSource;
  destination_root: string;
  destination_db: string;
  required_space_bytes: number;
  available_space_bytes: number;
  conflicts: string[];
  exclusions: string[];
}

export interface LegacyMigrationJob {
  job_id: string;
  plan_id: string;
  status: string;
  progress: number;
  current_step: string;
  error?: MigrationError | null;
  receipt?: MigrationReceipt | null;
}

export interface CleanupItem {
  path: string;
  category: string;
  size_bytes: number;
  eligible: boolean;
  reason?: string | null;
}

export interface LegacyCleanupPreview {
  preview_id: string;
  receipt_id: string;
  expires_at: number;
  items: CleanupItem[];
  total_size_bytes: number;
  destination_healthy: boolean;
}

export interface LegacyCleanupResult {
  receipt_id: string;
  cleaned_count: number;
  failed_count: number;
  cleaned_bytes: number;
  errors: string[];
  status: string;
}

