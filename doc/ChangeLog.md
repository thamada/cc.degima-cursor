# 変更履歴

> **注意**:
>   本ドキュメントは変更履歴です。日付はdateコマンドで確認して2026-01-23 12:34:55のように年-月-日 時:分:秒のようにします。
>   最も最新のものから順に並べて記入します。

## 2026-11-26 00:15:09

- ソースと `doc/design.md` の差分を反映した（Python 3 パッケージ構成、CLI 用法、秘密鍵の OS 別 Dashboard パス、BatchMode 試行条件、SSH config の chmod／ホーム解決、Cursor 探索パスと detached 起動、Makefile ターゲット）。
- 実装言語を Rust から Python 3 へ全面移行したことを記載した（`degima_cursor/` — 標準ライブラリのみ。`Cargo.toml` / `src/*.rs` / `install.sh` を削除。`make run` は `python3 -m degima_cursor`）。

## 2026-11-25 04:12:36

- Remote-SSH 失敗の原因だった固定鍵 `cursor2_id_ed25519` をやめ、接続先で認証できる秘密鍵を自動選択するようにした（`src/ssh_key.rs` — `id_ed25519` / `id_rsa` / `cursor2` / degima-agent Dashboard 鍵を BatchMode で試行。`--ssh-key` 指定時はその鍵のみ）。

## 2026-11-25 04:09:15

- Rust をプロジェクトローカル `.local/{cargo,rustup}` へ入れるようにした（`install.sh` / `Makefile` — システム rustup 非依存。`make build` / `make run` は未インストール時に自動 install）。

## 2026-11-25 04:03:54

- Rust CLI `degima-cursor` を新規実装した（`src/` — SSH コマンド入力 → `~/.ssh/config` upsert → Cursor Remote-SSH で `/root` を開く。degima-agent Electron の AI Agent / `open-in-cursor.js` 相当）。
- `make run` でビルド＆実行できるようにした（`Makefile`）。
- 設計正本を `doc/design.md` に記載した。
