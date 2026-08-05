import { setTheme } from "@fluentui/web-components/theme/index.js";
import { webDarkTheme, webLightTheme } from "@fluentui/tokens";

/**
 * Fluent's web components render effectively unstyled until the design-token
 * CSS custom properties are applied via `setTheme`. Without this call things
 * like <fluent-switch>/<fluent-radio> render with no visible track/thumb at
 * all, and <fluent-button appearance="..."> has no color to draw its border.
 *
 * Takes the resolved dark/light state directly rather than watching
 * `prefers-color-scheme` itself — see `src/lib/stores/appearance.ts`, which is the
 * single place that resolves "system/light/dark" (the user's Settings choice) down to
 * one boolean, for the Fluent tokens, `data-theme`, and xterm's colors alike.
 */
export function applyFluentTheme(dark: boolean) {
  setTheme(dark ? webDarkTheme : webLightTheme);
}
