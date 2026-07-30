import { setTheme } from "@fluentui/web-components/theme/index.js";
import { webDarkTheme, webLightTheme } from "@fluentui/tokens";

/**
 * Fluent's web components render effectively unstyled until the design-token
 * CSS custom properties are applied via `setTheme`. Without this call things
 * like <fluent-switch>/<fluent-radio> render with no visible track/thumb at
 * all, and <fluent-button appearance="..."> has no color to draw its border.
 */
export function initFluentTheme() {
  const media = window.matchMedia("(prefers-color-scheme: dark)");

  const apply = () => setTheme(media.matches ? webDarkTheme : webLightTheme);

  apply();
  media.addEventListener("change", apply);
}
