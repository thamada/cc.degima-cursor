# degima-cursor — 設計仕様

種別: 現行実装の設計正本／リポジトリ: `degima-cursor`

> **目的:** DEGIMA Device を Cursor IDE の Remote-SSH で `/root` として開く CLI（degima-agent の **AI Agent** 相当）を再実装できること。

## 1. 概要

| 項目 | 内容 |
| --- | --- |
| 言語 | Rust |
| 実行 | `make run`（対話入力または引数で SSH コマンドを渡す） |
| ソース | `src/` |
| 対応機能 | degima-agent Electron の AI Agent（`open-in-cursor.js`） |

処理フロー:

1. SSH コマンドを取得（引数またはプロンプト）
2. `user` / `host` / `port` を解析
3. 秘密鍵パスを解決（既定 `~/.ssh/cursor2_id_ed25519`）
4. `~/.ssh/config` に Device 別 Host ブロックを upsert
5. Cursor を Remote-SSH で `/root` へ detached 起動

## 2. 入力

### 2-1. SSH コマンド

形式は degima-agent `parseSshCmd` と同じ。

```
ssh root@dk11-kusaka24.degima.ai -p 10008
```

| 要素 | 規則 |
| --- | --- |
| 前置 | 必ず `ssh ` で始まる |
| user@host | `@` あり。user 省略時は `root` |
| bare host | `@` なしの非オプション token |
| `-p PORT` | 省略時は `22` |

### 2-2. 秘密鍵

明示指定が無い場合、接続先へ `BatchMode` で認証できる鍵を自動選択する（ターミナルで通る鍵と Cursor Remote-SSH の `IdentityFile` を揃えるため）。

| 優先順 | ソース |
| --- | --- |
| 1 | `--ssh-key PATH` / `DEGIMA_CURSOR_SSH_KEY` / `SSH_KEY`（指定時はその鍵のみ。拒否ならエラー） |
| 2 | `~/.ssh/id_ed25519` |
| 3 | `~/.ssh/id_rsa` |
| 4 | `~/.ssh/cursor2_id_ed25519` |
| 5 | degima-agent Dashboard 鍵（例: `~/Library/Application Support/degima-agent/ssh-keys/degima_dashboard_ed25519`） |

いずれも無い／拒否の場合はエラー終了。成功時は採用した鍵パスを標準出力へ表示する。

## 3. SSH config

degima-agent と同じマーカー／エイリアス規約を使い、同一 Device を両ツールで共有できる。

```
# BEGIN degima-terminal rocket dk11-kusaka24.degima.ai:10008
Host degima-dk11-kusaka24.degima.ai-10008
  HostName dk11-kusaka24.degima.ai
  User root
  Port 10008
  IdentityFile /Users/.../.ssh/cursor2_id_ed25519
  IdentitiesOnly yes
  StrictHostKeyChecking accept-new
# END degima-terminal rocket dk11-kusaka24.degima.ai:10008
```

- `Host` エイリアス: `degima-{host}-{port}`（host の非 `[a-zA-Z0-9.-]` は `-`）
- 既存マーカー区間があれば置換、なければ末尾追記
- レガシー（マーカー行に `{host}:{port}` が無いブロック）は削除
- `.ssh` chmod `0700`、`config` chmod `0600`

## 4. Cursor 起動

| OS | 探索 | 引数 |
| --- | --- | --- |
| macOS | `cursor` / `/usr/local/bin/cursor` / `/Applications/Cursor.app/.../bin/cursor` | `--folder-uri vscode-remote://ssh-remote+{sshHost}/root --new-window` |
| Linux | 同上 | `--remote ssh-remote+{sshHost} /root` |
| Windows | `%LOCALAPPDATA%\Programs\cursor\Cursor.exe` 等 | `--folder-uri ... --new-window` |

起動後はプロセスを待たず CLI は終了する。

## 5. モジュール

| ファイル | 役割 |
| --- | --- |
| `src/main.rs` | 引数／対話入力、鍵解決、オーケストレーション |
| `src/ssh_cmd.rs` | `parse_ssh_cmd` |
| `src/ssh_config.rs` | `ensure_device_ssh_config` |
| `src/cursor.rs` | Cursor 探索・Remote-SSH 起動 |

## 6. ローカル Rust 環境

システム全域の rustup には依存しない。`install.sh` がプロジェクト配下へ入れる。

| パス | 内容 |
| --- | --- |
| `.local/cargo` | `CARGO_HOME`（`cargo` / `rustc` / `rustup`） |
| `.local/rustup` | `RUSTUP_HOME`（toolchain） |
| `.local/env.sh` | PATH / `CARGO_HOME` / `RUSTUP_HOME` を export するヘルパ |

`make build` / `make run` 等は `.local/cargo/bin/cargo` が無ければ先に `install.sh` を実行する。

## 7. Makefile

| ターゲット | 動作 |
| --- | --- |
| `install` | `.local` へ rustup/cargo をインストール |
| `run`（既定） | ローカル `cargo build` 後 `cargo run` |
| `build` | デバッグビルド（未インストールなら install） |
| `release` | リリースビルド |
| `check` | `cargo check` |
| `clean` | `cargo clean` と `.local` 削除 |
