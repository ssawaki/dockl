import { writable, get } from "svelte/store";
import { load } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";
import { applyFluentTheme } from "$lib/fluentTheme";

export type ThemeMode = "system" | "light" | "dark";
export type WindowMaterial = "solid" | "mica" | "acrylic";

const STORE_PATH = "settings.json";

// Matches theme.css's --dockl-solid-bg. Duplicated here (rather than trusting the CSS
// attribute-selector cascade alone) because setting it as an inline style directly is a
// stronger guarantee: inline styles win over any stylesheet rule regardless of
// selector specificity, so this can't end up losing to something else unexpectedly.
const SOLID_BG = { light: "#f3f2f1", dark: "#202020" };

/** The width master lists had before they became resizable. */
export const MASTER_LIST_DEFAULT_WIDTH = 280;
/** Narrow enough to still show a name, wide enough to leave the detail panel usable. */
export const MASTER_LIST_MIN_WIDTH = 180;
export const MASTER_LIST_MAX_WIDTH = 600;

export function clampMasterListWidth(width: number): number {
  return Math.min(MASTER_LIST_MAX_WIDTH, Math.max(MASTER_LIST_MIN_WIDTH, Math.round(width)));
}

export const themeMode = writable<ThemeMode>("system");
export const windowMaterial = writable<WindowMaterial>("mica");
/**
 * Whether hovering the sidebar expands it temporarily, *on top of* the titlebar toggle.
 *
 * The toggle button is always available; this only adds hover as a second way in. It
 * replaced a four-way mode picker (icons / hover / always / toggle) whose options mostly
 * described states the toggle already reaches: "always" is the toggle pinned open,
 * "icons" is it closed. What the modes couldn't express was combining the two, which is
 * the one thing this switch does.
 */
export const sidebarHoverExpand = writable<boolean>(false);

// Whether the user has pinned the sidebar open via the titlebar button (Titlebar.svelte).
export const sidebarToggleExpanded = writable<boolean>(false);

/**
 * Width of the list beside a detail panel, in pixels — shared by every master/detail
 * page rather than stored per page, since it reflects one preference ("how much room do
 * I want the list to have"), not a per-screen layout.
 */
export const masterListWidth = writable<number>(MASTER_LIST_DEFAULT_WIDTH);

const media = window.matchMedia("(prefers-color-scheme: dark)");

function resolveDark(mode: ThemeMode): boolean {
  if (mode === "light") return false;
  if (mode === "dark") return true;
  return media.matches;
}

/**
 * Applies the current themeMode/windowMaterial everywhere it matters: the `data-theme`/
 * `data-material` attributes theme.css keys off of, Fluent's design tokens, the native
 * window's Mica/Acrylic material (via Rust — it has no way to see `data-theme` itself),
 * and already-mounted xterm instances (via a DOM event, since they aren't Svelte
 * components subscribed to this store).
 */
async function apply() {
  const mode = get(themeMode);
  const material = get(windowMaterial);
  const dark = resolveDark(mode);

  document.documentElement.dataset.theme = dark ? "dark" : "light";
  document.documentElement.dataset.material = material;
  document.body.style.backgroundColor = material === "solid" ? (dark ? SOLID_BG.dark : SOLID_BG.light) : "";
  applyFluentTheme(dark);

  try {
    await invoke("set_window_material", { material, dark });
  } catch (e) {
    console.error("set_window_material failed:", e);
  }

  window.dispatchEvent(new CustomEvent("dockl-theme-change", { detail: { dark } }));
}

// Only matters while themeMode is "system" — otherwise the OS changing doesn't affect
// what's already an explicit override.
media.addEventListener("change", () => {
  if (get(themeMode) === "system") void apply();
});

/** Loads persisted appearance settings and applies them — call once from the root layout. */
export async function initAppearance() {
  const store = await load(STORE_PATH, { autoSave: true });
  themeMode.set((await store.get<ThemeMode>("themeMode")) ?? "system");
  windowMaterial.set((await store.get<WindowMaterial>("windowMaterial")) ?? "mica");

  // Migrated from the mode picker this replaced: the two modes that involved hovering
  // become the switch turned on, the two that didn't become it off. Deliberately a new
  // key — `sidebarHoverExpand` was itself an older setting with a different meaning
  // ("hover" versus "icons"), so reading it back would misinterpret old files.
  const storedHover = await store.get<boolean>("sidebarHoverExpandEnabled");
  if (storedHover !== undefined && storedHover !== null) {
    sidebarHoverExpand.set(storedHover);
  } else {
    const legacyMode = await store.get<string>("sidebarMode");
    sidebarHoverExpand.set(legacyMode === "hover" || legacyMode === "always");
  }
  sidebarToggleExpanded.set((await store.get<boolean>("sidebarToggleExpanded")) ?? false);
  // Clamped on read too: a stored value could be out of range if the limits ever change,
  // and a list wider than the window would leave no detail panel at all.
  masterListWidth.set(
    clampMasterListWidth((await store.get<number>("masterListWidth")) ?? MASTER_LIST_DEFAULT_WIDTH),
  );

  await apply();
}

/** Persists and applies a new theme mode (Settings' 外観 section). */
export async function setThemeMode(mode: ThemeMode) {
  themeMode.set(mode);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("themeMode", mode);
  await apply();
}

/** Persists and applies a new window background material (Settings' 外観 section). */
export async function setWindowMaterial(material: WindowMaterial) {
  windowMaterial.set(material);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("windowMaterial", material);
  await apply();
}

/**
 * Persists the master list width. Called when a drag ends, not on every pointer move —
 * writing the settings file at pointer-event rate would be dozens of writes per drag.
 */
export async function setMasterListWidth(width: number) {
  const clamped = clampMasterListWidth(width);
  masterListWidth.set(clamped);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("masterListWidth", clamped);
}

/** Persists whether hovering expands the sidebar (Settings' 外観 section). */
export async function setSidebarHoverExpand(enabled: boolean) {
  sidebarHoverExpand.set(enabled);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("sidebarHoverExpandEnabled", enabled);
}

/** Persists whether the sidebar is pinned open (Titlebar's toggle button). */
export async function setSidebarToggleExpanded(expanded: boolean) {
  sidebarToggleExpanded.set(expanded);
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("sidebarToggleExpanded", expanded);
}

/** The currently resolved dark/light state, for one-off reads outside Svelte's reactivity (XtermController). */
export function isDarkNow(): boolean {
  return document.documentElement.dataset.theme === "dark";
}
