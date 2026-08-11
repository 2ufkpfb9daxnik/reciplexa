# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 1 | 1 |
| `SYN` | 0 | 5 | 5 |
| `RES` | 0 | 1 | 1 |
| `DAT` | 0 | 2 | 2 |
| `EVAL` | 0 | 1 | 1 |
| `BND` | 0 | 2 | 2 |
| `MAC` | 0 | 3 | 3 |
| `TYP` | 0 | 4 | 4 |
| `ROW` | 0 | 1 | 1 |
| `EFF` | 0 | 2 | 2 |
| `RSC` | 0 | 1 | 1 |
| `MOD` | 0 | 2 | 2 |
| `PKG` | 0 | 7 | 7 |
| `KER` | 0 | 0 | 0 |
| `EDT` | 0 | 0 | 0 |
| `IR` | 0 | 1 | 1 |
| `ERR` | 0 | 1 | 1 |
| `MEM` | 0 | 1 | 1 |
| `TST` | 0 | 2 | 2 |
| **合計** | **0** | **37** | **37** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `PKG`: gap 0 + partial 7 = **7**
- `SYN`: gap 0 + partial 5 = **5**
- `TYP`: gap 0 + partial 4 = **4**
- `MAC`: gap 0 + partial 3 = **3**
- `DAT`: gap 0 + partial 2 = **2**
- `BND`: gap 0 + partial 2 = **2**
- `EFF`: gap 0 + partial 2 = **2**
- `MOD`: gap 0 + partial 2 = **2**
- `TST`: gap 0 + partial 2 = **2**
- `LEX`: gap 0 + partial 1 = **1**
- `RES`: gap 0 + partial 1 = **1**
- `EVAL`: gap 0 + partial 1 = **1**
- `ROW`: gap 0 + partial 1 = **1**
- `RSC`: gap 0 + partial 1 = **1**
- `IR`: gap 0 + partial 1 = **1**
- `ERR`: gap 0 + partial 1 = **1**
- `MEM`: gap 0 + partial 1 = **1**





















---

## Top 30 `partial`（出現順・件数優先の大きい機能から）
1. **L3657** `PKG` — 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース
   - Slice A–C: local packages + path-dep lock + math/japanese/graphics stubs + workspace.rpxm parse; registry deferred
2. **L3701** `PKG` — 2. パッケージmanifest
   - parse_rpxm + PackageManifest; remaining schema depth vs full PKG-001
3. **L3713** `PKG` — 2.3 制限付きRPX形式
   - restricted sexp tokenize in rpxm.rs; not every static schema leaf
4. **L3717** `PKG` — 2.4 静的schema
   - restricted sexp + known-field gate; residual static schema leaves open
5. **L3725** `PKG` — 3. Manifestの基本構文
   - minimal/explicit/parensed manifest forms parsed; residual schema depth open
6. **L3745** `PKG` — 4. パッケージ名
   - package name parsed as identity; full naming-rule matrix still light
7. **L3825** `PKG` — 7.3 内部モジュール
   - non-public cross-package import rejected; intra-package graph incomplete
8. **L104** `SYN` — 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader
   - 字句〜markup+型表面/bytes 核実装済; intersect 糖衣・単位型・厳密 XID 等に残差
9. **L170** `SYN` — 3. 識別子
   - NFC/kebab/?!/_/不可視拒否ok; 厳密 XID_Start/Continue は近似のまま
10. **L174** `SYN` — 3.1 Unicode識別子
   - is_alphabetic 近似; 厳密 Unicode XID は OPEN（残差）
11. **L358** `SYN` — 16. 型構文
   - fn/app/forall/row/effects/union 等; intersect/not/diff 糖衣に残差
12. **L386** `SYN` — 16.7 型alias
   - type-alias 登録あり; 再帰 alias 検査は弱い（残差）
13. **L2029** `TYP` — 13.6 `TYP-001` Gradual set-theoretic types
   - Bounded Dynamic/casts/evidence/EffectRow/ROW fragment; 完全集合論ソルバは TYP-ALG 後回し
14. **L2033** `TYP` — 概要・状態
   - Dynamic/unify/EffectRow/cast fragment; 完全集合論代数は後回し
15. **L2037** `TYP` — 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure
   - Bounded Dynamic(S)+three-way+CastEvidence+try/check-cast; foreign/guarantee は deferred/meta
16. **L2525** `TYP` — `DD-TYP-FN-002`: fixed-arity function subtyping
   - Fun 部分型（arg contra / ret cov）断片; 完全 DD は残差
17. **L1457** `MAC` — 13.5 `MAC-001` 最小式マクロ・展開・衛生性
   - 式マクロ核（expand_language / ...+ / hygiene DAG）実装済; 診断帰属・衝突検査に残差
18. **L1633** `MAC` — 7.3 Templateの構文妥当性
   - def 時 var 検査あり; template 完全構文妥当性は弱い
19. **L1805** `MAC` — 14.3 概念的なidentity
   - MacroSourceMap SyntaxNodeId + BindingMap use-sites; Core 文字列 BindingId 二重表現は残差
20. **L545** `DAT` — 13.2.3 `DAT-001` 代数的データ型・constructor・pattern・match
   - data/match + App instantiate + poly ctor/∀; ctor-refined display / dual-ns は残差・後回し
21. **L717** `DAT` — 8. Variance
   - per-param cov/contra/invar/phantom 推論あり; variance 部分型 lattice は残差
22. **L1217** `BND` — 13.4 `BND-001` `val`、`var`、`let`、`letrec`、`fn`
   - let/letrec/var/set + typed store/annotations/escape; Identity 代数完全形は残差
23. **L1293** `BND` — `DD-BND-018`: local state identity
   - local-state effect + Cell Rc identity; formal scope identity は薄い
24. **L2921** `EFF` — 13.8 `EFF-001` Algebraic effects and handlers
   - deep one-shot + ambient + with/handler 実装済; multi-shot/return は意図的後回し
25. **L2925** `EFF` — 状態
   - deep one-shot + ambient + with/handler; multi-shot/return は意図的後回し
26. **L3017** `MOD` — 13.9 `MOD-001` モジュール・シグネチャ・Functor・分割コンパイル
   - outer unit + import/link + .rpi export filter/path 実装済; signatures/functors は意図的後回し
27. **L3029** `MOD` — DD-001.2 中心的な決定
   - import as/only/rename+qualified + .rpi boundary 実装済; functors/signatures は意図的後回し
28. **L6310** `TST` — 13.15 `TST-001` Tests and conformance
   - lang_kernel_suite: STA/DYN/INT/SYN-C + LANG-* ; 版付きフル matrix は意図的後回し
29. **L6454** `TST` — 21.1 型検査器要件
   - typecheck_language_source あり; ModuleEnv/imported sigs は残差
30. **L66** `LEX` — 13.1 `LEX-001` Lossless lexer/CST
   - rowan CST+unparse+shebang/BOM trivia 実装済; markup 糖衣パッケージ・単位型は PKG 依存で後回し


---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（0 件）。

