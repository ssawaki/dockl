<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { sidebarHoverExpand, sidebarToggleExpanded } from "$lib/stores/appearance";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { t } from "$lib/stores/i18n";
  import Icon from "$lib/components/ui/Icon.svelte";
  import Tooltip from "$lib/components/ui/Tooltip.svelte";
  import boxIcon from "@fluentui/svg-icons/icons/box_20_regular.svg?raw";
  import boxIconFilled from "@fluentui/svg-icons/icons/box_20_filled.svg?raw";
  import imageIcon from "@fluentui/svg-icons/icons/image_20_regular.svg?raw";
  import imageIconFilled from "@fluentui/svg-icons/icons/image_20_filled.svg?raw";
  import hardDriveIcon from "@fluentui/svg-icons/icons/hard_drive_20_regular.svg?raw";
  import hardDriveIconFilled from "@fluentui/svg-icons/icons/hard_drive_20_filled.svg?raw";
  import routerIcon from "@fluentui/svg-icons/icons/router_20_regular.svg?raw";
  import routerIconFilled from "@fluentui/svg-icons/icons/router_20_filled.svg?raw";
  import storageIcon from "@fluentui/svg-icons/icons/storage_20_regular.svg?raw";
  import storageIconFilled from "@fluentui/svg-icons/icons/storage_20_filled.svg?raw";
  import settingsIcon from "@fluentui/svg-icons/icons/settings_20_regular.svg?raw";
  import settingsIconFilled from "@fluentui/svg-icons/icons/settings_20_filled.svg?raw";
  import beakerIcon from "@fluentui/svg-icons/icons/beaker_20_regular.svg?raw";
  import beakerIconFilled from "@fluentui/svg-icons/icons/beaker_20_filled.svg?raw";

  let mainItems = $derived([
    { href: resolve("/"), label: $t("nav.containers"), icon: boxIcon, iconActive: boxIconFilled },
    {
      href: resolve("/images"),
      label: $t("nav.images"),
      icon: imageIcon,
      iconActive: imageIconFilled,
    },
    {
      href: resolve("/volumes"),
      label: $t("nav.volumes"),
      icon: hardDriveIcon,
      iconActive: hardDriveIconFilled,
    },
    {
      href: resolve("/networks"),
      label: $t("nav.networks"),
      icon: routerIcon,
      iconActive: routerIconFilled,
    },
    {
      href: resolve("/storage"),
      label: $t("nav.storage"),
      icon: storageIcon,
      iconActive: storageIconFilled,
    },
  ]);

  let settingsItem = $derived({
    href: resolve("/settings"),
    label: $t("nav.settings"),
    icon: settingsIcon,
    iconActive: settingsIconFilled,
  });

  // Dev-only: links to the design reference page. The whole route is stripped out of
  // production builds (see scripts/build-without-dev-routes.mjs), and `import.meta.env.DEV`
  // is statically replaced with `false` in that build, so this branch — link and all — is
  // dead-code-eliminated from the compiled output too.
  const devDesignItem = {
    href: resolve("/dev-design"),
    label: "Design Patterns",
    icon: beakerIcon,
    iconActive: beakerIconFilled,
  };

  // Pinned open by the titlebar button (see Titlebar.svelte), which is always available.
  let pinnedExpanded = $derived($sidebarToggleExpanded);

  // Tooltips only earn their place while the rail is showing icons alone: pinned open the
  // label is right there, and with hover-expand on, hovering an item is itself what widens
  // the rail, so a tooltip would just race the label it's standing in for.
  let tooltipDisabled = $derived(pinnedExpanded || $sidebarHoverExpand);

  /** The item being hovered/focused, i.e. the one the tooltip is currently labelling. */
  let tip = $state<{ el: HTMLElement; label: string } | null>(null);

  function openTip(el: HTMLElement, label: string) {
    if (tooltipDisabled) return;
    tip = { el, label };
  }

  function closeTip() {
    tip = null;
  }

  // The rail can widen (or get pinned open) while a tooltip is up — expanding to show
  // the very label the tooltip is standing in for.
  $effect(() => {
    if (tooltipDisabled) tip = null;
  });
</script>

{#snippet navItem(item: (typeof mainItems)[number])}
  {@const active = $page.url.pathname === item.href}
  <!-- No `title`: <Tooltip> replaces the native one, which can't be styled, is slow to
       appear, and would double up with it while the rail is collapsed. `focusin` is
       filtered to keyboard focus — clicking an item focuses it too, and a tooltip left
       hanging around after the click has already been made is pure noise. -->
  <!-- `data-sveltekit-keepfocus`: SvelteKit blurs the active element on every navigation
       (client.js's `!keepfocus && document.activeElement.blur()`), which for a nav rail
       means clicking an item drops focus to nowhere — the arrow keys below then have
       nothing to move from. Keeping focus on the item that was just clicked is also what
       makes click-then-arrow work as one continuous gesture. -->
  <!-- Every `href` reaching this snippet is built with resolve() above; the rule can't
       follow the value through the snippet parameter. -->
  <!-- eslint-disable svelte/no-navigation-without-resolve -->
  <a
    href={item.href}
    class="sidebar-item"
    class:active
    aria-current={active ? "page" : undefined}
    data-sveltekit-keepfocus
    data-roving-item
    onpointerenter={(e) => openTip(e.currentTarget, item.label)}
    onpointerleave={closeTip}
    onpointerdown={closeTip}
    onfocusin={(e) =>
      e.currentTarget.matches(":focus-visible") && openTip(e.currentTarget, item.label)}
    onfocusout={closeTip}
  >
    <span class="sidebar-item-icon">
      <Icon svg={active ? item.iconActive : item.icon} size={19} />
    </span>
    <span class="sidebar-item-label">{item.label}</span>
  </a>
  <!-- eslint-enable svelte/no-navigation-without-resolve -->
{/snippet}

<!-- `.sidebar-slot` reserves the resting 44px width in the flex layout; `.sidebar` itself
     is absolutely positioned inside it so widening on hover overlays the main content
     instead of pushing it (see .sidebar-slot / .sidebar below). -->
<div class="sidebar-slot" class:always-expand={pinnedExpanded}>
  <!-- Roving tabindex, as the master lists use: Tab enters and leaves the whole nav in one
       stop, and ArrowUp/Down move between items inside it. Without it, tabbing through the
       window means six stops before reaching the page content. The dividers and the spacer
       aren't matched by the selector, so they're skipped rather than being focus stops. -->
  <nav
    class="sidebar"
    class:hover-expand={$sidebarHoverExpand}
    class:always-expand={pinnedExpanded}
    use:rovingFocus={{ selector: "[data-roving-item]" }}
  >
    {#each mainItems as item (item.href)}
      {#if item.href === resolve("/storage")}
        <div class="divider"></div>
      {/if}
      {@render navItem(item)}
    {/each}
    <div class="spacer"></div>
    {#if import.meta.env.DEV}
      {@render navItem(devDesignItem)}
    {/if}
    {@render navItem(settingsItem)}
  </nav>
</div>

{#if tip}
  <Tooltip anchor={tip.el} label={tip.label} placement="right" onClose={closeTip} />
{/if}

<style>
  /* Floor is set by the longest label: an item is 5px margin + 34px icon + 8px gap +
     the text, and 「ネットワーク」 is six full-width characters at 13px — about 78px —
     so roughly 130px is needed. The rest is breathing room; narrowing much past this
     starts clipping (the sidebar is `overflow: hidden`). */
  .sidebar-slot,
  .sidebar {
    --sidebar-expanded-width: 160px;
  }

  .sidebar-slot {
    position: relative;
    width: 44px;
    flex-shrink: 0;
    transition: width 120ms ease;
  }

  /* Unlike hover-expand (which overlays the main content, see .sidebar below),
     always-expand needs the reserved layout space widened too, since there's no
     hover state to distinguish "about to overlay" from "at rest". */
  .sidebar-slot.always-expand {
    width: var(--sidebar-expanded-width);
  }

  .sidebar {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    z-index: 20;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 4px;
    width: 44px;
    padding-top: 8px;
    padding-bottom: 8px;
    overflow: hidden;
    background: transparent;
    transition:
      width 120ms ease,
      background 120ms ease;
  }

  .sidebar.hover-expand:hover,
  .sidebar.always-expand {
    width: var(--sidebar-expanded-width);
  }

  /* Hover overlays other page content (not just the window backdrop), so it stays more
     opaque than always-expand to read clearly as a floating panel over whatever's
     beneath it — but still translucent, with blur/shadow, rather than a flat fill.

     `:not(.always-expand)` because while it's pinned open the rail is already at full
     width and sitting in reserved layout space, overlaying nothing. Adding the floating-
     panel treatment there would make it visibly change on hover for no reason — there's
     nothing for it to float above. */
  .sidebar.hover-expand:not(.always-expand):hover {
    background: var(--dockl-titlebar-strong);
    backdrop-filter: blur(16px);
    box-shadow: 2px 0 12px rgba(0, 0, 0, 0.2);
  }

  /* Always-expand sits flush in reserved layout space (see .sidebar-slot.always-expand)
     rather than overlaying anything, so it can stay translucent like the titlebar above
     it and let the Mica/Acrylic backdrop show through instead of reading as a solid panel. */
  .sidebar.always-expand {
    background: var(--dockl-titlebar);
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    margin: 0 5px;
    border-radius: var(--dockl-radius);
    color: var(--dockl-text-secondary);
    text-decoration: none;
    cursor: default;
    white-space: nowrap;
  }

  .sidebar-item:hover {
    background: var(--dockl-surface-hover);
  }

  .sidebar-item.active {
    background: var(--dockl-surface);
    color: var(--dockl-accent);
  }

  .sidebar-item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    flex-shrink: 0;
  }

  .sidebar-item-label {
    font-size: 13px;
  }

  .divider {
    align-self: center;
    width: 26px;
    height: 1px;
    margin: 4px 0;
    background: var(--dockl-border);
    flex-shrink: 0;
  }

  .spacer {
    flex: 1;
  }
</style>
