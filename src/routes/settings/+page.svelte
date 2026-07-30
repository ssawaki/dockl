<script lang="ts">
  import { onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { isEnabled as autostartIsEnabled, enable as autostartEnable, disable as autostartDisable } from "@tauri-apps/plugin-autostart";
  import { setupCurrentDistro } from "$lib/ipc/setup";
  import { ensureConnected } from "$lib/connection";

  let store: Store | null = null;
  let trayEnabled = $state(false);
  let autostartEnabled = $state(false);
  let distro = $state<string | null>(null);
  let reconnecting = $state(false);
  let errorMessage = $state<string | null>(null);

  onMount(async () => {
    store = await load("settings.json", { autoSave: true });
    trayEnabled = (await store.get<boolean>("trayEnabled")) ?? false;
    autostartEnabled = await autostartIsEnabled();
    distro = await setupCurrentDistro();
  });

  async function toggleTray(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    trayEnabled = checked;
    await store?.set("trayEnabled", checked);
  }

  async function toggleAutostart(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    try {
      if (checked) {
        await autostartEnable();
      } else {
        await autostartDisable();
      }
      autostartEnabled = checked;
    } catch (err) {
      errorMessage = String(err);
    }
  }

  async function reconnect() {
    reconnecting = true;
    errorMessage = null;
    try {
      const connected = await ensureConnected();
      if (!connected) {
        errorMessage = "接続情報が保存されていません。初回セットアップを完了してください。";
        return;
      }
      distro = await setupCurrentDistro();
    } catch (err) {
      errorMessage = String(err);
    } finally {
      reconnecting = false;
    }
  }
</script>

<div class="settings-view">
  <h1>設定</h1>

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  <section class="dockl-surface">
    <h2>接続</h2>
    <p class="section-desc">WSL2側Dockerとの通信方式。現時点で利用できるのは「ブリッジなし」のみです。</p>

    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-radio-group value="shell_out" orientation="vertical">
      <fluent-radio value="shell_out" checked>
        ブリッジなし（CLIシェルアウト）
      </fluent-radio>
      <fluent-radio value="managed_bridge" disabled>
        プロキシブリッジ（近日対応）
      </fluent-radio>
      <fluent-radio value="user_managed_tcp" disabled>
        Docker標準ブリッジ / ユーザー管理TCP（近日対応）
      </fluent-radio>
    </fluent-radio-group>

    <div class="row">
      <span class="label">現在の接続先:</span>
      <span>{distro ?? "未接続"}</span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-button appearance="outline" disabled={reconnecting} onclick={reconnect}>
        <span class="btn-content">
          {#if reconnecting}
            <fluent-spinner size="tiny"></fluent-spinner>
          {/if}
          {reconnecting ? "接続中..." : "再接続"}
        </span>
      </fluent-button>
    </div>
  </section>

  <section class="dockl-surface">
    <h2>タスクトレイ</h2>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="row toggle-row">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-switch checked={trayEnabled} onchange={toggleTray}></fluent-switch>
      <span>ウィンドウを閉じてもタスクトレイに常駐する</span>
    </label>
    <p class="section-desc hint">※ トレイアイコン実装後に有効になります（現在は設定の保存のみ）。</p>
  </section>

  <section class="dockl-surface">
    <h2>自動起動</h2>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="row toggle-row">
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-switch checked={autostartEnabled} onchange={toggleAutostart}></fluent-switch>
      <span>Windowsサインイン時にDocklを自動起動する</span>
    </label>
  </section>

  <section class="dockl-surface">
    <h2>WSLブリッジ管理</h2>
    <p class="section-desc">プロキシブリッジのインストール／アンインストール／再インストール。近日対応予定です。</p>
    <div class="row">
      <fluent-button appearance="outline" disabled>インストール</fluent-button>
      <fluent-button appearance="outline" disabled>アンインストール</fluent-button>
      <fluent-button appearance="outline" disabled>再インストール</fluent-button>
    </div>
  </section>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 640px;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }

  section {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h2 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .section-desc {
    font-size: 12px;
    color: var(--dockl-text-secondary);
    margin: 0;
  }

  .hint {
    font-style: italic;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .toggle-row {
    cursor: pointer;
  }

  .label {
    color: var(--dockl-text-secondary);
  }

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
</style>
