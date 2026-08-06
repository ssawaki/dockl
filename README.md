# Dockl

WSL2 上の Docker を Windows 11 から操作する、軽量なネイティブ GUI。

Docker Desktop を入れずに WSL2 へ直接 Docker を導入している環境では、Windows 側からコンテナの状態を見る手段がありません。Dockl はその一点のために作られています。WSL2 側には**何もインストールしません**。

## 特徴

**Docker Desktop に依存しない** — WSL2 内の Docker daemon と直接やり取りします。Docker Desktop がインストールされていても、していなくても動きます。

**WSL2 側に何も置かない** — 常駐サービスもバイナリも配置せず、ポートも開きません。アンインストールは Windows 側のアプリを消すだけで完結します。

**Windows 11 になじむ外観** — Fluent 2 デザイン、Mica / Acrylic の背景効果、ライト/ダークのシステム連動。

**ポーリングしない** — `docker events` を購読しているため、コンテナの起動・停止は待たずに一覧へ反映されます。

## できること

| 領域 | 内容 |
|---|---|
| **コンテナ** | 一覧・詳細、起動/停止/再起動/一時停止/削除、Compose プロジェクト単位のグルーピングと up/down |
| **ログ** | ストリーミング表示、検索、折り返し切り替え |
| **ターミナル** | コンテナへのアタッチ（`docker exec -it` 相当）。タブやコンテナを切り替えてもセッションは維持されます |
| **統計** | CPU / メモリ / ブロック I/O / ネットワーク I/O / PID 数、スパークライン付き |
| **イメージ・ボリューム・ネットワーク** | 一覧・詳細・削除・prune |
| **ストレージ** | ディスク使用量の内訳とビルドキャッシュの prune |
| **その他** | 日本語 / 英語 UI、タスクトレイ常駐、Windows ログイン時の自動起動、キーボード操作 |

## 動作要件

- Windows 11
- WSL2（systemd の有効化は不要）
- WSL2 ディストロ内に Docker がインストール済みで、`docker` コマンドが使えること

Docker のインストール形態は問いません。rootless インストールでも、ソケットの場所を Docker CLI のコンテキストから解決するため、追加設定なしで動作します。

## インストール

配布用ビルドはまだ用意していないため、手元でビルドしてください。

```bash
npm install
npm run tauri build
```

インストーラは `src-tauri/target/release/bundle/nsis/` に生成されます。管理者権限は不要で、常に現在のユーザーにのみインストールされます。

初回起動時にセットアップ画面が開き、WSL2 ディストロを選んで接続するだけで使い始められます。

## 接続方式

WSL2 の Docker とどう話すかを 3 つから選べます。設定画面でいつでも切り替えられます。

### リレープロセス（dial-stdio）— 推奨・既定

`docker system dial-stdio` を経由して Docker Engine API を使います。Dockl が起動した子プロセスだけが daemon に到達できるため、**認証すべき経路がそもそも存在しません**。ポートは開かず、設定も後片付けも不要です。

### ブリッジなし（CLI シェルアウト）

操作のたびに `wsl.exe -- docker ...` を実行します。最も単純で、WSL2 側に一切影響しません。プロセス起動のオーバーヘッドがあるぶん、上記より応答は劣ります。

### TCP 接続（Docker Engine API）

`dockerd` を `tcp://127.0.0.1:2375` で待ち受けさせて接続します。

> **注意**: この方式は Engine API を**無認証で公開**します。Docker Engine API は `--privileged` なコンテナを起動できるため、同じマシン上の任意のプロセスから WSL2 側の root 権限を取られるのと同等のリスクがあります。既に他の用途で TCP を公開している場合を除き、dial-stdio を使ってください。

## 開発

```bash
npm install          # 依存の取得と Git hooks の設定
npm run tauri dev    # 開発ビルドで起動
```

### 技術スタック

Tauri v2（Rust）+ SvelteKit（Svelte 5）。UI は Fluent UI Web Components、ターミナルは xterm.js。フロントエンドは SPA として静的ビルドされます。

### 品質チェック

```bash
npm run check:all    # svelte-check と cargo check を並列実行
npm run lint         # Prettier と ESLint
npm run format       # Prettier で整形
```

コミット時には lefthook が変更ファイルだけを Prettier / ESLint / rustfmt に通します。型チェックは時間がかかるためフックには含めていません。

## ライセンス

MIT
