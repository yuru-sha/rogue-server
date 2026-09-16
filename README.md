# rogue-server

The MVP currently includes three enemy types in addition to the core combat loop.

最小構成のRogue風ターン制ダンジョンサーバーです。ゲーム状態はサーバーが管理し、クライアントはJSON over WebSocketで入力とスナップショットを交換します。現在は地下1階・地下2階の往復と各階の状態保持、勝利アイテム、視界付き敵AI、敵種別ごとの行動速度・移動特性・Trollの再生・瀕死Batの退避、食料、満腹度、経験値、レベルアップ、武器・防具・指輪・杖装備、遠隔攻撃、毒・火炎罠、回復・力のポーション、マッピング・テレポートスクロール、消耗品スタック、終了スコアに対応しています。

## 起動

```sh
cargo run
curl http://127.0.0.1:8080/health
```

別のターミナルでASCIIクライアントを起動します。

```sh
cargo run --bin rogue-cli
```

操作一覧は次で確認できます。

```sh
cargo run --bin rogue-cli -- --help
```

接続先を変更する場合は `ROGUE_WS` を指定します。

```sh
ROGUE_WS=ws://127.0.0.1:9000/ws cargo run --bin rogue-cli
```

listen先は`ROGUE_LISTEN`で変更できます。

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
