<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";

export interface HelpTopic {
  id: string;
  category: string;
  title: string;
  icon: string;
  badge?: string;
  summary: string;
  content: string[];
  tips?: string[];
  shortcuts?: { key: string; desc: string }[];
}

const props = withDefaults(
  defineProps<{
    show: boolean;
    context?: string;
  }>(),
  {
    show: false,
    context: "home",
  }
);

const emit = defineEmits<{
  (e: "close"): void;
}>();

const searchQuery = ref("");
const selectedTopicId = ref("workspace-layout");
const articleContainer = ref<HTMLElement | null>(null);

const TOPICS: HelpTopic[] = [
  {
    id: "workspace-layout",
    category: "Getting Started",
    title: "Workspace Layout & 3-Pane View",
    icon: "🖥️",
    badge: "Core",
    summary: "Overview of the studio interface: Navigation Sidebar, Gallery Canvas, and Metadata Inspector.",
    content: [
      "Omera features an ultra-responsive 3-pane studio workspace optimized for high-volume creative curation.",
      "Left Sidebar: Navigate Library folders, Smart Albums, Tags, Sensitive Media, and Prompt Insights.",
      "Center Canvas: High-performance virtualized Grid, Masonry Waterfall, or Table layout with dynamic thumbnail zoom.",
      "Right Inspector: Comprehensive parameter inspector showing Generation Prompts, KSampler configs, CFG, Seeds, Models, and LoRA chips."
    ],
    tips: [
      "Press Tab or click the panel toggles in the top MenuBar (View menu) to collapse the Sidebar or Inspector for fullscreen browsing.",
      "Double-click or press Space / Enter on any card to summon the Fullscreen Quick Look Lightbox."
    ]
  },
  {
    id: "keyboard-shortcuts",
    category: "Getting Started",
    title: "Keyboard Shortcuts & Fast Curation",
    icon: "⌨️",
    summary: "Master all single-key shortcuts for rapid culling, rating, comparing, and navigation.",
    content: [
      "Keyboard shortcuts allow prompt engineers and artists to triage thousands of images without taking their hands off the keyboard.",
      "Rating & Culling: Use numeric keys 1-5 to rate images; 0 clears rating. Press Delete or Backspace to move to system Trash.",
      "Compare Mode: Select 2 to 4 images and press 'C' to launch the Side-by-Side compare matrix with synchronized pan and zoom."
    ],
    shortcuts: [
      { key: "1 - 5", desc: "Set star rating on selection" },
      { key: "0", desc: "Clear rating on selection" },
      { key: "F", desc: "Toggle favorite bookmark" },
      { key: "C", desc: "Side-by-side compare selected images" },
      { key: "Ctrl + G", desc: "Group selection into a new burst stack" },
      { key: "Ctrl + Shift + G", desc: "Ungroup active burst stack" },
      { key: "Alt + S", desc: "Set selected image as stack hero cover" },
      { key: "Space / Enter", desc: "Open fullscreen Quick Look Lightbox" },
      { key: "/ or Ctrl+F", desc: "Focus search bar" },
      { key: "F1", desc: "Open this Help Guide & Documentation" }
    ]
  },
  {
    id: "folder-modes",
    category: "Library Management",
    title: "Folder Modes & Pipeline Ingestion",
    icon: "📂",
    badge: "Architecture",
    summary: "Mode A (External Link), Mode B (Managed Vault), and Mode C (AIGC Pipeline Harvesting).",
    content: [
      "Mode A: External Link mode leaves your image files in-place on disk (read-only index). Perfect for existing generation outputs.",
      "Mode B: Managed Project Vault stores images within dedicated app storage with copy/move drag-and-drop ingestion.",
      "Mode C: AIGC Ingestion Pipeline monitors WebUI/ComfyUI output directories, debounces new generations, and safely harvests them into organized collections."
    ],
    tips: [
      "You can toggle between Subfolder recursive search and Direct single-level browsing using the folder options in the sidebar."
    ]
  },
  {
    id: "stacks-and-bursts",
    category: "Intelligent Curation",
    title: "Image Stacking & Burst Grouping",
    icon: "🃏",
    badge: "Pro",
    summary: "Poker-deck card stacking, automatic prompt clustering, hero cover selection, and draft culling.",
    content: [
      "Burst stacks consolidate variations of the same generation into a clean, collapsible poker-deck card.",
      "Auto-Stacking: Choose Tools > Organize by Prompt to cluster generations created within similar timeframes or prompt similarity.",
      "Hero Selection: The top card (Hero) represents the stack. Select any card within an expanded stack and press Alt+S to assign it as the new cover.",
      "Batch Cull: Click 'Cull Drafts' in the batch action bar or menu to automatically keep heroes and 4-5 star artworks while moving lower-rated iterations to Trash."
    ]
  },
  {
    id: "search-and-filtering",
    category: "Discovery & Analytics",
    title: "Search Syntax & Structured Filters",
    icon: "🔍",
    summary: "Structured search tokens, parameter ranges, negation, and multi-facet filtering.",
    content: [
      "Search supports structured key-value queries: prompt:cat, model:sdxl, steps:>=30, cfg:>7, rating:>=4.",
      "Negation & Exclusion: Use -tag:draft or -model:pony to exclude specific attributes.",
      "Video Facets: Filter video creations with container:mp4, fps:>=24, or duration:>=5s.",
      "Sensitive Media: Toggle the 18+ filter or search is:nsfw / is:sfw to quickly locate sensitive artworks."
    ]
  },
  {
    id: "ai-semantic-and-tagger",
    category: "Intelligent Curation",
    title: "AI Semantic Search & WD14 Tagger",
    icon: "🧠",
    badge: "AI Powered",
    summary: "Local CLIP/SigLIP text-to-image natural language search and local ONNX auto-tagger.",
    content: [
      "Natural Language Semantic Search: Toggle Semantic mode in the search bar to describe concepts freely (e.g. 'cyberpunk rainy alleyway at night').",
      "Local Vector Indexing: Generate 768-dim embeddings locally with zero data sent to external clouds via Tools > CLIP Semantic Search Index.",
      "Local WD14 Tagger: Batch extract Danbooru & general anime tags with high accuracy using Tools > Auto-Tag with WD14."
    ],
    tips: [
      "Semantic inference failures are tracked per-model and will not block gallery rendering or cause retry loops."
    ]
  },
  {
    id: "models-and-loras",
    category: "Intelligent Curation",
    title: "Checkpoint Models & LoRA Trigger Words",
    icon: "🎨",
    summary: "Model hash reverse lookups, Civitai caching, and one-click LoRA trigger word injection.",
    content: [
      "Tools > Model Manager displays all unique checkpoint base models indexed across your creations with Civitai SHA256 reverse lookups.",
      "Tools > LoRA Trigger Library automatically identifies active LoRA weights in ComfyUI / WebUI generation strings and copies trigger tokens to clipboard."
    ]
  },
  {
    id: "export-and-showcase",
    category: "Export & Collaboration",
    title: "Batch Export, Transcoding & HTML Showcase",
    icon: "📦",
    summary: "Lossless/lossy format conversion, privacy sanitization, companion sidecars, and standalone HTML showcase.",
    content: [
      "Batch Export (Ctrl+E): Convert selections to WebP, JPEG, or PNG with strict resolution downscaling constraints and collision resolution.",
      "Privacy Stripping: Select 'Strip Prompt Only', 'Strip All AI Metadata', or 'Full Sanitize' to clean files for public release.",
      "Standalone HTML Showcase: Generate a self-contained, zero-dependency responsive photo gallery with prompt inspection playable directly in any browser."
    ]
  },
  {
    id: "video-support",
    category: "Library Management",
    title: "Video & Animation AIGC Support",
    icon: "🎬",
    badge: "New in v0.4.0",
    summary: "MP4/WebM video indexing, AnimateDiff/Hunyuan/Wan2.1 workflows, and lightbox video controls.",
    content: [
      "Omera automatically parses generation parameters and workflow metadata embedded within MP4 and WebM video files.",
      "Full Lightbox playback supports looping, playback speed adjustment (0.25x - 2.0x), and frame-stepping (',' and '.')."
    ]
  },
  {
    id: "database-maintenance",
    category: "Reference & Maintenance",
    title: "Database Maintenance & Cloud Backups",
    icon: "⚙️",
    summary: "SQLite VACUUM compaction, safe WAL migrations, and S3 / WebDAV snapshot cloud backups.",
    content: [
      "Database Compaction: Tools > Database Manager allows running live SQLite VACUUM and integrity checks without downtime.",
      "Cloud Snapshots: Configure AWS S3, Cloudflare R2, MinIO, or WebDAV in Settings to create point-in-time encrypted snapshot backups."
    ]
  },
  {
    id: "settings-reference",
    category: "Reference & Maintenance",
    title: "Settings Reference & Privacy Policy",
    icon: "🛡️",
    summary: "Application preferences, theme appearance, thumbnail cache budgets, and local-first data protection.",
    content: [
      "Theme: Switch between Dark, Light, and Cyberpunk theme accents with high-contrast text rendering.",
      "Thumbnail Cache: Set maximum disk cache budget (512MB - 16GB) with automatic LRU tiered cleanup.",
      "Zero Telemetry Guarantee: Omera is 100% local-first and never sends prompts, images, or metrics to external analytics."
    ]
  }
];

const categories = computed(() => {
  const cats = new Set<string>();
  TOPICS.forEach((t) => cats.add(t.category));
  return Array.from(cats);
});

const filteredTopics = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return TOPICS;
  return TOPICS.filter(
    (item) =>
      item.title.toLowerCase().includes(q) ||
      item.summary.toLowerCase().includes(q) ||
      item.category.toLowerCase().includes(q) ||
      item.content.some((c) => c.toLowerCase().includes(q))
  );
});

const activeTopic = computed(() => {
  return TOPICS.find((t) => t.id === selectedTopicId.value) || TOPICS[0];
});

function selectTopic(id: string) {
  selectedTopicId.value = id;
  nextTick(() => {
    if (articleContainer.value) {
      articleContainer.value.scrollTop = 0;
    }
  });
}

// Context-aware deep linking
watch(
  () => props.show,
  (isOpen) => {
    if (isOpen) {
      const ctx = (props.context || "").toLowerCase();
      if (ctx.includes("stack")) selectTopic("stacks-and-bursts");
      else if (ctx.includes("video")) selectTopic("video-support");
      else if (ctx.includes("setting")) selectTopic("settings-reference");
      else if (ctx.includes("model") || ctx.includes("lora")) selectTopic("models-and-loras");
      else if (ctx.includes("clip") || ctx.includes("semantic") || ctx.includes("tag")) selectTopic("ai-semantic-and-tagger");
      else if (ctx.includes("export")) selectTopic("export-and-showcase");
      else if (ctx.includes("database") || ctx.includes("backup")) selectTopic("database-maintenance");
      else if (ctx.includes("search")) selectTopic("search-and-filtering");
    }
  },
  { immediate: true }
);
</script>

<template>
  <div v-if="show" class="help-drawer-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="help-drawer" role="dialog" aria-labelledby="help-drawer-title" aria-modal="true">
      <!-- Drawer Header -->
      <div class="drawer-header">
        <div class="header-branding">
          <span class="header-icon">📖</span>
          <div>
            <h2 id="help-drawer-title" class="drawer-title">Feature Guide & Documentation</h2>
            <p class="drawer-subtitle">Offline studio manuals, shortcuts, and workflow guides</p>
          </div>
        </div>
        <button
          type="button"
          class="btn-close"
          aria-label="Close Help Guide"
          @click="emit('close')"
        >
          ✕
        </button>
      </div>

      <!-- Search Input -->
      <div class="drawer-search-bar">
        <span class="search-icon">🔎</span>
        <input
          v-model="searchQuery"
          type="search"
          class="drawer-search-input"
          placeholder="Search guides, parameters, shortcuts (e.g. stacks, video, models)..."
        />
        <button v-if="searchQuery" class="clear-search-btn" @click="searchQuery = ''">✕</button>
      </div>

      <!-- Drawer Body: Split Navigation and Reader -->
      <div class="drawer-body">
        <!-- Sidebar Navigation -->
        <nav class="drawer-nav" aria-label="Help topics">
          <div v-for="cat in categories" :key="cat" class="nav-category-group">
            <h3 class="category-header">{{ cat }}</h3>
            <ul class="topic-list">
              <li
                v-for="item in filteredTopics.filter((t) => t.category === cat)"
                :key="item.id"
              >
                <button
                  type="button"
                  class="topic-button"
                  :class="{ active: selectedTopicId === item.id }"
                  @click="selectTopic(item.id)"
                >
                  <span class="topic-icon">{{ item.icon }}</span>
                  <span class="topic-label">{{ item.title }}</span>
                  <span v-if="item.badge" class="topic-badge">{{ item.badge }}</span>
                </button>
              </li>
            </ul>
          </div>

          <div v-if="filteredTopics.length === 0" class="empty-search">
            <p>No documentation found matching "{{ searchQuery }}".</p>
          </div>
        </nav>

        <!-- Article Content Viewer -->
        <main ref="articleContainer" class="drawer-content" tabindex="0">
          <div class="article-header">
            <div class="article-category">{{ activeTopic.category }}</div>
            <h1 class="article-title">{{ activeTopic.icon }} {{ activeTopic.title }}</h1>
            <p class="article-summary">{{ activeTopic.summary }}</p>
          </div>

          <div class="article-body">
            <p v-for="(paragraph, idx) in activeTopic.content" :key="idx" class="article-paragraph">
              {{ paragraph }}
            </p>

            <!-- Shortcuts Table if provided -->
            <div v-if="activeTopic.shortcuts && activeTopic.shortcuts.length" class="shortcuts-section">
              <h4 class="section-heading">⚡ Essential Shortcuts</h4>
              <div class="shortcut-matrix">
                <div v-for="(sc, sidx) in activeTopic.shortcuts" :key="sidx" class="shortcut-row">
                  <kbd class="key-pill">{{ sc.key }}</kbd>
                  <span class="shortcut-desc">{{ sc.desc }}</span>
                </div>
              </div>
            </div>

            <!-- Pro Tips Callout if provided -->
            <div v-if="activeTopic.tips && activeTopic.tips.length" class="pro-tips-card">
              <div class="tip-badge">💡 Pro Tip</div>
              <ul class="tip-list">
                <li v-for="(tip, tidx) in activeTopic.tips" :key="tidx">{{ tip }}</li>
              </ul>
            </div>
          </div>
        </main>
      </div>

      <!-- Drawer Footer -->
      <div class="drawer-footer">
        <span class="footer-hint">Press <kbd class="kbd-hint">Esc</kbd> to close · <kbd class="kbd-hint">F1</kbd> to summon anywhere</span>
        <button type="button" class="btn btn-secondary" @click="emit('close')">Done</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.help-drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.help-drawer {
  width: 780px;
  max-width: 90vw;
  height: 100%;
  background: var(--bg-surface, #1e1e24);
  color: var(--text-main, #e0e0e0);
  display: flex;
  flex-direction: column;
  box-shadow: -8px 0 32px rgba(0, 0, 0, 0.5);
  border-left: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  animation: slideIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slideIn {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

@media (prefers-reduced-motion: reduce) {
  .help-drawer-backdrop,
  .help-drawer {
    animation: none;
  }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  background: var(--bg-header, rgba(255, 255, 255, 0.02));
}

.header-branding {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  font-size: 26px;
}

.drawer-title {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--text-title, #ffffff);
}

.drawer-subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--text-muted, #8e8e93);
}

.btn-close {
  background: transparent;
  border: none;
  font-size: 18px;
  color: var(--text-muted, #8e8e93);
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  transition: all 0.12s;
}

.btn-close:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.drawer-search-bar {
  position: relative;
  padding: 10px 20px;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.15);
}

.search-icon {
  position: absolute;
  left: 30px;
  font-size: 14px;
  color: var(--text-muted, #8e8e93);
  pointer-events: none;
}

.drawer-search-input {
  width: 100%;
  padding: 8px 32px 8px 34px;
  background: var(--bg-input, #141418);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.12));
  border-radius: 6px;
  color: #fff;
  font-size: 13px;
  outline: none;
  transition: border-color 0.12s;
}

.drawer-search-input:focus {
  border-color: var(--accent-color, #4f80ff);
}

.clear-search-btn {
  position: absolute;
  right: 28px;
  background: none;
  border: none;
  color: var(--text-muted, #8e8e93);
  cursor: pointer;
}

.drawer-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.drawer-nav {
  width: 260px;
  border-right: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  overflow-y: auto;
  padding: 12px 8px;
  background: var(--bg-sidebar, rgba(0, 0, 0, 0.1));
}

.category-header {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted, #8e8e93);
  margin: 12px 10px 6px;
}

.topic-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.topic-button {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-main, #d0d0d0);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: all 0.1s;
}

.topic-button:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
}

.topic-button.active {
  background: var(--accent-subtle, rgba(79, 128, 255, 0.2));
  color: var(--accent-color, #4f80ff);
  font-weight: 500;
}

.topic-icon {
  font-size: 14px;
}

.topic-label {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.topic-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 10px;
  background: rgba(79, 128, 255, 0.15);
  color: var(--accent-color, #4f80ff);
  font-weight: 600;
}

.empty-search {
  padding: 20px 10px;
  text-align: center;
  color: var(--text-muted, #8e8e93);
  font-size: 12px;
}

.drawer-content {
  flex: 1;
  padding: 24px 30px;
  overflow-y: auto;
  outline: none;
}

.article-category {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--accent-color, #4f80ff);
  margin-bottom: 6px;
}

.article-title {
  margin: 0 0 10px;
  font-size: 22px;
  font-weight: 700;
  color: #fff;
}

.article-summary {
  font-size: 14px;
  color: var(--text-muted, #a0a0a8);
  line-height: 1.5;
  margin: 0 0 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
}

.article-paragraph {
  font-size: 14px;
  line-height: 1.65;
  color: var(--text-main, #d8d8de);
  margin-bottom: 14px;
}

.shortcuts-section {
  margin-top: 24px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  border-radius: 8px;
  padding: 16px;
}

.section-heading {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
}

.shortcut-matrix {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 8px;
}

.shortcut-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.key-pill {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.12);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-bottom: 2px solid rgba(255, 255, 255, 0.3);
  padding: 2px 7px;
  border-radius: 4px;
  color: #fff;
  min-width: 48px;
  text-align: center;
}

.shortcut-desc {
  color: var(--text-muted, #b0b0b8);
}

.pro-tips-card {
  margin-top: 20px;
  background: rgba(79, 128, 255, 0.08);
  border-left: 3px solid var(--accent-color, #4f80ff);
  padding: 14px 16px;
  border-radius: 0 8px 8px 0;
}

.tip-badge {
  font-size: 12px;
  font-weight: 700;
  color: var(--accent-color, #4f80ff);
  margin-bottom: 6px;
}

.tip-list {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-main, #d8d8de);
}

.drawer-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  background: var(--bg-footer, rgba(0, 0, 0, 0.2));
}

.footer-hint {
  font-size: 12px;
  color: var(--text-muted, #7c7c82);
}

.kbd-hint {
  background: rgba(255, 255, 255, 0.1);
  padding: 1px 4px;
  border-radius: 3px;
}

.btn {
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  font-weight: 500;
  border: none;
  transition: all 0.12s;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.16);
}
</style>
