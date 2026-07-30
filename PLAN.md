# Dockl — Windows 11ネイティブ WSL2 Docker GUI 実装計画

## Context

Docker DesktopのGUIが重く、かつユーザーはDocker Desktopを導入していない（WSL2 Ubuntu側に直接Dockerを導入済み）。そのため現状、Windows側からWSL2内のコンテナ状態を確認する手段がない。この課題を解決するため、OrbStackのような軽量ネイティブGUIをWindows 11専用で新規開発する。

会話の中で以下の技術方針がユーザーと合意済み：
- **技術スタック**: Tauri v2 (Rustコア) + Svelteフロントエンド — パフォーマンス最優先のため、仮想DOMを持たないSvelteを採用
- **デザイン**: Fluent UI Web Components (Fluent 2) + `window-vibrancy`によるMica/Acrylic背景効果 — Windows 11にネイティブに溶け込む見た目
- **ターミナル描画**: xterm.js（VS Code統合ターミナルと同等の実績。WebView2上でも高品質な描画が可能なことを確認済み）
- **WSL2連携**: ハイブリッド方式
  - 一覧/状態/ログ/統計/起動停止などの非対話操作 → 3つの「接続モード」から選択可能（下記参照）、デフォルトはWSL2内に常駐する軽量ブリッジ経由でDocker Engine APIに直接アクセス（`bollard`クレート使用）
  - 対話的アタッチ（`docker exec -it`相当）→ 接続モードによらず共通で`wsl.exe`を`portable-pty`でConPTY経由スポーンし、実ターミナルとして扱う
  - Docker Compose → ラベルベースでプロジェクトをグルーピングして一覧表示、実際のup/down操作は`wsl.exe -- docker compose`にシェルアウト（compose CLIの挙動をそのまま踏襲するため）

### 接続モード（Settings/セットアップウィザードで選択可能）

WSL2側Dockerとの通信方式を1つに固定せず、`ConnectionMode`として抽象化し、ユーザーが選べるようにする。

| モード | 仕組み | 特徴 |
|---|---|---|
| **ブリッジなし（CLIシェルアウト）** | 一覧/状態/ログ/統計すべて`wsl.exe -d <distro> -- docker ...`の都度実行（ログは`docker logs -f`をpipe購読、統計は定期ポーリング） | WSL側に何も配置不要・ポートも開かない。セットアップ最速・最安全。反面ポーリング/プロセス起動オーバーヘッドあり |
| **プロキシブリッジ（デフォルト推奨）** | `dockl-bridged`をWSL内でビルド・常駐、Bearerトークン認証付きTCP | パフォーマンスと安全性のバランスが良い。WSL側にsystemdサービスが1つ増える |
| **Docker標準ブリッジ（ユーザー管理TCP）** | ユーザーが自分で`dockerd -H tcp://...`等を設定済みの場合、その`ディストロ:ポート`を直接指定して接続（認証はユーザー自身のTLS設定等に委ねるBYO方式） | 既に他用途でTCP公開している上級者向け。アプリ側は配備処理を一切行わない |

実装は`docker_bridge`内に共通トレイト（`DockerConnection`）を用意し、各モードがそれを実装する形にする。M1では「ブリッジなし」「プロキシブリッジ」の2モードを実装し、「ユーザー管理TCP」はプロキシブリッジのクライアントコードを流用できるためM5〜M6で追加する。

**開発環境確認済み**: WSL2 Ubuntu (running, systemd有効, Docker 29.6.1, docker.sock存在, ユーザーは`docker`グループ所属、`sudo -n`パスワードレス、`cargo`/`gcc`インストール済み)。この環境確認により、ブリッジバイナイはWindows側からのクロスコンパイル不要で、**WSL内で直接`cargo build --release`**すればよいことが判明（当初計画の「クロスコンパイル」リスクは解消）。

---

## アーキテクチャ概要

```
Windows 11 (Tauri v2 App / WebView2)
├─ Svelteフロントエンド（Fluent UI Web Components, xterm.js）
└─ Rustコア (src-tauri)
    ├─ docker_bridge/  … bollard経由でTCPブリッジと通信
    ├─ pty_session/    … portable-pty経由でwsl.exeをアタッチ用にスポーン
    ├─ compose/        … ラベルベースのグルーピング＋wsl.exeシェルアウト
    ├─ setup/          … WSL検出・ブリッジ配備・検証（アプリ初回起動時の内蔵ウィザード）
    ├─ tray/, settings/, notifications/
    └─ TCP 127.0.0.1:PORT (Bearerトークン認証) ──┐
                                                   ▼
WSL2 Ubuntu ディストロ
└─ systemd常駐サービス dockl-bridge.service
    └─ 自前の小さな認証付きプロキシバイナリ（dockl-bridged、WSL内でcargo buildして生成）
        127.0.0.1:PORT ⇔ /var/run/docker.sock
```

---

## セキュリティ方針（重要な確認事項）

Docker Engine APIをTCPで公開する場合、認証なしは避ける。理由:
- Windowsのループバック（127.0.0.1）宛通信は既定でファイアウォール検査対象外。同一Windows上の他プロセスやブラウザ経由のDNSリバインディング攻撃からも到達可能。
- Docker Engine APIは`--privileged -v /:/host`のようなコンテナを起動できるため、無認証で叩かれると実質WSL2側のroot権限を奪われるのと同義（RCE級リスク）。
- Docker Desktop自身も生TCPではなく、ACLで保護された名前付きパイプを使っている。

→ **自前の軽量プロキシ（dockl-bridged）でBearerトークン認証を行う方式を採用**（dockerdネイティブのTCPリスナー直接有効化は認証機構が持てないため不採用）。

---

## 配布方式

- Tauriの**NSISインストーラ**を採用（MSI/WiXは不採用）。
- `tauri.conf.json`の`bundle.windows.nsis.installMode`を**`"currentUser"`固定**にする。常に管理者権限不要のユーザーインストールのみとし、per-machineの選択肢自体を出さない。
- 理由: MSI(WiX)でper-user installを実現するには自前でWiXテンプレートをカスタマイズする必要があるが、NSISはTauriの標準設定だけでper-user installを一級サポートしている。

### インストール／セットアップの二段構え

1. **インストーラ（NSIS）time**: Windows側アプリ本体のみを配置（Program Files or `%LocalAppData%\Programs`、スタートメニュー登録）。WSL側には一切触れない。
2. **アプリ初回起動時**: アプリ内蔵の「セットアップウィザード」（`Setup/SetupWizard.svelte`）が自動表示され、WSL2ディストロ検出→prereqチェック（systemd/Docker有無）→ブリッジ配備（`sudo`プロンプトはPTY経由でライブ表示）→疎通検証、という対話フローを実行。
   - 理由: インストーラは管理者権限下やサイレント実行が前提であり、特定ユーザーのWSL環境に対話的に踏み込む処理とは実行コンテキストが一致しない。
   - 「今はスキップ」も選択可能にし、未設定の間は接続状態バッジに明示。設定画面からいつでも再実行可能。
3. **設定画面からのブリッジ管理（いつでも実行可能）**: 初回セットアップ時だけでなく、`Settings/SettingsPage.svelte`から**いつでも**「ブリッジをインストール」「ブリッジをアンインストール」「再インストール（バージョン更新）」を実行できるようにする。接続モードを後から`ShellOut`⇔`ManagedBridge`に切り替えたい場合や、単純にWSL側をクリーンにしたい場合に対応。進捗はセットアップウィザードと同じPTYストリーミングUIを再利用。
4. **アプリのアンインストール時の後片付け**（M5仕上げ項目）: アプリ側「ブリッジを削除」ボタン（上記と同一機能）を使い切ってからのアンインストールを推奨案内。加えてアンインストーラのカスタムアクションでも`systemctl disable --now dockl-bridge`＋関連ファイル削除を試行（distro/settingsが読み取れる場合のベストエフォート）。

---

## プロジェクト構成

```
dockl/
├── src-tauri/                       # Rustコア
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/main.json       # Tauri v2 権限定義
│   ├── resources/dockl-bridge/      # WSL内でビルドするブリッジのRustソース一式を同梱
│   └── src/
│       ├── main.rs / lib.rs
│       ├── state.rs                 # AppState (Arc<Mutex<...>>)
│       ├── error.rs                 # thiserror -> フロントエンドへシリアライズ可能なAppError
│       ├── docker_bridge/
│       │   ├── connection.rs          # DockerConnectionトレイト、ConnectionMode enum定義
│       │   ├── managed_bridge.rs      # プロキシブリッジ実装 (bollard http://127.0.0.1:PORT + Bearerヘッダ)
│       │   ├── shell_out.rs           # ブリッジなしモード: wsl.exe docker ... の都度実行
│       │   ├── user_managed.rs        # ユーザー管理TCP実装 (M5〜M6追加)
│       │   ├── discovery.rs          # ディストロ検出、port/token読み取り、wsl.exeラッパー
│       │   ├── containers.rs / images.rs / volumes.rs / networks.rs  # 各ConnectionMode共通インターフェース経由
│       │   ├── logs.rs               # streaming logs -> event
│       │   └── stats.rs              # streaming stats -> event（1秒間隔にスロットル）
│       ├── pty_session/
│       │   ├── session.rs            # wsl.exeをportable-ptyでスポーン、reader/writerスレッド
│       │   └── resize.rs
│       ├── compose/
│       │   ├── discovery.rs          # com.docker.compose.project/serviceラベルでグルーピング
│       │   └── actions.rs            # wsl.exe -d <distro> -- docker compose -f <path> <cmd>
│       ├── setup/
│       │   ├── wsl_detect.rs         # `wsl.exe -l -v`パース（UTF-16LE注意）
│       │   ├── deploy.rs             # ソース転送＋WSL内cargo build＋systemdユニット有効化（sudo対話はPTY経由）。初回セットアップ・設定画面からの再実行の両方から呼ばれる共通ロジック
│       │   ├── uninstall.rs          # systemctl disable --now＋関連ファイル削除。設定画面・アプリアンインストール時の両方から呼ばれる
│       │   └── verify.rs             # デプロイ後のping疎通確認
│       ├── tray/mod.rs               # Tauri v2 TrayIconBuilder
│       ├── settings/                 # トレイ挙動・自動起動・ディストロ選択の永続化
│       ├── commands/                 # #[tauri::command] 薄いラッパー群
│       └── notifications.rs          # コンテナクラッシュ時のWindowsトースト
├── src/                             # Svelteフロントエンド
│   ├── routes/ (Setup / Containers / Compose / Images / Volumes / Networks / Settings)
│   ├── components/
│   │   ├── Titlebar.svelte           # decorations:false のためのカスタムタイトルバー
│   │   ├── LogViewer.svelte           # xterm読み取り専用モード
│   │   ├── TerminalSession.svelte     # xterm対話アタッチモード
│   │   └── Sparkline.svelte
│   ├── lib/xterm/XtermController.ts
│   ├── lib/ipc/{containers,compose,pty,setup}.ts   # 型付きinvoke()ラッパー
│   ├── stores/{containers,composeProjects,settings,connection}.ts
│   └── styles/theme.css              # Fluentトークン、Mica透過用の背景設定
├── package.json / vite.config.ts / svelte.config.js
```

**Rust主要クレート**: `tauri` v2, `tauri-plugin-single-instance`, `tauri-plugin-autostart`, `tauri-plugin-store`, `tauri-plugin-notification`, `window-vibrancy`, `bollard`, `portable-pty`, `tokio`, `serde`/`serde_json`, `thiserror`+`anyhow`, `uuid`

**npm主要パッケージ**: `svelte`, `vite`, `@tauri-apps/api` v2, `@fluentui/web-components`, `@xterm/xterm` + `@xterm/addon-fit` + `@xterm/addon-search`

---

## WSL2ブリッジの詳細設計

- **ブリッジ実体**: 自前の小さな認証付きプロキシバイナリ（`dockl-bridged`）。ソースコードをアプリのresourcesに同梱し、**配備時にWSL内で`cargo build --release`して生成**（開発機にWSL+cargo+gccがあることを前提。ない場合の代替としてビルド済みバイナリのフォールバック同梱も検討）。
- **待受**: `127.0.0.1:PORT`のみ（外部公開しない）。ポートは初回セットアップ時に決定し空きを確認の上、Windows側`settings.json`に固定保存。
- **認証**: `ExecStartPre`でサービス起動毎に256bitランダムトークンを再生成し`/opt/dockl-bridge/token`（0600）に書き込み。Windows側は`wsl.exe -d <distro> -- cat /opt/dockl-bridge/token`で都度読み取り、`Authorization: Bearer`ヘッダとして利用。トークンはWSL2/ディストロ再起動のたびに自動失効・再発行される。
- **実行ユーザー**: ユーザーが`docker`グループに所属していれば、systemdサービスは`root`ではなく当該ユーザーで実行可能（docker.sockはgroup-rw）。root実行の複雑さを避けられる。ただしグループ未所属の場合のフォールバック（root実行 or グループ追加案内）も用意。
- **systemd常駐**: `/etc/systemd/system/dockl-bridge.service`（`Requires=docker.service`）。WSL2側で`systemd=true`が`/etc/wsl.conf`に設定されている前提（未設定の場合はセットアップウィザードが検出し、設定＋`wsl --shutdown`を案内）。
- **再接続フロー**: アプリ起動毎に①保存済み port/distro読み込み→②トークン再取得（この呼び出し自体がディストロのコールドブートも兼ねる）→③bollardクライアント構築→④`ping`疎通確認→失敗時は指数バックオフで再試行→UI上に接続状態バッジを表示。フォアグラウンド中は15〜30秒間隔のウォッチドッグで自動再接続。

---

## Tauri IPCコントラクト（抜粋）

- セットアップ: `setup_list_distros`, `setup_check_prereqs`, `setup_select_connection_mode`(ShellOut/ManagedBridge/UserManagedTcp), `setup_verify`
- ブリッジ管理（初回セットアップ・設定画面どちらからも呼べる共通コマンド）: `bridge_status`(未インストール/インストール済みバージョン/稼働状態), `bridge_install`(進捗は`bridge:progress`イベント), `bridge_uninstall`(進捗は`bridge:progress`イベント), `bridge_reinstall`
- コンテナ: `list_containers`, `inspect_container`, `container_action`(start/stop/restart/remove/pause), `stream_logs`→`logs:{id}`イベント, `stream_stats`→`stats:{id}`イベント
- イメージ/ボリューム/ネットワーク: `list_images`/`remove_image`/`prune_images` 等、同様のパターン
- 対話アタッチ: `start_attach_session`→`pty:{id}:data`イベント, `pty_write`, `pty_resize`, `pty_close`
- Compose: `list_compose_projects`（ラベルから高速導出）, `compose_action`→ストリーミング出力
- 設定/ウィンドウ: `get_settings`/`update_settings`, `window_minimize`/`window_toggle_maximize`/`window_close`, `connection:status`イベント
- 全体: バックグラウンドでbollardの`events`ストリームを購読し`docker:event`として配信 → UIの自動更新とクラッシュ通知に利用

---

## マイルストーン

1. **M1 基盤（接続モード抽象化＋一覧/状態/起動停止）**: Tauriスキャフォールド（Fluentテーマ、Micaタイトルバー）、`DockerConnection`トレイト設計、セットアップウィザード一連（検出→接続モード選択→[ManagedBridge選択時のみ配備]→検証）、ShellOutモードとManagedBridgeモードの両方でコンテナ一覧・start/stop/restart/remove/pause、接続状態表示・再接続。
   - Done: `wsl --shutdown`後の再起動を含め、実機でセットアップからコンテナ操作まで一通り動作（ShellOut/ManagedBridge両モードで確認）。
2. **M2 ログストリーミング**: `stream_logs`＋xterm `LogViewer`（ANSIカラー、tail、follow/pause）、`docker:event`によるリスト自動更新。
3. **M3 対話アタッチ/exec**: `pty_session`（portable-pty経由wsl.exeスポーン）、`TerminalSession`コンポーネント、単体「WSLシェルを開く」機能。
   - リスクが高い箇所（ConPTYの入れ子・リサイズ伝播）のため、実質M1と並行して早期にスパイクする。
4. **M4 Compose対応**: ラベルベースのプロジェクトビュー、up/down/restart/pullのストリーミング実行。
5. **M5 アプリシェル仕上げ**: タスクトレイ常駐/非常駐切替、単一インスタンス制御、自動起動トグル、Windowsテーマ追従、設定画面（接続モード切替、ブリッジのインストール/アンインストール/再インストールをいつでも実行可能なUI含む）、アプリアンインストール時のブリッジ後片付け。
6. **M6 追加機能（優先度に応じて）**: `user_managed.rs`（ユーザー管理TCP接続モード）追加、イメージ/ボリューム/ネットワーク管理、CPU/メモリのスパークライン表示、prune/クリーンアップダッシュボード、クラッシュ通知、クイックアクション（コンテナID/execコマンドのコピー等）。

---

## 主要リスクと対策

- **WSL2ネットワークモード（NAT/mirrored）差異**: どちらのモードでもloopback待受は動作する前提で設計し、モード別分岐は作らず、セットアップ検証ステップを実際の安全網とする。
- **systemd未有効化**: 非常によくある詰まりどころ。事前チェックで検出し、`/etc/wsl.conf`書き込み＋`wsl --shutdown`案内を明示する。
- **sudoパスワードプロンプト**: 配備処理はM3で作るPTY基盤を流用し、対話的にプロンプトを見せる（本開発機はpasswordless sudoだが、一般ユーザー環境では要パスワードが普通）。
- **コールドブート時のトークン読み取り競合**: リトライ＋バックオフで対応。「ファイル未検出」と「接続拒否」を区別してエラー表示。
- **bollardのヘッダ注入対応可否**: M1初期にスパイクして検証。対応不可なら薄いhyper/reqwestクライアントを自前実装するフォールバックを用意。
- **ConPTY＋wsl.exeの入れ子特有の癖**（リサイズ伝播等）: M3を待たずM1と並行して早期プロトタイプ検証。
- **WebView2ランタイム依存**: Windows 11ではほぼ標準搭載だが、Tauri標準のブートストラップ機構で防御的に対応。
- **WSL内ビルド依存**（cargo/gcc未導入環境）: 開発機には既にあるが、エンドユーザー環境には無い前提で、フォールバックとしてビルド済みバイナリの同梱も検討する。
- **Docker導入方法の差異**（apt/snap/get.docker.com/rootless）: ソケットパスやサービスユニット名が変わり得るため、事前チェックで実際のパスを検出し、ブリッジの`--socket`引数をハードコードせず可変にする。
- **Composeファイルの移動/削除**: ラベルに記録されたパスが古い場合、サイレント失敗ではなく「ファイルが見つからないため再指定してください」という復旧可能なエラーとして扱う。

---

## 検証方法（マイルストーン毎）

- **M1**: 実機のWSL2 Ubuntu + 公式apt版Docker Engine（Docker Desktop無し）でゼロ状態からセットアップウィザード完走、`wsl --shutdown`後の再接続、手動での`systemctl restart dockl-bridge`によるトークンローテーション後の自動再接続、`docker run -d nginx`等でのstart/stop/restart/remove/pause確認。
- **M2**: ANSIカラー出力するコンテナでの色表示確認、大量ログでのtail動作、コンテナ再起動をまたいだストリーミング継続確認。
- **M3**: `/bin/bash`と`/bin/sh`のみのコンテナ双方へのアタッチ、`top`/`vim`等のフルスクリーンアプリでの再描画・リサイズ確認、終了時（コンテナ停止/シェル終了/Ctrl-D）のクリーンアップ確認、単体WSLシェル起動の確認。
- **M4**: 実際の複数サービスcompose構成で`docker compose ps`とグルーピング結果を突き合わせ、up/down/pull/restartをCLI実行結果と比較検証。
- **M5**: トレイ常駐からの複数回起動で単一インスタンスが機能すること、自動起動有効時の再起動後の起動確認、Windowsダーク/ライト切替への追従確認、設定画面からのブリッジインストール→アンインストール→再インストールが実機で正しくWSL側の状態を変えること、アプリアンインストール時のブリッジ後片付け確認。
- **M6**: 各機能ごとのスポットチェック（例: 別ターミナルからの`docker kill`でトースト通知が飛ぶこと、prune実行後の解放容量が`docker system df`と一致すること）。

---

## 重要ファイル（実装時の要）

- `src-tauri/src/docker_bridge/discovery.rs`
- `src-tauri/src/setup/deploy.rs`
- `src-tauri/src/pty_session/session.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/tauri.conf.json`
- `src/lib/ipc/containers.ts`
- `src/lib/xterm/XtermController.ts`
