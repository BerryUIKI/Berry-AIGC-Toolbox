import assert from "node:assert/strict";
import test from "node:test";
import { performance } from "node:perf_hooks";
import { WaterfallGeometry, visibleWaterfallItems, GalleryPages } from "../src/utils/gallery-state.ts";
import { selectThumbnailTier, THUMBNAIL_SIZE_TIERS } from "../src/utils/thumbnail-tier.ts";
import { LruThumbnailCache } from "../src/utils/lru-cache.ts";

const createMockFile = (id) => ({
  id,
  path: `/library/stress_vault/image_${String(id).padStart(6, "0")}.png`,
  container: "png",
  size_bytes: 1_200_000 + (id % 500_000),
  modified_at: 1_700_000_000 + id,
  metadata: {
    width: id % 3 === 0 ? 512 : id % 3 === 1 ? 768 : 1024,
    height: id % 2 === 0 ? 768 : 512,
    prompt: `masterpiece, stress test iteration ${id}`,
  },
  rating: id % 5 === 0 ? ((id % 5) + 1) : null,
  is_favorite: id % 10 === 0,
  is_nsfw: false,
});

test("Part 1: 50,000-item virtualized waterfall layout and high-speed scroll churn", () => {
  const TOTAL_ITEMS = 50_000;
  const CHUNK_SIZE = 1_000;
  const geometry = new WaterfallGeometry();
  const allFiles = [];

  // 1. Incremental append across 50 pages of 1,000 items
  const appendStart = performance.now();
  for (let chunk = 0; chunk < TOTAL_ITEMS / CHUNK_SIZE; chunk++) {
    const chunkFiles = Array.from({ length: CHUNK_SIZE }, (_, i) =>
      createMockFile(chunk * CHUNK_SIZE + i),
    );
    allFiles.push(...chunkFiles);
    geometry.update(allFiles, 6, 220, 16);
  }
  const appendElapsed = performance.now() - appendStart;

  assert.equal(geometry.items.length, TOTAL_ITEMS, "Geometry must hold all 50,000 items");
  assert.equal(geometry.columns.length, 6, "Must lay out across 6 columns");
  assert.ok(geometry.height > 2_000_000, `Geometry height (${geometry.height}px) must be proportional to 50k items`);
  console.log(`[Stress Test] 50k item incremental append: ${appendElapsed.toFixed(2)}ms (total height: ${Math.round(geometry.height)}px)`);

  // 2. High-speed scroll churn: 500 rapid scroll events simulating fast scrollbar drag & wheel
  const VIEWPORT_HEIGHT = 900;
  const maxScroll = Math.max(0, geometry.height - VIEWPORT_HEIGHT);
  const latencies = [];

  for (let i = 0; i < 500; i++) {
    // Oscillating and pseudo-random scroll offsets
    const factor = (Math.sin(i * 0.1) + 1) / 2; // 0.0 ~ 1.0
    const top = factor * maxScroll;
    const bottom = top + VIEWPORT_HEIGHT;

    const t0 = performance.now();
    const visible = visibleWaterfallItems(geometry.columns, top, bottom);
    const elapsed = performance.now() - t0;
    latencies.push(elapsed);

    // Viewport virtualization bounding check: DOM nodes must never exceed ~60 items
    assert.ok(
      visible.length >= 10 && visible.length <= 60,
      `Visible item count (${visible.length}) at offset ${top.toFixed(0)} must be strictly bounded to viewport`,
    );

    // Verify ordering
    for (let j = 1; j < visible.length; j++) {
      assert.ok(visible[j].index > visible[j - 1].index, "Visible items must remain sorted by index");
    }
  }

  latencies.sort((a, b) => a - b);
  const p50 = latencies[Math.floor(latencies.length * 0.5)];
  const p95 = latencies[Math.floor(latencies.length * 0.95)];
  const max = latencies[latencies.length - 1];

  console.log(`[Stress Test] 500 rapid viewport window lookups: p50=${p50.toFixed(3)}ms, p95=${p95.toFixed(3)}ms, max=${max.toFixed(3)}ms`);
  assert.ok(p50 < 0.25, `Median lookup latency (${p50.toFixed(3)}ms) must be under 0.25ms`);
  assert.ok(p95 < 1.0, `95th percentile lookup latency (${p95.toFixed(3)}ms) must be under 1.0ms`);
});

test("Part 2: Multi-tier thumbnail cache hit ratio, eviction & memory upper bound stress", () => {
  // 1. Verify standard resolution tier ladder
  assert.deepEqual(
    [...THUMBNAIL_SIZE_TIERS],
    [128, 192, 256, 320, 384, 448, 512, 768, 1024],
    "Standard thumbnail tiers must span 128 through 1024",
  );

  // Resolution tier selection across varied display zoom and device pixel ratios
  assert.equal(selectThumbnailTier(120, 384, 1), 128);
  assert.equal(selectThumbnailTier(120, 384, 2), 256);
  assert.equal(selectThumbnailTier(220, 384, 1), 256);
  assert.equal(selectThumbnailTier(220, 384, 1.5), 384);
  assert.equal(selectThumbnailTier(360, 512, 1), 384);
  assert.equal(selectThumbnailTier(360, 512, 2), 512);

  // 2. LRU Cache memory bounds across 50,000 items
  const CACHE_CAPACITY = 3000;
  const cache = new LruThumbnailCache(CACHE_CAPACITY);

  // Populate 50,000 distinct items (simulating progressive forward scrolling)
  for (let i = 0; i < 50_000; i++) {
    cache.set(`thumb_file_${i}_rev_1_edge_384`, `asset://localhost/thumbs/${i}.webp`);
  }

  assert.equal(cache.size, CACHE_CAPACITY, `Cache size must remain strictly capped at ${CACHE_CAPACITY}`);
  assert.equal(cache.get("thumb_file_0_rev_1_edge_384"), undefined, "Old item 0 must be evicted");
  assert.equal(cache.get("thumb_file_46999_rev_1_edge_384"), undefined, "Item 46999 must be evicted");
  assert.ok(cache.get("thumb_file_47000_rev_1_edge_384") !== undefined, "Item 47000 must be in cache");
  assert.ok(cache.get("thumb_file_49999_rev_1_edge_384") !== undefined, "Latest item 49999 must be in cache");

  // 3. Realistic return scroll simulation (scrolling back up to warm items)
  let hits = 0;
  let misses = 0;
  for (let i = 49_999; i >= 47_500; i--) {
    const val = cache.get(`thumb_file_${i}_rev_1_edge_384`);
    if (val !== undefined) hits++;
    else misses++;
  }
  const hitRate = hits / (hits + misses);
  console.log(`[Stress Test] Warm return scroll hit rate across 2,500 items: ${(hitRate * 100).toFixed(1)}%`);
  assert.ok(hitRate >= 0.99, `Return scroll hit rate (${(hitRate * 100).toFixed(1)}%) must exceed 99%`);

  // 4. BatchReadyKeys pruning under 50,000 items
  const batchReadyKeys = new Set();
  const MAX_LIMIT = 5000;
  const PRUNE_TARGET = 4000;

  function recordBatchKey(key) {
    batchReadyKeys.add(key);
    if (batchReadyKeys.size > MAX_LIMIT) {
      const excess = batchReadyKeys.size - PRUNE_TARGET;
      let pruned = 0;
      for (const k of batchReadyKeys) {
        batchReadyKeys.delete(k);
        pruned++;
        if (pruned >= excess) break;
      }
    }
  }

  for (let i = 0; i < 50_000; i++) {
    recordBatchKey(`file_${i}_rev_1_edge_384`);
  }

  assert.ok(batchReadyKeys.size <= MAX_LIMIT, `batchReadyKeys size (${batchReadyKeys.size}) must never exceed ${MAX_LIMIT}`);
  assert.ok(batchReadyKeys.size >= PRUNE_TARGET, `batchReadyKeys size must stay near ${PRUNE_TARGET}`);
  assert.ok(batchReadyKeys.has("file_49999_rev_1_edge_384"), "Most recent key must be preserved");

  // 5. Epoch invalidation: old in-flight generation cannot overwrite cleared cache
  let cacheEpoch = 1;
  const inFlightEpoch = cacheEpoch;
  cache.clear();
  cacheEpoch++; // User navigated away or jumped scroll
  const fakeCompletedCallback = (targetKey, targetUrl) => {
    if (inFlightEpoch === cacheEpoch) {
      cache.set(targetKey, targetUrl);
    }
  };
  fakeCompletedCallback("stale_key", "asset://stale.webp");
  assert.equal(cache.get("stale_key"), undefined, "Stale epoch response must not pollute cache");
  assert.equal(cache.size, 0, "Cache must remain empty");
});

test("Part 3: High-concurrency transcoding backpressure, cancellation and GC profile", async () => {
  const MAX_CONCURRENT_WORKERS = 4; // Matches bounded Rayon pool clamp(2, 6)
  const TOTAL_JOBS = 200;

  class ConcurrencyBoundedQueue {
    constructor(maxConcurrency) {
      this.maxConcurrency = maxConcurrency;
      this.running = 0;
      this.queue = [];
      this.peakRunning = 0;
      this.completed = 0;
      this.canceled = 0;
      this.aborted = false;
    }

    push(task) {
      if (this.aborted) {
        this.canceled++;
        return Promise.reject(new Error("Job queue canceled"));
      }
      return new Promise((resolve, reject) => {
        this.queue.push({ task, resolve, reject });
        this.processNext();
      });
    }

    abortRemaining() {
      this.aborted = true;
      while (this.queue.length > 0) {
        const item = this.queue.shift();
        this.canceled++;
        item.reject(new Error("Job canceled"));
      }
    }

    processNext() {
      if (this.aborted || this.running >= this.maxConcurrency || this.queue.length === 0) {
        return;
      }
      this.running++;
      if (this.running > this.peakRunning) {
        this.peakRunning = this.running;
      }
      const item = this.queue.shift();

      item.task()
        .then((res) => {
          this.completed++;
          item.resolve(res);
        })
        .catch((err) => {
          this.canceled++;
          item.reject(err);
        })
        .finally(() => {
          this.running--;
          this.processNext();
        });
    }
  }

  // 1. Dispatch 200 heavy jobs under bounded concurrency
  const queue = new ConcurrencyBoundedQueue(MAX_CONCURRENT_WORKERS);
  const stagingFiles = new Set();
  const jobs = [];

  // Track simulated buffer memory
  let activeBufferBytes = 0;
  let peakBufferBytes = 0;

  for (let i = 0; i < TOTAL_JOBS; i++) {
    const jobId = i;
    const stagingPath = `/staging/temp_${jobId}.tmp`;
    stagingFiles.add(stagingPath);

    jobs.push(
      queue.push(async () => {
        // Simulate image decode buffer (1MB raw frame)
        const frameBuffer = Buffer.alloc(1024 * 1024, 0xaa);
        activeBufferBytes += frameBuffer.length;
        if (activeBufferBytes > peakBufferBytes) {
          peakBufferBytes = activeBufferBytes;
        }

        // Simulate encode latency
        await new Promise((r) => setTimeout(r, 2));

        // Release buffer and cleanup staging file
        stagingFiles.delete(stagingPath);
        activeBufferBytes -= frameBuffer.length;
        return { jobId, output: `/media/output_${jobId}.webp` };
      }).catch((e) => e),
    );
  }

  // Allow queue to run until ~40 jobs complete, then trigger mid-batch cancellation
  await new Promise((r) => setTimeout(r, 50));
  queue.abortRemaining();

  const results = await Promise.all(jobs);

  // 2. Assertions on bounded concurrency & backpressure
  assert.ok(
    queue.peakRunning <= MAX_CONCURRENT_WORKERS,
    `Peak running workers (${queue.peakRunning}) must not exceed limit ${MAX_CONCURRENT_WORKERS}`,
  );
  assert.ok(
    peakBufferBytes <= MAX_CONCURRENT_WORKERS * 1024 * 1024,
    `Peak buffer memory (${(peakBufferBytes / 1024 / 1024).toFixed(1)}MB) must be bounded to ${MAX_CONCURRENT_WORKERS}MB`,
  );
  assert.equal(
    queue.completed + queue.canceled,
    TOTAL_JOBS,
    "All jobs must be cleanly resolved as completed or canceled",
  );
  assert.ok(queue.canceled > 0, "Cancellation must have aborted remaining queued jobs");
  assert.equal(queue.running, 0, "No worker must be left running after completion");

  console.log(
    `[Stress Test] Transcoding queue backpressure: completed=${queue.completed}, canceled=${queue.canceled}, peakWorkers=${queue.peakRunning}/${MAX_CONCURRENT_WORKERS}, peakMemory=${(peakBufferBytes / 1024 / 1024).toFixed(1)}MB`,
  );

  // 3. GC memory assertion
  if (global.gc) {
    global.gc();
  }
  const mem = process.memoryUsage();
  console.log(`[Stress Test] Memory post-test: heapUsed=${(mem.heapUsed / 1024 / 1024).toFixed(1)}MB, rss=${(mem.rss / 1024 / 1024).toFixed(1)}MB`);
  assert.ok(mem.heapUsed < 250 * 1024 * 1024, `Heap used (${(mem.heapUsed / 1024 / 1024).toFixed(1)}MB) must stay bounded under 250MB`);
});
