import { writable, derived } from "svelte/store";
import { load } from "@tauri-apps/plugin-store";
import { en, type MessageKey } from "$lib/i18n/locales/en";

export type { MessageKey };
import { ja } from "$lib/i18n/locales/ja";
import { jaEn } from "$lib/i18n/locales/ja-en";

export type Locale = "ja" | "ja-en" | "en";

const STORE_PATH = "settings.json";
const catalogs: Record<Locale, Record<MessageKey, string>> = { ja, "ja-en": jaEn, en };

function detectDefaultLocale(): Locale {
  return navigator.language.toLowerCase().startsWith("ja") ? "ja" : "en";
}

export const locale = writable<Locale>(detectDefaultLocale());

/**
 * `$t("key", { name })` in templates. A derived store (not a plain function) so that
 * switching `locale` produces a new function value, which re-runs every subscribed
 * template automatically — the same reactivity trick components already rely on for
 * `$themeMode`-driven UI.
 */
export const t = derived(locale, ($locale) => {
  const catalog = catalogs[$locale];
  return (key: MessageKey, params?: Record<string, string | number>): string => {
    let msg: string = catalog[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) msg = msg.replaceAll(`{${k}}`, String(v));
    }
    return msg;
  };
});

/** Persists and applies a new locale (Settings' 言語 section). */
export async function setLocale(l: Locale) {
  locale.set(l);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("locale", l);
}

/** Loads the persisted locale, if any — call once from the root layout. */
export async function initI18n() {
  const store = await load(STORE_PATH, { autoSave: true });
  const saved = await store.get<Locale>("locale");
  if (saved) locale.set(saved);
}
