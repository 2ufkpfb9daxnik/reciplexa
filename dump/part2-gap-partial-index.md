# 第II部 — gap / partial 優先度インデックス

各エージェントが埋め作業の優先度を付けるための在庫表。出典は `part2-conformance.md` の見出しステータスを、L4 見出しの `XXX-001` 機能ブロックに帰属させたもの。（下位見出しの多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 残件

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 0 | 0 |
| `SYN` | 0 | 0 | 0 |
| `RES` | 0 | 0 | 0 |
| `DAT` | 0 | 0 | 0 |
| `EVAL` | 0 | 0 | 0 |
| `BND` | 0 | 0 | 0 |
| `MAC` | 0 | 0 | 0 |
| `TYP` | 0 | 0 | 0 |
| `ROW` | 0 | 0 | 0 |
| `EFF` | 0 | 0 | 0 |
| `RSC` | 0 | 0 | 0 |
| `MOD` | 0 | 0 | 0 |
| `PKG` | 0 | 1 | 1 |
| `KER` | 0 | 0 | 0 |
| `EDT` | 0 | 0 | 0 |
| `IR` | 0 | 0 | 0 |
| `ERR` | 0 | 0 | 0 |
| `MEM` | 0 | 0 | 0 |
| `TST` | 0 | 0 | 0 |
| **合計** | **0** | **1** | **1** |

（全体カウンタは `part2-conformance-stats.json` 参照。機能別内訳は L4 `XXX-001` ブロック帰属。）

### 合計の多い順（優先度の目安）
- `PKG`: gap 0 + partial 1 = **1**

---

## Remaining `partial`（出現順）
1. **L3657** `PKG` — 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース
   - Slice A–C: local packages + path-dep lock + math/japanese/graphics stubs + workspace.rpxm; registry/残schemaは意図的後回し（親overview）
