# Reciplexa 実装AI向け実行プロンプト

以下を専門のSoftware Engineering AIへそのまま渡してください。

---

あなたはReciplexa / RPXを継続実装する自律Agentです。作業ディレクトリはrepository rootです。

## 最初に読むもの

1. `dump/README.md`
2. `dump/active-roadmap.md`
3. `dump/implemented-features.md`
4. 対象Stepが変更する契約だけ `dump/specification.md`

第II部見出し単位の確認が必要なときだけ `dump/part2-conformance.md` を読む。coverage/deferred/metaの派生文書を最初から全読しない。

## 実行指示

- `dump/active-roadmap.md` の最初の未完了Stepから開始し、受入条件が満たされるまで継続する。
- Step 1〜6は完了済み。再実装せず、Step 7の最初の未完了itemから進む（いまは item 3 製品級組版。item 2/2b Direct Native v2 は完了）。
- 安全な実装判断、crate責任分割、テスト構成は自律的に決める。確認待ちだけを理由に止まらない。
- 仕様・製品挙動・データ互換性・破壊的操作を変える必要がある場合だけ、人間へ具体的な選択肢を示す。
- ユーザーの既存変更を上書き、破棄、巻き戻ししない。

## 固定された製品方針

### Hybrid Native v1（参照）

Hybrid Native v1 は差分試験の参照経路である。標準packageの本番loadは Direct Native v2。`.rpx` body fileがないことだけを理由に「完全direct native」と表記しない（portable fallback は `OPEN-NATIVE-PKG-001`）。

### Direct Native v2

Direct Native v2 は必須マイルストーンである。完了条件と現状は [`direct-native-v2-plan.md`](direct-native-v2-plan.md)。portable fallbackは仕様上のOPEN契約として残す。暗黙に削除しない。

native registry keyは `package/module/export` とし、source-localな既存`BindingId`を永続ABI identityへ流用せず、stub bind時に compilation `BindingId` へ対応付ける。

### 最初の製品Vertical Slice

package形式の図形と文字の1ページを、GUI編集、source保存、再読込、PDF/SVG出力まで往復保証する。

対象:

- circle、line、text等の基本graphics
- move、resize、insert、delete、reorder、text content変更
- authoring sourceを壊さないCST範囲書換え

対象外:

- markup作者同期
- 製品級JLReq / OpenType MATH
- `document/page`のdoc-*作者編集

対象外機能は黙って変更せず、read-onlyまたは理由付きsoft-refuseにする。

## 仕様と文書

- 規範は `dump/specification.md`
- 実行順は `dump/active-roadmap.md`
- 現状は `dump/implemented-features.md`
- 第II部適合は `dump/part2-conformance.md`

実装都合だけで仕様を変えない。仕様が矛盾または実装不能なら、根拠、影響、代案を示し、承認が必要な外部挙動でなければ仕様と実装を同じcommitで整合させる。

`gap=0`、`ok`件数、限定coverageを製品完成率として扱わない。`unit complete`、`stub complete`、`spec conformant`、`product slice complete`を区別する。

## 作業開始前

1. `git status --short --branch` で既存変更を確認する。
2. 対象Stepの受入条件とnon-goalsを列挙する。
3. 関連コード、既存テスト、CI commandを調べる。
4. 変更前baselineを対象テストで再現する。
5. 既存failureと新規failureを区別する。

network access、dependency追加、大規模rename、破壊的git操作を安易に行わない。

## Commit方針

本計画中のcommit作成はユーザーから許可済み。各commit前の個別確認は不要。pushはしない。

1 commitは1つの検証可能な目的に限定する。production変更と直接の回帰testは同じcommitに入れる。無関係な整形、生成物、別bugfixを混ぜない。

推奨単位:

- handoff / docs authority
- 1つのgate failure修復
- 1つの挙動を変えないrefactor
- effect check API
- package Core typecheck API
- pipeline wiring
- package AST CRUD
- GUI routing / properties
- Vertical Slice E2E
- final status synchronization

commit前に必ず確認:

1. `git status --short`
2. staged / unstaged diffの自己review
3. 対象unit / integration test
4. 変更したホストを実際に起動する。`cargo check` や unit test が通っても、Windows 既定スタックでの `cargo run` が落ちることがある。`--smoke` は ingest のみで winit を起動しない。package 経路を触ったら少なくとも:
   `cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx`
   `cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/smoke.pdf`
   GUI の event loop / ウィンドウ経路を触ったら、さらに `cargo run --offline -p reciplexa-gui -- examples/text_line.rpx` を起動し、winit が main thread 以外で panic しないことを確認する。event loop を worker に移さない。Windows の 8MiB スタックは PE `/STACK`（`build.rs`）で上げる。
5. `cargo fmt --all --check`
6. 変更範囲のClippy
7. 公開契約を変えた場合のdocs / examples / conformance
8. secret、credential、一時生成物、巨大binary、意図しない削除がない
9. conflict markerとwhitespace errorがない
10. 既存ユーザー変更を含めていない

合理的な節目ごとにworkspace全gateを実行する。hook失敗後は原因を直して新しいcommitを作る。amend、rebase、force、pushを行わない。

commit messageはrepositoryの直近履歴に合わせ、目的と理由が分かる簡潔なConventional Commit形式を使う。

## 標準gate

Windows / offlineを前提に、repositoryの既存環境を使う。

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
cargo check --workspace --all-targets --offline
cargo check --offline -p reciplexa-gui
cargo run --offline -p reciplexa-gui -- --smoke examples/text_line.rpx
cargo run --offline -p reciplexa -- examples/text_line.rpx .tmp/smoke.pdf
# window path: cargo run --offline -p reciplexa-gui -- examples/text_line.rpx
```

高コストな全gateの前に対象crateのtest / Clippyを実行する。全gateの実測結果は `dump/implemented-features.md` に同じHEADの結果として記録する。

## 停止条件

次の場合のみ停止して報告する。

- 仕様または製品方針の根本矛盾
- data loss、security、互換性破壊の危険
- credential、外部service、network、push、履歴書換えが必要
- ユーザー変更を破棄しないと進めない
- 同じfailureへ合理的な複数案を試しても進展しない

報告には、再現command、root cause、試した内容、安全な選択肢、推奨案を含める。

## 完了報告

各Step完了時に以下を `dump/active-roadmap.md` と `dump/implemented-features.md` へ反映する。

- 完了した受入条件
- commit hashと目的
- 実行したvalidation
- 保証する範囲とnon-goals
- 残るOPENと次のStep

今すぐrepositoryを確認し、`dump/active-roadmap.md` の最初の未完了StepからAtomic Commitを積み重ねて実行してください。
