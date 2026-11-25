# degima-cursor — 設計仕様

種別: 現行実装の設計正本／リポジトリ: `degima-cursor`

> **目的:** DEGIMA Device を Cursor IDE の Remote-SSH で `/root` として開く CLI（degima-agent の **AI Agent** 相当）を再実装できること。

## 1. 概要

| 項目 | 内容 |
| --- | --- |
| 言語 | Python 3（標準ライブラリのみ。外部パッケージ依存なし） |
| 実行 | `make run` または `python3 -m degima_cursor`（対話入力または引数で SSH コマンドを渡す） |
| ソース | `degima_cursor/` |
| 対応機能 | degima-agent Electron の AI Agent（`open-in-cursor.js`） |
| リモートパス | 常に `/root` |

処理フロー:

1. SSH コマンドを取得（引数またはプロンプト）
2. `user` / `host` / `port` を解析
3. 秘密鍵パスを解決（既定は接続可能な鍵を自動選択）
4. `~/.ssh/config` に Device 別 Host ブロックを upsert
5. Cursor を Remote-SSH で `/root` へ detached 起動

## 2. 入力

### 2-1. CLI

| 形式 | 意味 |
| --- | --- |
| `python3 -m degima_cursor ["ssh user@host -p PORT"]` | SSH コマンド省略時は `SSH command: ` で対話入力 |
| `--ssh-key PATH` | 使用する秘密鍵を明示（後述の環境変数より優先） |
| `-h` / `--help` | 用法を表示して終了コード `2` |

成功時は標準出力に接続先と採用した鍵パスを表示する。失敗時は `error: ...` を標準エラーへ出して終了コード `1`。

### 2-2. SSH コマンド

形式は degima-agent `parseSshCmd` と同じ。

```
ssh root@dk11-kusaka24.degima.ai -p 10008
```

| 要素 | 規則 |
| --- | --- |
| 前置 | 必ず `ssh ` で始まる |
| user@host | `@` あり。user 省略時は `root` |
| bare host | `@` なしの非オプション token |
| `-p PORT` | 省略時は `22`（`0`–`65535`） |

### 2-3. 秘密鍵

明示指定が無い場合、接続先へ `BatchMode=yes` / `ConnectTimeout=5` / `IdentitiesOnly=yes` で `ssh ... true` を試し、認証できる鍵を自動選択する（ターミナルで通る鍵と Cursor Remote-SSH の `IdentityFile` を揃えるため）。

| 優先順 | ソース |
| --- | --- |
| 1 | `--ssh-key PATH` / `DEGIMA_CURSOR_SSH_KEY` / `SSH_KEY`（指定時はその鍵のみ。拒否ならエラー） |
| 2 | `~/.ssh/id_ed25519` |
| 3 | `~/.ssh/id_rsa` |
| 4 | `~/.ssh/cursor2_id_ed25519` |
| 5 | degima-agent Dashboard 鍵（OS 別。下表） |

| OS | Dashboard 鍵パス |
| --- | --- |
| macOS | `~/Library/Application Support/degima-agent/ssh-keys/degima_dashboard_ed25519` |
| Linux | `~/.config/degima-agent/ssh-keys/degima_dashboard_ed25519` |
| Windows | `%APPDATA%\degima-agent\ssh-keys\degima_dashboard_ed25519` |

いずれも無い／拒否の場合はエラー終了。成功時は採用した鍵パスを標準出力へ表示する。

## 3. SSH config

degima-agent と同じマーカー／エイリアス規約を使い、同一 Device を両ツールで共有できる。

```
# BEGIN degima-terminal rocket dk11-kusaka24.degima.ai:10008
Host degima-dk11-kusaka24.degima.ai-10008
  HostName dk11-kusaka24.degima.ai
  User root
  Port 10008
  IdentityFile /Users/.../.ssh/id_ed25519
  IdentitiesOnly yes
  StrictHostKeyChecking accept-new
# END degima-terminal rocket dk11-kusaka24.degima.ai:10008
```

- `Host` エイリアス: `degima-{host}-{port}`（host の非 `[a-zA-Z0-9.-]` は `-`）
- 既存マーカー区間があれば置換、なければ末尾追記
- レガシー（マーカー行に `{host}:{port}` が無いブロック）は削除
- `.ssh` chmod `0700`、`config` chmod `0600`（Windows では chmod をスキップ）
- ホームは `HOME`、なければ `USERPROFILE`

## 4. Cursor 起動

| OS | 探索 | 引数 |
| --- | --- | --- |
| macOS | `PATH` 上の `cursor` / `/usr/local/bin/cursor` / `/Applications/Cursor.app/Contents/Resources/app/bin/cursor` | `--folder-uri vscode-remote://ssh-remote+{sshHost}/root --new-window` |
| Linux | `PATH` 上の `cursor` / `/usr/local/bin/cursor` | `--remote ssh-remote+{sshHost} /root` |
| Windows | `%LOCALAPPDATA%\Programs\cursor\Cursor.exe` / 同配下 `resources\app\bin\cursor.cmd` / `PATH` 上の `cursor.cmd` | `--folder-uri ... --new-window` |

起動は `subprocess.Popen` で detached（非 Windows は `start_new_session=True`、Windows は `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`）。プロセスを待たず CLI は終了する。Cursor が見つからなければエラー。

## 5. モジュール

| ファイル | 役割 |
| --- | --- |
| `degima_cursor/__init__.py` | パッケージ版 `__version__` |
| `degima_cursor/__main__.py` | 引数／対話入力、鍵解決、オーケストレーション |
| `degima_cursor/ssh_cmd.py` | `SshTarget` / `parse_ssh_cmd` |
| `degima_cursor/ssh_config.py` | `ensure_device_ssh_config` / Host エイリアス・マーカー upsert |
| `degima_cursor/ssh_key.py` | `resolve_identity_path`（BatchMode 試行） |
| `degima_cursor/cursor.py` | Cursor 探索・Remote-SSH 起動 |

## 6. 実行環境

システムに `python3` があれば追加インストール不要。外部パッケージ依存はない。`PYTHON` Make 変数でインタプリタを上書きできる（既定 `python3`）。

## 7. Makefile

| ターゲット | 動作 |
| --- | --- |
| `run`（既定） | `python3 -m degima_cursor` |
| `check` | `python3 -m compileall` で `degima_cursor/` の構文チェック |
| `clean` | `__pycache__` / `.local` / `target` 削除 |
