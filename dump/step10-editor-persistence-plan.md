# Step 10 — Editor persistence v1

**Status:** active  
**Depends on:** Step 9 complete  
**Normative:** `roadmap.md` Step 10, Phase 9（永続化、Undo、Migration）

## 現状（Step 9 時点）

- `reciplexa-codec`: atomic write、journal/recovery、`RevisionUndoLog`、migration graph、extension partial recovery — **ライブラリ＋`phase9_persist` で検証済**
- `reciplexa-gui`: in-memory `undo_stack` / `redo_stack`（source 文字列）、`fs::write` による `.rpx` 保存、save 時に `.rpxsnap` journal + compact — **製品配線が未完**

## 非ゴール

- コラボレーション codec / ネットワーク同期
- Capability / Secret / Task / native pointer の snapshot 混入（禁止のまま）
- markup 作者同期

## Ordered slices

1. **Atomic `.rpx` save** — `atomic_write` で primary 保存；torn write 回帰テスト；既存 journal sidecar 維持 ✓
2. **Crash recovery UX** — 起動/オープン時に journal 候補を検出し、primary を上書きせず recovered 提示；`recover_from_journal` 配線 ✓
3. **Revision undo log** — GUI undo/redo を `RevisionUndoLog` + `document_snapshot_from_source` に接続；transaction 単位の coalesce（typing / props）維持 ✓
4. **IME / caret / selection rebase** — source revision 変更後に byte span と `StableNodeId` selection を再投影 ✓
5. **Schema migration + partial recovery** — 未知 extension の GUI 表示；古い `.rpxsnap` の migrate 経路 ✓
6. **Step 10 gate** — workspace gate + E2E（save crash sim、undo/redo roundtrip、recovery）+ §2.4.5（active）

## Complete when

- Undo/Redo、atomic save、crash recovery、snapshot compaction が transaction 単位で動く
- schema version と migration graph があり、未知 extension と partial recovery を安全に扱う
- Stable Node ID が save/load で維持され、Capability、Secret、Task、native pointer を保存しない
- IME composition、caret、selection が source/document revision 変更後に rebase される
