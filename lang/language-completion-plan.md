# RPX 言語完成計画（第II部 → 機能サマリ → PKG）

`lang/specification.md` 第II部の全見出しは [`part2-conformance.md`](part2-conformance.md) で 1 件ずつ追跡する。`deferred` / `meta` の意味は [`part2-deferred.md`](part2-deferred.md) / [`part2-meta.md`](part2-meta.md) を参照。

## 完成の定義（ユーザー合意）

1. **第II部**の規範的要求を `specification.md` と突き合わせ、実装が矛盾しないこと（重い項目も含む）
2. **ゲート緑**: `cargo fmt` / `clippy -D warnings` / `test --workspace` / `check -p reciplexa-gui`
3. **リージョンカバレッジ 100%**（言語関連クレートを優先）
4. 上記達成後 → **実装済み機能一覧**を文書化し、仕様との差分を可視化
5. その後 PKG: **静的図形** / **jlreq（日本語組版）** / **数式（Satysfi 参考）** を省略なく実装

## 現在の適合状況（`part2-conformance-stats.json` 参照）

| status | 意味 |
|---|---|
| `ok` | 仕様と一致（または実装義務なしで N/A） |
| `partial` | 一部実装、ギャップ残り |
| `gap` | 未実装または矛盾 |
| `deferred` | 意図的後回し（仕様未決定 OPEN-* 含む）— **ユーザー未決定ではなくレビュー時の分類** |
| `meta` | 用語・プロセス・メタ理論（コード表面なし） |

| metric | count |
|---|---|
| total | 1589 |
| ok | 319 |
| partial | 507 |
| gap | 338 |
| deferred | 293 |
| meta | 132 |

## 埋め作業の優先順（言語核）

1. **SYN/LEX** — shebang CST、不可視文字、package path、bytes、型表面、formatter …
2. **DAT/TYP** — 型適用、occurrence typing、variance、optional record pattern …
3. **MAC/MOD/BND/EFF** — 残 gap、multi-shot は仕様確定後
4. **ERR/MEM/LIT/EDT** — 第II部で言語に直結する節から（GUI/第III部専用は依存待ち）
5. **TST** — 適合試験 ID と `lang_kernel_suite` の拡充
6. **カバレッジ** — `reciplexa-core` / `syntax` / `eval` / `bind` / `macro` を 100% region

## PKG フェーズ（言語完成後）

| パッケージ | 参照 | 状態 |
|---|---|---|
| graphics / length / color | SYN §12, PKG-001 | Slice A: local `packages/` + import |
| math | Satysfi 数式パイプライン | 未着手 |
| japanese / jlreq | jlreq + 組版 | 未着手 |
| 文書 surface 移行 | `(page)` interim 廃止 | strangler |

詳細: [`package-plan.md`](package-plan.md)

## ゲート

```powershell
$env:CARGO_TARGET_DIR = "d:\reciplexa\target"
$env:TEMP = "d:\reciplexa\.tmp"
$env:TMP = "d:\reciplexa\.tmp"
cargo fmt --all
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --offline -p reciplexa-gui
```
