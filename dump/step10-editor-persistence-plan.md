# Step 10 — Editor persistence v1

**Status:** complete（gate `implemented-features.md` §2.4.5; gate HEAD `8ec30dc`）  
**Depends on:** Step 9 complete  
**Normative:** `roadmap.md` Step 10, Phase 9（永続化、Undo、Migration）

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
6. **Step 10 gate** — workspace gate + E2E + §2.4.5 ✓

## Audit record（2026-08-21）

独立 subagent（`b7e2df91`; **Composer 2.5**）判定: **complete**（人間 GUI 2026-08-21 記録後）

| Slice | 実装 | 監査 |
|-------|------|------|
| 1 Atomic save | complete | GUI `save_rpx` → `atomic_write`; E2E 2件 |
| 2 Crash recovery UX | complete | `.rpjsrc` journal + `resolve_open_recovery` ダイアログ; E2E |
| 3 Revision undo | complete | `AuthoringUndo` + `RevisionUndoLog`; E2E roundtrip |
| 4 Caret/selection rebase | complete | `rebase.rs` + `apply_source_revision`; 単体/E2E |
| 5 Sidecar migration | complete | `sidecar.rs` + `migrate_existing_sidecar`; E2E |
| 6 Gate | complete | workspace green；独立監査 + 人間 GUI 記録 |

## Complete when

- Undo/Redo、atomic save、crash recovery、snapshot compaction が transaction 単位で動く ✓
- schema version と migration graph があり、未知 extension と partial recovery を安全に扱う ✓
- Stable Node ID が save/load で維持され、Capability、Secret、Task、native pointer を保存しない ✓
- IME composition、caret、selection が source/document revision 変更後に rebase される ✓

## Landed modules

- `crates/reciplexa-gui/src/persistence.rs` — open recovery, edit journal, authoring undo
- `crates/reciplexa-gui/src/rebase.rs` — layer/caret rebase
- `crates/reciplexa-codec/src/sidecar.rs` — migration + partial extension recovery
- `crates/reciplexa-gui/tests/persistence_e2e.rs` — persistence E2E
