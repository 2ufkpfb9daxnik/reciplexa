# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 3 | 3 |
| `SYN` | 0 | 24 | 24 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 55 | 55 |
| `EVAL` | 0 | 7 | 7 |
| `BND` | 0 | 19 | 19 |
| `MAC` | 0 | 14 | 14 |
| `TYP` | 92 | 101 | 193 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 10 | 10 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 20 | 20 |
| `PKG` | 0 | 50 | 50 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 75 | 56 | 131 |
| `IR` | 1 | 5 | 6 |
| `ERR` | 19 | 37 | 56 |
| `MEM` | 14 | 102 | 116 |
| `TST` | 4 | 27 | 31 |
| **合計** | **205** | **538** | **743** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `TYP`: gap 92 + partial 101 = **193**
- `EDT`: gap 75 + partial 56 = **131**
- `MEM`: gap 14 + partial 102 = **116**
- `ERR`: gap 19 + partial 37 = **56**
- `DAT`: gap 0 + partial 55 = **55**
- `PKG`: gap 0 + partial 50 = **50**
- `TST`: gap 4 + partial 27 = **31**
- `SYN`: gap 0 + partial 24 = **24**
- `MOD`: gap 0 + partial 20 = **20**
- `BND`: gap 0 + partial 19 = **19**
- `MAC`: gap 0 + partial 14 = **14**
- `EFF`: gap 0 + partial 10 = **10**
- `EVAL`: gap 0 + partial 7 = **7**
- `IR`: gap 1 + partial 5 = **6**
- `LEX`: gap 0 + partial 3 = **3**
- `RES`: gap 0 + partial 2 = **2**
- `ROW`: gap 0 + partial 2 = **2**
- `RSC`: gap 0 + partial 2 = **2**
- `KER`: gap 0 + partial 2 = **2**

---

## Top 30 `gap`（出現順）
1. **L2254** `TYP` — dynamic typingの終端状態
   - no dedicated dynamic cast Core forms yet
2. **L2274** `TYP` — safe static use
   - TYP-DYN conformance cases not implemented as suite
3. **L2278** `TYP` — partial overlap
   - TYP-DYN conformance cases not implemented as suite
4. **L2282** `TYP` — disjoint use
   - TYP-DYN conformance cases not implemented as suite
5. **L2286** `TYP` — cast precision
   - TYP-DYN conformance cases not implemented as suite
6. **L2294** `TYP` — foreign ingress
   - TYP-DYN conformance cases not implemented as suite
7. **L2298** `TYP` — implicit cast failure
   - TYP-DYN conformance cases not implemented as suite
8. **L2302** `TYP` — explicit safe cast
   - TYP-DYN conformance cases not implemented as suite
9. **L2306** `TYP` — fixed-arity function cast
   - TYP-DYN conformance cases not implemented as suite
10. **L2310** `TYP` — function result cast
   - TYP-DYN conformance cases not implemented as suite
11. **L2314** `TYP` — effect-compatible function cast
   - TYP-DYN conformance cases not implemented as suite
12. **L2318** `TYP` — effect-incompatible function cast
   - TYP-DYN conformance cases not implemented as suite
13. **L2322** `TYP` — polymorphic value boundary
   - TYP-DYN conformance cases not implemented as suite
14. **L2326** `TYP` — dynamicからforall
   - TYP-DYN conformance cases not implemented as suite
15. **L2330** `TYP` — numeric promotion
   - TYP-DYN conformance cases not implemented as suite
16. **L2334** `TYP` — namespace collision warning
   - TYP-DYN conformance cases not implemented as suite
17. **L2374** `TYP` — `DD-TYP-ALG-001`: 宣言的型関係とalgorithmic判定の分離
   - no declarative vs algorithmic subtype separation in code
18. **L2378** `TYP` — `DD-TYP-ALG-002`: algorithmic判定の三値結果
   - no three-valued decide-subtype / diagnostic taxonomy
19. **L2382** `TYP` — `proved`
   - no three-valued decide-subtype / diagnostic taxonomy
20. **L2386** `TYP` — `disproved`
   - no three-valued decide-subtype / diagnostic taxonomy
21. **L2390** `TYP` — `unknown`
   - no three-valued decide-subtype / diagnostic taxonomy
22. **L2394** `TYP` — `DD-TYP-ALG-003`: 診断分類
   - no three-valued decide-subtype / diagnostic taxonomy
23. **L2398** `TYP` — `type-error`
   - no three-valued decide-subtype / diagnostic taxonomy
24. **L2402** `TYP` — `annotation-required`
   - no three-valued decide-subtype / diagnostic taxonomy
25. **L2406** `TYP` — `checker-limitation`
   - no three-valued decide-subtype / diagnostic taxonomy
26. **L2410** `TYP` — `checker-resource-limit`
   - no three-valued decide-subtype / diagnostic taxonomy
27. **L2414** `TYP` — `unsupported-language-feature`
   - no three-valued decide-subtype / diagnostic taxonomy
28. **L2418** `TYP` — `DD-TYP-ALG-004`: soundness、completeness、terminationの優先順位
   - no three-valued decide-subtype / diagnostic taxonomy
29. **L2422** `TYP` — `DD-TYP-ALG-005`: principal type
   - no three-valued decide-subtype / diagnostic taxonomy
30. **L2426** `TYP` — `DD-TYP-ALG-006`: 決定的なsolver budget
   - no three-valued decide-subtype / diagnostic taxonomy

---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（205 件）。

