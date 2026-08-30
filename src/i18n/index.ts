import { ref, computed } from "vue";
import { en } from "./locales/en";
import { zhCN } from "./locales/zh-CN";
import { zhTW } from "./locales/zh-TW";
import { ja } from "./locales/ja";
import { de } from "./locales/de";
import { fr } from "./locales/fr";
import { es } from "./locales/es";

export type LocaleKey = "en" | "zh-CN" | "zh-TW" | "ja" | "de" | "fr" | "es";

export interface LocaleOption {
  key: LocaleKey;
  label: string;
}

export const SUPPORTED_LOCALES: LocaleOption[] = [
  { key: "en", label: "English" },
  { key: "zh-CN", label: "简体中文" },
  { key: "zh-TW", label: "繁體中文" },
  { key: "ja", label: "日本語" },
  { key: "de", label: "Deutsch" },
  { key: "fr", label: "Français" },
  { key: "es", label: "Español" },
];

const messages: Record<LocaleKey, typeof en> = {
  en,
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  ja,
  de,
  fr,
  es,
};

function getInitialLocale(): LocaleKey {
  try {
    const saved = localStorage.getItem("berry_locale") as LocaleKey;
    if (saved && saved in messages) {
      return saved;
    }
    const navLang = navigator.language;
    if (navLang.startsWith("zh-TW") || navLang.startsWith("zh-HK")) return "zh-TW";
    if (navLang.startsWith("zh")) return "zh-CN";
    if (navLang.startsWith("ja")) return "ja";
    if (navLang.startsWith("de")) return "de";
    if (navLang.startsWith("fr")) return "fr";
    if (navLang.startsWith("es")) return "es";
  } catch {
    // fallback
  }
  return "en";
}

export const currentLocale = ref<LocaleKey>(getInitialLocale());

export function setLocale(locale: LocaleKey) {
  currentLocale.value = locale;
  try {
    localStorage.setItem("berry_locale", locale);
  } catch {
    // ignore
  }
}

export const t = computed(() => messages[currentLocale.value] || messages.en);
