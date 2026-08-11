# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 2 | 2 |
| `SYN` | 0 | 5 | 5 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 2 | 2 |
| `EVAL` | 0 | 4 | 4 |
| `BND` | 0 | 2 | 2 |
| `MAC` | 0 | 9 | 9 |
| `TYP` | 0 | 22 | 22 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 8 | 8 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 2 | 2 |
| `PKG` | 0 | 8 | 8 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 0 | 0 | 0 |
| `IR` | 0 | 6 | 6 |
| `ERR` | 0 | 3 | 3 |
| `MEM` | 0 | 12 | 12 |
| `TST` | 0 | 23 | 23 |
| **合計** | **0** | **116** | **116** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `TST`: gap 0 + partial 23 = **23**
- `TYP`: gap 0 + partial 22 = **22**
- `MEM`: gap 0 + partial 12 = **12**
- `MAC`: gap 0 + partial 9 = **9**
- `EFF`: gap 0 + partial 8 = **8**
- `PKG`: gap 0 + partial 8 = **8**
- `IR`: gap 0 + partial 6 = **6**
- `SYN`: gap 0 + partial 5 = **5**
- `EVAL`: gap 0 + partial 4 = **4**
- `ERR`: gap 0 + partial 3 = **3**
- `LEX`: gap 0 + partial 2 = **2**
- `RES`: gap 0 + partial 2 = **2**
- `DAT`: gap 0 + partial 2 = **2**
- `BND`: gap 0 + partial 2 = **2**
- `ROW`: gap 0 + partial 2 = **2**
- `RSC`: gap 0 + partial 2 = **2**
- `MOD`: gap 0 + partial 2 = **2**
- `KER`: gap 0 + partial 2 = **2**














---

## Top 30 `partial`（出現順・件数優先の大きい機能から）
1. **L6310** `TST` — 13.15 `TST-001` Tests and conformance
   - lang_kernel_suite: STA/DYN/INT/SYN-C + LANG-* BND/DAT/EFF/ERR/TYP/PKG/ADT; full matrix open
2. **L6326** `TST` — テスト原則
   - conformance.rs + lang_kernel_suite TEST-* IDs; full artifact trace incomplete
3. **L6340** `TST` — 14.1 全体EBNF（未完成）
   - skeleton EBNF; real grammar in reciplexa-syntax
4. **L6348** `TST` — 14.3 予約語
   - is_reserved_special_form / keywords; full policy 未決定
5. **L6352** `TST` — 14.4 糖衣とCore
   - elaborate covers many sugars; table incomplete
6. **L6362** `TST` — 15.1 Kind
   - kinds implicit in CoreType/rows; Module/Signature kinds absent
7. **L6366** `TST` — 15.2 共通判断
   - infer/check judgments in check.rs; module sig judgment absent
8. **L6370** `TST` — 15.3 基本規則
   - T-VAR/if/record etc. partially in checker
9. **L6374** `TST` — 15.4 一般化
   - let generalization light; value restriction incomplete
10. **L6378** `TST` — 15.5 Subtypingと制約解決
   - unify + subtype stubs; full constraint solver deferred
11. **L6400** `TST` — 16.3 効果伝播
   - deep one-shot handlers; multi-shot deferred
12. **L6404** `TST` — 16.4 SourceEdit
   - syntax edit + source_sync; not full SourceEdit calculus
13. **L6432** `TST` — 実装アーキテクチャ
   - pipeline exists as vertical slice; not final
14. **L6436** `TST` — 20.1 最終目標パイプライン
   - bytes→CST→elaborate→check→eval→lower present; phase gaps
15. **L6440** `TST` — 20.2 必要データ構造
   - many structures exist; ModuleEnv/Typed Core incomplete
16. **L6444** `TST` — 20.3 現行crateとの対応
   - crate map outdated in places (core/eval/bind now carry more); still useful
17. **L6454** `TST` — 21.1 型検査器要件
   - typecheck_language_source; ModuleEnv/imported sigs incomplete
18. **L6468** `TST` — 22.2 静的意味
   - check/unify + STA-001/007 + BND ann/ADT/casts; full STA matrix open
19. **L6476** `TST` — 22.4 統合
   - GUI/source_sync + TEST-INT-002 .rpi boundary; not full INT matrix
20. **L6484** `TST` — 22.6 横断適合試験
   - cross-wires STA/DYN/INT + LANG-* ; not versioned full cross suite
21. **L6498** `TST` — 段階2: 仕様が明確になった
   - spec largely written; grammar/Core still holes
22. **L6506** `TST` — 段階4: 適合試験を通過
   - some conformance tests pass; not versioned full suite gate
23. **L6522** `TST` — 段階8: 実装と形式仕様の対応を確認
   - crate↔spec mapping informal; no versioned correspondence report
24. **L2029** `TYP` — 13.6 `TYP-001` Gradual set-theoretic types
   - Bounded Dynamic/casts/evidence/EffectRow/ROW fragment; full set-theoretic solver deferred at TYP-ALG
25. **L2033** `TYP` — 概要・状態
   - Dynamic/unify/EffectRow/cast fragment; full set-theoretic algebra deferred
26. **L2037** `TYP` — 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure
   - Bounded Dynamic(S)+three-way use+CastEvidence+try/check-cast; foreign/guarantee remain deferred/meta
27. **L2089** `TYP` — `DD-TYP-DYN-009`: dynamic上限のwidening
   - Widen evidence for dynamic S→dynamic U when S<:U; deep provenance TBD
28. **L2113** `TYP` — `DD-TYP-DYN-013`: implicit cast failure
   - implicit Cast failure→EvalError; structured dynamic-type-error taxonomy still thin
29. **L2169** `TYP` — `DD-TYP-DYN-017`: evidence compositionと最適化
   - compose_evidence/simplify_evidence Identity absorption; widen-chain opt TBD
30. **L2173** `TYP` — `DD-TYP-DYN-018`: cast provenance
   - CastProvenance struct separate from evidence; full boundary IDs TBD


---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（0 件）。

