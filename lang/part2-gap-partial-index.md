# 第II部 — gap / partial 優先度インデックス

親エージェントが埋め作業の優先度を付けるための在庫表。件数は `part2-conformance.md` の見出しステータスを、直前の `XXX-001` 機能ブロックに帰属させたもの。（サブ節の多くは親機能 ID の下にぶら下がる。）

## 機能ブロック別 件数

| 機能 | gap | partial | 合計 |
|---|---:|---:|---:|
| `LEX` | 0 | 4 | 4 |
| `SYN` | 4 | 29 | 33 |
| `RES` | 0 | 2 | 2 |
| `DAT` | 11 | 50 | 61 |
| `EVAL` | 3 | 7 | 10 |
| `BND` | 1 | 19 | 20 |
| `MAC` | 11 | 21 | 32 |
| `TYP` | 59 | 19 | 78 |
| `LIT` | 76 | 49 | 125 |
| `ROW` | 0 | 2 | 2 |
| `EFF` | 1 | 10 | 11 |
| `RSC` | 0 | 2 | 2 |
| `MOD` | 30 | 18 | 48 |
| `PKG` | 2 | 48 | 50 |
| `EDT` | 77 | 56 | 133 |
| `IR` | 1 | 5 | 6 |
| `ERR` | 96 | 17 | 113 |
| `MEM` | 24 | 99 | 123 |
| `TST` | 4 | 27 | 31 |
| `KER` | 0 | 2 | 2 |
| **合計** | **338** | **506** | **844** |

（全体カウンタは `part2-conformance-stats.json` 準拠。機能別内訳は概算。）

### 合計が多い順（優先度の目安）
- `EDT`: gap 77 + partial 56 = **133**
- `LIT`: gap 76 + partial 49 = **125**
- `MEM`: gap 24 + partial 99 = **123**
- `ERR`: gap 96 + partial 17 = **113**
- `TYP`: gap 59 + partial 19 = **78**
- `DAT`: gap 11 + partial 50 = **61**
- `PKG`: gap 2 + partial 48 = **50**
- `MOD`: gap 30 + partial 18 = **48**
- `SYN`: gap 4 + partial 29 = **33**
- `MAC`: gap 11 + partial 21 = **32**
- `TST`: gap 4 + partial 27 = **31**
- `BND`: gap 1 + partial 19 = **20**

---

## Top 30 `gap`（文書出現順）
1. **L1184** `SYN` — 8.4 特殊文字
   - newline/unicode等のkernel helper未
2. **L2312** `SYN` — 18.4 未終了文字列
   - 未終了文字列の仮想閉じ無し→Error token
3. **L2391** `SYN` — 18.10 型検査のcascade抑制
   - 型cascade抑制は本層に無し
4. **L2401** `SYN` — 18.11 Formatter
   - 専用formatter未
5. **L2930** `DAT` — 2.5 Runtime reflection
   - no runtime reflection API for data
6. **L3259** `DAT` — 8. Variance
   - no variance/positivity checker yet
7. **L3265** `DAT` — 8.2 共変
   - no variance/positivity checker yet
8. **L3283** `DAT` — 8.3 反変
   - no variance/positivity checker yet
9. **L3295** `DAT` — 8.4 不変
   - no variance/positivity checker yet
10. **L3308** `DAT` — 8.5 Phantom parameter
   - no variance/positivity checker yet
11. **L3341** `DAT` — 9.2 Strict positivity
   - no variance/positivity checker yet
12. **L3435** `DAT` — 10.4 Group全体のpositivity
   - no variance/positivity checker yet
13. **L3443** `DAT` — 11.1 不変性
   - no variance/positivity checker yet
14. **L3842** `DAT` — 18.5 Optional field
   - DAT §18.5 optional record fields not in CorePattern; plan gap
15. **L4054** `DAT` — 21.3 到達不能case
   - no unreachable-case warning yet
16. **L4670** `EVAL` — `DD-TYP-IF-001`: 条件分岐による型の絞り込み
   - no occurrence typing / intersect-diff narrowing in check.rs
17. **L4983** `EVAL` — union result
   - Union type stub only; if branch union typing not enforced
18. **L4997** `EVAL` — occurrence typing
   - occurrence typing absent (DD-TYP-IF-001)
19. **L5936** `BND` — `DD-BND-021`: local state effectの除去
   - no effect-row removal for local state
20. **L7022** `MAC` — 3.3 宣言位置
   - 宣言位置マクロ使用の明示拒否なし

## Top 30 `partial`（文書出現順）
1. **L343** `LEX` — 13.1 `LEX-001` Lossless lexer/CST
   - rowan CST+unparse+shebang trivia; BOM専用node/仮想tokenは不足
2. **L353** `LEX` — 構文・字句
   - lexer/kind/parse; Comment kind未使用・OPEN字句あり
3. **L385** `LEX` — 正常例・拒否例
   - trivia保持はok; 単位suffixはinterim
4. **L417** `LEX` — 相互作用・テスト
   - TEST-LEX round-trip一部; incremental未
5. **L432** `SYN` — 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader
   - 字句〜markup+型表面/bytes核; intersect/糖衣等にギャップ
6. **L449** `SYN` — 0. 結論
   - 結論の多くは実装; 一部PKG/型は未完
7. **L471** `SYN` — 1. Source fileと文字コード
   - UTF-8+shebang trivia; BOM専用CST nodeなし
8. **L493** `SYN` — 1.2 BOM
   - BOM検出・本体strip; CST専用nodeなし
9. **L689** `SYN` — 3. 識別子
   - NFC/kebab/?!/_/不可視拒否ok; 厳密XIDは近似
10. **L691** `SYN` — 3.1 Unicode識別子
   - is_alphabetic近似; 厳密XIDではない
11. **L1045** `SYN` — 7.5 `f64`
   - 実行値はf64; 任意精度intは未
12. **L1435** `SYN` — 13. Source fileの宣言グループ
   - val/rec/localあり; 自由式・注釈対応は弱い
13. **L1437** `SYN` — 13.1 Top-level
   - top-level宣言対応; 自由式を完全拒否せず
14. **L1455** `SYN` — 13.2 型注釈
   - (type name Ty)登録; 対val必須は未
15. **L1573** `SYN` — 13.6 重複binding
   - let/rec内duplicateは検出; 全域弱い
16. **L1790** `SYN` — 16. 型構文
   - fn/app/forall/row/effects/union/record; intersect/not/diff未
17. **L1840** `SYN` — 16.3 集合論的型
   - unionのみ; intersect/not/diff未
18. **L1952** `SYN` — 16.7 型alias
   - type-alias登録; 再帰alias検査弱い
19. **L1967** `SYN` — 17. `markup` reader
   - reader/command核ok; sugar/型は部分
20. **L1986** `SYN` — 17.2 Coreとpackageの分担
   - CST readerあり; package分担は暫定
21. **L2111** `SYN` — 17.6 任意のcode埋込み
   - @(…)埋め込みCST可; walkはIdent名前提
22. **L2129** `SYN` — 17.7 糖衣展開
   - document/macro prototype; 完全sugar未
23. **L2253** `SYN` — 17.11 Markupの型
   - markup値の静的型は暫定/stub
24. **L2274** `SYN` — 18. 構文エラー回復
   - CST+ErrorNode; MissingToken等は未
25. **L2294** `SYN` — 18.2 不足した閉じ括弧
   - 未閉じはerror; 仮想MissingTokenなし
26. **L2322** `SYN` — 18.5 未終了コメント
   - 未終了コメントerror; 仮想)なし
27. **L2330** `SYN` — 18.6 Markupの`[]`
   - markup [] エラーは汎用ErrorNode
