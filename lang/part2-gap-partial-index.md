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
| `MAC` | 0 | 3 | 3 |
| `TYP` | 0 | 4 | 4 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 2 | 2 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 2 | 2 |
| `PKG` | 0 | 8 | 8 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 0 | 0 | 0 |
| `IR` | 0 | 1 | 1 |
| `ERR` | 0 | 3 | 3 |
| `MEM` | 0 | 1 | 1 |
| `TST` | 0 | 2 | 2 |
| **合計** | **0** | **49** | **49** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `PKG`: gap 0 + partial 8 = **8**
- `SYN`: gap 0 + partial 5 = **5**
- `EVAL`: gap 0 + partial 4 = **4**
- `TYP`: gap 0 + partial 4 = **4**
- `MAC`: gap 0 + partial 3 = **3**
- `ERR`: gap 0 + partial 3 = **3**
- `LEX`: gap 0 + partial 2 = **2**
- `RES`: gap 0 + partial 2 = **2**
- `DAT`: gap 0 + partial 2 = **2**
- `BND`: gap 0 + partial 2 = **2**
- `ROW`: gap 0 + partial 2 = **2**
- `EFF`: gap 0 + partial 2 = **2**
- `RSC`: gap 0 + partial 2 = **2**
- `MOD`: gap 0 + partial 2 = **2**
- `KER`: gap 0 + partial 2 = **2**
- `TST`: gap 0 + partial 2 = **2**
- `IR`: gap 0 + partial 1 = **1**
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
7. **L3761** `PKG` — 5. パッケージversion
   - version field parsed; full semver algebra deferred
8. **L3825** `PKG` — 7.3 内部モジュール
   - non-public cross-package import rejected; intra-package graph incomplete
9. **L104** `SYN` — 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader
   - 字句〜markup+型表面/bytes 核実装済; intersect 糖衣・単位型・厳密 XID 等に残差
10. **L170** `SYN` — 3. 識別子
   - NFC/kebab/?!/_/不可視拒否ok; 厳密 XID_Start/Continue は近似のまま
11. **L174** `SYN` — 3.1 Unicode識別子
   - is_alphabetic 近似; 厳密 Unicode XID は OPEN（残差）
12. **L358** `SYN` — 16. 型構文
   - fn/app/forall/row/effects/union 等; intersect/not/diff 糖衣に残差
13. **L386** `SYN` — 16.7 型alias
   - type-alias 登録あり; 再帰 alias 検査は弱い（残差）
14. **L1117** `EVAL` — 13.3 `EVAL-001` Strict lexical Core evaluator
   - eval.rs CBV Core; also letrec/match/effects beyond min-Core v1
15. **L1161** `EVAL` — 評価文脈
   - evaluation contexts via Outcome/resume; not formal EC grammar
16. **L1173** `EVAL` — 適合試験
   - eval_tests + lang_kernel_suite EVAL/DYN-003; named EVAL corpus incomplete
17. **L1213** `EVAL` — 解決後の最小Core
   - resolve BindingMap + SyntaxNodeId on defs (lang_kernel_suite); eval still name-string Core
18. **L2029** `TYP` — 13.6 `TYP-001` Gradual set-theoretic types
   - Bounded Dynamic/casts/evidence/EffectRow/ROW fragment; 完全集合論ソルバは TYP-ALG 後回し
19. **L2033** `TYP` — 概要・状態
   - Dynamic/unify/EffectRow/cast fragment; 完全集合論代数は後回し
20. **L2037** `TYP` — 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure
   - Bounded Dynamic(S)+three-way+CastEvidence+try/check-cast; foreign/guarantee は deferred/meta
21. **L2525** `TYP` — `DD-TYP-FN-002`: fixed-arity function subtyping
   - Fun 部分型（arg contra / ret cov）断片; 完全 DD は残差
22. **L1457** `MAC` — 13.5 `MAC-001` 最小式マクロ・展開・衛生性
   - 式マクロ核（expand_language / ...+ / hygiene DAG）実装済; 診断帰属・衝突検査に残差
23. **L1633** `MAC` — 7.3 Templateの構文妥当性
   - def 時 var 検査あり; template 完全構文妥当性は弱い
24. **L1805** `MAC` — 14.3 概念的なidentity
   - MacroSourceMap SyntaxNodeId + BindingMap use-sites; Core 文字列 BindingId 二重表現は残差
25. **L5061** `ERR` — 13.13 `ERR-001` 通常の失敗・Failure effect・後始末・Defect・最上位実行境界
   - raise/handle/or-raise/as-result + Never + DYN-005; bracket/cleanup/defect 依存待ち(第III部)
26. **L5169** `ERR` — 5.4 Handlerの結果型
   - handler result typing via shared handle infer (interim)
27. **L5545** `ERR` — 28. Diagnostic
   - reciplexa-diagnostic + FailureDiagnosticBundle; ERR primary model light vs §28
28. **L66** `LEX` — 13.1 `LEX-001` Lossless lexer/CST
   - rowan CST+unparse+shebang/BOM trivia; markup sugar package portion deferred rowan CST+unparse+shebang trivia; BOM専用node/仮想tokenは不足
29. **L84** `LEX` — 正常例・拒否例
   - trivia保持はok; 単位suffixはinterim
30. **L537** `RES` — 13.2.1 `RES-001` 名前解決とnamespace
   - resolve_language_source + BindingMap; multi-ns/phase/import ambiguity incomplete


---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（0 件）。

