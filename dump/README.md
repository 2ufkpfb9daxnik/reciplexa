# Reciplexa 開発案内

このファイルが、人間と実装AIの共通の入口です。
`dump/` が開発文書の正本ディレクトリです。存在しない `lang/` を参照しないでください。

## 最初に読むもの

1. 現在の作業順と完了条件: [`active-roadmap.md`](active-roadmap.md)
2. 現在動く機能と未完事項: [`implemented-features.md`](implemented-features.md)
3. 規範契約を確認するときだけ: [`specification.md`](specification.md)
4. 第II部の見出し単位適合を確認するときだけ: [`part2-conformance.md`](part2-conformance.md)

実装AIには [`implementation-agent-prompt.md`](implementation-agent-prompt.md) をそのまま渡してください。

## 文書の役割

- `specification.md`: 何を満たすべきかを定める規範
- `active-roadmap.md`: いま何を、どの順で実装するかを定める唯一の実行計画
- `implemented-features.md`: 現在動くもの、実測gate、OPEN事項の唯一の現状正本
- `part2-conformance.md`: 第II部見出し単位の適合台帳。製品完成率ではない
- `package-plan.md`: Step 7 package実運用化（local/offline slice完了、残存OPEN）の詳細
- `direct-native-v2-plan.md`: Direct Native v2（標準package）の実装順と完了条件
- `product-typesetting-plan.md`: Step 7 item 3（font-backed JLReq / OpenType MATH）の実装順と完了条件

coverage、deferred、meta、gap/partialの各ファイルは測定・監査用です。通常の実装判断では先に読む必要はありません。

## 現在の方向性

- 現行標準package実装は **Direct Native v2**（DN2 stub + 型付き Rust callable）である。
- Hybrid Native v1 の合成RPXは差分試験の参照本文として残す。本番loadには使わない。
- portable fallback / ABI negotiation は `OPEN-NATIVE-PKG-001` として OPEN のまま追跡する。
- 最初に閉じる製品体験は、package形式の図形と文字の1ページをGUIで編集し、保存、再読込、PDF/SVG出力まで往復できるVertical Sliceである。
- markup、`document/page`のdoc-*作者編集はこのSliceの対象外とし、read-onlyまたは明示的soft-refuseにする。
- Step 7 item 3 の JLReq / MATH Profile v1 は landed（stub 参照経路は残す）。ruby/`vert`、完全 MATH assembly、`OPEN-TEXT-LAYOUT-001` 残 bullets、first-class editable math は OPEN。

## 人間が確認すること

通常は専門AIに自律実行させて構いません。人間の判断が必要なのは、仕様・製品挙動・データ互換性・破壊的変更が変わる場合です。

Vertical Slice完了時には、GUIで基本図形と文字を編集し、保存後に同じ表示へ戻ることを一度確認してください。

コミット前には unit test だけでなく、変更したホストを `cargo run` で起動すること。`--smoke examples/text_line.rpx` は ingest のみで、winit の event loop は起動しない。ウィンドウ経路を触ったら `cargo run -p reciplexa-gui -- examples/text_line.rpx` も確認する。GUI の event loop はプロセスの main thread に置く。Windows の 8MiB スタックは PE `/STACK`（各ホストの `build.rs`）で上げる。
