import type { MessageKey } from "./en";

// Fully Japanese — Docker action verbs are translated too (see ja-en.ts for the hybrid variant).
export const ja: Record<MessageKey, string> = {
  "nav.containers": "コンテナ",
  "nav.images": "イメージ",
  "nav.volumes": "ボリューム",
  "nav.networks": "ネットワーク",
  "nav.storage": "ストレージ",
  "nav.settings": "設定",

  "app.connecting": "WSL2に接続中...",
  "app.starting": "WSL2を起動中...",
  "app.connectFailed": "WSL2に接続できませんでした",
  "app.retry": "再試行",

  "common.cancel": "キャンセル",
  "common.close": "閉じる",
  "common.loading": "読み込み中...",
  "common.refresh": "更新",
  "common.connecting": "接続中...",
  "common.copy": "コピー",
  "common.copied": "{value} をコピーしました",
  "common.unknown": "不明",
  "common.comingSoon": "近日対応予定です。",
  "common.yes": "はい",
  "common.no": "いいえ",
  "common.confirmIrreversible": "この操作は取り消せません。",

  "resource.images": "イメージ",
  "resource.volumes": "ボリューム",
  "resource.networks": "ネットワーク",

  // Deliberately not plain "削除". These run `docker {resource} prune`, sweeping every
  // unused resource of a kind at once, while the 削除 button on each row is that one
  // item's `rm` — with both labelled 削除 on the same page, nothing distinguished them
  // (same conflation `action.down` had). "一括" says it's a sweep; "削除" is kept rather
  // than a softer word like 整理 because volumes lose their data here, and the label
  // shouldn't read as tidying-up.
  "prune.button": "未使用の{resource}を一括削除",
  "prune.pending": "未使用の{resource}を一括削除しています...",
  "prune.success": "未使用の{resource}を一括削除しました",
  "prune.error": "一括削除に失敗しました: {error}",
  "confirmRemove.messageInUse":
    "「{name}」を削除します。他のコンテナで使用中の場合は削除に失敗します。",

  "table.name": "名前",
  "table.composeProject": "Composeプロジェクト",
  "table.driver": "ドライバー",
  "table.scope": "スコープ",
  "table.size": "サイズ",

  "titlebar.wslDistro": "接続中のWSLディストロ",
  "titlebar.openShell": "WSLシェルを開く",
  "titlebar.toggleSidebar": "サイドバーの開閉",
  "titlebar.minimize": "最小化",
  "titlebar.maximize": "最大化",

  "action.start": "開始",
  "action.stop": "停止",
  "action.restart": "再起動",
  "action.remove": "削除",
  "action.pause": "一時停止",
  "action.unpause": "再開",
  "action.up": "開始",
  // Deliberately not "削除". This runs `docker compose down` — removing the project's
  // containers *and* its network — while `action.remove` next to it is a single
  // container's `rm -f`; with both labelled "削除" on identical trash icons, nothing
  // distinguished them. There's no established Japanese for "down", so the transliterated
  // command name keeps the tie to the CLI while "終了" says what happens.
  "action.down": "終了（ダウン）",
  "action.prune": "一括削除",

  "toast.start.pending": "{name} を開始しています...",
  "toast.start.success": "{name} を開始しました",
  "toast.start.error": "{name} の開始に失敗しました: {error}",
  "toast.stop.pending": "{name} を停止しています...",
  "toast.stop.success": "{name} を停止しました",
  "toast.stop.error": "{name} の停止に失敗しました: {error}",
  "toast.restart.pending": "{name} を再起動しています...",
  "toast.restart.success": "{name} を再起動しました",
  "toast.restart.error": "{name} の再起動に失敗しました: {error}",
  "toast.remove.pending": "{name} を削除しています...",
  "toast.remove.success": "{name} を削除しました",
  "toast.remove.error": "{name} の削除に失敗しました: {error}",
  "toast.pause.pending": "{name} を一時停止しています...",
  "toast.pause.success": "{name} を一時停止しました",
  "toast.pause.error": "{name} の一時停止に失敗しました: {error}",
  "toast.unpause.pending": "{name} を再開しています...",
  "toast.unpause.success": "{name} を再開しました",
  "toast.unpause.error": "{name} の再開に失敗しました: {error}",
  "toast.up.pending": "{name} を開始しています...",
  "toast.up.success": "{name} を開始しました",
  "toast.up.error": "{name} の開始に失敗しました: {error}",
  // "終了" to match `action.down`'s label, and to keep these distinguishable from
  // `toast.remove.*` — the two were word-for-word identical, so a toast gave no hint
  // whether one container or a whole Compose project had just gone away.
  "toast.down.pending": "{name} を終了しています...",
  "toast.down.success": "{name} を終了しました",
  "toast.down.error": "{name} の終了に失敗しました: {error}",

  "errors.connectTimeout":
    "WSL2が応答しなくなりました（{seconds}秒以内に応答がありません）。時間を置くと復帰することがあります。少し待ってから再試行してください。",
  "errors.notConfigured": "Dockerに接続していません。設定画面からセットアップを完了してください。",
  "errors.wslUnavailable": "WSL2に接続できませんでした: {detail}",
  "errors.noDistroFound": "Dockerが入ったWSL2ディストロが見つかりませんでした。",

  "errors.noDistroSelected": "接続先が不明です",
  "errors.connectionSwitchFailed": "接続方式の切り替えに失敗しました: {error}",
  "errors.notConnectedSetupRequired":
    "接続情報が保存されていません。初回セットアップを完了してください。",
  "errors.copyFailed": "コピーに失敗しました: {error}",

  "toastStack.copyError": "エラーをコピー",
  "toastStack.showDetails": "詳細を表示",

  "containers.loading": "コンテナを読み込み中...",
  "containers.empty": "コンテナが見つかりません。",
  "containers.section.running": "起動中",
  "containers.section.stopped": "停止中",
  "containers.status.allRunning": "すべて起動中",
  "containers.status.partiallyRunning": "一部起動中",
  "containers.showInExplorer": "エクスプローラーで表示",
  "containers.expand": "展開",
  "containers.collapse": "折りたたむ",
  "containers.expandAriaLabel": "{name} を展開",
  "containers.collapseAriaLabel": "{name} を折りたたむ",
  "containers.confirmRemove.title": "コンテナを削除",
  "containers.confirmRemove.message": "「{name}」を停止してから削除します。",
  "containers.confirmComposeDown.title": "Composeプロジェクトを終了",
  "containers.confirmComposeDown.message":
    "「{project}」を停止し、コンテナとネットワークを削除します（ボリュームは保持されます）。",
  "errors.explorerOpenFailed": "エクスプローラーで開けませんでした: {error}",

  "containers.tab.info": "情報",
  "containers.tab.stats": "統計",
  "containers.tab.logs": "ログ",
  "containers.tab.terminal": "ターミナル",
  "containers.detail.placeholder": "左の一覧からコンテナを選択してください。",
  "containers.detail.terminalUnavailable": "起動中のコンテナのみターミナルに接続できます。",
  "containers.detail.currentStatus": "現在の状態: {status}",

  "stats.notRunningHint": "起動中のコンテナのみCPU・メモリ・I/Oの統計情報を取得できます。",
  "stats.memory": "メモリ",
  "stats.max": "MAX {percent}%（{cores}）",
  "stats.blockIO": "ブロック I/O",
  "stats.blockIO.value": "読取 {read} / 書込 {write}",
  "stats.networkIO": "ネットワーク I/O",
  "stats.loading": "統計情報を取得中...",
  "stats.storageTotal": "合計（イメージ込み）: {size}",

  "terminal.wrapOff": "折り返しなし",
  "terminal.wrapOn": "折り返しあり",
  "terminal.wrapOffAriaLabel": "折り返しをオフにする",
  "terminal.wrapOnAriaLabel": "折り返しをオンにする",
  "logs.ended": "ログ出力が終了しました（コンテナが停止しました）",
  "terminal.sessionEnded": "セッションが終了しました",

  "terminal.search.placeholder": "検索",
  "terminal.search.ariaLabel": "ターミナル内を検索",
  "terminal.search.noMatches": "見つかりません",
  "terminal.search.previous": "前へ",
  "terminal.search.previousAriaLabel": "前を検索",
  "terminal.search.next": "次へ",
  "terminal.search.nextAriaLabel": "次を検索",
  "terminal.search.closeAriaLabel": "検索を閉じる",

  "wslShell.title": "WSLシェル",

  "tcpEndpoint.title": "TCPエンドポイント（Docker Engine API）",
  "tcpEndpoint.description1a":
    "「TCP接続」は、ご自身で公開したDocker Engine API互換のエンドポイント（",
  "tcpEndpoint.description1b":
    "）に接続する方式です。Podmanの`podman system service`などが該当します。Docklはこのポートを開きません — 中継プロセス方式ならポートを開かずに同じAPIへ同じ速度で接続できるため、Dockl側で開く理由がないからです。",
  "tcpEndpoint.warning":
    "公開されているエンドポイントに認証が無い場合、このWindows上で動くあらゆるプログラムがDocker APIを操作でき、WSL2内のroot権限を渡すのと同等になります。Windowsファイアウォールはループバック通信を検査せず、Webページから到達させることも可能です。信頼できるマシンでのみご利用ください。",
  "tcpEndpoint.checkConnection": "接続を確認",
  "tcpEndpoint.connectionOk": "接続を確認しました",
  "tcpEndpoint.connectionFailed": "接続できませんでした: {error}",
  "tcpEndpoint.teardown":
    "以前のバージョンのDocklでこのポートを設定した場合、以下を実行すると設定を削除してDockerデーモンを元の状態に戻せます（稼働中のコンテナは再起動されます）:",

  "toastOutput.empty": "出力はありません。",

  "images.confirmRemove.title": "イメージを削除",
  "images.table.image": "イメージ",
  "images.table.created": "作成",
  "images.table.containersInUse": "使用中コンテナ数",
  "images.loading": "イメージを読み込み中...",
  "images.empty": "イメージが見つかりません。",
  "images.section.inUse": "使用中",
  "images.section.unused": "未使用",
  "images.detail.placeholder": "左の一覧からイメージを選択してください。",
  "images.detail.id": "イメージID",
  "images.detail.tags": "タグ",
  "images.detail.noTags": "このイメージにはタグがありません（dangling）。",
  "images.moreTags": "+{count}",
  "images.prune.message": "どのコンテナからも参照されていないイメージを一括削除します。",
  "images.prune.includeTagged": "タグ付きの未使用イメージも削除する",

  "volumes.confirmRemove.title": "ボリュームを削除",
  "volumes.table.mountpoint": "マウントポイント",
  "volumes.loading": "ボリュームを読み込み中...",
  "volumes.empty": "ボリュームが見つかりません。",
  "volumes.detail.placeholder": "左の一覧からボリュームを選択してください。",
  "volumes.prune.message":
    "どのコンテナからも参照されていないボリュームを一括削除します。中のデータは失われます。",
  "volumes.prune.includeNamed": "名前付きの未使用ボリュームも削除する",

  "networks.confirmRemove.title": "ネットワークを削除",
  "networks.loading": "ネットワークを読み込み中...",
  "networks.empty": "ネットワークが見つかりません。",
  "networks.builtinCannotRemove": "Docker標準のネットワークは削除できません",
  "networks.detail.placeholder": "左の一覧からネットワークを選択してください。",
  "networks.detail.id": "ネットワークID",
  "networks.detail.internal": "内部ネットワーク",
  "networks.prune.message": "どのコンテナからも使用されていないネットワークを一括削除します。",

  "storage.loading": "ディスク使用量を読み込み中...",
  "storage.totalLabel": "Dockerが使用しているストレージの合計",
  "storage.totalHint": "目安の値です（カテゴリ間で共有される領域が二重に計上される場合があります）",
  "storage.table.kind": "種類",
  "storage.table.count": "個数",
  "storage.table.active": "使用中",
  "storage.table.reclaimable": "解放可能",
  "storage.kind.localVolumes": "ボリューム",
  "storage.kind.buildCache": "ビルドキャッシュ",
  "storage.pruneBuildCache": "ビルドキャッシュを一括削除",
  "storage.pruneBuildCache.message":
    "どのイメージにも使われていないビルドキャッシュを一括削除します。",
  "storage.pruneBuildCache.includeReusable": "今後のビルドで再利用されるキャッシュも削除する",
  "storage.pruneBuildCache.pending": "ビルドキャッシュを一括削除しています...",
  "storage.pruneBuildCache.success": "ビルドキャッシュを一括削除しました",

  "settings.appearance.heading": "外観",
  "settings.appearance.theme.label": "テーマ",
  "settings.appearance.theme.system": "システム",
  "settings.appearance.theme.light": "ライト",
  "settings.appearance.theme.dark": "ダーク",
  "settings.appearance.background.label": "背景",
  "settings.appearance.background.mica": "Mica",
  "settings.appearance.background.acrylic": "Acrylic",
  "settings.appearance.background.solid": "単色",
  "settings.appearance.sidebarHoverExpand": "サイドバーをホバーで展開する",
  "settings.appearance.error": "外観設定の表示に失敗しました（{error}）",

  "settings.language.heading": "言語",
  "settings.language.ja": "日本語",
  "settings.language.jaEn": "日本語 + English",
  "settings.language.en": "English",

  "settings.connection.heading": "接続",
  "settings.connection.description":
    "WSL2側Dockerとの通信方式（コンテナ/イメージ/ボリューム/ネットワークの一覧・操作に使われます）。",
  "settings.connection.recommended": "推奨",
  "settings.connection.dialStdio": "中継プロセス方式（dial-stdio）",
  "settings.connection.dialStdio.desc":
    "Dockl自身が起動した中継プロセス経由でDocker Engine APIに接続します。WSL2側に一切変更を加えないため、セットアップも解除も不要です。",
  "settings.connection.shellOut": "ブリッジなし（CLIシェルアウト）",
  "settings.connection.shellOut.desc":
    "操作のたびにdocker CLIを実行します。最も単純ですが、呼び出しごとにwsl.exeを起動するため低速です。中継プロセス方式が使えない場合は自動的にこの方式で動作します。",
  "settings.connection.tcp": "TCP接続（Docker Engine API）",
  "settings.connection.tcp.desc":
    "ご自身でTCP公開したDocker Engine API互換のエンドポイント（Podmanなど）に接続します。Docklはポートを開きません。中継プロセス方式と速度差はないため、そちらが使えない環境向けの選択肢です。",
  "settings.connection.switching": "切り替えています...",
  "settings.connection.current": "現在の接続先:",
  "settings.connection.notConnected": "未接続",
  "settings.connection.reconnect": "再接続",
  "settings.connection.tcpSetupLabel": "TCPエンドポイント（Docker Engine API）:",
  "settings.connection.setupButton": "詳細...",

  "settings.tray.heading": "タスクトレイ",
  "settings.tray.toggle": "ウィンドウを閉じてもタスクトレイに常駐する",

  "settings.autostart.heading": "自動起動",
  "settings.autostart.toggle": "Windowsサインイン時にDocklを自動起動する",

  "setup.welcome": "ようこそ",
  "setup.lead": "Dockerを実行しているWSL2ディストロを選択してください。",
  "setup.detecting": "WSLディストロを検出中...",
  "setup.noneFound": "WSL2のディストロが見つかりませんでした。",
  "setup.noneFoundHint":
    "WSL2をインストールし、Dockerをセットアップした上でもう一度お試しください。",
  "setup.running": "起動中",
  "setup.stopped": "停止中",
  "setup.default": "既定",
  "setup.connect": "接続する",
};
