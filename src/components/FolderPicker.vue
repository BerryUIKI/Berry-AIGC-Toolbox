<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Folder } from "../types";

const emit = defineEmits<{
  added: [folder: Folder];
}>();

const adding = ref(false);
const error = ref("");

async function pickFolder() {
  error.value = "";
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected !== "string") return; // cancelled
  adding.value = true;
  try {
    const folder = await invoke<Folder>("add_folder", { path: selected });
    emit("added", folder);
  } catch (e) {
    error.value = String(e);
  } finally {
    adding.value = false;
  }
}
</script>

<template>
  <div class="picker">
    <button :disabled="adding" @click="pickFolder">
      {{ adding ? "Adding…" : "Add folder" }}
    </button>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.picker {
  margin-bottom: 1rem;
}

button {
  font: inherit;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.35);
  background: #2f6fed;
  color: #fff;
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: default;
}

.error {
  color: #d33;
  font-size: 0.85em;
  margin: 0.5rem 0 0;
}
</style>
