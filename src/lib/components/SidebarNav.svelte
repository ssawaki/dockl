<script lang="ts">
  import { page } from "$app/stores";
  import Icon from "$lib/components/Icon.svelte";
  import boxIcon from "@fluentui/svg-icons/icons/box_20_regular.svg?raw";
  import boxIconFilled from "@fluentui/svg-icons/icons/box_20_filled.svg?raw";
  import settingsIcon from "@fluentui/svg-icons/icons/settings_20_regular.svg?raw";
  import settingsIconFilled from "@fluentui/svg-icons/icons/settings_20_filled.svg?raw";

  const items = [
    { href: "/", label: "コンテナ", icon: boxIcon, iconActive: boxIconFilled },
    { href: "/settings", label: "設定", icon: settingsIcon, iconActive: settingsIconFilled },
  ];
</script>

<nav class="sidebar">
  {#each items as item (item.href)}
    {@const active = $page.url.pathname === item.href}
    <a href={item.href} class="sidebar-item" class:active title={item.label}>
      <Icon svg={active ? item.iconActive : item.icon} size={19} />
    </a>
  {/each}
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 44px;
    flex-shrink: 0;
    padding-top: 8px;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--dockl-radius);
    color: var(--dockl-text-secondary);
    text-decoration: none;
  }

  .sidebar-item:hover {
    background: var(--dockl-surface-hover);
  }

  .sidebar-item.active {
    background: var(--dockl-surface);
    color: var(--dockl-accent);
  }
</style>
