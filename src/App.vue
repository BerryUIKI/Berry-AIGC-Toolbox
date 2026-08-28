<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AppInfo {
  app_version: string;
  schema_version: number;
  database_path: string;
}

const info = ref<AppInfo | null>(null);
const error = ref("");

onMounted(async () => {
  try {
    info.value = await invoke<AppInfo>("get_app_info");
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <main class="shell">
    <h1>Berry AIGC Toolbox</h1>
    <p class="tagline">Metadata indexer and viewer for AI-generated images</p>

    <section class="status">
      <template v-if="info">
        <p><strong>App version:</strong> {{ info.app_version }}</p>
        <p><strong>Database schema:</strong> v{{ info.schema_version }}</p>
        <p class="path"><strong>Database:</strong> {{ info.database_path }}</p>
      </template>
      <p v-else-if="error" class="error">{{ error }}</p>
      <p v-else>Starting…</p>
    </section>
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #0f0f0f;
  background-color: #f6f6f6;
  font-synthesis: none;
  -webkit-font-smoothing: antialiased;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}

.shell {
  max-width: 42rem;
  margin: 0 auto;
  padding: 12vh 1.5rem;
  text-align: center;
}

.tagline {
  color: #888;
}

.status {
  margin-top: 2rem;
  padding: 1.25rem;
  border: 1px solid rgba(128, 128, 128, 0.35);
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.06);
}

.path {
  word-break: break-all;
  font-size: 0.85em;
  color: #888;
}

.error {
  color: #d33;
}
</style>
