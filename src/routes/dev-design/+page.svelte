<script lang="ts">
  /**
   * Dev-only design reference: every token/component pattern the app uses, in one
   * place, so changes to theme.css or a shared component can be eyeballed without
   * hunting through real feature pages. The whole `dev-design` route folder is
   * physically removed before `vite build` runs (see scripts/build-without-dev-routes.mjs)
   * so none of this — markup, icons, or code — ends up in the Tauri production bundle.
   */
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Tooltip, { type TooltipPlacement } from "$lib/components/ui/Tooltip.svelte";
  import { pushToast, resolveToast, showToast } from "$lib/stores/toasts";

  import checkmarkIcon from "@fluentui/svg-icons/icons/checkmark_circle_20_filled.svg?raw";
  import errorIcon from "@fluentui/svg-icons/icons/error_circle_20_filled.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";
  import copyIcon from "@fluentui/svg-icons/icons/copy_16_regular.svg?raw";
  import boxIconFilled from "@fluentui/svg-icons/icons/box_20_filled.svg?raw";
  import hardDriveIconFilled from "@fluentui/svg-icons/icons/hard_drive_20_filled.svg?raw";
  import routerIconFilled from "@fluentui/svg-icons/icons/router_20_filled.svg?raw";
  import broomIcon from "@fluentui/svg-icons/icons/broom_16_regular.svg?raw";
  import arrowClockwiseIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_regular.svg?raw";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";

  const colorTokens = [
    { name: "--dockl-accent", role: "アクセント" },
    { name: "--dockl-accent-hover", role: "アクセント (hover)" },
    { name: "--dockl-link", role: "リンク" },
    { name: "--dockl-danger", role: "危険" },
    { name: "--dockl-success", role: "成功" },
    { name: "--dockl-warning", role: "警告" },
    { name: "--dockl-text-primary", role: "本文" },
    { name: "--dockl-text-secondary", role: "補助テキスト" },
    { name: "--dockl-surface", role: "サーフェス" },
    { name: "--dockl-surface-hover", role: "サーフェス (hover)" },
    { name: "--dockl-border", role: "ボーダー" },
    { name: "--dockl-menu-bg", role: "メニュー/ダイアログ背景" },
  ];

  const icons = [
    { svg: checkmarkIcon, label: "checkmark" },
    { svg: errorIcon, label: "error" },
    { svg: dismissIcon, label: "dismiss" },
    { svg: copyIcon, label: "copy" },
    { svg: boxIconFilled, label: "box" },
    { svg: hardDriveIconFilled, label: "hard-drive" },
    { svg: routerIconFilled, label: "router" },
    { svg: broomIcon, label: "broom" },
    { svg: arrowClockwiseIcon, label: "arrow-clockwise" },
    { svg: deleteIcon, label: "delete" },
  ];

  const tooltipDemos: { text: string; label: string; placement: TooltipPlacement }[] = [
    { text: "top", label: "上に表示", placement: "top" },
    { text: "right", label: "右に表示", placement: "right" },
    { text: "bottom", label: "下に表示", placement: "bottom" },
    { text: "left", label: "左に表示", placement: "left" },
    {
      text: "長いラベル",
      label: "長いラベルは max-width: 240px で折り返します。サイドバーのように幅の狭い要素でも読めます。",
      placement: "right",
    },
  ];

  let tip = $state<{ el: HTMLElement; label: string; placement: TooltipPlacement } | null>(null);

  let confirmOpen = $state(false);

  function demoLoadingToast(status: "success" | "error") {
    const id = pushToast("処理を実行しています...");
    setTimeout(() => {
      resolveToast(id, status, status === "success" ? "処理が完了しました" : "処理に失敗しました: sample error");
    }, 1200);
  }

  function demoOutputToast() {
    const id = pushToast("コマンドを実行しています...");
    setTimeout(() => {
      resolveToast(
        id,
        "success",
        "コマンドが完了しました（詳細ボタンで出力を表示）",
        "$ docker compose up -d\n Container web-1  Started\n Container db-1   Started",
      );
    }, 1000);
  }
</script>

<PageHeader title="Design Patterns" />

<p class="intro">
  theme.css のトークンと共通コンポーネントの見本です。dev-design は build 時に自動で除外されるため本番出力には含まれません。
</p>

<section class="dockl-surface block">
  <h2>カラートークン</h2>
  <div class="swatch-grid">
    {#each colorTokens as t (t.name)}
      <div class="swatch">
        <span class="swatch-color" style={`background: var(${t.name})`}></span>
        <span class="swatch-name">{t.name}</span>
        <span class="swatch-role">{t.role}</span>
      </div>
    {/each}
  </div>
</section>

<section class="dockl-surface block">
  <h2>タイポグラフィ</h2>
  <h1 class="sample-h1">見出し (h1) — 20px / 600</h1>
  <p class="sample-body">本文テキスト (13px) — var(--dockl-text-primary)</p>
  <p class="sample-secondary">補助テキスト (13px) — var(--dockl-text-secondary)</p>
</section>

<section class="dockl-surface block">
  <h2>ボタン</h2>
  <div class="row">
    <fluent-button appearance="primary">primary</fluent-button>
    <fluent-button appearance="outline">outline</fluent-button>
    <fluent-button appearance="subtle">subtle</fluent-button>
    <fluent-button appearance="transparent">transparent</fluent-button>
    <fluent-button appearance="outline" disabled>disabled</fluent-button>
  </div>
</section>

<section class="dockl-surface block">
  <h2>バッジ</h2>
  <div class="row">
    <fluent-badge appearance="filled" color="brand">brand</fluent-badge>
    <fluent-badge appearance="filled" color="success">success</fluent-badge>
    <fluent-badge appearance="filled" color="danger">danger</fluent-badge>
    <fluent-badge appearance="filled" color="warning">warning</fluent-badge>
    <fluent-badge appearance="outline" color="informative">outline</fluent-badge>
  </div>
</section>

<section class="dockl-surface block">
  <h2>フォームコントロール</h2>
  <div class="row">
    <fluent-checkbox checked>checkbox</fluent-checkbox>
    <fluent-switch checked></fluent-switch>
    <fluent-dropdown style="min-width: 140px">
      <fluent-listbox>
        <fluent-option value="a" selected>option A</fluent-option>
        <fluent-option value="b">option B</fluent-option>
      </fluent-listbox>
    </fluent-dropdown>
  </div>
  <fluent-radio-group orientation="horizontal">
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label><fluent-radio value="1" checked></fluent-radio> radio 1</label>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label><fluent-radio value="2"></fluent-radio> radio 2</label>
  </fluent-radio-group>
</section>

<section class="dockl-surface block">
  <h2>スピナー</h2>
  <div class="row">
    <fluent-spinner size="tiny"></fluent-spinner>
    <fluent-spinner size="small"></fluent-spinner>
    <fluent-spinner size="medium"></fluent-spinner>
  </div>
</section>

<section class="dockl-surface block">
  <h2>アイコン (@fluentui/svg-icons)</h2>
  <div class="icon-grid">
    {#each icons as i (i.label)}
      <div class="icon-cell">
        <Icon svg={i.svg} size={20} />
        <span>{i.label}</span>
      </div>
    {/each}
  </div>
</section>

<section class="dockl-surface block">
  <h2>トースト</h2>
  <div class="row">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={() => demoLoadingToast("success")}>成功 (loading→success)</fluent-button>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={() => demoLoadingToast("error")}>失敗 (loading→error)</fluent-button>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={() => showToast("success", "即時成功トースト")}>
      即時成功
    </fluent-button>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={() => showToast("error", "即時失敗トースト")}>即時失敗</fluent-button>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={demoOutputToast}>出力付き（詳細ボタン表示）</fluent-button>
  </div>
</section>

<section class="dockl-surface block">
  <h2>ツールチップ (Tooltip)</h2>
  <p class="note">ホバーまたはキーボードフォーカスで即座に表示。アンカー側の辺から広がる短いアニメーションが付きます。</p>
  <div class="row">
    {#each tooltipDemos as demo (demo.text)}
      <button
        class="tooltip-demo"
        onpointerenter={(e) => (tip = { el: e.currentTarget, ...demo })}
        onpointerleave={() => (tip = null)}
        onfocusin={(e) => e.currentTarget.matches(":focus-visible") && (tip = { el: e.currentTarget, ...demo })}
        onfocusout={() => (tip = null)}
      >
        {demo.text}
      </button>
    {/each}
  </div>
</section>

{#if tip}
  <Tooltip anchor={tip.el} label={tip.label} placement={tip.placement} onClose={() => (tip = null)} />
{/if}

<section class="dockl-surface block">
  <h2>ダイアログ</h2>
  <div class="row">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" onclick={() => (confirmOpen = true)}>確認ダイアログを開く</fluent-button>
  </div>
</section>

{#if confirmOpen}
  <ConfirmDialog
    title="サンプルの削除"
    message="これはデザイン確認用のダミーダイアログです。"
    onConfirm={() => (confirmOpen = false)}
    onCancel={() => (confirmOpen = false)}
  />
{/if}

<style>
  .intro {
    font-size: 13px;
    color: var(--dockl-text-secondary);
    margin: 0 0 16px;
  }

  .block {
    padding: 16px;
    margin-bottom: 16px;
  }

  .block h2 {
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 12px;
  }

  .note {
    font-size: 12px;
    color: var(--dockl-text-secondary);
    margin: 0 0 12px;
  }

  .tooltip-demo {
    padding: 6px 12px;
    font-family: var(--dockl-font);
    font-size: 13px;
    color: var(--dockl-text-primary);
    background: var(--dockl-surface);
    border: 1px solid var(--dockl-border);
    border-radius: var(--dockl-radius);
    cursor: default;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }

  .row:last-child {
    margin-bottom: 0;
  }

  .swatch-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 12px;
  }

  .swatch {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .swatch-color {
    display: block;
    height: 40px;
    border-radius: var(--dockl-radius);
    border: 1px solid var(--dockl-border);
  }

  .swatch-name {
    font-family: Consolas, "Cascadia Code", monospace;
    font-size: 11px;
  }

  .swatch-role {
    font-size: 11px;
    color: var(--dockl-text-secondary);
  }

  .sample-h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .sample-body {
    font-size: 13px;
    color: var(--dockl-text-primary);
    margin: 0 0 4px;
  }

  .sample-secondary {
    font-size: 13px;
    color: var(--dockl-text-secondary);
    margin: 0;
  }

  .icon-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: 12px;
  }

  .icon-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--dockl-text-secondary);
  }
</style>
