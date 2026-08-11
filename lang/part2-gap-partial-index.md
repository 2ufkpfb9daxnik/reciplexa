# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 2 | 2 |
| `SYN` | 0 | 17 | 17 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 0 | 26 | 26 |
| `EVAL` | 0 | 4 | 4 |
| `BND` | 0 | 19 | 19 |
| `MAC` | 0 | 9 | 9 |
| `TYP` | 0 | 86 | 86 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 0 | 8 | 8 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 0 | 16 | 16 |
| `PKG` | 0 | 32 | 32 |
| `KER` | 0 | 2 | 2 |
| `EDT` | 0 | 19 | 19 |
| `IR` | 0 | 6 | 6 |
| `ERR` | 0 | 20 | 20 |
| `MEM` | 0 | 12 | 12 |
| `TST` | 0 | 27 | 27 |
| **合計** | **0** | **311** | **311** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `TYP`: gap 0 + partial 86 = **86**
- `PKG`: gap 0 + partial 32 = **32**
- `TST`: gap 0 + partial 27 = **27**
- `DAT`: gap 0 + partial 26 = **26**
- `ERR`: gap 0 + partial 20 = **20**
- `BND`: gap 0 + partial 19 = **19**
- `EDT`: gap 0 + partial 19 = **19**
- `SYN`: gap 0 + partial 17 = **17**
- `MOD`: gap 0 + partial 16 = **16**
- `MEM`: gap 0 + partial 12 = **12**
- `MAC`: gap 0 + partial 9 = **9**
- `EFF`: gap 0 + partial 8 = **8**
- `IR`: gap 0 + partial 6 = **6**
- `EVAL`: gap 0 + partial 4 = **4**
- `LEX`: gap 0 + partial 2 = **2**
- `RES`: gap 0 + partial 2 = **2**
- `ROW`: gap 0 + partial 2 = **2**
- `RSC`: gap 0 + partial 2 = **2**
- `KER`: gap 0 + partial 2 = **2**




---

## Top 30 `partial`（出現順・件数優先の大きい機能から）
1. **L5582** `MEM` — 13.14 `MEM-001` Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path; eval uses Rc not Perceus by default
2. **L5618** `MEM` — 1. メモリとResourceの分離
   - policy: values vs resources; RSC/bracket separation incomplete
3. **L5622** `MEM` — 1.1 通常値
   - policy: values vs resources; RSC/bracket separation incomplete
4. **L5626** `MEM` — 1.2 外部Resource
   - policy: values vs resources; RSC/bracket separation incomplete
5. **L5630** `MEM` — 1.3 基本原則
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
6. **L5634** `MEM` — 2. Perceusによる自動メモリ管理
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
7. **L5638** `MEM` — 2.1 利用者から見える意味
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
8. **L5642** `MEM` — 2.2 回収時点
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
9. **L5646** `MEM` — 2.3 物理identity
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
10. **L5650** `MEM` — 3. Compilation pipeline
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
11. **L5654** `MEM` — 3.1 適用順序
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
12. **L5658** `MEM` — 3.2 Surface所有権注釈
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
13. **L5662** `MEM` — 3.3 Trusted boundary
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
14. **L5666** `MEM` — 4. 所有権Core IR
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
15. **L5670** `MEM` — 4.1 必須のCore要素
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
16. **L5674** `MEM` — 4.2 評価順序
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
17. **L5678** `MEM` — 5. dup
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
18. **L5682** `MEM` — 5.1 意味
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
19. **L5686** `MEM` — 5.2 挿入条件
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
20. **L5690** `MEM` — 5.3 利用者からの不可視性
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
21. **L5694** `MEM` — 6. drop
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
22. **L5698** `MEM` — 6.1 意味
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
23. **L5702** `MEM` — 6.2 最終使用位置
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
24. **L5706** `MEM` — 6.3 制御フロー
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
25. **L5710** `MEM` — 7. 分岐とJoin point
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
26. **L5714** `MEM` — 7.1 排他的分岐
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
27. **L5718** `MEM` — 7.2 Join時の整合
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
28. **L5722** `MEM` — 8. Reuse
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
29. **L5726** `MEM` — 8.1 位置付け
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path
30. **L5730** `MEM` — 8.2 一意性
   - reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path


---

全文の残 gap 一覧: [`part2-remaining-gaps.md`](part2-remaining-gaps.md)（0 件）。

