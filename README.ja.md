# rogue-server

[English](README.md) | [日本語](README.ja.md)

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/yuru-sha/rogue-server)

The MVP currently includes three enemy types in addition to the core combat loop.

最小構成のRogue風ターン制ダンジョンサーバーです。ゲーム状態はサーバーが管理し、クライアントはJSON over WebSocketで入力とスナップショットを交換します。現在は地下1階・地下2階の往復と各階の状態保持、勝利アイテム、視界付き敵AI、敵種別ごとの行動速度・移動特性・Trollの再生・瀕死Batの退避、食料、満腹度、経験値、レベルアップ、武器・防具・指輪・杖装備、遠隔攻撃、毒・火炎罠、回復・力のポーション、マッピング・テレポートスクロール、消耗品スタック、終了スコアに対応しています。

## 起動

Go 1.27.4 が必要です。

```sh
go run ./cmd/rogue-server
curl http://127.0.0.1:8080/health
```
別のターミナルでASCIIクライアントを起動します。

```sh
go run ./cmd/rogue-cli
```

操作一覧は次で確認できます。

```sh
go run ./cmd/rogue-cli --help
```
接続先を変更する場合は `ROGUE_WS` を指定します。

```sh
ROGUE_WS=ws://127.0.0.1:9000/ws go run ./cmd/rogue-cli
```
listen先は`ROGUE_LISTEN`で変更できます。デフォルトはloopbackです。非loopbackに
bindするとネットワークに公開されるため、適切なアクセス制御の内側でのみ運用してください。
WebSocketセッション数の上限はデフォルト64、無通信タイムアウトは5分です。
`ROGUE_MAX_SESSIONS`には正の接続数、`ROGUE_IDLE_TIMEOUT`にはGoのduration形式
（例: `2m`）を指定して変更できます。SIGINT/SIGTERMによる終了時には接続中の
WebSocketセッションも閉じます。

サーバーの構造化ログはJSON形式で標準エラー出力に出力します。

## コマンド例

```json
{"version":1,"request_id":"1","command":{"type":"move","dx":1,"dy":0}}
{"version":1,"request_id":"2","command":{"type":"wait"}}
{"version":1,"request_id":"3","command":{"type":"pickup"}}
{"version":1,"request_id":"4","command":{"type":"eat"}}
{"version":1,"request_id":"5","command":{"type":"equip"}}
{"version":1,"request_id":"6","command":{"type":"equip_armor"}}
{"version":1,"request_id":"7","command":{"type":"shoot","dx":1,"dy":0}}
{"version":1,"request_id":"8","command":{"type":"search"}}
{"version":1,"request_id":"9","command":{"type":"drink"}}
{"version":1,"request_id":"10","command":{"type":"quit"}}
```

同じseedをWebSocket URLの`seed`クエリへ渡すと、マップと初期配置を再現できます。

インベントリ検査は `inventory`、所持品を1個落とす操作は `drop` として利用できます。
`drop` には `food`、`potion`、`strength_potion`、`scroll`、`teleport_scroll`、`amulet`、`weapon`、`armor`、`ring`、`staff` のいずれかを指定します。
識別は `identify` に `weapon`、`armor`、`ring`、`staff` のいずれかを指定します。テレポートスクロールは `teleport` で使用します。
同一種類かつ同一メタデータのアイテムは1スロットにまとまり、使用またはドロップで1個ずつ減ります。ボーナスや識別・呪い状態が異なるアイテムは分離されます。

```text
ws://127.0.0.1:8080/ws?seed=42
```

## ドキュメント

- [仕様](docs/SPEC.md)
- [MVP受け入れ条件](docs/MVP.md)

## GitHub運用

Issue Form と既定の Pull Request テンプレートは `yuru-sha/.github` の共通設定を利用します。Release Notes のカテゴリは `.github/release.yml` で管理し、`orca:*` を含む共通ラベルは `yuru-sha/project-template` から同期します。
