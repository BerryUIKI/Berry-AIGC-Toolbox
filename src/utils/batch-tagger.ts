import type { BatchTagProgress, BatchTagResult, TaggerConfig } from "../types";

export type BatchTagStatus = "idle" | "running" | "completed" | "canceled" | "error";

export interface BatchAutoTagOptions {
  fileIds: number[];
  config: TaggerConfig;
  onProgress?: (progress: BatchTagProgress) => void | Promise<void>;
}

export class BatchAutoTagController {
  private _status: BatchTagStatus = "idle";
  private _progress: BatchTagProgress | null = null;
  private _errorMessage: string | null = null;
  private _isCanceled: boolean = false;
  private _unlisten: (() => void) | null = null;
  private _invoke: (cmd: string, args?: Record<string, unknown>) => Promise<any>;
  private _listen: (event: string, handler: (e: { payload: any }) => void) => Promise<() => void>;

  constructor(
    invokeFn?: (cmd: string, args?: Record<string, unknown>) => Promise<any>,
    listenFn?: (event: string, handler: (e: { payload: any }) => void) => Promise<() => void>,
  ) {
    this._invoke =
      invokeFn ??
      (async (cmd, args) => {
        const { invoke } = await import("@tauri-apps/api/core");
        return invoke(cmd, args);
      });
    this._listen =
      listenFn ??
      (async (event, handler) => {
        const { listen } = await import("@tauri-apps/api/event");
        return listen(event, handler);
      });
  }

  get status(): BatchTagStatus {
    return this._status;
  }

  get isRunning(): boolean {
    return this._status === "running";
  }

  get progress(): BatchTagProgress | null {
    return this._progress;
  }

  get errorMessage(): string | null {
    return this._errorMessage;
  }

  async start(options: BatchAutoTagOptions): Promise<BatchTagResult> {
    if (this.isRunning) {
      throw new Error("Batch auto-tagging is already in progress");
    }

    this._status = "running";
    this._isCanceled = false;
    this._errorMessage = null;
    this._progress = {
      current: 0,
      total: options.fileIds.length,
      percent: 0,
      current_file: "",
      processed_files: 0,
      failed_files: 0,
      tags_added: 0,
      is_complete: false,
      is_canceled: false,
    };

    try {
      this._unlisten = await this._listen("tagger-batch-progress", async (event) => {
        const p = event.payload as BatchTagProgress;
        this._progress = p;
        if (options.onProgress) {
          await options.onProgress(p);
        }
      });

      const result: BatchTagResult = await this._invoke("batch_auto_tag_files", {
        fileIds: options.fileIds,
        config: options.config,
      });

      if (this._isCanceled) {
        this._status = "canceled";
      } else {
        this._status = "completed";
      }

      return result;
    } catch (err: any) {
      if (this._isCanceled) {
        this._status = "canceled";
        return {
          processed_files: this._progress?.processed_files ?? 0,
          tags_added: this._progress?.tags_added ?? 0,
        };
      }
      this._status = "error";
      this._errorMessage = String(err?.message || err);
      throw err;
    } finally {
      if (this._unlisten) {
        this._unlisten();
        this._unlisten = null;
      }
    }
  }

  async cancel(): Promise<void> {
    if (!this.isRunning) return;
    this._isCanceled = true;
    try {
      await this._invoke("cancel_batch_auto_tag");
    } catch (e) {
      console.error("Failed to cancel batch auto tag:", e);
    }
  }
}
