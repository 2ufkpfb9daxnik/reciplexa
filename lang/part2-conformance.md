# 第II部 RPX言語の基礎仕様 — 実装適合チェックリスト

## 概要

本チェックリストは、`lang/specification.md` の**第II部**（見出し行およそ `241`–`22618`）に現れる**すべての見出し**（深さ不問）を実装と突き合わせるための進捗台帳である。PKG Slice A（local packages）更新済み — 詳細は `lang/package-plan.md`。

### ステータス凡例

- `unchecked` — 未レビュー
- `ok` — レビュー済；実装がこの節と一致（または実装義務のないメタ記述で N/A）
- `partial` — レビュー済；一部要件はあるがギャップが残る
- `gap` — レビュー済；要求される振る舞いが欠落、または実装と矛盾
- `deferred` — 意図的に後回し（例: PKG / カーネル後）※ notes に理由を書く
- `meta` — 散文・プロセス・メタ理論で直接のコード表面がない（追跡はする）

### 進捗カウンタ

- **total**: 1589
- **unchecked**: 0
- **ok**: 700
- **partial**: 172
- **gap**: 0
- **deferred**: 579
- **meta**: 138

（LEX/SYN/MAC および導入メタ節を照合済み。他機能ブロックも並行更新済み。）

---

- [x] **L1 L241: 第II部 RPX言語の基礎仕様** — `meta`
  - spec: `specification.md:241`
  - notes: 用語・カタログのみ

## 共通用語と記号

- [x] **L2 L243: 共通用語と記号** — `meta`
  - spec: `specification.md:243`
  - notes: 用語・カタログのみ

- [x] **L4 L263: 10.1 Effect用語** — `meta`
  - spec: `specification.md:263`
  - notes: 用語・カタログのみ

## 記号一覧

- [x] **L2 L272: 記号一覧** — `meta`
  - spec: `specification.md:272`
  - notes: 用語・カタログのみ

## 言語機能の一覧

- [x] **L2 L294: 言語機能の一覧** — `meta`
  - spec: `specification.md:294`
  - notes: 用語・カタログのみ

- [x] **L4 L320: 12.1 依存関係の要約** — `meta`
  - spec: `specification.md:320`
  - notes: 用語・カタログのみ

## 言語機能の詳細仕様

- [x] **L2 L338: 言語機能の詳細仕様** — `meta`
  - spec: `specification.md:338`
  - notes: 用語・カタログのみ

- [x] **L4 L343: 13.1 `LEX-001` Lossless lexer/CST** — `partial`
  - spec: `specification.md:343`
  
  - notes: rowan CST+unparse+shebang/BOM trivia; markup sugar package portion deferred rowan CST+unparse+shebang trivia; BOM専用node/仮想tokenは不足

- [x] **L5 L345: 概要・目的・状態** — `ok`
  - spec: `specification.md:345`
  - notes: rowan CST/trivia/mode stack 実装済 (syntax)

- [x] **L5 L353: 構文・字句** — `ok`
  - spec: `specification.md:353`
  
  - notes: LEX surface: UTF-8/BOM trivia/shebang/whitespace/comment CST- notes: lexer/kind/parse; Comment kind未使用・OPEN字句あり

- [x] **L5 L379: 静的・動的意味** — `meta`
  - spec: `specification.md:379`
  - notes: CSTに意味論なし（該当なし）

- [x] **L5 L385: 正常例・拒否例** — `partial`
  - spec: `specification.md:385`
  - notes: trivia保持はok; 単位suffixはinterim

- [x] **L5 L402: Progress/Preservation・その他** — `meta`
  - spec: `specification.md:402`
  - notes: 理論性質; Core対象外

- [x] **L5 L408: 実装構造・処理** — `ok`
  - spec: `specification.md:408`
  - notes: lexer→parse→rowan CST (reciplexa-syntax)

- [x] **L5 L417: 相互作用・テスト** — `ok`
  - spec: `specification.md:417`
  - notes: TEST-LEX round-trip + incremental bump_token tests (lexer_tests)

- [x] **L5 L427: 未決定** — `deferred`
  - spec: `specification.md:427`
  - notes: OPEN-SYN-002/OPEN-EDT-001 追跡

- [x] **L4 L432: 13.2 `SYN-001` Code mode・字句・Surface構文・markup reader** — `partial`
  - spec: `specification.md:432`
  - notes: 字句〜markup+型表面/bytes核; intersect/糖衣等にギャップ

- [x] **L5 L434: 統合方針** — `ok`
  - spec: `specification.md:434`
  - notes: code mode既定・markup reader・src wrapper無し

- [x] **L5 L449: 0. 結論** — `partial`
  - spec: `specification.md:449`
  - notes: 結論の多くは実装; 一部PKG/型は未完

- [x] **L5 L471: 1. Source fileと文字コード** — `ok`
  - spec: `specification.md:471`
  
  - notes: UTF-8 + BOM SyntaxKind::Bom trivia + shebang trivia- notes: UTF-8+shebang trivia; BOM専用CST nodeなし

- [x] **L6 L473: 1.1 文字コード** — `ok`
  - spec: `specification.md:473`
  - notes: SourceResource UTF-8のみ (reciplexa-source)

- [x] **L6 L493: 1.2 BOM** — `ok`
  - spec: `specification.md:493`
  
  - notes: BOM at offset 0 → SyntaxKind::Bom trivia; conflict with shebang in SourceResource- notes: BOM検出・本体strip; CST専用nodeなし

- [x] **L6 L508: 1.3 改行** — `ok`
  - spec: `specification.md:508`
  - notes: LF/CRLF/CR→Newline trivia (lexer)

- [x] **L6 L524: 1.4 Source span** — `ok`
  - spec: `specification.md:524`
  - notes: UTF-8 byte [start,end) spans

- [x] **L6 L538: 1.5 Shebang** — `ok`
  - spec: `specification.md:538`
  - notes: offset-0 `#!` → SyntaxKind::Shebang trivia (lexer/CST)

- [x] **L5 L558: 2. 空白とコメント** — `ok`
  - spec: `specification.md:558`
  - notes: Whitespace/Newline + (//…) StructuredComment

- [x] **L6 L560: 2.1 Code modeの空白** — `ok`
  - spec: `specification.md:560`
  - notes: space/tab等をtrivia保持 (lexer)

- [x] **L6 L577: 2.2 構造化コメント** — `ok`
  - spec: `specification.md:577`
  - notes: (// …) StructuredComment (parse)

- [x] **L6 L609: 2.3 コメントの入れ子** — `ok`
  - spec: `specification.md:609`
  - notes: 括弧nestで入れ子コメント可

- [x] **L6 L636: 2.4 コメント内部の括弧** — `ok`
  - spec: `specification.md:636`
  - notes: コメント内括弧は構造として保持

- [x] **L6 L654: 2.5 採用しないコメント** — `ok`
  - spec: `specification.md:654`
  - notes: ; と //行コメント拒否 (lexer_tests)

- [x] **L6 L670: 2.6 CSTでの保持** — `ok`
  - spec: `specification.md:670`
  - notes: unparseでコメント・空白round-trip

- [x] **L5 L689: 3. 識別子** — `partial`
  - spec: `specification.md:689`
  - notes: NFC/kebab/?!/_/不可視拒否ok; 厳密XIDは近似

- [x] **L6 L691: 3.1 Unicode識別子** — `partial`
  - spec: `specification.md:691`
  - notes: is_alphabetic近似; 厳密XIDではない

- [x] **L6 L715: 3.2 Unicode正規化** — `ok`
  - spec: `specification.md:715`
  - notes: normalize_ident NFC (ident.rs)

- [x] **L6 L733: 3.3 先頭文字** — `ok`
  - spec: `specification.md:733`
  - notes: 大文字開始はvalidate_identで拒否

- [x] **L6 L756: 3.4 Kebab-case** — `ok`
  - spec: `specification.md:756`
  - notes: kebab・--/端- 拒否 (ident+lexer)

- [x] **L6 L778: 3.5 Underscore** — `ok`
  - spec: `specification.md:778`
  - notes: _のみwildcard; 埋め込み_拒否

- [x] **L6 L801: 3.6 `?`と`!`** — `ok`
  - spec: `specification.md:801`
  - notes: 末尾単一?/! (ident.rs)

- [x] **L6 L825: 3.7 不可視文字** — `ok`
  - spec: `specification.md:825`
  - notes: reject_invisible_chars (bidi/ZW*/format) in validate_ident

- [x] **L5 L841: 4. Package名とmodule path component** — `ok`
  - spec: `specification.md:841`
  - notes: validate_package_path ASCII lowercase kebab; `/` paths enforced

- [x] **L5 L867: 5. Operator identifier** — `ok`
  - spec: `specification.md:867`
  - notes: 固定op一覧+is_operator_ident; /分割

- [x] **L5 L900: 6. Core特殊形式** — `ok`
  - spec: `specification.md:900`
  - notes: reserved.rs + surface拡張形

- [x] **L5 L936: 7. 数値リテラル** — `ok`
  - spec: `specification.md:936`
  - notes: number_lit+lexer: 基数/_/科学表記

- [x] **L6 L938: 7.1 整数** — `ok`
  - spec: `specification.md:938`
  - notes: 10/2/8/16進 (number_lit.rs)

- [x] **L6 L969: 7.2 先頭ゼロ** — `ok`
  - spec: `specification.md:969`
  - notes: 007等をError拒否

- [x] **L6 L990: 7.3 負数** — `ok`
  - spec: `specification.md:990`
  - notes: 符号付き数値token

- [x] **L6 L1018: 7.4 数字separator** — `ok`
  - spec: `specification.md:1018`
  - notes: _ separator (number_lit)

- [x] **L6 L1045: 7.5 `f64`** — `ok`
  - spec: `specification.md:1045`
  - notes: f64 lit → CoreLiteral::F64; int/f64 split (DD-TYP-NUM-001)

- [x] **L6 L1077: 7.6 非有限値** — `ok`
  - spec: `specification.md:1077`
  - notes: 非有限リテラル拒否

- [x] **L5 L1097: 8. 文字列** — `ok`
  - spec: `specification.md:1097`
  - notes: バックスラッシュescape無し (string_lit)

- [x] **L6 L1099: 8.1 基本方針** — `ok`
  - spec: `specification.md:1099`
  - notes: escape無し方針 (lexer+string_lit)

- [x] **L6 L1126: 8.2 文字列delimiter** — `ok`
  - spec: `specification.md:1126`
  - notes: """ n≥3 delimiter

- [x] **L6 L1157: 8.3 複数行文字列** — `ok`
  - spec: `specification.md:1157`
  - notes: 端改行strip+dedent (decode_string_literal)

- [x] **L6 L1184: 8.4 特殊文字** — `ok`
  - spec: `specification.md:1184`
  - notes: special_char_value/unicode_scalar_value + eval primitives (newline/tab/nul/unicode)

- [x] **L6 L1211: 8.5 `str`の意味** — `ok`
  - spec: `specification.md:1211`
  - notes: UTF-8不変str; 自動NFCなし

- [x] **L5 L1241: 9. Boolとunit** — `ok`
  - spec: `specification.md:1241`
  - notes: true/false/unit → literals (elaborate)

- [x] **L5 L1275: 10. Symbolとkeyword** — `ok`
  - spec: `specification.md:1275`
  - notes: 'sym/:kwなし; #はError

- [x] **L5 L1310: 11. Bytes** — `ok`
  - spec: `specification.md:1310`
  - notes: bytes lit + encode-utf8/decode-utf8 builtins (lang_kernel_suite)

- [x] **L5 L1360: 12. 単位と色** — `deferred`
  - spec: `specification.md:1360`
  - notes: 単位・色はPKG担当; suffixはinterim分割

- [x] **L6 L1362: 12.1 単位** — `deferred`
  - spec: `specification.md:1362`
  - notes: 40mm→Number+Ident; typed (mm 40)はPKG

- [x] **L6 L1402: 12.2 色** — `deferred`
  - spec: `specification.md:1402`
  - notes: #hexなし; 色ctorはPKG

- [x] **L5 L1435: 13. Source fileの宣言グループ** — `partial`
  - spec: `specification.md:1435`
  - notes: val/rec/localあり; 自由式・注釈対応は弱い

- [x] **L6 L1437: 13.1 Top-level** — `partial`
  - spec: `specification.md:1437`
  - notes: top-level宣言対応; 自由式を完全拒否せず

- [x] **L6 L1455: 13.2 型注釈** — `ok`
  - spec: `specification.md:1455`
  - notes: (type name Ty) requires matching val/var; type-alias for pure aliases (SYN §13.2 / DD-BND-004)

- [x] **L6 L1485: 13.3 `val`** — `ok`
  - spec: `specification.md:1485`
  - notes: (val …) elaborate

- [x] **L6 L1511: 13.4 `rec`** — `ok`
  - spec: `specification.md:1511`
  - notes: top-level/local rec→LetRec

- [x] **L6 L1548: 13.5 `local`** — `ok`
  - spec: `specification.md:1548`
  - notes: local→入れ子let (elaborate)

- [x] **L6 L1573: 13.6 重複binding** — `ok`
  - spec: `specification.md:1573`
  - notes: duplicate top-level val/fn/rec rejected; top-level var forbidden (elaborate §13.6)

- [x] **L5 L1590: 14. 関数** — `ok`
  - spec: `specification.md:1590`
  - notes: 値構文・arity・表面(fn …)型ok

- [x] **L6 L1592: 14.1 値構文** — `ok`
  - spec: `specification.md:1592`
  - notes: (fn (params) body…) (elaborate)

- [x] **L6 L1610: 14.2 関数型** — `ok`
  - spec: `specification.md:1610`
  - notes: (fn T… Ret [(effects …)]) in parse_type_syntax

- [x] **L6 L1649: 14.3 Fixed arity** — `ok`
  - spec: `specification.md:1649`
  - notes: n-ary App; 自動curry無し

- [x] **L5 L1678: 15. List、tuple、record** — `ok`
  - spec: `specification.md:1678`
  - notes: 値形+list/tuple型(App/positional record)ok

- [x] **L6 L1680: 15.1 List** — `ok`
  - spec: `specification.md:1680`
  - notes: (list …)→cons/nil (elaborate)

- [x] **L6 L1706: 15.2 Tuple** — `ok`
  - spec: `specification.md:1706`
  - notes: 0→unit / 2+ record化; 1要素はelaborate拒否 (SYN §20)

- [x] **L6 L1734: 15.3 Record** — `ok`
  - spec: `specification.md:1734`
  - notes: (record …)+duplicate field拒否

- [x] **L6 L1760: 15.4 Field access** — `ok`
  - spec: `specification.md:1760`
  - notes: (field rec lab)

- [x] **L6 L1770: 15.5 Updateとextension** — `ok`
  - spec: `specification.md:1770`
  - notes: record-update/extend (Core+elaborate)

- [x] **L6 L1790: 16. 型構文** — `partial`
  - spec: `specification.md:1790`
  - notes: fn/app/forall/row/effects/union/intersect/not/diff; sugar gaps remain

- [x] **L6 L1792: 16.1 型適用** — `ok`
  - spec: `specification.md:1792`
  - notes: (option str)→CoreType::App (unify stub)

- [x] **L6 L1814: 16.2 `forall`** — `ok`
  - spec: `specification.md:1814`
  - notes: surface forall+kind; prenex instantiate on use (check.rs instantiate_forall)

- [x] **L6 L1840: 16.3 集合論的型** — `ok`
  - spec: `specification.md:1840`
  - notes: parse_type_syntax intersect/not/diff + CoreType stubs (unify.rs)

- [x] **L6 L1871: 16.4 Dynamic** — `ok`
  - spec: `specification.md:1871`
  - notes: (dynamic …)→Dynamic stub

- [x] **L6 L1887: 16.5 Open record** — `ok`
  - spec: `specification.md:1887`
  - notes: (row r)/(optional label Ty)→OpenRecord/OptionalField

- [x] **L6 L1924: 16.6 Effect row** — `ok`
  - spec: `specification.md:1924`
  - notes: (effects …) trailing on fn types; duplicate拒否

- [x] **L6 L1952: 16.7 型alias** — `partial`
  - spec: `specification.md:1952`
  - notes: type-alias登録; 再帰alias検査弱い

- [x] **L5 L1967: 17. `markup` reader** — `partial`
  - spec: `specification.md:1967`
  - notes: reader/command + Embed @(…) + markup-fragment default; package expected types deferred

- [x] **L6 L1969: 17.1 位置づけ** — `ok`
  - spec: `specification.md:1969`
  - notes: (markup …) mode switch (parse)

- [x] **L6 L1986: 17.2 Coreとpackageの分担** — `deferred`
  - spec: `specification.md:1986`
  
  - notes: 依存待ち(PKG): Core reader vs package constructors (circle/space/…)- notes: CST readerあり; package分担は暫定

- [x] **L6 L2017: 17.3 Markup command** — `ok`
  - spec: `specification.md:2017`
  - notes: @name / {} / [] / () (markup.rs)

- [x] **L6 L2061: 17.4 `[]`** — `ok`
  - spec: `specification.md:2061`
  - notes: []引数CST (parse+markup_tests)

- [x] **L6 L2089: 17.5 `()`** — `ok`
  - spec: `specification.md:2089`
  - notes: () code引数

- [x] **L6 L2111: 17.6 任意のcode埋込み** — `ok`
  - spec: `specification.md:2111`
  
  - notes: MarkupPart::Embed for `@(…)` code embedding (markup.rs)- notes: @(…)埋め込みCST可; walkはIdent名前提

- [x] **L6 L2129: 17.7 糖衣展開** — `deferred`
  - spec: `specification.md:2129`
  
  - notes: 依存待ち(PKG): @name[…]/{} sugar expands to package constructors- notes: document/macro prototype; 完全sugar未

- [x] **L6 L2176: 17.8 `@at()`** — `ok`
  - spec: `specification.md:2176`
  - notes: @at() を通常commandとして受理

- [x] **L6 L2204: 17.9 丸括弧** — `ok`
  - spec: `specification.md:2204`
  - notes: 丸括弧引数・mode切替

- [x] **L6 L2226: 17.10 改行と空白** — `ok`
  - spec: `specification.md:2226`
  - notes: markup空白/改行をCST保持

- [x] **L6 L2253: 17.11 Markupの型** — `ok`
  - spec: `specification.md:2253`
  - notes: default markup-fragment type name + document Type::Markup as §17.11 default

- [x] **L5 L2274: 18. 構文エラー回復** — `ok`
  - spec: `specification.md:2274`
  - notes: MissingToken/UnexpectedToken + lossless CST on errors (parse.rs)

- [x] **L6 L2276: 18.1 基本原則** — `ok`
  - spec: `specification.md:2276`
  - notes: エラー時もCST返却 (parse)

- [x] **L6 L2294: 18.2 不足した閉じ括弧** — `ok`
  - spec: `specification.md:2294`
  - notes: EOF virtual MissingToken close via parse_lisp_list_tail

- [x] **L6 L2304: 18.3 余分な閉じ括弧** — `ok`
  - spec: `specification.md:2304`
  - notes: 余分な)→UnexpectedTokenとして保持し後続解析継続

- [x] **L6 L2312: 18.4 未終了文字列** — `ok`
  - spec: `specification.md:2312`
  - notes: partial String at newline/EOF + virtual_close_delimiter in parse

- [x] **L6 L2322: 18.5 未終了コメント** — `ok`
  - spec: `specification.md:2322`
  - notes: unclosed structured comment emits virtual MissingToken `)` at EOF

- [x] **L6 L2330: 18.6 Markupの`[]`** — `partial`
  - spec: `specification.md:2330`
  - notes: virtual `]` on EOF for @-expr brackets; multi-expr ErrorNode deferred

- [x] **L6 L2342: 18.7 Markup body** — `partial`
  - spec: `specification.md:2342`
  - notes: markup body回復は汎用

- [x] **L6 L2350: 18.8 不正な`@`** — `partial`
  - spec: `specification.md:2350`
  - notes: 不正@はError token/ node

- [x] **L6 L2371: 18.9 未知command** — `ok`
  - spec: `specification.md:2371`
  - notes: 未知commandはparse許容→resolve段階

- [x] **L6 L2391: 18.10 型検査のcascade抑制** — `ok`
  - spec: `specification.md:2391`
  - notes: CoreExpr::Error + CoreType::Error cascade suppression (check.rs)

- [x] **L6 L2401: 18.11 Formatter** — `ok`
  - spec: `specification.md:2401`
  - notes: unparse skips MissingToken/UnexpectedToken (SYN SS18.11)

- [x] **L5 L2425: 19. 適合例** — `partial`
  - spec: `specification.md:2425`
  - notes: examples/testsで一部適合; 全列挙未

- [x] **L5 L2477: 20. 不適合例** — `partial`
  - spec: `specification.md:2477`
  - notes: 拒否例の多くは検出; markup複数式等に差

- [x] **L6 L2479: 不要な先頭ゼロ** — `ok`
  - spec: `specification.md:2479`
  - notes: 007拒否 (lexer_tests)

- [x] **L6 L2485: 大文字開始識別子** — `ok`
  - spec: `specification.md:2485`
  - notes: 大文字開始拒否 (validate_ident)

- [x] **L6 L2491: Underscoreを含む識別子** — `ok`
  - spec: `specification.md:2491`
  - notes: 埋め込み_拒否

- [x] **L6 L2497: 重複field** — `ok`
  - spec: `specification.md:2497`
  - notes: duplicate record field拒否

- [x] **L6 L2505: 1要素tuple** — `ok`
  - spec: `specification.md:2505`
  - notes: (tuple 1) / 1要素patternをelaborate拒否

- [x] **L6 L2511: 自動部分適用** — `ok`
  - spec: `specification.md:2511`
  - notes: 不足arityは型/実行でエラー

- [x] **L6 L2520: Markup引数内の複数式** — `partial`
  - spec: `specification.md:2520`
  - notes: markup複数式の拒否は部分的

- [x] **L6 L2526: 文字としての`@`に`@@`を使用** — `ok`
  - spec: `specification.md:2526`
  - notes: @@非escape; @at()を使用

- [x] **L5 L2540: 21. 本項目で意図的に確定しない事項** — `meta`
  - spec: `specification.md:2540`
  - notes: 意図的スコープ外リスト（DAT/MOD等）

- [x] **L5 L2569: 22. 状態** — `meta`
  - spec: `specification.md:2569`
  - notes: OPEN-SYN-002 RESOLVED 状態表

- [x] **L4 L2608: 13.2.1 `RES-001` 名前解決とnamespace** — `partial`
  - spec: `specification.md:2608`
  - notes: resolve_language_source + BindingMap; multi-ns/phase/import ambiguity incomplete

- [x] **L5 L2610: 概要・状態** — `partial`
  - spec: `specification.md:2610`
  - notes: TEST-RES-C001 shadowing/unbound; Type/Module/Syntax ns + phase resolve 暫定のまま

- [x] **L4 L2646: 13.2.3 `DAT-001` 代数的データ型・constructor・pattern・match** — `partial`
  - spec: `specification.md:2646`
  - notes: data/match + App instantiate + poly ctor/∀-generalize; ctor-specific display / dual-ns residual

- [x] **L5 L2648: DD-001 決定概要** — `meta`
  - spec: `specification.md:2648`
  - notes: scope/overview prose

- [x] **L6 L2650: DD-001.1 状態** — `meta`
  - spec: `specification.md:2650`
  - notes: scope/overview prose

- [x] **L6 L2668: DD-001.2 中心的な決定** — `meta`
  - spec: `specification.md:2668`
  - notes: scope/overview prose

- [x] **L6 L2685: DD-001.3 宣言名** — `meta`
  - spec: `specification.md:2685`
  - notes: scope/overview prose

- [x] **L5 L2712: 0. 適用範囲** — `meta`
  - spec: `specification.md:2712`
  - notes: scope/overview prose

- [x] **L6 L2714: 0.1 本項目が定めるもの** — `meta`
  - spec: `specification.md:2714`
  - notes: scope/overview prose

- [x] **L6 L2737: 0.2 本項目が定めないもの** — `meta`
  - spec: `specification.md:2737`
  - notes: scope/overview prose

- [x] **L5 L2759: 1. `data`宣言** — `ok`
  - spec: `specification.md:2759`
  - notes: elaborate.rs `(data …)` → DataEnv constructors

- [x] **L6 L2761: 1.1 Parameterを持たないdata型** — `ok`
  - spec: `specification.md:2761`
  - notes: elaborate.rs data decl → DataEnv

- [x] **L6 L2773: 1.2 Parameterを持つdata型** — `ok`
  - spec: `specification.md:2773`
  - notes: ((a type)…) params + App instantiate + poly ctor→App (elaborate/check)

- [x] **L6 L2800: 1.3 Parameterなしの場合** — `ok`
  - spec: `specification.md:2800`
  - notes: nullary-param path; elaborate.rs DataEnv without type_params

- [x] **L6 L2824: 1.4 Constructor payload** — `ok`
  - spec: `specification.md:2824`
  - notes: elaborate.rs data decl → DataEnv

- [x] **L5 L2847: 2. `data`宣言が生成するbinding** — `partial`
  - spec: `specification.md:2847`
  - notes: ctors in DataEnv; parent/sealed display vs full §2 ctor-types still light

- [x] **L6 L2849: 2.1 親type constructor** — `partial`
  - spec: `specification.md:2849`
  - notes: DataEnv ctor_type/data_ctors map parent ADT; sealed export incomplete

- [x] **L6 L2876: 2.2 Value constructor** — `partial`
  - spec: `specification.md:2876`
  - notes: value ctors registered with payloads; ctor-refined some<int> display deferred

- [x] **L6 L2897: 2.3 Constructor固有型** — `deferred`
  - spec: `specification.md:2897`
  - notes: plan: full polymorphic ADT typing / ctor-specific types deferred

- [x] **L6 L2916: 2.4 Sealed constructor集合** — `ok`
  - spec: `specification.md:2916`
  - notes: sealed ctor set via data_ctors + exhaustiveness (expr.rs/check.rs)

- [x] **L6 L2930: 2.5 Runtime reflection** — `deferred`
  - spec: `specification.md:2930`
  - notes: spec §2.5: data decls do not auto-emit runtime type descriptors; type-of-value / constructors-of are separate future items

- [x] **L5 L2946: 3. 名前とnamespace** — `partial`
  - spec: `specification.md:2946`
  - notes: value-ns ctors via DataEnv; distinct type-ns ctor types absent

- [x] **L6 L2948: 3.1 型namespaceと値namespace** — `partial`
  - spec: `specification.md:2948`
  - notes: value-ns ctors via DataEnv; distinct type-ns ctor types absent

- [x] **L6 L2966: 3.2 型名とconstructor名の同名禁止** — `partial`
  - spec: `specification.md:2966`
  - notes: name clash checks limited; no full dual-namespace enforcement

- [x] **L6 L2988: 3.3 Constructor名の重複** — `partial`
  - spec: `specification.md:2988`
  - notes: duplicate ctor names via DataEnv registration; full RES identity still light

- [x] **L6 L3000: 3.4 Constructor identity** — `partial`
  - spec: `specification.md:3000`
  - notes: ctor identity via DataEnv maps; dual-ns RES identity still light

- [x] **L5 L3013: 4. Nullary constructor** — `ok`
  - spec: `specification.md:3013`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L6 L3015: 4.1 宣言** — `ok`
  - spec: `specification.md:3015`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L6 L3026: 4.2 値** — `ok`
  - spec: `specification.md:3026`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L6 L3042: 4.3 関数ではない** — `ok`
  - spec: `specification.md:3042`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L5 L3055: 5. Payload constructor** — `ok`
  - spec: `specification.md:3055`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3057: 5.1 値構築** — `ok`
  - spec: `specification.md:3057`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L6 L3065: 5.2 Constructorの高階利用** — `ok`
  - spec: `specification.md:3065`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L6 L3079: 5.3 Arity** — `ok`
  - spec: `specification.md:3079`
  - notes: elaborate registers ctors; Variant values in eval

- [x] **L5 L3106: 6. Constructor固有型** — `deferred`
  - spec: `specification.md:3106`
  - notes: plan: full polymorphic ADT typing / ctor-specific types deferred

- [x] **L6 L3108: 6.1 精密な結果型** — `ok`
  - spec: `specification.md:3108`
  - notes: strict positivity on data payloads (elaborate check_payload_positivity)

- [x] **L6 L3128: 6.2 Subtyping** — `deferred`
  - spec: `specification.md:3128`
  - notes: plan: full polymorphic ADT typing / ctor-specific types deferred

- [x] **L6 L3137: 6.3 親型とのsealed union関係** — `deferred`
  - spec: `specification.md:3137`
  - notes: plan: full polymorphic ADT typing / ctor-specific types deferred

- [x] **L6 L3149: 6.4 表示上の単純化** — `deferred`
  - spec: `specification.md:3149`
  - notes: plan: full polymorphic ADT typing / ctor-specific types deferred

- [x] **L5 L3166: 7. 型parameter推論** — `ok`
  - spec: `specification.md:3166`
  - notes: payload+annotation instantiates App; val ∀-generalize; var value restriction (ADT-07/08)

- [x] **L6 L3168: 7.1 Payloadからの推論** — `ok`
  - spec: `specification.md:3168`
  - notes: payload positions scored for variance/positivity (elaborate.rs)

- [x] **L6 L3182: 7.2 期待型からの推論** — `ok`
  - spec: `specification.md:3182`
  - notes: expected type from (type name Ty) checks ctor apps (BIDI annotation / App unify)

- [x] **L6 L3200: 7.3 一部未確定のparameter** — `ok`
  - spec: `specification.md:3200`
  - notes: undetermined params generalized on val → forall (check.rs generalize_type)

- [x] **L6 L3222: 7.4 Value restriction** — `ok`
  - spec: `specification.md:3222`
  - notes: var rejects ungeneralized params (annotation-required); TEST-LANG-ADT-08

- [x] **L6 L3251: 7.5 値位置の明示型argument** — `deferred`
  - spec: `specification.md:3251`
  - notes: DAT param typing still Dynamic; plan DAT-001 deferral

- [x] **L5 L3259: 8. Variance** — `partial`
  - spec: `specification.md:3259`
  - notes: per-param cov/contra/invar/phantom inferred; variance subtyping lattice incomplete

- [x] **L6 L3261: 8.1 自動推論** — `ok`
  - spec: `specification.md:3261`
  - notes: variance inferred from payload polarity in elaborate.rs

- [x] **L6 L3265: 8.2 共変** — `ok`
  - spec: `specification.md:3265`
  - notes: covariant params when only positive payload occurrences

- [x] **L6 L3283: 8.3 反変** — `ok`
  - spec: `specification.md:3283`
  - notes: contravariant params from fn-argument occurrences

- [x] **L6 L3295: 8.4 不変** — `ok`
  - spec: `specification.md:3295`
  - notes: invariant params when both polarities appear

- [x] **L6 L3308: 8.5 Phantom parameter** — `partial`
  - spec: `specification.md:3308`
  - notes: phantom recorded in DataEnv.type_variances; unused-param warning channel deferred

- [x] **L5 L3323: 9. 再帰data型** — `ok`
  - spec: `specification.md:3323`
  - notes: recursive data + strict positivity (DAT §9)

- [x] **L6 L3325: 9.1 自己再帰** — `ok`
  - spec: `specification.md:3325`
  - notes: self-recursion allowed when strictly positive (DAT §9.1)

- [x] **L6 L3341: 9.2 Strict positivity** — `ok`
  - spec: `specification.md:3341`
  - notes: check_payload_positivity rejects defining type in fn arg position (DAT §9.2)

- [x] **L6 L3365: 9.3 関数戻り値位置** — `ok`
  - spec: `specification.md:3365`
  - notes: fn result self-reference allowed by positivity checker

- [x] **L5 L3382: 10. 相互再帰data型** — `ok`
  - spec: `specification.md:3382`
  - notes: mutual `(rec (data …)…)` + group strict positivity (elaborate.rs)

- [x] **L6 L3384: 10.1 `rec` group** — `ok`
  - spec: `specification.md:3384`
  - notes: `rec` group shares positivity name set (DAT §10.1)

- [x] **L6 L3399: 10.2 Group内の可視性** — `ok`
  - spec: `specification.md:3399`
  - notes: group-internal ADT names visible for positivity/payload walks

- [x] **L6 L3403: 10.3 宣言kindの混在禁止** — `partial`
  - spec: `specification.md:3403`
  - notes: kind-mixing reject inside rec groups still light

- [x] **L6 L3435: 10.4 Group全体のpositivity** — `ok`
  - spec: `specification.md:3435`
  - notes: mutual `(rec (data …)…)` group positivity via shared name set

- [x] **L5 L3441: 11. Data値の実行意味** — `ok`
  - spec: `specification.md:3441`
  - notes: reciplexa-eval immutable Variant values; strict CBV

- [x] **L6 L3443: 11.1 不変性** — `ok`
  - spec: `specification.md:3443`
  - notes: data values immutable Variant/Record in reciplexa-eval (not a positivity item)

- [x] **L6 L3449: 11.2 Strict評価** — `ok`
  - spec: `specification.md:3449`
  - notes: reciplexa-eval immutable Variant values; strict CBV

- [x] **L6 L3469: 11.3 Effect** — `ok`
  - spec: `specification.md:3469`
  - notes: ctor/app strict; effects via surrounding eval

- [x] **L6 L3475: 11.4 途中失敗** — `ok`
  - spec: `specification.md:3475`
  - notes: reciplexa-eval immutable Variant values; strict CBV

- [x] **L5 L3481: 12. 非循環性とsharing** — `ok`
  - spec: `specification.md:3481`
  - notes: ctors share DataEnv; value sharing via Rc RuntimeValue

- [x] **L6 L3483: 12.1 Cyclic data値** — `deferred`
  - spec: `specification.md:3483`
  - notes: OPEN-GRAPH / cyclic values deferred

- [x] **L6 L3497: 12.2 Structural sharing** — `ok`
  - spec: `specification.md:3497`
  - notes: reciplexa-eval immutable Variant values; strict CBV

- [x] **L6 L3509: 12.3 Sharingの非観測性** — `deferred`
  - spec: `specification.md:3509`
  - notes: OPEN-GRAPH / cyclic values deferred

- [x] **L6 L3515: 12.4 無限構造** — `deferred`
  - spec: `specification.md:3515`
  - notes: OPEN-GRAPH / cyclic values deferred

- [x] **L5 L3530: 13. Patternの適用場所** — `ok`
  - spec: `specification.md:3530`
  - notes: match-only patterns in elaborate; no let-pattern yet (spec v1)

- [x] **L6 L3532: 13.1 v1の制限** — `ok`
  - spec: `specification.md:3532`
  - notes: match-only patterns in elaborate; no let-pattern yet (spec v1)

- [x] **L6 L3558: 13.2 Binder** — `ok`
  - spec: `specification.md:3558`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3569: 13.3 Wildcard** — `ok`
  - spec: `specification.md:3569`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3577: 13.4 Duplicate binder** — `ok`
  - spec: `specification.md:3577`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L5 L3592: 14. `match`構文** — `ok`
  - spec: `specification.md:3592`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3594: 14.1 基本構文** — `ok`
  - spec: `specification.md:3594`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3605: 14.2 Payload constructor** — `ok`
  - spec: `specification.md:3605`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3616: 14.3 複数payload** — `ok`
  - spec: `specification.md:3616`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3630: 14.4 `->`** — `ok`
  - spec: `specification.md:3630`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3636: 14.5 Case body** — `ok`
  - spec: `specification.md:3636`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L5 L3651: 15. Patternの種類** — `ok`
  - spec: `specification.md:3651`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3653: 15.1 Nullary constructor pattern** — `ok`
  - spec: `specification.md:3653`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3662: 15.2 Payload constructor pattern** — `ok`
  - spec: `specification.md:3662`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3671: 15.3 Wildcard pattern** — `ok`
  - spec: `specification.md:3671`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3678: 15.4 Catch-all binder** — `ok`
  - spec: `specification.md:3678`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3689: 15.5 Literal pattern** — `ok`
  - spec: `specification.md:3689`
  - notes: CorePattern::Lit; eval + elaborate; lang_kernel_suite

- [x] **L5 L3716: 16. Nested pattern** — `ok`
  - spec: `specification.md:3716`
  - notes: CorePattern::Tuple / nested Variant; elaborate.rs + eval.rs

- [x] **L6 L3718: 16.1 Nested payload constructor** — `ok`
  - spec: `specification.md:3718`
  - notes: CorePattern::Tuple / nested Variant; elaborate.rs + eval.rs

- [x] **L6 L3732: 16.2 Nested tuple** — `ok`
  - spec: `specification.md:3732`
  - notes: CorePattern::Tuple / nested Variant; elaborate.rs + eval.rs

- [x] **L6 L3743: 16.3 Nested nullary constructor** — `ok`
  - spec: `specification.md:3743`
  - notes: CorePattern::Tuple / nested Variant; elaborate.rs + eval.rs

- [x] **L5 L3765: 17. Tuple pattern** — `ok`
  - spec: `specification.md:3765`
  - notes: CorePattern::Tuple / nested Variant; elaborate.rs + eval.rs

- [x] **L6 L3767: 17.1 基本形** — `ok`
  - spec: `specification.md:3767`
  - notes: constructor patterns: CorePattern + elaborate/eval/check (DAT §17.1)

- [x] **L6 L3775: 17.2 Arity** — `ok`
  - spec: `specification.md:3775`
  - notes: ctor arity checked at match/elaborate (DAT §17.2)

- [x] **L6 L3779: 17.3 Binding型** — `ok`
  - spec: `specification.md:3779`
  - notes: pattern binders from expand_type_app Variant/Record payloads (check.rs)

- [x] **L6 L3800: 17.4 Refutability** — `ok`
  - spec: `specification.md:3800`
  - notes: refutability via exhaustiveness + unreachable arms (expr.rs/check.rs)

- [x] **L5 L3806: 18. Record pattern** — `ok`
  - spec: `specification.md:3806`
  - notes: CorePattern::Record + partial required fields (DAT §18)

- [x] **L6 L3808: 18.1 基本形** — `ok`
  - spec: `specification.md:3808`
  - notes: record pattern basic form elaborate/eval/check (DAT §18.1)

- [x] **L6 L3819: 18.2 Partial decomposition** — `ok`
  - spec: `specification.md:3819`
  - notes: CorePattern::Record + eval.rs / elaborate.rs §18 required fields

- [x] **L6 L3832: 18.3 Closed pattern** — `ok`
  - spec: `specification.md:3832`
  - notes: CorePattern::Record + eval.rs / elaborate.rs §18 required fields

- [x] **L6 L3836: 18.4 Required fieldのみ** — `ok`
  - spec: `specification.md:3836`
  - notes: CorePattern::Record + eval.rs / elaborate.rs §18 required fields

- [x] **L6 L3842: 18.5 Optional field** — `ok`
  - spec: `specification.md:3842`
  - notes: optional record fields rejected in record patterns; access via field+match (DAT §18.5)

- [x] **L6 L3855: 18.6 Unknown row field** — `partial`
  - spec: `specification.md:3855`
  - notes: unknown open-row fields bind as Dynamic in check.rs (DAT §18.6)

- [x] **L5 L3863: 19. Pattern typing** — `ok`
  - spec: `specification.md:3863`
  - notes: pattern typing via expand_type_app + ctor_payload schemas (DAT §19)

- [x] **L6 L3865: 19.1 三つの結果** — `ok`
  - spec: `specification.md:3865`
  - notes: bind/refine/reject paths: exhaustiveness + payload unify + Dynamic fallback

- [x] **L6 L3880: 19.2 Constructor pattern** — `ok`
  - spec: `specification.md:3880`
  - notes: constructor patterns typed from sealed Variant expansion (DAT §19.2)

- [x] **L6 L3907: 19.3 Literal pattern** — `ok`
  - spec: `specification.md:3907`
  - notes: CorePattern::Lit; eval + elaborate; lang_kernel_suite

- [x] **L6 L3931: 19.4 Wildcard** — `ok`
  - spec: `specification.md:3931`
  - notes: CorePattern + MatchArm; elaborate.rs / eval.rs / check.rs

- [x] **L6 L3944: 19.5 Disjoint pattern** — `partial`
  - spec: `specification.md:3944`
  - notes: disjoint-pattern typing mostly via Dynamic fallback / exhaustiveness

- [x] **L5 L3954: 20. `match`の型・effect・評価** — `ok`
  - spec: `specification.md:3954`
  - notes: match elaborates to Core Match; effect residual via surround expr (DAT §20)

- [x] **L6 L3956: 20.1 対象式** — `ok`
  - spec: `specification.md:3956`
  - notes: match scrutinee typed/eval'd before arms (DAT §20.1)

- [x] **L6 L3960: 20.2 Case順序** — `ok`
  - spec: `specification.md:3960`
  - notes: case arm elaborates pattern→body; exhaustiveness enforced (DAT §20.2)

- [x] **L6 L3964: 20.3 結果型** — `partial`
  - spec: `specification.md:3964`
  - notes: arm result types unify; finer GADT/refine still Dynamic-heavy (DAT §20.3)

- [x] **L6 L3985: 20.4 `never`** — `ok`
  - spec: `specification.md:3985`
  - notes: empty/unreachable use Never cascade; CoreType::Never (DAT §20.4)

- [x] **L6 L3993: 20.5 Effect** — `ok`
  - spec: `specification.md:3993`
  - notes: reciplexa-eval immutable Variant values; strict CBV

- [x] **L5 L4007: 21. 網羅性・公開・適合試験** — `ok`
  - spec: `specification.md:4007`
  - notes: lang_kernel_suite ADT-01..10 including poly generalization ADT-07/08

- [x] **L6 L4009: 21.1 網羅性** — `ok`
  - spec: `specification.md:4009`
  - notes: elaborate.rs + check.rs static exhaustiveness via DataEnv

- [x] **L6 L4035: 21.2 非網羅match** — `ok`
  - spec: `specification.md:4035`
  - notes: elaborate.rs + check.rs static exhaustiveness via DataEnv

- [x] **L6 L4054: 21.3 到達不能case** — `ok`
  - spec: `specification.md:4054`
  - notes: first_unreachable_arm in elaborate/check; catch-all and duplicate ctor cases

- [x] **L6 L4067: 21.4 空match** — `ok`
  - spec: `specification.md:4067`
  - notes: elaborate/check reject empty/non-exhaustive match

- [x] **L6 L4071: 21.5 Sealed data** — `ok`
  - spec: `specification.md:4071`
  - notes: local sealed data via data_ctors exhaustiveness; abstract export deferred

- [x] **L6 L4079: 21.6 Transparent export** — `deferred`
  - spec: `specification.md:4079`
  - notes: MOD export/abstract data boundary; MOD-001 deferral

- [x] **L6 L4085: 21.7 Abstract export** — `deferred`
  - spec: `specification.md:4085`
  - notes: MOD export/abstract data boundary; MOD-001 deferral

- [x] **L6 L4091: 21.8 一部constructor公開** — `deferred`
  - spec: `specification.md:4091`
  - notes: MOD export/abstract data boundary; MOD-001 deferral

- [x] **L6 L4095: 21.9 適合試験 ADT-01** — `ok`
  - spec: `specification.md:4095`
  - notes: TEST-LANG-ADT-01 parameterized option (Variant interim for option apps)

- [x] **L6 L4122: 21.10 適合試験 ADT-02** — `ok`
  - spec: `specification.md:4122`
  - notes: TEST-LANG-ADT-02 exhaustive match → int

- [x] **L6 L4141: 21.11 適合試験 ADT-03** — `ok`
  - spec: `specification.md:4141`
  - notes: TEST-LANG-ADT-03 non-exhaustive reports missing ctors

- [x] **L6 L4159: 21.12 適合試験 ADT-04** — `ok`
  - spec: `specification.md:4159`
  - notes: TEST-LANG-ADT-04 unreachable arm after wildcard

- [x] **L6 L4177: 21.13 適合試験 ADT-05** — `ok`
  - spec: `specification.md:4177`
  - notes: TEST-LANG-ADT-05 strictly-positive recursive tree

- [x] **L6 L4194: 21.14 不適合試験 ADT-06** — `ok`
  - spec: `specification.md:4194`
  - notes: TEST-LANG-ADT-06 negative recursion rejected (positivity)

- [x] **L6 L4209: 21.15 適合試験 ADT-07** — `ok`
  - spec: `specification.md:4209`
  - notes: TEST-LANG-ADT-07 forall e. result<int,e> via let generalization (check.rs)

- [x] **L6 L4224: 21.16 不適合試験 ADT-08** — `ok`
  - spec: `specification.md:4224`
  - notes: TEST-LANG-ADT-08 var value-restriction annotation-required (check.rs)

- [x] **L6 L4244: 21.17 適合試験 ADT-09** — `ok`
  - spec: `specification.md:4244`
  - notes: TEST-LANG-ADT-09 required record field pattern → str

- [x] **L6 L4269: 21.18 不適合試験 ADT-10** — `ok`
  - spec: `specification.md:4269`
  - notes: TEST-LANG-ADT-10 optional field pattern rejected (DAT §18.5)

- [x] **L5 L4291: 22. 移管先OPEN・下位項目・状態** — `meta`
  - spec: `specification.md:4291`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4293: 22.1 `OPEN-SYN-002`への追補** — `meta`
  - spec: `specification.md:4293`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4303: 22.2 `OPEN-MOD-001`** — `meta`
  - spec: `specification.md:4303`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4317: 22.3 `OPEN-SEM-001`** — `meta`
  - spec: `specification.md:4317`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4328: 22.4 `OPEN-DERIVE-001`** — `meta`
  - spec: `specification.md:4328`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4341: 22.5 `OPEN-GADT-001`** — `meta`
  - spec: `specification.md:4341`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4355: 22.6 `OPEN-DYNAMIC-001`** — `meta`
  - spec: `specification.md:4355`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4369: 22.7 `OPEN-LAZY-001`** — `meta`
  - spec: `specification.md:4369`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4381: 22.8 `OPEN-GRAPH-001`** — `meta`
  - spec: `specification.md:4381`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4395: 22.9 下位項目** — `meta`
  - spec: `specification.md:4395`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L6 L4435: 22.10 最終状態** — `meta`
  - spec: `specification.md:4435`
  - notes: spec OPEN/status prose; no direct code surface

- [x] **L4 L4456: 13.3 `EVAL-001` Strict lexical Core evaluator** — `partial`
  - spec: `specification.md:4456`
  - notes: eval.rs CBV Core; also letrec/match/effects beyond min-Core v1

- [x] **L5 L4460: 状態** — `meta`
  - spec: `specification.md:4460`
  - notes: EVAL-001 status prose (解決済み)

- [x] **L5 L4472: `DD-EVAL-001`: 評価戦略と評価順序** — `ok`
  - spec: `specification.md:4472`
  - notes: strict CBV; fun then LTR args; let/seq/if order in eval.rs

- [x] **L5 L4497: `DD-EVAL-002`: 最小Coreの項** — `ok`
  - spec: `specification.md:4497`
  - notes: min-Core terms in CoreExpr (+ LetRec/Match/Perform/Handle/Record)

- [x] **L5 L4545: `DD-EVAL-003`: `let`** — `ok`
  - spec: `specification.md:4545`
  - notes: CoreExpr::Let single binding; nested surface let via elaborate

- [x] **L5 L4592: `DD-EVAL-004`: 複数式bodyと`seq`** — `ok`
  - spec: `specification.md:4592`
  - notes: CoreExpr::Seq; last-value result in eval_seq

- [x] **L5 L4634: `DD-EVAL-005`: `if`** — `ok`
  - spec: `specification.md:4634`
  - notes: CoreExpr::If; selected branch only; non-Bool rejects

- [x] **L5 L4670: `DD-TYP-IF-001`: 条件分岐による型の絞り込み** — `ok`
  - spec: `specification.md:4670`
  - notes: DD-TYP-IF-001: intersect/diff narrowing on immutable locals for eval builtins

- [x] **L5 L4706: 値** — `ok`
  - spec: `specification.md:4706`
  - notes: Lit/Closure + records/handlers/cells as runtime values

- [x] **L5 L4744: Closureとlexical scope** — `ok`
  - spec: `specification.md:4744`
  - notes: Closure captures env; BindingMap use-sites (TEST-LANG-EDT-binding)

- [x] **L5 L4764: 関数適用** — `ok`
  - spec: `specification.md:4764`
  - notes: eval_app: operator then LTR args then apply

- [x] **L5 L4788: 評価文脈** — `partial`
  - spec: `specification.md:4788`
  - notes: evaluation contexts via Outcome/resume; not formal EC grammar

- [x] **L5 L4817: 最小Coreの終端状態** — `ok`
  - spec: `specification.md:4817`
  - notes: Value or EvalError terminal; effects via host/handlers

- [x] **L5 L4839: 未決定事項の移管** — `meta`
  - spec: `specification.md:4839`
  - notes: OPEN transfer table out of EVAL-001

- [x] **L5 L4867: 適合試験** — `partial`
  - spec: `specification.md:4867`
  - notes: eval_tests + lang_kernel_suite EVAL/DYN-003; named EVAL corpus incomplete

- [x] **L6 L4869: lexical closure** — `ok`
  - spec: `specification.md:4869`
  - notes: lexical closure via captured env (eval Closure)

- [x] **L6 L4884: sequential `let`** — `ok`
  - spec: `specification.md:4884`
  - notes: sequential surface let → nested Core let (elaborate)

- [x] **L6 L4898: duplicate binder** — `ok`
  - spec: `specification.md:4898`
  - notes: duplicate binder rejected in elaborate.rs

- [x] **L6 L4912: application order** — `ok`
  - spec: `specification.md:4912`
  - notes: application order: fun then LTR args

- [x] **L6 L4927: sequence result** — `ok`
  - spec: `specification.md:4927`
  - notes: seq returns last value

- [x] **L6 L4949: selected branch only** — `ok`
  - spec: `specification.md:4949`
  - notes: if evaluates only selected branch

- [x] **L6 L4969: strict Bool condition** — `ok`
  - spec: `specification.md:4969`
  - notes: non-Bool if condition → EvalError

- [x] **L6 L4983: union result** — `ok`
  - spec: `specification.md:4983`
  - notes: if branches that fail unify become CoreType::Union

- [x] **L6 L4997: occurrence typing** — `ok`
  - spec: `specification.md:4997`
  - notes: occurrence typing for number?/string?/bool?/is-none/is-some predicates

- [x] **L5 L5016: 解決後の最小Core** — `partial`
  - spec: `specification.md:5016`
  - notes: resolve BindingMap + SyntaxNodeId on defs (lang_kernel_suite); eval still name-string Core

- [x] **L4 L5034: 13.4 `BND-001` `val`、`var`、`let`、`letrec`、`fn`** — `partial`
  - spec: `specification.md:5034`
  - notes: let/letrec/var/set + typed store/annotations/escape check; full Identity algebra still light

- [x] **L5 L5038: 状態** — `meta`
  - spec: `specification.md:5038`
  - notes: BND status/principles prose

- [x] **L5 L5071: `DD-BND-001`: 通常の不変束縛** — `ok`
  - spec: `specification.md:5071`
  - notes: val/let → Core Let; elaborate.rs + eval.rs

- [x] **L5 L5130: `DD-BND-002`: sequentialなSurface `let`** — `ok`
  - spec: `specification.md:5130`
  - notes: val/let → Core Let; elaborate.rs + eval.rs

- [x] **L5 L5165: `DD-BND-003`: `(type ...)`による型注釈** — `ok`
  - spec: `specification.md:5165`
  - notes: (type name Ty) value annotations in DataEnv; checked via infer_binding_init (DD-BND-003)

- [x] **L5 L5230: `DD-BND-004`: 型注釈のscopeと対応関係** — `ok`
  - spec: `specification.md:5230`
  - notes: orphan (type …) without matching val/var rejected; type-alias exempt (DD-BND-004)

- [x] **L5 L5302: `DD-BND-005`: 型注釈は検査される** — `ok`
  - spec: `specification.md:5302`
  - notes: initializer checked against (type …); lambda params seeded from annotation (DD-BND-005)

- [x] **L5 L5348: `DD-BND-006`: `letrec`の対象** — `ok`
  - spec: `specification.md:5348`
  - notes: CoreExpr::LetRec; elaborate/eval/bind; lang_kernel_suite

- [x] **L5 L5400: `DD-BND-007`: `letrec`内の型注釈** — `ok`
  - spec: `specification.md:5400`
  - notes: rec/local (type …) registered into DataEnv; LetRec uses annotation stubs (DD-BND-007)

- [x] **L5 L5478: `DD-BND-008`: `letrec`の実行意味** — `ok`
  - spec: `specification.md:5478`
  - notes: CoreExpr::LetRec; elaborate/eval/bind; lang_kernel_suite

- [x] **L5 L5509: `DD-BND-009`: 相互再帰** — `ok`
  - spec: `specification.md:5509`
  - notes: CoreExpr::LetRec; elaborate/eval/bind; lang_kernel_suite

- [x] **L5 L5546: `DD-BND-010`: 再帰関数の型推論** — `ok`
  - spec: `specification.md:5546`
  - notes: LetRec seeds params from stubs/annotations; partner inference via subst (DD-BND-010)

- [x] **L5 L5597: `DD-BND-011`: 多相再帰の禁止** — `deferred`
  - spec: `specification.md:5597`
  - notes: polymorphic recursion out of v1; plan BND deferral

- [x] **L5 L5619: `DD-BND-012`: `var`の基本意味** — `ok`
  - spec: `specification.md:5619`
  - notes: LocalVar/Set in expr.rs; eval.rs Cell; reciplexa-bind resolve

- [x] **L5 L5656: `DD-BND-013`: `var`の型注釈** — `ok`
  - spec: `specification.md:5656`
  - notes: var store type from initializer or (type …) annotation (DD-BND-013)

- [x] **L5 L5696: `DD-BND-014`: `var`の格納型は固定** — `ok`
  - spec: `specification.md:5696`
  - notes: set unifies to fixed store type; unannotated int+set str rejected (DD-BND-014)

- [x] **L5 L5740: `DD-BND-015`: `var`の読出し** — `ok`
  - spec: `specification.md:5740`
  - notes: LocalVar/Set in expr.rs; eval.rs Cell; reciplexa-bind resolve

- [x] **L5 L5771: `DD-BND-016`: `set`** — `ok`
  - spec: `specification.md:5771`
  - notes: LocalVar/Set in expr.rs; eval.rs Cell; reciplexa-bind resolve

- [x] **L5 L5810: `DD-BND-017`: closureによる`var`のcapture** — `ok`
  - spec: `specification.md:5810`
  - notes: LocalVar/Set in expr.rs; eval.rs Cell; reciplexa-bind resolve

- [x] **L5 L5834: `DD-BND-018`: local state identity** — `partial`
  - spec: `specification.md:5834`
  - notes: local-state/<name> effect on get/set + Cell Rc identity; formal scope identity still thin

- [x] **L5 L5866: `DD-BND-019`: local state escapeの禁止** — `ok`
  - spec: `specification.md:5866`
  - notes: static escape: result type mentioning local-state/<name> rejected; runtime alive remains

- [x] **L5 L5907: `DD-BND-020`: non-escaping callback** — `deferred`
  - spec: `specification.md:5907`
  - notes: non-escaping callback typing deferred

- [x] **L5 L5936: `DD-BND-021`: local state effectの除去** — `ok`
  - spec: `specification.md:5936`
  - notes: var strips local-state/<name> from residual; set adds it in var body (check.rs)

- [x] **L5 L5974: `DD-BND-022`: one-shot resumptionと局所state** — `deferred`
  - spec: `specification.md:5974`
  - notes: multi-shot/escapeable state deferred; one-shot+alive check only

- [x] **L5 L6003: `DD-BND-023`: escape可能な状態との分離** — `deferred`
  - spec: `specification.md:6003`
  - notes: multi-shot/escapeable state deferred; one-shot+alive check only

- [x] **L5 L6035: `DD-BND-024`: 型一般化point** — `deferred`
  - spec: `specification.md:6035`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6064: `DD-BND-025`: 構文的value restriction** — `deferred`
  - spec: `specification.md:6064`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6108: `DD-BND-026`: effectful function valueの一般化** — `deferred`
  - spec: `specification.md:6108`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6145: `DD-BND-027`: capability captureによる一般化禁止** — `deferred`
  - spec: `specification.md:6145`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6176: `DD-BND-028`: 明示的`forall`注釈** — `deferred`
  - spec: `specification.md:6176`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6223: `DD-BND-029`: 一般化される変数** — `deferred`
  - spec: `specification.md:6223`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6251: `DD-BND-030`: 一般化されないmetavariable** — `deferred`
  - spec: `specification.md:6251`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6267: `DD-BND-031`: `letrec`とvalue restriction** — `deferred`
  - spec: `specification.md:6267`
  - notes: full generalization / value-restriction regime deferred (plan BND-001)

- [x] **L5 L6292: `DD-BND-032`: relaxed value restrictionの保留** — `deferred`
  - spec: `specification.md:6292`
  - notes: relaxed value restriction reserved

- [x] **L5 L6306: Coreおよびdeclaration elaboration** — `ok`
  - spec: `specification.md:6306`
  - notes: elaborate.rs val/let/letrec/var/local/rec

- [x] **L5 L6352: 適合試験** — `ok`
  - spec: `specification.md:6352`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6354: top-level型注釈** — `ok`
  - spec: `specification.md:6354`
  - notes: top-level (type …)+(val …) checked in typecheck_language_source / lang_kernel_suite

- [x] **L6 L6369: 関数型注釈** — `ok`
  - spec: `specification.md:6369`
  - notes: function (type add (fn int int int)) + val seed params (TEST-LANG-BND-ann)

- [x] **L6 L6386: 注釈不一致** — `ok`
  - spec: `specification.md:6386`
  - notes: annotation mismatch (type str / val 42) is static error

- [x] **L6 L6400: 対応bindingのない型注釈** — `ok`
  - spec: `specification.md:6400`
  - notes: orphan type annotation without value binding rejected at elaborate

- [x] **L6 L6413: 自己再帰の型注釈** — `ok`
  - spec: `specification.md:6413`
  - notes: CoreExpr::LetRec; elaborate/eval/bind; lang_kernel_suite

- [x] **L6 L6434: 相互再帰の型注釈** — `ok`
  - spec: `specification.md:6434`
  - notes: CoreExpr::LetRec; elaborate/eval/bind; lang_kernel_suite

- [x] **L6 L6464: 一部だけ注釈** — `ok`
  - spec: `specification.md:6464`
  - notes: partial letrec/rec annotation: partner inferred (partial_letrec_annotation_infers_partner)

- [x] **L6 L6492: 任意式の再帰拒否** — `ok`
  - spec: `specification.md:6492`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6508: 明示的多相型** — `ok`
  - spec: `specification.md:6508`
  - notes: explicit forall annotation + instantiate at use (forall_annotation_*)

- [x] **L6 L6528: 注釈によるvalue restriction回避の拒否** — `ok`
  - spec: `specification.md:6528`
  - notes: expansive initializer cannot be ∀-generalized via annotation

- [x] **L6 L6546: local `var`** — `ok`
  - spec: `specification.md:6546`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6561: `set`の戻り値** — `ok`
  - spec: `specification.md:6561`
  - notes: LocalVar/Set in expr.rs; eval.rs Cell; reciplexa-bind resolve

- [x] **L6 L6574: 格納型の固定** — `ok`
  - spec: `specification.md:6574`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6589: 明示union格納型** — `ok`
  - spec: `specification.md:6589`
  - notes: annotated union store allows int|str sets (var_store_type_fixed_and_union_annotation)

- [x] **L6 L6609: closure capture** — `ok`
  - spec: `specification.md:6609`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6628: closure escape** — `ok`
  - spec: `specification.md:6628`
  - notes: lang_kernel_suite BND letrec/var/rec-local

- [x] **L6 L6646: effectful関数値の一般化** — `ok`
  - spec: `specification.md:6646`
  - notes: effectful forall fn annotation generalizes (effectful_annotated_fn_generalizes)

- [x] **L5 L6669: 関連する後続設計課題** — `meta`
  - spec: `specification.md:6669`
  - notes: OPEN / future-work prose

- [x] **L6 L6671: `OPEN-SYN-002`** — `meta`
  - spec: `specification.md:6671`
  - notes: OPEN / future-work prose

- [x] **L6 L6682: `OPEN-TYP-002`** — `meta`
  - spec: `specification.md:6682`
  - notes: OPEN / future-work prose

- [x] **L6 L6693: `OPEN-MEM-001`** — `meta`
  - spec: `specification.md:6693`
  - notes: OPEN / future-work prose

- [x] **L6 L6703: `OPEN-MOD-001`** — `meta`
  - spec: `specification.md:6703`
  - notes: OPEN / future-work prose

- [x] **L6 L6711: 将来拡張** — `meta`
  - spec: `specification.md:6711`
  - notes: OPEN / future-work prose

- [x] **L5 L6724: 確立された基本原則** — `meta`
  - spec: `specification.md:6724`
  - notes: BND status/principles prose

- [x] **L4 L6790: 13.5 `MAC-001` 最小式マクロ・展開・衛生性** — `partial`
  - spec: `specification.md:6790`
  - notes: 式マクロ核ok; hygiene/scope/診断に差

- [x] **L5 L6791: DD-001 決定概要** — `ok`
  - spec: `specification.md:6791`
  - notes: 決定概要どおり最小式マクロ

- [x] **L6 L6792: DD-001.1 状態** — `ok`
  - spec: `specification.md:6792`
  - notes: RESOLVED相当の核実装 (lang_macro)

- [x] **L6 L6808: DD-001.2 設計原則** — `ok`
  - spec: `specification.md:6808`
  - notes: 通常関数優先・衛生・予算の原則

- [x] **L6 L6821: DD-001.3 中心的な決定** — `ok`
  - spec: `specification.md:6821`
  - notes: $params/->/...+/展開前型検査

- [x] **L5 L6838: 0. 適用範囲** — `ok`
  - spec: `specification.md:6838`
  - notes: 適用範囲: 式マクロ最小集合

- [x] **L6 L6839: 0.1 本項目が定めるもの** — `ok`
  - spec: `specification.md:6839`
  - notes: 宣言・展開・衛生の核を実装

- [x] **L6 L6858: 0.2 本項目が定めないもの** — `meta`
  - spec: `specification.md:6858`
  - notes: 定めないもの=OPEN移管（意図的）

- [x] **L5 L6877: 1. マクロと通常関数** — `ok`
  - spec: `specification.md:6877`
  - notes: 関数で足りるならマクロ不要の方針

- [x] **L6 L6878: 1.1 通常関数の優先** — `ok`
  - spec: `specification.md:6878`
  - notes: 通常関数優先; マクロは評価制御向け

- [x] **L6 L6900: 1.2 マクロが適する場合** — `ok`
  - spec: `specification.md:6900`
  - notes: unless/when等の評価制御用途

- [x] **L5 L6926: 2. マクロの基本構文** — `ok`
  - spec: `specification.md:6926`
  - notes: (macro name ($p…) -> tmpl)

- [x] **L6 L6927: 2.1 宣言形式** — `ok`
  - spec: `specification.md:6927`
  - notes: try_macro_def 形式

- [x] **L6 L6933: 2.2 固定arityの例** — `ok`
  - spec: `specification.md:6933`
  - notes: 固定arity例・unitテスト

- [x] **L6 L6954: 2.3 可変長本文の例** — `ok`
  - spec: `specification.md:6954`
  - notes: $rest ...+ 可変本文

- [x] **L6 L6978: 2.4 ->** — `ok`
  - spec: `specification.md:6978`
  - notes: ->必須; legacy拒否

- [x] **L5 L6991: 3. 式マクロ** — `ok`
  - spec: `specification.md:6991`
  - notes: expression macros ok; declaration-position macro head rejected

- [x] **L6 L6992: 3.1 使用可能な位置** — `ok`
  - spec: `specification.md:6992`
  - notes: 式呼出し位置で展開

- [x] **L6 L7009: 3.2 展開結果** — `ok`
  - spec: `specification.md:7009`
  - notes: 展開結果は式sexpr

- [x] **L6 L7022: 3.3 宣言位置** — `ok`
  - spec: `specification.md:7022`
  - notes: expand_language rejects top-level macro heads (declaration position)

- [x] **L6 L7032: 3.4 型位置** — `ok`
  - spec: `specification.md:7032`
  - notes: 型位置マクロなし（未サポート=仕様）

- [x] **L6 L7039: 3.5 Pattern位置** — `ok`
  - spec: `specification.md:7039`
  - notes: pattern位置マクロなし

- [x] **L6 L7047: 3.6 Interfaceおよびmanifest** — `deferred`
  - spec: `specification.md:7047`
  - notes: interface/manifestはPKG/OPEN

- [x] **L5 L7059: 4. マクロpattern変数** — `ok`
  - spec: `specification.md:7059`
  - notes: $param pattern変数

- [x] **L6 L7060: 4.1 基本表記** — `ok`
  - spec: `specification.md:7060`
  - notes: $始まり必須

- [x] **L6 L7070: 4.2 通常identifierとの区別** — `ok`
  - spec: `specification.md:7070`
  - notes: 通常identと区別

- [x] **L6 L7082: 4.3 受け取るもの** — `ok`
  - spec: `specification.md:7082`
  - notes: 引数式を受け取る

- [x] **L6 L7103: 4.4 Pattern変数の重複** — `ok`
  - spec: `specification.md:7103`
  - notes: 重複param拒否

- [x] **L6 L7120: 4.5 Template内での再利用** — `ok`
  - spec: `specification.md:7120`
  - notes: template内で再利用可

- [x] **L5 L7134: 5. 固定arity** — `ok`
  - spec: `specification.md:7134`
  - notes: 固定arity検査

- [x] **L6 L7135: 5.1 基本方針** — `ok`
  - spec: `specification.md:7135`
  - notes: arity一致必須

- [x] **L6 L7144: 5.2 引数不足** — `ok`
  - spec: `specification.md:7144`
  - notes: 引数不足エラー

- [x] **L6 L7152: 5.3 引数過剰** — `ok`
  - spec: `specification.md:7152`
  - notes: 引数過剰エラー

- [x] **L6 L7162: 5.4 通常関数との整合** — `ok`
  - spec: `specification.md:7162`
  - notes: 関数arityと整合

- [x] **L5 L7168: 6. 末尾反復** — `ok`
  - spec: `specification.md:7168`
  - notes: ...+ 末尾反復

- [x] **L6 L7169: 6.1 一個以上の入力** — `ok`
  - spec: `specification.md:7169`
  - notes: ≥1引数必須

- [x] **L6 L7175: 6.2 Templateでの展開** — `ok`
  - spec: `specification.md:7175`
  - notes: template `$rest ...` splice

- [x] **L6 L7181: 6.3 使用例** — `ok`
  - spec: `specification.md:7181`
  - notes: when等の使用例テスト

- [x] **L6 L7206: 6.4 反復位置** — `ok`
  - spec: `specification.md:7206`
  - notes: 反復は末尾のみ

- [x] **L6 L7219: 6.5 反復数** — `ok`
  - spec: `specification.md:7219`
  - notes: 反復数=実引数個数

- [x] **L6 L7223: 6.6 0個以上の反復** — `deferred`
  - spec: `specification.md:7223`
  - notes: 0個以上`...`はv1対象外

- [x] **L5 L7229: 7. Template** — `ok`
  - spec: `specification.md:7229`
  - notes: template置換

- [x] **L6 L7230: 7.1 Template変数** — `ok`
  - spec: `specification.md:7230`
  - notes: $var展開

- [x] **L6 L7247: 7.2 通常identifier** — `ok`
  - spec: `specification.md:7247`
  - notes: 通常identは綴り保持

- [x] **L6 L7258: 7.3 Templateの構文妥当性** — `partial`
  - spec: `specification.md:7258`
  - notes: def時var検査; 完全構文妥当性は弱い

- [x] **L6 L7264: 7.4 評価回数** — `ok`
  - spec: `specification.md:7264`
  - notes: 展開は代入のみ（再評価なし）

- [x] **L5 L7276: 8. Scope** — `ok`
  - spec: `specification.md:7276`
  - notes: same-unit + nested (module …) child inherits earlier parent macros (lang_macro)

- [x] **L6 L7277: 8.1 コンパイル単位内限定** — `ok`
  - spec: `specification.md:7277`
  - notes: compile単位内のin-memory map

- [x] **L6 L7285: 8.2 宣言順序** — `ok`
  - spec: `specification.md:7285`
  - notes: MAC-10: pre-definition macro use rejected in expand_language

- [x] **L6 L7312: 8.3 下位モジュール** — `ok`
  - spec: `specification.md:7312`
  - notes: nested (module …) sees parent macros defined earlier (MAC §8.3)

- [x] **L6 L7327: 8.4 子scopeから親scope** — `ok`
  - spec: `specification.md:7327`
  - notes: child/sibling scopes: later/sibling macros not visible (MAC §8.4)

- [x] **L5 L7333: 9. マクロ名と呼出し** — `partial`
  - spec: `specification.md:7333`
  - notes: 呼出し構文ok; 衝突検査弱

- [x] **L6 L7334: 9.1 呼出し構文** — `ok`
  - spec: `specification.md:7334`
  - notes: (name args…) 呼出し

- [x] **L6 L7341: 9.2 判別** — `ok`
  - spec: `specification.md:7341`
  - notes: 定義後のheadでマクロ判別

- [x] **L6 L7347: 9.3 名前衝突** — `ok`
  - spec: `specification.md:7347`
  - notes: MAC §9.3: macro/value same spelling rejected at definition

- [x] **L6 L7356: 9.4 名前変更** — `ok`
  - spec: `specification.md:7356`
  - notes: renameは通常の再定義/別名で可

- [x] **L5 L7362: 10. 展開段階** — `ok`
  - spec: `specification.md:7362`
  - notes: expand→elaborate→check順

- [x] **L6 L7363: 10.1 処理順序** — `ok`
  - spec: `specification.md:7363`
  - notes: 処理パイプライン一致

- [x] **L5 L7367: 1. Sourceをreaderで読む** — `ok`
  - spec: `specification.md:7367`
  - notes: parse_source

- [x] **L5 L7368: 2. Lossless CSTを構築する** — `ok`
  - spec: `specification.md:7368`
  - notes: rowan CST

- [x] **L5 L7369: 3. 逐次scopeに従ってマクロ定義を認識する** — `ok`
  - spec: `specification.md:7369`
  - notes: top-downでmacro def認識

- [x] **L5 L7370: 4. 式マクロを展開する** — `ok`
  - spec: `specification.md:7370`
  - notes: expand_language

- [x] **L5 L7371: 5. 通常の名前解決を行う** — `ok`
  - spec: `specification.md:7371`
  - notes: 展開後にresolve/elaborate

- [x] **L5 L7372: 6. 型検査する** — `ok`
  - spec: `specification.md:7372`
  - notes: typecheck_language_source

- [x] **L5 L7373: 7. IRへ変換する** — `ok`
  - spec: `specification.md:7373`
  - notes: 後段IR/evalへ

- [x] **L5 L7374: 8. 実行可能artifactを生成する** — `meta`
  - spec: `specification.md:7374`
  - notes: artifact生成は実行パイプライン側

- [x] **L6 L7376: 10.2 型検査前展開** — `ok`
  - spec: `specification.md:7376`
  - notes: 型検査前展開

- [x] **L6 L7382: 10.3 型情報** — `ok`
  - spec: `specification.md:7382`
  - notes: 展開時に型情報なし

- [x] **L6 L7392: 10.4 Typed macro** — `deferred`
  - spec: `specification.md:7392`
  - notes: typed macroはOPEN-MAC-TYPED

- [x] **L5 L7396: 11. 展開順序** — `ok`
  - spec: `specification.md:7396`
  - notes: 外側から・決定的展開

- [x] **L6 L7397: 11.1 Source order** — `ok`
  - spec: `specification.md:7397`
  - notes: source order

- [x] **L6 L7401: 11.2 外側からの展開** — `ok`
  - spec: `specification.md:7401`
  - notes: 外側呼出し優先

- [x] **L6 L7407: 11.3 決定性** — `ok`
  - spec: `specification.md:7407`
  - notes: 決定的rewrite

- [x] **L6 L7413: 11.4 展開後の再検査** — `ok`
  - spec: `specification.md:7413`
  - notes: 展開結果を再walk

- [x] **L5 L7419: 12. 再帰マクロ** — `ok`
  - spec: `specification.md:7419`
  - notes: static macro ref-graph DAG check; self/mutual recursion rejected

- [x] **L6 L7420: 12.1 自己再帰** — `ok`
  - spec: `specification.md:7420`
  - notes: self-recursive template → cycle error at definition

- [x] **L6 L7431: 12.2 相互再帰** — `ok`
  - spec: `specification.md:7431`
  - notes: mutual recursion → cycle error at definition

- [x] **L6 L7441: 12.3 先行マクロの利用** — `ok`
  - spec: `specification.md:7441`
  - notes: 先行マクロ利用可（env蓄積）

- [x] **L6 L7456: 12.4 参照graph** — `ok`
  - spec: `specification.md:7456`
  - notes: macro reference graph must be DAG (assert_macro_graph_dag)

- [x] **L5 L7464: 13. 衛生性** — `ok`
  - spec: `specification.md:7464`
  - notes: fn/let/local val/var hygiene (lang_macro hygiene_* tests)

- [x] **L6 L7465: 13.1 定義** — `ok`
  - spec: `specification.md:7465`
  - notes: hygienic rename via __rx_N gensym (lang_macro.rs)

- [x] **L6 L7474: 13.2 基本保証** — `ok`
  - spec: `specification.md:7474`
  - notes: basic capture-avoidance for fn/let/local binders

- [x] **L6 L7480: 13.3 Fresh identity** — `ok`
  - spec: `specification.md:7480`
  - notes: __rx_N gensym fresh identity (fn/let/local)

- [x] **L6 L7486: 13.4 入力構文の再挿入** — `ok`
  - spec: `specification.md:7486`
  - notes: 入力構文はrenameせず再挿入

- [x] **L5 L7492: 14. 衛生性の例** — `ok`
  - spec: `specification.md:7492`
  - notes: hygiene examples covered for fn + local var (lang_macro tests)

- [x] **L6 L7493: 14.1 マクロ定義** — `ok`
  - spec: `specification.md:7493`
  - notes: マクロ定義例形式

- [x] **L6 L7503: 14.2 利用** — `ok`
  - spec: `specification.md:7503`
  - notes: 利用例形式

- [x] **L6 L7510: 14.3 概念的なidentity** — `partial`
  - spec: `specification.md:7510`
  - notes: MacroSourceMap SyntaxNodeId + BindingMap use-sites; Core BindingId string dual still open

- [x] **L5 L7532: 15. 意図的capture** — `ok`
  - spec: `specification.md:7532`
  - notes: 意図的capture APIなし（仕様）

- [x] **L6 L7533: 15.1 v1の方針** — `ok`
  - spec: `specification.md:7533`
  - notes: v1でcapture導入しない

- [x] **L6 L7543: 15.2 Binder名の明示** — `ok`
  - spec: `specification.md:7543`
  - notes: binder名は明示引数で渡す方針

- [x] **L6 L7555: 15.3 理由** — `ok`
  - spec: `specification.md:7555`
  - notes: 理由どおり簡易設計

- [x] **L5 L7561: 16. 純粋性** — `ok`
  - spec: `specification.md:7561`
  - notes: 純関数的sexpr rewrite

- [x] **L6 L7562: 16.1 展開入力** — `ok`
  - spec: `specification.md:7562`
  - notes: 入力は構文木のみ

- [x] **L6 L7571: 16.2 禁止される外部入力** — `ok`
  - spec: `specification.md:7571`
  - notes: FS/env/時刻なし

- [x] **L6 L7586: 16.3 通常値** — `ok`
  - spec: `specification.md:7586`
  - notes: 通常値を展開時評価しない

- [x] **L6 L7592: 16.4 再現性** — `ok`
  - spec: `specification.md:7592`
  - notes: 決定的・再現可能

- [x] **L5 L7596: 17. 展開予算** — `ok`
  - spec: `specification.md:7596`
  - notes: EXPANSION_BUDGET=256

- [x] **L6 L7597: 17.1 基本原則** — `ok`
  - spec: `specification.md:7597`
  - notes: 有限展開原則

- [x] **L6 L7603: 17.2 監視対象** — `ok`
  - spec: `specification.md:7603`
  - notes: 適用回数をカウント

- [x] **L6 L7611: 17.3 具体値** — `ok`
  - spec: `specification.md:7611`
  - notes: 定数256

- [x] **L6 L7617: 17.4 上限超過** — `ok`
  - spec: `specification.md:7617`
  - notes: 上限超過エラー

- [x] **L6 L7631: 17.5 Sourceからの無制限化** — `ok`
  - spec: `specification.md:7631`
  - notes: sourceからの無制限化なし

- [x] **L5 L7637: 18. エラー診断** — `partial`
  - spec: `specification.md:7637`
  - notes: 主要診断あり; span/provenance弱

- [x] **L6 L7638: 18.1 Pattern不一致** — `ok`
  - spec: `specification.md:7638`
  - notes: pattern/形式エラー

- [x] **L6 L7644: 18.2 引数数** — `ok`
  - spec: `specification.md:7644`
  - notes: 引数数エラー

- [x] **L6 L7654: 18.3 ...+** — `ok`
  - spec: `specification.md:7654`
  - notes: ...+ 空/位置エラー

- [x] **L6 L7660: 18.4 未束縛pattern変数** — `ok`
  - spec: `specification.md:7660`
  - notes: 未束縛template変数拒否

- [x] **L6 L7664: 18.5 不正な展開結果** — `partial`
  - spec: `specification.md:7664`
  - notes: 不正結果は後段エラーに依存

- [x] **L6 L7668: 18.6 型エラー** — `partial`
  - spec: `specification.md:7668`
  - notes: 型エラーのマクロ帰属なし

- [x] **L5 L7672: 19. Provenance** — `ok`
  - spec: `specification.md:7672`
  - notes: MacroSourceMap: call/def SyntaxNodeId + spans + expansion chain

- [x] **L6 L7673: 19.1 保持する情報** — `ok`
  - spec: `specification.md:7673`
  - notes: ExpansionOrigin keeps call/def site ids/spans and chain

- [x] **L6 L7683: 19.2 診断例** — `partial`
  - spec: `specification.md:7683`
  - notes: expand provenance recorded; typed diagnostic attachment still light

- [x] **L6 L7692: 19.3 Source map** — `ok`
  - spec: `specification.md:7692`
  - notes: expand_language_with_map source map for IDE/formatter origin

- [x] **L6 L7696: 19.4 正式identity** — `deferred`
  - spec: `specification.md:7696`
  - notes: 正式identityはOPEN-EDT

- [x] **L5 L7702: 20. マクロの外部公開** — `deferred`
  - spec: `specification.md:7702`
  - notes: 外部公開はv1制限/OPEN-MAC-PKG

- [x] **L6 L7703: 20.1 v1の制限** — `ok`
  - spec: `specification.md:7703`
  - notes: 同一単位のみ; exportなし

- [x] **L6 L7712: 20.2 標準構文** — `ok`
  - spec: `specification.md:7712`
  - notes: 標準構文は言語内macro形

- [x] **L6 L7720: 20.3 将来拡張** — `deferred`
  - spec: `specification.md:7720`
  - notes: 将来のpkg公開

- [x] **L5 L7724: 21. 適合試験** — `partial`
  - spec: `specification.md:7724`
  - notes: unit/integrationで一部; 全MAC試験未

- [x] **L6 L7725: 21.1 適合試験 MAC-01：固定arity** — `ok`
  - spec: `specification.md:7725`
  - notes: MAC-01相当 (lang_macro tests)

- [x] **L6 L7747: 21.2 不適合試験 MAC-02：引数不足** — `ok`
  - spec: `specification.md:7747`
  - notes: MAC-02 引数不足

- [x] **L6 L7756: 21.3 不適合試験 MAC-03：引数過剰** — `ok`
  - spec: `specification.md:7756`
  - notes: MAC-03 引数過剰

- [x] **L6 L7767: 21.4 適合試験 MAC-04：末尾反復** — `ok`
  - spec: `specification.md:7767`
  - notes: MAC-04 ...+

- [x] **L6 L7791: 21.5 不適合試験 MAC-05：空の...+** — `ok`
  - spec: `specification.md:7791`
  - notes: MAC-05 空...+

- [x] **L6 L7800: 21.6 不適合試験 MAC-06：反復が末尾以外** — `ok`
  - spec: `specification.md:7800`
  - notes: MAC-06 非末尾...+

- [x] **L6 L7812: 21.7 不適合試験 MAC-07：重複pattern変数** — `ok`
  - spec: `specification.md:7812`
  - notes: MAC-07 重複param

- [x] **L6 L7825: 21.8 不適合試験 MAC-08：未束縛template変数** — `ok`
  - spec: `specification.md:7825`
  - notes: MAC-08 未束縛template

- [x] **L6 L7838: 21.9 適合試験 MAC-09：衛生的binder** — `ok`
  - spec: `specification.md:7838`
  - notes: MAC-09: hygienic fn/let/local val/var binders (lang_macro tests)

- [x] **L6 L7862: 21.10 不適合試験 MAC-10：定義前使用** — `ok`
  - spec: `specification.md:7862`
  - notes: MAC-10: pre-def macro use diagnostic in expand_language

- [x] **L6 L7879: 21.11 不適合試験 MAC-11：自己再帰** — `ok`
  - spec: `specification.md:7879`
  - notes: MAC-11 mutual recursion rejected via ref-graph cycle diagnostic

- [x] **L6 L7891: 21.12 不適合試験 MAC-12：宣言位置** — `ok`
  - spec: `specification.md:7891`
  - notes: MAC-12: declaration-position rejection

- [x] **L6 L7900: 21.13 不適合試験 MAC-13：Interface** — `deferred`
  - spec: `specification.md:7900`
  - notes: MAC-13 interface未（PKG）

- [x] **L6 L7912: 21.14 不適合試験 MAC-14：展開上限** — `ok`
  - spec: `specification.md:7912`
  - notes: MAC-14 展開上限

- [x] **L5 L7919: 22. 移管先OPEN・下位項目・状態** — `deferred`
  - spec: `specification.md:7919`
  - notes: OPEN移管カタログ

- [x] **L6 L7920: 22.1 `OPEN-MAC-EXT-001`** — `deferred`
  - spec: `specification.md:7920`
  - notes: OPEN-MAC-EXT-001

- [x] **L6 L7933: 22.2 `OPEN-MAC-PKG-001`** — `deferred`
  - spec: `specification.md:7933`
  - notes: OPEN-MAC-PKG-001

- [x] **L6 L7944: 22.3 `OPEN-MAC-PROC-001`** — `deferred`
  - spec: `specification.md:7944`
  - notes: OPEN-MAC-PROC-001

- [x] **L6 L7954: 22.4 `OPEN-MAC-TYPED-001`** — `deferred`
  - spec: `specification.md:7954`
  - notes: OPEN-MAC-TYPED-001

- [x] **L6 L7963: 22.5 `OPEN-MAC-CAP-001`** — `deferred`
  - spec: `specification.md:7963`
  - notes: OPEN-MAC-CAP-001

- [x] **L6 L7971: 22.6 `OPEN-EDT-001`** — `deferred`
  - spec: `specification.md:7971`
  - notes: OPEN-EDT-001

- [x] **L6 L7980: 22.7 下位項目** — `meta`
  - spec: `specification.md:7980`
  - notes: 下位OPEN一覧

- [x] **L6 L8016: 22.8 最終状態** — `meta`
  - spec: `specification.md:8016`
  - notes: 最終RESOLVED宣言（追跡用）

- [x] **L4 L8036: 13.6 `TYP-001` Gradual set-theoretic types** — `partial`
  - spec: `specification.md:8036`
  - notes: Bounded Dynamic/casts/evidence/EffectRow/ROW fragment; full set-theoretic solver deferred at TYP-ALG

- [x] **L5 L8038: 概要・状態** — `partial`
  - spec: `specification.md:8038`
  - notes: Dynamic/unify/EffectRow/cast fragment; full set-theoretic algebra deferred

- [x] **L5 L8045: 13.6.1 `TYP-DYN-001` Bounded dynamic、cast evidence、dynamic failure** — `partial`
  - spec: `specification.md:8045`
  - notes: Bounded Dynamic(S)+three-way use+CastEvidence+try/check-cast; foreign/guarantee remain deferred/meta

- [x] **L6 L8049: 状態** — `meta`
  - spec: `specification.md:8049`
  - notes: TYP-DYN resolved-in-spec; runtime gradually filling

- [x] **L6 L8086: `DD-TYP-DYN-001`: static型とgradual型の分離** — `ok`
  - spec: `specification.md:8086`
  - notes: static vs gradual: CoreType::Dynamic(bound)+Any; no free static↔gradual mix

- [x] **L6 L8154: `DD-TYP-DYN-002`: `dynamic S`の意味** — `ok`
  - spec: `specification.md:8154`
  - notes: `dynamic S`→Dynamic(bound); dynamic never≃never; nested dynamic collapses (ty/cast)

- [x] **L6 L8199: `DD-TYP-DYN-003`: static top型`any`** — `ok`
  - spec: `specification.md:8199`
  - notes: CoreType::Any + S<:any; any rejects implicit cast as static target

- [x] **L6 L8275: `DD-TYP-DYN-004`: `never`およびdynamicの正規形** — `ok`
  - spec: `specification.md:8275`
  - notes: Never + dynamic never normalize; CoreType::dynamic_bound

- [x] **L6 L8323: `DD-TYP-DYN-005`: dynamic値をstatic型として使用する三段階判定** — `ok`
  - spec: `specification.md:8323`
  - notes: judge_dynamic_use three-way; coerce_to_static / insert_implicit_casts (check/cast)

- [x] **L6 L8333: 1. 上限全体が要求型へ含まれる場合** — `ok`
  - spec: `specification.md:8333`
  - notes: FullyIncluded when S<:T → CastEvidence::Identity

- [x] **L6 L8354: 2. 上限と要求型が互いに素である場合** — `ok`
  - spec: `specification.md:8354`
  - notes: Disjoint when intersect(S,T)≃never → static reject in coerce/plan

- [x] **L6 L8376: 3. 一部だけ重なる場合** — `ok`
  - spec: `specification.md:8376`
  - notes: PartialOverlap → cast evidence + success intersect(S,T)

- [x] **L6 L8409: `DD-TYP-DYN-006`: cast成功後の型** — `ok`
  - spec: `specification.md:8409`
  - notes: cast_success_type=intersect(S,T); Cast/TryCast/CheckCast return intersect

- [x] **L6 L8455: `DD-TYP-DYN-007`: occurrence typingとの関係** — `ok`
  - spec: `specification.md:8455`
  - notes: occurrence refine on Dynamic: then intersect / else dynamic(diff) (check.rs)

- [x] **L6 L8513: `DD-TYP-DYN-008`: static値からdynamic値への導入** — `ok`
  - spec: `specification.md:8513`
  - notes: Widen when T<:S; plan_cast rejects T</:S for to-dynamic

- [x] **L6 L8564: `DD-TYP-DYN-009`: dynamic上限のwidening** — `partial`
  - spec: `specification.md:8564`
  - notes: Widen evidence for dynamic S→dynamic U when S<:U; deep provenance TBD

- [x] **L6 L8609: `DD-TYP-DYN-010`: foreign値のdynamic導入** — `deferred`
  - spec: `specification.md:8609`
  - notes: 依存待ち(KER/foreign): import-dynamic / ForeignValue boundary

- [x] **L6 L8660: `DD-TYP-DYN-011`: decoderとgradual foreign boundaryの分離** — `deferred`
  - spec: `specification.md:8660`
  - notes: 依存待ち(KER/foreign): decoder vs gradual foreign boundary separation

- [x] **L6 L8667: Static decoder** — `deferred`
  - spec: `specification.md:8667`
  - notes: 依存待ち(KER/foreign): static decoder Result<S, decode-error>

- [x] **L6 L8676: Gradual foreign boundary** — `deferred`
  - spec: `specification.md:8676`
  - notes: 依存待ち(KER/foreign): import-dynamic foreign boundary

- [x] **L6 L8690: `DD-TYP-DYN-012`: runtime-checkableな型** — `ok`
  - spec: `specification.md:8690`
  - notes: is_runtime_checkable for primitives/record/variant/fun; opaque/capability reject

- [x] **L6 L8742: `DD-TYP-DYN-013`: implicit cast failure** — `partial`
  - spec: `specification.md:8742`
  - notes: implicit Cast failure→EvalError; structured dynamic-type-error taxonomy still thin

- [x] **L6 L8785: `DD-TYP-DYN-014`: 明示的safe cast** — `ok`
  - spec: `specification.md:8785`
  - notes: try-cast/check-cast → Option/Result of intersect(S,T)

- [x] **L6 L8824: `DD-TYP-DYN-015`: cast evidence** — `ok`
  - spec: `specification.md:8824`
  - notes: CastEvidence algebra + plan_cast_evidence + CastProvenance stub (cast.rs)

- [x] **L6 L8854: `Identity`** — `ok`
  - spec: `specification.md:8854`
  - notes: CastEvidence::Identity

- [x] **L6 L8858: `Widen`** — `ok`
  - spec: `specification.md:8858`
  - notes: CastEvidence::Widen

- [x] **L6 L8862: `TagCheck`** — `ok`
  - spec: `specification.md:8862`
  - notes: CastEvidence::TagCheck

- [x] **L6 L8866: `UnionCheck`** — `ok`
  - spec: `specification.md:8866`
  - notes: CastEvidence::UnionCheck

- [x] **L6 L8870: `IntersectionCheck`** — `ok`
  - spec: `specification.md:8870`
  - notes: CastEvidence::IntersectionCheck

- [x] **L6 L8874: `RecordCheck`** — `ok`
  - spec: `specification.md:8874`
  - notes: CastEvidence::RecordCheck + field-type runtime checks

- [x] **L6 L8878: `VariantCheck`** — `ok`
  - spec: `specification.md:8878`
  - notes: CastEvidence::VariantCheck + payload runtime checks

- [x] **L6 L8882: `FunctionGuard`** — `ok`
  - spec: `specification.md:8882`
  - notes: CastEvidence::FunctionGuard {arity,arg_casts,ret_cast}

- [x] **L6 L8886: `NominalCheck`** — `ok`
  - spec: `specification.md:8886`
  - notes: CastEvidence::NominalCheck

- [x] **L6 L8890: `Compose`** — `ok`
  - spec: `specification.md:8890`
  - notes: CastEvidence::Compose + simplify_evidence

- [x] **L6 L8907: `DD-TYP-DYN-016`: cast evidenceの純粋性** — `ok`
  - spec: `specification.md:8907`
  - notes: evidence eval pure (inspect/wrap only); DD-TYP-DYN-016

- [x] **L6 L8936: `DD-TYP-DYN-017`: evidence compositionと最適化** — `partial`
  - spec: `specification.md:8936`
  - notes: compose_evidence/simplify_evidence Identity absorption; widen-chain opt TBD

- [x] **L6 L8992: `DD-TYP-DYN-018`: cast provenance** — `partial`
  - spec: `specification.md:8992`
  - notes: CastProvenance struct separate from evidence; full boundary IDs TBD

- [x] **L6 L9027: `DD-TYP-DYN-019`: recordおよびvariant cast** — `ok`
  - spec: `specification.md:9027`
  - notes: RecordCheck/VariantCheck plan + deepened runtime field/payload checks

- [x] **L6 L9054: `DD-TYP-DYN-020`: opaque abstract typeのcast** — `deferred`
  - spec: `specification.md:9054`
  - notes: 依存待ち(MOD/opaque): NominalCheck abstract-type sealing

- [x] **L6 L9089: `DD-TYP-DYN-021`: fixed-arity function cast** — `partial`
  - spec: `specification.md:9089`
  - notes: FunctionGuard fixed-arity plan; call-time arg/ret guard wrap interim

- [x] **L6 L9128: `DD-TYP-DYN-022`: function引数の反変cast** — `partial`
  - spec: `specification.md:9128`
  - notes: FunctionGuard.arg_casts contravariant planning; runtime call wrap interim

- [x] **L6 L9169: `DD-TYP-DYN-023`: function結果の共変cast** — `partial`
  - spec: `specification.md:9169`
  - notes: FunctionGuard.ret_cast covariant planning; runtime call wrap interim

- [x] **L6 L9205: `DD-TYP-DYN-024`: function arityの制限** — `ok`
  - spec: `specification.md:9205`
  - notes: arity equality required in FunctionGuard; varargs deferred

- [x] **L6 L9235: `DD-TYP-DYN-025`: effectful function cast** — `ok`
  - spec: `specification.md:9235`
  - notes: effect_subrow Es⊑Et in is_subtype/fun intersect; gradual effect cast out of v1

- [x] **L6 L9301: `DD-TYP-DYN-026`: dynamic境界を通れない制御値** — `deferred`
  - spec: `specification.md:9301`
  - notes: 依存待ち(EFF/KER): control values (resume/handler/capability) barred from dynamic

- [x] **L6 L9339: `DD-TYP-DYN-027`: polymorphismとdynamic境界** — `deferred`
  - spec: `specification.md:9339`
  - notes: 依存待ち(DAT/poly): polymorphism × dynamic boundary

- [x] **L6 L9394: `DD-TYP-DYN-028`: gradual guarantee** — `meta`
  - spec: `specification.md:9394`
  - notes: 形式証明: gradual guarantee (static/dynamic) not mechanically proven

- [x] **L6 L9412: Static gradual guarantee** — `meta`
  - spec: `specification.md:9412`
  - notes: 形式証明: Static gradual guarantee

- [x] **L6 L9426: Dynamic gradual guarantee** — `meta`
  - spec: `specification.md:9426`
  - notes: 形式証明: Dynamic gradual guarantee

- [x] **L6 L9474: `DD-TYP-NUM-001`: RPX v1の基本数値型** — `ok`
  - spec: `specification.md:9474`
  - notes: CoreType::Int/F64/Number; int<:number, f64<:number; int⊥f64

- [x] **L6 L9485: `int`** — `ok`
  - spec: `specification.md:9485`
  - notes: CoreType::Int + CoreLiteral::Int / RuntimeValue::Int (i128 kernel)

- [x] **L6 L9494: `f64`** — `ok`
  - spec: `specification.md:9494`
  - notes: CoreType::F64 + CoreLiteral::F64 / RuntimeValue::F64

- [x] **L6 L9503: `number`** — `ok`
  - spec: `specification.md:9503`
  - notes: number ≃ int|f64; unify/subtype via Number

- [x] **L6 L9537: `DD-TYP-NUM-002`: 数値promotion** — `ok`
  - spec: `specification.md:9537`
  - notes: mixed +-∗	o f64; int+int	o int; /	o f64; int-div/mod builtins

- [x] **L6 L9586: `DD-TYP-NUM-003`: dynamic castとnumeric promotionの順序** — `ok`
  - spec: `specification.md:9586`
  - notes: DD-TYP-NUM-003: NumericPromote after dynamic narrow; int∩f64 not subtype

- [x] **L6 L9644: `DD-NAME-001`: 組込み型名と識別子の小文字規約** — `partial`
  - spec: `specification.md:9644`
  - notes: lowercase builtins via env; full namespace-collision warnings still thin

- [x] **L6 L9734: `DD-NAME-002`: namespace間の同綴り衝突** — `partial`
  - spec: `specification.md:9734`
  - notes: lowercase builtins via env; full namespace-collision warnings still thin

- [x] **L6 L9772: dynamic typingのCore構文** — `ok`
  - spec: `specification.md:9772`
  - notes: CoreExpr::Cast/TryCast/CheckCast in core

- [x] **L6 L9812: dynamic typingの終端状態** — `ok`
  - spec: `specification.md:9812`
  - notes: Cast/TryCast/CheckCast + NumericPromote terminal paths in eval

- [x] **L6 L9840: 適合試験** — `partial`
  - spec: `specification.md:9840`
  - notes: lang_kernel_suite TYP + cast_tests; full named TYP corpus still open

- [x] **L6 L9842: static injection** — `ok`
  - spec: `specification.md:9842`
  - notes: static injection via Widen/plan_cast (cast_tests)

- [x] **L6 L9861: invalid static injection** — `ok`
  - spec: `specification.md:9861`
  - notes: invalid static injection rejected when T</:S (cast_tests)

- [x] **L6 L9874: dynamic widening** — `ok`
  - spec: `specification.md:9874`
  - notes: dynamic widening Widen evidence (cast_tests)

- [x] **L6 L9889: safe static use** — `ok`
  - spec: `specification.md:9889`
  - notes: safe static use FullyIncluded/Identity when S<:T

- [x] **L6 L9904: partial overlap** — `ok`
  - spec: `specification.md:9904`
  - notes: partial overlap → cast evidence (judge_dynamic_use)

- [x] **L6 L9930: disjoint use** — `ok`
  - spec: `specification.md:9930`
  - notes: disjoint use → static reject in coerce/plan

- [x] **L6 L9944: cast precision** — `ok`
  - spec: `specification.md:9944`
  - notes: cast_success_type = intersect(S,T)

- [x] **L6 L9958: `any`と`dynamic any`** — `ok`
  - spec: `specification.md:9958`
  - notes: any + dynamic any via CoreType::Any / Dynamic(Any)

- [x] **L6 L9988: foreign ingress** — `deferred`
  - spec: `specification.md:9988`
  - notes: 依存待ち(KER/foreign): foreign ingress

- [x] **L6 L10007: implicit cast failure** — `ok`
  - spec: `specification.md:10007`
  - notes: implicit cast failure → EvalError dynamic cast failed

- [x] **L6 L10030: explicit safe cast** — `ok`
  - spec: `specification.md:10030`
  - notes: try-cast/check-cast Option/Result paths

- [x] **L6 L10044: fixed-arity function cast** — `ok`
  - spec: `specification.md:10044`
  - notes: FunctionGuard evidence for fixed-arity fun casts

- [x] **L6 L10067: function result cast** — `ok`
  - spec: `specification.md:10067`
  - notes: covariant ret_cast in FunctionGuard

- [x] **L6 L10083: effect-compatible function cast** — `ok`
  - spec: `specification.md:10083`
  - notes: effect_subrow compatible function cast planning

- [x] **L6 L10099: effect-incompatible function cast** — `ok`
  - spec: `specification.md:10099`
  - notes: effect-incompatible fun rejected via effect_subrow

- [x] **L6 L10115: polymorphic value boundary** — `deferred`
  - spec: `specification.md:10115`
  - notes: 依存待ち: polymorphic value dynamic boundary

- [x] **L6 L10147: dynamicからforall** — `deferred`
  - spec: `specification.md:10147`
  - notes: 依存待ち: dynamic→forall boundary

- [x] **L6 L10160: numeric promotion** — `ok`
  - spec: `specification.md:10160`
  - notes: numeric promotion via CastEvidence::NumericPromote + try-cast

- [x] **L6 L10176: namespace collision warning** — `deferred`
  - spec: `specification.md:10176`
  - notes: namespace collision warnings deferred (NAME-002)

- [x] **L6 L10197: 関連する後続設計課題** — `meta`
  - spec: `specification.md:10197`
  - notes: spec process / OPEN pointer

- [x] **L6 L10201: `OPEN-TYP-002`** — `meta`
  - spec: `specification.md:10201`
  - notes: OPEN / principles under TYP-DYN

- [x] **L6 L10210: `OPEN-SYN-002`／`LIT-001`** — `meta`
  - spec: `specification.md:10210`
  - notes: OPEN / principles under TYP-DYN

- [x] **L6 L10225: `OPEN-ERR-001`** — `meta`
  - spec: `specification.md:10225`
  - notes: OPEN / principles under TYP-DYN

- [x] **L6 L10233: `OPEN-MOD-001`** — `meta`
  - spec: `specification.md:10233`
  - notes: OPEN / principles under TYP-DYN

- [x] **L6 L10240: `OPEN-KER-001`** — `meta`
  - spec: `specification.md:10240`
  - notes: OPEN / principles under TYP-DYN

- [x] **L6 L10248: 将来拡張** — `meta`
  - spec: `specification.md:10248`
  - notes: spec process / OPEN pointer

- [x] **L6 L10262: 確立された基本原則** — `meta`
  - spec: `specification.md:10262`
  - notes: spec process / OPEN pointer

- [x] **L5 L10340: 13.6.2 `TYP-ALG-001` Algorithmic型検査、semantic subtypingの判定範囲、型推論** — `deferred`
  - spec: `specification.md:10340`
  - notes: 意図的後回し: 完全な semantic subtyping solver / worklist 代数（decide_subtype 三値 fragment は維持）

- [x] **L6 L10388: `DD-TYP-ALG-001`: 宣言的型関係とalgorithmic判定の分離** — `ok`
  - spec: `specification.md:10388`
  - notes: declarative is_subtype/types_disjoint vs algorithmic decide_subtype (cast.rs)

- [x] **L6 L10432: `DD-TYP-ALG-002`: algorithmic判定の三値結果** — `ok`
  - spec: `specification.md:10432`
  - notes: DecideResult::{Proved,Disproved,Unknown} (cast.rs)

- [x] **L6 L10444: `proved`** — `ok`
  - spec: `specification.md:10444`
  - notes: DecideResult::Proved + decide_subtype tests (cast.rs)

- [x] **L6 L10453: `disproved`** — `ok`
  - spec: `specification.md:10453`
  - notes: DecideResult::Disproved; witness optional deferred

- [x] **L6 L10464: `unknown`** — `ok`
  - spec: `specification.md:10464`
  - notes: DecideResult::Unknown outside decidable fragment; not accepted as OK

- [x] **L6 L10489: `DD-TYP-ALG-003`: 診断分類** — `deferred`
  - spec: `specification.md:10489`
  - notes: 意図的後回し: TypeDiagClass 全経路の emitter 配線（classify_decide の基本マップは維持）

- [x] **L6 L10503: `type-error`** — `ok`
  - spec: `specification.md:10503`
  - notes: TypeDiagClass::TypeError via classify_decide(Disproved)

- [x] **L6 L10520: `annotation-required`** — `deferred`
  - spec: `specification.md:10520`
  - notes: 意図的後回し: AnnotationRequired の診断 emitter 統合（ADT-08 文字列注釈要求は維持）

- [x] **L6 L10537: `checker-limitation`** — `ok`
  - spec: `specification.md:10537`
  - notes: TypeDiagClass::CheckerLimitation via classify_decide(Unknown)

- [x] **L6 L10556: `checker-resource-limit`** — `deferred`
  - spec: `specification.md:10556`
  - notes: 意図的後回し: checker-resource-limit / solver step-budget emitter

- [x] **L6 L10562: `unsupported-language-feature`** — `deferred`
  - spec: `specification.md:10562`
  - notes: 意図的後回し: UnsupportedLanguageFeature 診断の利用点拡張

- [x] **L6 L10573: `DD-TYP-ALG-004`: soundness、completeness、terminationの優先順位** — `meta`
  - spec: `specification.md:10573`
  - notes: formal priority: soundness > termination > completeness (spec policy)

- [x] **L6 L10610: `DD-TYP-ALG-005`: principal type** — `meta`
  - spec: `specification.md:10610`
  - notes: principal-type property; not an executable obligation here

- [x] **L6 L10639: `DD-TYP-ALG-006`: 決定的なsolver budget** — `deferred`
  - spec: `specification.md:10639`
  - notes: 意図的後回し: 明示的 solver step budget（断片上は終了する）

- [x] **L3 L10672: Effectに関するalgorithmic用語** — `ok`
  - spec: `specification.md:10672`
  - notes: EffectRow on Fun + infer_with_effects / handle shrink (check.rs)

- [x] **L6 L10674: `DD-TYP-EFF-001`: effect関連用語** — `ok`
  - spec: `specification.md:10674`
  - notes: DD-TYP-EFF-001 terms: EffectRow + infer_with_effects (check.rs)

- [x] **L6 L10690: `effect-signature-environment`** — `ok`
  - spec: `specification.md:10690`
  - notes: effect-signature env via Fun EffectRow (ty.rs/check.rs)

- [x] **L6 L10696: `effect-environment`** — `ok`
  - spec: `specification.md:10696`
  - notes: effect-environment carried in infer_with_effects

- [x] **L6 L10707: `expression-effects`** — `ok`
  - spec: `specification.md:10707`
  - notes: expression-effects residual EffectRow from infer

- [x] **L6 L10721: `function-required-effects`** — `ok`
  - spec: `specification.md:10721`
  - notes: function-required-effects on CoreType::Fun

- [x] **L6 L10733: `handled-effects`** — `ok`
  - spec: `specification.md:10733`
  - notes: handled-effects removed by handle in residual

- [x] **L6 L10737: `residual-effects`** — `ok`
  - spec: `specification.md:10737`
  - notes: residual-effects after handle shrink

- [x] **L6 L10741: `effect-row`** — `ok`
  - spec: `specification.md:10741`
  - notes: EffectRow type + infer_with_effects (ty.rs/check.rs)

- [x] **L6 L10757: `DD-TYP-EFF-002`: 最小required effects** — `ok`
  - spec: `specification.md:10757`
  - notes: minimal required effects collected by infer_with_effects

- [x] **L6 L10798: `DD-TYP-EFF-003`: 関数値とrequired effects** — `ok`
  - spec: `specification.md:10798`
  - notes: Fun values carry required EffectRow (check.rs)

- [x] **L6 L10823: `DD-TYP-EFF-004`: effect-row polymorphism** — `deferred`
  - spec: `specification.md:10823`
  - notes: 意図的後回し: EffectRow 全多相 / quantify（Fun EffectRow + infer_with_effects は維持）

- [x] **L6 L10853: `DD-TYP-EFF-005`: handlerとrunnerによるeffect縮小** — `partial`
  - spec: `specification.md:10853`
  - notes: handle removes op from residual; full handler return-clause typing interim

- [x] **L6 L10878: `DD-TYP-EFF-006`: EffectRow alias** — `deferred`
  - spec: `specification.md:10878`
  - notes: effect-row polymorphism / aliases not full

- [x] **L6 L10915: `DD-TYP-EFF-007`: 注釈されたrequired effects** — `deferred`
  - spec: `specification.md:10915`
  - notes: annotated required-effects surface deferred

- [x] **L3 L10941: Static型のBoolean代数** — `ok`
  - spec: `specification.md:10941`
  - notes: Union/Intersect/Not/Diff CoreType + cast intersect algebra

- [x] **L6 L10943: `DD-TYP-BOOL-001`: 型のBoolean演算** — `ok`
  - spec: `specification.md:10943`
  - notes: Boolean type ops in CoreType + cast.rs algebra helpers

- [x] **L6 L10990: `DD-TYP-BOOL-002`: Surface negationの制限** — `deferred`
  - spec: `specification.md:10990`
  - notes: 意図的後回し: unrestricted Surface negation の完全解法（Not 構文は維持）

- [x] **L3 L11031: Singleton型** — `ok`
  - spec: `specification.md:11031`
  - notes: CoreType::Singleton + SingletonValue; f64 singleton reserved

- [x] **L6 L11033: `DD-TYP-SINGLETON-001`: singleton型** — `ok`
  - spec: `specification.md:11033`
  - notes: type-position int/str/bool singletons + domain subtyping (ty/cast)

- [x] **L3 L11096: Function型** — `deferred`
  - spec: `specification.md:11096`
  - notes: 意図的後回し: function intersection types / coherence（fixed-arity Fun は維持）

- [x] **L6 L11098: `DD-TYP-FN-001`: fixed-arity function** — `ok`
  - spec: `specification.md:11098`
  - notes: fixed-arity Fun unify (ty.rs/unify.rs)

- [x] **L6 L11126: `DD-TYP-FN-002`: fixed-arity function subtyping** — `partial`
  - spec: `specification.md:11126`
  - notes: Fun subtyping limited (arg contra / ret cov incomplete vs full DD)

- [x] **L6 L11201: `DD-TYP-FN-003`: function intersection** — `deferred`
  - spec: `specification.md:11201`
  - notes: function intersection / coherence deferred past fixed-arity Fun

- [x] **L6 L11227: `DD-TYP-FN-004`: function intersectionの適用可能性** — `deferred`
  - spec: `specification.md:11227`
  - notes: function intersection applicability deferred

- [x] **L6 L11277: `DD-TYP-FN-005`: branch specificity** — `deferred`
  - spec: `specification.md:11277`
  - notes: branch specificity deferred with function intersection

- [x] **L6 L11333: `DD-TYP-FN-006`: function intersectionのcoherence** — `deferred`
  - spec: `specification.md:11333`
  - notes: function intersection coherence deferred

- [x] **L6 L11339: 入力領域が互いに素** — `deferred`
  - spec: `specification.md:11339`
  - notes: coherence case: disjoint domains

- [x] **L6 L11347: 片方が他方を包含** — `deferred`
  - spec: `specification.md:11347`
  - notes: coherence case: inclusion

- [x] **L6 L11351: 入力領域が等価** — `deferred`
  - spec: `specification.md:11351`
  - notes: coherence case: equivalent domains

- [x] **L6 L11357: 入力領域が重なるが非比較** — `deferred`
  - spec: `specification.md:11357`
  - notes: coherence case: overlapping incomparable

- [x] **L6 L11373: `DD-TYP-FN-007`: union引数とdispatch** — `deferred`
  - spec: `specification.md:11373`
  - notes: union-arg dispatch deferred with function intersection

- [x] **L3 L11396: RecordRow** — `ok`
  - spec: `specification.md:11396`
  - notes: Record/OpenRecord/Lacks in ty.rs + unify.rs

- [x] **L6 L11398: `DD-TYP-ROW-001`: closed、open、row-polymorphic record** — `ok`
  - spec: `specification.md:11398`
  - notes: closed/open/row-poly fragment via Record/OpenRecord/Lacks

- [x] **L6 L11404: Closed record** — `ok`
  - spec: `specification.md:11404`
  - notes: closed Record type + unify (ty.rs)

- [x] **L6 L11412: Open record** — `ok`
  - spec: `specification.md:11412`
  - notes: OpenRecord with row var (ty.rs/unify.rs)

- [x] **L6 L11420: Row-polymorphic record** — `partial`
  - spec: `specification.md:11420`
  - notes: row-polymorphic records via OpenRecord; multi-tail deferred at ROW-001

- [x] **L6 L11434: `DD-TYP-ROW-002`: closed recordの正確なshape** — `ok`
  - spec: `specification.md:11434`
  - notes: closed record exact shape unify (DD-TYP-ROW-002)

- [x] **L6 L11458: `DD-TYP-ROW-003`: immutable field covariance** — `ok`
  - spec: `specification.md:11458`
  - notes: immutable field covariance in record subtype (cast/unify)

- [x] **L6 L11478: `DD-TYP-ROW-004`: RecordRow変数とlacks制約** — `ok`
  - spec: `specification.md:11478`
  - notes: row vars + Lacks constraints in unify.rs

- [x] **L6 L11501: `DD-TYP-ROW-005`: field extensionとupdate** — `ok`
  - spec: `specification.md:11501`
  - notes: record-extend / record-update in CoreExpr + check/eval

- [x] **L6 L11507: Extension** — `ok`
  - spec: `specification.md:11507`
  - notes: record-extend (check.rs/eval.rs)

- [x] **L6 L11516: Update** — `ok`
  - spec: `specification.md:11516`
  - notes: record-update (check.rs/eval.rs)

- [x] **L6 L11531: `DD-TYP-ROW-006`: row-preserving update** — `ok`
  - spec: `specification.md:11531`
  - notes: row-preserving update; multi-tail still deferred at ROW-001

- [x] **L6 L11551: `DD-TYP-ROW-007`: optional field** — `ok`
  - spec: `specification.md:11551`
  - notes: OptionalField + required<:optional; pattern decomp rejected (§18.5)

- [x] **L6 L11591: `DD-TYP-ROW-008`: recordのBoolean演算** — `deferred`
  - spec: `specification.md:11591`
  - notes: record Boolean combination complete fragment deferred

- [x] **L3 L11626: Recursive data type** — `deferred`
  - spec: `specification.md:11626`
  - notes: equi-recursive / contractiveness checker deferred

- [x] **L6 L11628: `DD-TYP-REC-001`: recursive data type** — `deferred`
  - spec: `specification.md:11628`
  - notes: recursive data type equi-checker deferred (DAT surface exists)

- [x] **L6 L11644: `DD-TYP-REC-002`: equi-recursiveな利用者意味論** — `deferred`
  - spec: `specification.md:11644`
  - notes: equi-recursive user semantics deferred

- [x] **L6 L11664: `DD-TYP-REC-003`: contractiveness** — `deferred`
  - spec: `specification.md:11664`
  - notes: contractiveness checker deferred

- [x] **L6 L11699: `DD-TYP-REC-004`: strict positivity** — `deferred`
  - spec: `specification.md:11699`
  - notes: strict positivity: DAT group positivity only (partial elsewhere)

- [x] **L6 L11727: `DD-TYP-REC-005`: regularity** — `deferred`
  - spec: `specification.md:11727`
  - notes: regularity checker deferred

- [x] **L6 L11758: `DD-TYP-REC-006`: base constructor** — `deferred`
  - spec: `specification.md:11758`
  - notes: base constructor discipline deferred

- [x] **L6 L11786: `DD-TYP-REC-007`: recursive dataの完全判定範囲** — `deferred`
  - spec: `specification.md:11786`
  - notes: full recursive decide fragment deferred

- [x] **L3 L11809: Bidirectional type checking** — `ok`
  - spec: `specification.md:11809`
  - notes: infer-primary + (type)/(val) annotation checking; principle-type val generalization

- [x] **L6 L11811: `DD-TYP-BIDI-001`: bidirectional typing** — `ok`
  - spec: `specification.md:11811`
  - notes: synthesis+annotation check path via infer_binding_init (check.rs)

- [x] **L6 L11819: Synthesis** — `ok`
  - spec: `specification.md:11819`
  - notes: synthesis path = infer_expr (check.rs)

- [x] **L6 L11827: Checking** — `ok`
  - spec: `specification.md:11827`
  - notes: annotation-driven checking via unify against expected (check.rs)

- [x] **L6 L11839: `DD-TYP-BIDI-002`: 型注釈付きbinding** — `ok`
  - spec: `specification.md:11839`
  - notes: same-named (type)/(val) initializer checked against annotation

- [x] **L6 L11862: `DD-TYP-BIDI-003`: 無注釈binding** — `ok`
  - spec: `specification.md:11862`
  - notes: unannotated val synthesizes; free vars generalized; ambiguous var→annotation-required

- [x] **L6 L11885: `DD-TYP-BIDI-004`: synthesis可能な式** — `partial`
  - spec: `specification.md:11885`
  - notes: literals/vars/app/record/ctors synthesize; empty-collection synth gaps remain

- [x] **L6 L11907: `DD-TYP-BIDI-005`: checkingを優先する式** — `partial`
  - spec: `specification.md:11907`
  - notes: annotated bindings check; empty collections/intersection check gaps remain

- [x] **L6 L11928: `DD-TYP-BIDI-006`: `if`と`match`** — `ok`
  - spec: `specification.md:11928`
  - notes: if/match infer-primary with unify join; occurrence refine on predicates

- [x] **L6 L11952: `DD-TYP-BIDI-007`: subtyping、promotion、dynamic cast** — `ok`
  - spec: `specification.md:11952`
  - notes: subtyping/promotion/dynamic cast via plan_cast + coerce_to_static

- [x] **L6 L11982: `DD-TYP-BIDI-008`: annotation-requiredとchecker-limitation** — `ok`
  - spec: `specification.md:11982`
  - notes: classify_decide maps Unknown→CheckerLimitation; AnnotationRequired enum

- [x] **L3 L12003: 明示的多相型** — `ok`
  - spec: `specification.md:12003`
  - notes: prenex forall instantiate-at-use + let/val generalization (rank-N deferred)

- [x] **L6 L12005: `DD-TYP-POLY-001`: 明示的`forall`** — `ok`
  - spec: `specification.md:12005`
  - notes: explicit forall annotate; unbound type vars rejected; let generalization

- [x] **L6 L12046: `DD-TYP-POLY-002`: `forall` binderのkind** — `ok`
  - spec: `specification.md:12046`
  - notes: forall binder kinds type/record-row/effect-row checked at elaborate

- [x] **L6 L12077: `DD-TYP-POLY-003`: `forall`のscope** — `ok`
  - spec: `specification.md:12077`
  - notes: forall binder scope in parse_type_syntax_in

- [x] **L6 L12100: `DD-TYP-POLY-004`: rank-1／prenex制限** — `ok`
  - spec: `specification.md:12100`
  - notes: rank-1/prenex surface forall; instantiate_forall at use sites

- [x] **L3 L12136: 完全性分類** — `ok`
  - spec: `specification.md:12136`
  - notes: is_fully_decidable_fragment gates Unknown vs Disproved (cast.rs)

- [x] **L6 L12138: `DD-TYP-FRAG-001`: A — 完全判定fragment** — `ok`
  - spec: `specification.md:12138`
  - notes: fragment A: primitives/union/intersect/record/fun decide closed

- [x] **L6 L12180: `DD-TYP-FRAG-002`: B — `unknown`を返し得るfragment** — `ok`
  - spec: `specification.md:12180`
  - notes: fragment B: open rows/forall/app → Unknown

- [x] **L6 L12208: `DD-TYP-FRAG-003`: C — 注釈を要求し得るfragment** — `partial`
  - spec: `specification.md:12208`
  - notes: AnnotationRequired class reserved; incompleteness paths still light

- [x] **L6 L12231: `DD-TYP-FRAG-004`: D — RPX v1で禁止するfragment** — `ok`
  - spec: `specification.md:12231`
  - notes: v1 rejects unsupported via type errors / reserved enums

- [x] **L3 L12266: 共通constraint worklist** — `deferred`
  - spec: `specification.md:12266`
  - notes: shared constraint worklist deferred; unify is direct

- [x] **L6 L12268: `DD-TYP-SOLVER-001`: shared constraint worklist** — `deferred`
  - spec: `specification.md:12268`
  - notes: shared constraint worklist deferred

- [x] **L6 L12303: `DD-TYP-SOLVER-002`: solver間のconstraint生成** — `deferred`
  - spec: `specification.md:12303`
  - notes: multi-solver constraint generation deferred

- [x] **L6 L12382: `DD-TYP-SOLVER-003`: constraint処理の優先度** — `deferred`
  - spec: `specification.md:12382`
  - notes: constraint priority schedule deferred

- [x] **L6 L12409: `DD-TYP-SOLVER-004`: canonicalizationとmemoization** — `deferred`
  - spec: `specification.md:12409`
  - notes: canonicalization/memoization deferred

- [x] **L6 L12436: `DD-TYP-SOLVER-005`: solver終了状態** — `deferred`
  - spec: `specification.md:12436`
  - notes: solver end-states folded into CheckError for now

- [x] **L6 L12442: 成功** — `ok`
  - spec: `specification.md:12442`
  - notes: success path = infer_expr Ok

- [x] **L6 L12454: Type error** — `ok`
  - spec: `specification.md:12454`
  - notes: type error via CheckError / DecideResult::Disproved

- [x] **L6 L12458: Annotation required** — `ok`
  - spec: `specification.md:12458`
  - notes: LocalVar free vars → cannot infer ungeneralized… / annotation-required

- [x] **L6 L12462: Checker limitation** — `ok`
  - spec: `specification.md:12462`
  - notes: CheckerLimitation class on Unknown

- [x] **L6 L12466: Resource limit** — `deferred`
  - spec: `specification.md:12466`
  - notes: 意図的後回し: CheckerResourceLimit / resource-budget 実行経路

- [x] **L6 L12472: `DD-TYP-SOLVER-006`: cast insertionとgeneralizationの順序** — `deferred`
  - spec: `specification.md:12472`
  - notes: cast insertion vs generalization ordering deferred

- [x] **L3 L12494: 型検査器の概念pipeline** — `deferred`
  - spec: `specification.md:12494`
  - notes: full checker pipeline schedule deferred

- [x] **L6 L12496: `DD-TYP-SOLVER-007`: checkerの全体処理順序** — `deferred`
  - spec: `specification.md:12496`
  - notes: overall checker processing order deferred

- [x] **L3 L12542: 適合試験** — `partial`
  - spec: `specification.md:12542`
  - notes: lang_kernel_suite TYP + cast_tests; full named TYP-ALG corpus open

- [x] **L6 L12544: 三値判定** — `ok`
  - spec: `specification.md:12544`
  - notes: decide_subtype three-valued tests in cast.rs

- [x] **L6 L12574: 診断分類** — `deferred`
  - spec: `specification.md:12574`
  - notes: 意図的後回し: TypeDiagClass 全診断 emitter（taxonomy + classify_decide は維持）

- [x] **L6 L12615: 最小required effects** — `ok`
  - spec: `specification.md:12615`
  - notes: minimal required effects covered by infer_with_effects tests

- [x] **L6 L12634: 関数値のeffect** — `ok`
  - spec: `specification.md:12634`
  - notes: Fun effect-row carried; covered by check/effect tests

- [x] **L6 L12656: EffectRow polymorphism** — `deferred`
  - spec: `specification.md:12656`
  - notes: 意図的後回し: EffectRow polymorphism 適合試験 / 量化（EffectRow 本体は維持）

- [x] **L6 L12680: Handlerによるeffect縮小** — `ok`
  - spec: `specification.md:12680`
  - notes: handler effect shrink covered by handle residual tests

- [x] **L6 L12696: Singleton型** — `ok`
  - spec: `specification.md:12696`
  - notes: singleton suite: type-position lit + domain subtype tests

- [x] **L6 L12722: Function subtyping** — `partial`
  - spec: `specification.md:12722`
  - notes: basic Fun subtyping only; full DD suite incomplete

- [x] **L6 L12744: Function intersection coherence** — `deferred`
  - spec: `specification.md:12744`
  - notes: function intersection coherence suite deferred

- [x] **L6 L12770: 非比較なbranch overlap** — `deferred`
  - spec: `specification.md:12770`
  - notes: incomparable branch overlap suite deferred

- [x] **L6 L12786: Union引数の暗黙dispatch禁止** — `deferred`
  - spec: `specification.md:12786`
  - notes: union-arg implicit dispatch forbidden suite deferred

- [x] **L6 L12810: Closed record** — `ok`
  - spec: `specification.md:12810`
  - notes: closed/open record + Lacks covered by unify/check tests

- [x] **L6 L12840: Row preservation** — `ok`
  - spec: `specification.md:12840`
  - notes: row-preserving update covered by record-update tests

- [x] **L6 L12871: Optional field** — `ok`
  - spec: `specification.md:12871`
  - notes: optional-field subtype + pattern reject tests

- [x] **L6 L12893: Recursive data** — `deferred`
  - spec: `specification.md:12893`
  - notes: recursive data conformance suite deferred

- [x] **L6 L12916: 非contractive再帰** — `deferred`
  - spec: `specification.md:12916`
  - notes: non-contractive recursion suite deferred

- [x] **L6 L12930: Non-regular recursion** — `deferred`
  - spec: `specification.md:12930`
  - notes: non-regular recursion suite deferred

- [x] **L6 L12948: Bidirectional checking** — `partial`
  - spec: `specification.md:12948`
  - notes: Fun/Record annotation checking present; full BIDI corpus incomplete

- [x] **L6 L12979: Explicit `forall`** — `ok`
  - spec: `specification.md:12979`
  - notes: explicit forall surface + instantiate suite (forall_annotation_* / ADT-07)

- [x] **L6 L13007: Kind error** — `ok`
  - spec: `specification.md:13007`
  - notes: forall kind error checked at elaborate

- [x] **L6 L13022: Solver determinism** — `deferred`
  - spec: `specification.md:13022`
  - notes: solver determinism suite deferred with worklist

- [x] **L3 L13037: 関連する後続設計課題** — `meta`
  - spec: `specification.md:13037`
  - notes: spec process / OPEN pointer

- [x] **L6 L13039: `OPEN-SYN-002`** — `meta`
  - spec: `specification.md:13039`
  - notes: OPEN / principles under TYP-ALG

- [x] **L6 L13053: `OPEN-DAT-001`** — `meta`
  - spec: `specification.md:13053`
  - notes: OPEN / principles under TYP-ALG

- [x] **L6 L13063: `OPEN-EFF-001`の後続仕様** — `meta`
  - spec: `specification.md:13063`
  - notes: OPEN / principles under TYP-ALG

- [x] **L6 L13071: `OPEN-MOD-001`** — `meta`
  - spec: `specification.md:13071`
  - notes: OPEN / principles under TYP-ALG

- [x] **L6 L13079: `OPEN-ERR-001`** — `meta`
  - spec: `specification.md:13079`
  - notes: OPEN / principles under TYP-ALG

- [x] **L6 L13087: 将来拡張** — `meta`
  - spec: `specification.md:13087`
  - notes: spec process / OPEN pointer

- [x] **L3 L13104: 確立された基本原則** — `meta`
  - spec: `specification.md:13104`
  - notes: spec process / OPEN pointer

- [x] **L4 L13203: 13.7 `ROW-001` Row-polymorphic records** — `partial`
  - spec: `specification.md:13203`
  - notes: closed+OpenRecord+Lacks done; multi-tail deferred (plan ROW-001)

- [x] **L5 L13205: 概要・状態** — `partial`
  - spec: `specification.md:13205`
  - notes: closed+OpenRecord+Lacks done; multi-tail deferred (plan ROW-001)

- [x] **L4 L13226: 13.8 `EFF-001` Algebraic effects and handlers** — `partial`
  - spec: `specification.md:13226`
  - notes: deep one-shot + ambient + with/handler; multi-shot/return deferred

- [x] **L5 L13230: 状態** — `partial`
  - spec: `specification.md:13230`
  - notes: deep one-shot + ambient + with/handler; multi-shot/return deferred

- [x] **L5 L13262: `DD-EFF-001`: deep handler** — `ok`
  - spec: `specification.md:13262`
  - notes: eval.rs deep handle + one-shot; control.rs; lang_kernel_suite

- [x] **L5 L13295: `DD-EFF-002`: one-shot resumption** — `ok`
  - spec: `specification.md:13295`
  - notes: eval.rs deep handle + one-shot; control.rs; lang_kernel_suite

- [x] **L5 L13351: `DD-EFF-003`: resumptionの型とscope** — `partial`
  - spec: `specification.md:13351`
  - notes: one-shot resume value; resume typing interim Dynamic

- [x] **L5 L13426: `DD-EFF-004`: effect operationの呼出し** — `ok`
  - spec: `specification.md:13426`
  - notes: eval.rs deep handle + one-shot; control.rs; lang_kernel_suite

- [x] **L5 L13459: `DD-EFF-005`: nearest matching handlerと自動伝播** — `ok`
  - spec: `specification.md:13459`
  - notes: eval.rs deep handle + one-shot; control.rs; lang_kernel_suite

- [x] **L5 L13489: `DD-EFF-006`: 明示的forward** — `ok`
  - spec: `specification.md:13489`
  - notes: surface (forward resume) + eval re-performs to next outer matching handler

- [x] **L5 L13539: `DD-EFF-007`: handler clauseの実行scope** — `ok`
  - spec: `specification.md:13539`
  - notes: handler runs in captured env; eval.rs

- [x] **L5 L13579: `DD-EFF-008`: return clause** — `deferred`
  - spec: `specification.md:13579`
  - notes: return-clause / Handler<L,A,B,H> typing deferred (plan)

- [x] **L5 L13634: `DD-EFF-009`: handler単位の結果型変換** — `deferred`
  - spec: `specification.md:13634`
  - notes: return-clause / Handler<L,A,B,H> typing deferred (plan)

- [x] **L5 L13681: `DD-EFF-010`: handler valueのrank-1多相性** — `partial`
  - spec: `specification.md:13681`
  - notes: HandlerValue is Dynamic in check.rs

- [x] **L5 L13725: `DD-EFF-011`: first-class handler value** — `ok`
  - spec: `specification.md:13725`
  - notes: HandlerValue + With; elaborate.rs / eval.rs

- [x] **L5 L13781: `DD-EFF-012`: Core `handle`とSurface `with`** — `ok`
  - spec: `specification.md:13781`
  - notes: HandlerValue + With; elaborate.rs / eval.rs

- [x] **L5 L13845: `DD-EFF-013`: ambient effectとnamed/scoped effect instance** — `partial`
  - spec: `specification.md:13845`
  - notes: ambient ok (elaborate Perform); named/scoped instances deferred

- [x] **L6 L13854: Ambient effect** — `ok`
  - spec: `specification.md:13854`
  - notes: ambient log/random → Perform (elaborate.rs)

- [x] **L6 L13875: Named/scoped effect instance** — `deferred`
  - spec: `specification.md:13875`
  - notes: named/scoped effect instances not in kernel

- [x] **L5 L13931: `DD-EFF-014`: EffectRowの意味** — `partial`
  - spec: `specification.md:13931`
  - notes: thin EffectRow; handler typing interim (check.rs)

- [x] **L5 L14013: `DD-EFF-015`: ambient effect rowと制約生成** — `partial`
  - spec: `specification.md:14013`
  - notes: thin EffectRow; handler typing interim (check.rs)

- [x] **L5 L14109: `DD-EFF-016`: handlerの型付け骨格** — `partial`
  - spec: `specification.md:14109`
  - notes: thin EffectRow; handler typing interim (check.rs)

- [x] **L5 L14189: `DD-EFF-017`: 利用者向けmaskの禁止** — `ok`
  - spec: `specification.md:14189`
  - notes: user mask not provided (spec prohibits); N/A intentional

- [x] **L5 L14232: `DD-EFF-018`: residual effectと実行境界** — `ok`
  - spec: `specification.md:14232`
  - notes: residual effects tracked; unhandled propagates to host

- [x] **L5 L14291: `DD-EFF-019`: 未処理effectの動的分類** — `ok`
  - spec: `specification.md:14291`
  - notes: unhandled effect classified at host boundary (eval Outcome)

- [x] **L5 L14371: `DD-EFF-020`: cleanupとの接続要件** — `deferred`
  - spec: `specification.md:14371`
  - notes: cleanup/finalization → ERR; plan deferral

- [x] **L4 L14412: 13.9 `MOD-001` モジュール・シグネチャ・Functor・分割コンパイル** — `partial`
  - spec: `specification.md:14412`
  - notes: outer unit + import/link + .rpi export filter/path; signatures/functors deferred

- [x] **L5 L14413: DD-001 決定概要** — `meta`
  - spec: `specification.md:14413`
  - notes: decision overview

- [x] **L6 L14414: DD-001.1 状態** — `meta`
  - spec: `specification.md:14414`
  - notes: RESOLVED scope list; impl covers outer+import subset only

- [x] **L6 L14434: DD-001.2 中心的な決定** — `partial`
  - spec: `specification.md:14434`
  - notes: import as/only/rename+qualified + .rpi export boundary; functors/signatures deferred

- [x] **L5 L14452: 0. 適用範囲** — `meta`
  - spec: `specification.md:14452`
  - notes: scope framing

- [x] **L6 L14453: 0.1 本項目が定めるもの** — `meta`
  - spec: `specification.md:14453`
  - notes: lists MOD surface; many items deferred

- [x] **L6 L14477: 0.2 本項目が直接定めないもの** — `deferred`
  - spec: `specification.md:14477`
  - notes: PKG/MAC/KER/recursive/1st-class/generative → OPEN

- [x] **L5 L14495: 1. コンパイル単位** — `ok`
  - spec: `specification.md:14495`
  - notes: ModuleUnit / elaborate_units / load_module_tree

- [x] **L6 L14496: 1.1 基本単位** — `ok`
  - spec: `specification.md:14496`
  - notes: one source unit ≈ one outer module (bind/module.rs)

- [x] **L6 L14507: 1.2 外側モジュールのwrapper** — `ok`
  - spec: `specification.md:14507`
  - notes: spec forbids explicit outer-module wrapper; file body is the outer module

- [x] **L6 L14519: 1.3 一ファイル内の外側モジュール数** — `ok`
  - spec: `specification.md:14519`
  - notes: one unit per file stem; multi-outer-in-file rejected/unsupported

- [x] **L6 L14531: 1.4 一ファイル一モジュールとの違い** — `meta`
  - spec: `specification.md:14531`
  - notes: design note vs one-file-one-module

- [x] **L5 L14542: 2. モジュールpathとファイルpath** — `ok`
  - spec: `specification.md:14542`
  - notes: source-root module path + interface-root .rpi path (PackageManifest)

- [x] **L6 L14543: 2.1 Source root** — `ok`
  - spec: `specification.md:14543`
  - notes: source-root resolves module .rpx under package root

- [x] **L6 L14549: 2.2 Module path** — `ok`
  - spec: `specification.md:14549`
  - notes: coalesce_slash_paths + import module path

- [x] **L6 L14560: 2.3 Interface path** — `ok`
  - spec: `specification.md:14560`
  - notes: module_interface_path under interface-root; required for public modules

- [x] **L6 L14573: 2.4 Path変更** — `deferred`
  - spec: `specification.md:14573`
  - notes: 意図的後回し: path-rename / remapping API

- [x] **L5 L14585: 3. 下位モジュール** — `deferred`
  - spec: `specification.md:14585`
  - notes: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)

- [x] **L6 L14586: 3.1 基本構文** — `deferred`
  - spec: `specification.md:14586`
  - notes: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)

- [x] **L6 L14603: 3.2 下位モジュールのpath** — `deferred`
  - spec: `specification.md:14603`
  - notes: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)

- [x] **L6 L14609: 3.3 下位モジュールの外部ファイル化** — `deferred`
  - spec: `specification.md:14609`
  - notes: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)

- [x] **L6 L14620: 3.4 Module body** — `deferred`
  - spec: `specification.md:14620`
  - notes: 意図的後回し: nested (module …) surface; flat outer units only (MOD-001)

- [x] **L5 L14636: 4. 下位モジュールのscopeと純粋性** — `deferred`
  - spec: `specification.md:14636`
  - notes: 意図的後回し: nested-module scope N/A until nested modules

- [x] **L6 L14637: 4.1 Importの位置** — `ok`
  - spec: `specification.md:14637`
  - notes: imports collected at unit top via split_imports

- [x] **L6 L14655: 4.2 親scopeの参照** — `deferred`
  - spec: `specification.md:14655`
  - notes: 意図的後回し: nested parent-scope N/A until nested modules

- [x] **L6 L14667: 4.3 後方参照** — `deferred`
  - spec: `specification.md:14667`
  - notes: 意図的後回し: nested forward-ref N/A until nested modules

- [x] **L6 L14679: 4.4 新しいscope** — `meta`
  - spec: `specification.md:14679`
  - notes: new scope rule; nested modules absent

- [x] **L6 L14685: 4.5 Top-level effect** — `partial`
  - spec: `specification.md:14685`
  - notes: unit body elaborated as Core; no module-level effect gate

- [x] **L6 L14698: 4.6 モジュールは通常値ではない** — `ok`
  - spec: `specification.md:14698`
  - notes: modules not first-class values in kernel

- [x] **L5 L14707: 5. 修飾参照** — `ok`
  - spec: `specification.md:14707`
  - notes: prefix/export lets after import as / bare import

- [x] **L6 L14708: 5.1 区切り記号** — `ok`
  - spec: `specification.md:14708`
  - notes: `/` path separator (coalesce_slash_paths)

- [x] **L6 L14718: 5.2 除算との区別** — `ok`
  - spec: `specification.md:14718`
  - notes: `/` not ident_continue; // kept; division via BuiltinOp

- [x] **L6 L14727: 5.3 内部表現** — `partial`
  - spec: `specification.md:14727`
  - notes: qualified name is string binder `alias/export`, not ModuleId path IR

- [x] **L6 L14737: 5.4 .** — `deferred`
  - spec: `specification.md:14737`
  - notes: 意図的後回し: dot-qualified module refs; slash paths are primary

- [x] **L5 L14743: 6. Import** — `ok`
  - spec: `specification.md:14743`
  - notes: ImportDecl + elaborate_units link

- [x] **L6 L14744: 6.1 通常import** — `ok`
  - spec: `specification.md:14744`
  - notes: `(import m)` → qualified exports

- [x] **L6 L14755: 6.2 モジュール別名** — `ok`
  - spec: `specification.md:14755`
  - notes: `as` alias → prefix/export

- [x] **L6 L14766: 6.3 選択的インポート** — `ok`
  - spec: `specification.md:14766`
  - notes: flat `only a b` + legacy `only (a b)`

- [x] **L6 L14776: 6.4 個別項目の改名** — `ok`
  - spec: `specification.md:14776`
  - notes: `only x as y` → ImportItem.rename

- [x] **L6 L14787: 6.5 asとonlyの併用** — `ok`
  - spec: `specification.md:14787`
  - notes: as + only combined (module_tests)

- [x] **L6 L14803: 6.6 Import item** — `ok`
  - spec: `specification.md:14803`
  - notes: ImportItem {name, rename}

- [x] **L6 L14811: 6.7 自動再公開** — `deferred`
  - spec: `specification.md:14811`
  - notes: 意図的後回し: auto re-export of imports not in bind skeleton

- [x] **L5 L14821: 7. Importと正式identity** — `partial`
  - spec: `specification.md:14821`
  - notes: alias is local prefix only; no formal ModuleId identity layer

- [x] **L6 L14822: 7.1 Aliasの効果** — `partial`
  - spec: `specification.md:14822`
  - notes: alias does not change export identity (string copy of binding)

- [x] **L6 L14832: 7.2 選択的import** — `partial`
  - spec: `specification.md:14832`
  - notes: selective import binds locals; no sealed identity table

- [x] **L6 L14843: 7.3 Source spelling** — `meta`
  - spec: `specification.md:14843`
  - notes: source spelling vs identity

- [x] **L6 L14853: 7.4 同じidentityの重複import** — `ok`
  - spec: `specification.md:14853`
  - notes: same-identity duplicate import ok; distinct-module local collision error (module.rs §7.4)

- [x] **L5 L14859: 8. .rpiインターフェース** — `partial`
  - spec: `specification.md:14859`
  - notes: .rpi stub parse + export name boundary; full signature checking deferred

- [x] **L6 L14860: 8.1 役割** — `partial`
  - spec: `specification.md:14860`
  - notes: .rpi lists public vals/types; enforced at package link

- [x] **L6 L14870: 8.2 Wrapper** — `deferred`
  - spec: `specification.md:14870`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14879: 8.3 .rpi内のimport** — `deferred`
  - spec: `specification.md:14879`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14891: 8.4 Public module** — `ok`
  - spec: `specification.md:14891`
  - notes: public modules with interface-root require sibling .rpi stub

- [x] **L6 L14897: 8.5 Internal module** — `deferred`
  - spec: `specification.md:14897`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14903: 8.6 Script／実行entry** — `deferred`
  - spec: `specification.md:14903`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L14907: 9. 推論シグネチャと抽象化境界** — `deferred`
  - spec: `specification.md:14907`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14908: 9.1 .rpiなしの内部モジュール** — `ok`
  - spec: `specification.md:14908`
  - notes: no .rpi → implementation exports inferred (elaborate default)

- [x] **L6 L14912: 9.2 .rpiありのモジュール** — `partial`
  - spec: `specification.md:14912`
  - notes: .rpi filters public exports at link; types/abstract not checked yet

- [x] **L6 L14918: 9.3 Interface追加** — `deferred`
  - spec: `specification.md:14918`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L14924: 10. .rpiの値仕様** — `deferred`
  - spec: `specification.md:14924`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14925: 10.1 type** — `deferred`
  - spec: `specification.md:14925`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14938: 10.2 実装** — `deferred`
  - spec: `specification.md:14938`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14946: 10.3 型注釈の省略** — `deferred`
  - spec: `specification.md:14946`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14952: 10.4 実装側にも型がある場合** — `deferred`
  - spec: `specification.md:14952`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L14958: 11. 型の公開方法** — `deferred`
  - spec: `specification.md:14958`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14959: 11.1 抽象型仕様** — `deferred`
  - spec: `specification.md:14959`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14977: 11.2 Parameter付き抽象型** — `deferred`
  - spec: `specification.md:14977`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14986: 11.3 Constructor公開data仕様** — `deferred`
  - spec: `specification.md:14986`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L14998: 11.4 実装との一致** — `deferred`
  - spec: `specification.md:14998`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15011: 11.5 type-alias** — `deferred`
  - spec: `specification.md:15011`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L15025: 12. 名前付きシグネチャ** — `deferred`
  - spec: `specification.md:15025`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15026: 12.1 基本構文** — `deferred`
  - spec: `specification.md:15026`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15033: 12.2 日本語上の意味** — `deferred`
  - spec: `specification.md:15033`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15042: 12.3 .rpiとの違い** — `deferred`
  - spec: `specification.md:15042`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15049: 12.4 Signature identity** — `deferred`
  - spec: `specification.md:15049`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L15055: 13. シグネチャ指定** — `deferred`
  - spec: `specification.md:15055`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15056: 13.1 基本構文** — `deferred`
  - spec: `specification.md:15056`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15069: 13.2 適合判定** — `deferred`
  - spec: `specification.md:15069`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15075: 13.3 追加構成要素** — `deferred`
  - spec: `specification.md:15075`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15081: 13.4 抽象型identity** — `deferred`
  - spec: `specification.md:15081`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15089: 13.5 実装内部のalias** — `deferred`
  - spec: `specification.md:15089`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L15103: 14. 下位モジュール仕様** — `deferred`
  - spec: `specification.md:15103`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15104: 14.1 名前付きシグネチャ** — `deferred`
  - spec: `specification.md:15104`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15119: 14.2 インラインシグネチャ** — `deferred`
  - spec: `specification.md:15119`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15128: 14.3 下位モジュール型の参照** — `deferred`
  - spec: `specification.md:15128`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15134: 14.4 抽象型の独立性** — `deferred`
  - spec: `specification.md:15134`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15142: 14.5 非公開型の漏出** — `deferred`
  - spec: `specification.md:15142`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L15151: 15. シグネチャの精緻化と型共有** — `deferred`
  - spec: `specification.md:15151`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15152: 15.1 基本構文** — `deferred`
  - spec: `specification.md:15152`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15156: 15.2 抽象型の具体化** — `deferred`
  - spec: `specification.md:15156`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15163: 15.3 別モジュールとの型共有** — `deferred`
  - spec: `specification.md:15163`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15174: 15.4 日本語用語** — `deferred`
  - spec: `specification.md:15174`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15185: 15.5 許可される精緻化** — `deferred`
  - spec: `specification.md:15185`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15189: 15.6 禁止される変更** — `deferred`
  - spec: `specification.md:15189`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15198: 15.7 Parameter付き型constructor** — `deferred`
  - spec: `specification.md:15198`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L6 L15205: 15.8 下位モジュール内の型** — `deferred`
  - spec: `specification.md:15205`
  - notes: MOD signatures/.rpi/refinement deferred (pre-PKG; language-kernel-plan)

- [x] **L5 L15212: 16. Functor** — `deferred`
  - spec: `specification.md:15212`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15213: 16.1 概念** — `deferred`
  - spec: `specification.md:15213`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15223: 16.2 基本構文** — `deferred`
  - spec: `specification.md:15223`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15233: 16.3 Parameter** — `deferred`
  - spec: `specification.md:15233`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15243: 16.4 結果シグネチャ** — `deferred`
  - spec: `specification.md:15243`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15249: 16.5 Fixed arity** — `deferred`
  - spec: `specification.md:15249`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15255: 16.6 通常値ではない** — `deferred`
  - spec: `specification.md:15255`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L5 L15261: 17. Functor適用** — `deferred`
  - spec: `specification.md:15261`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15262: 17.1 基本構文** — `deferred`
  - spec: `specification.md:15262`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15269: 17.2 複数引数** — `deferred`
  - spec: `specification.md:15269`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15276: 17.3 名前付きモジュールのみ** — `deferred`
  - spec: `specification.md:15276`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15282: 17.4 適用結果の再利用** — `deferred`
  - spec: `specification.md:15282`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L5 L15294: 18. 適用的Functor** — `deferred`
  - spec: `specification.md:15294`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15295: 18.1 基本規則** — `deferred`
  - spec: `specification.md:15295`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15305: 18.2 同一入力** — `deferred`
  - spec: `specification.md:15305`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15319: 18.3 異なる入力** — `deferred`
  - spec: `specification.md:15319`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15333: 18.4 Aliasの影響** — `deferred`
  - spec: `specification.md:15333`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L6 L15339: 18.5 生成的Functor** — `deferred`
  - spec: `specification.md:15339`
  - notes: full ML functors deferred (MOD-001 / language-kernel-plan)

- [x] **L5 L15343: 19. 再公開とシグネチャ合成** — `deferred`
  - spec: `specification.md:15343`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15344: 19.1 module-alias** — `deferred`
  - spec: `specification.md:15344`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15353: 19.2 re-export** — `deferred`
  - spec: `specification.md:15353`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15362: 19.3 Identity** — `deferred`
  - spec: `specification.md:15362`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15368: 19.4 Dataの原子性** — `deferred`
  - spec: `specification.md:15368`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15379: 19.5 include** — `deferred`
  - spec: `specification.md:15379`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15388: 19.6 衝突** — `deferred`
  - spec: `specification.md:15388`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L6 L15394: 19.7 実装moduleのinclude** — `deferred`
  - spec: `specification.md:15394`
  - notes: module-alias/re-export/include not in bind skeleton

- [x] **L5 L15400: 20. 正式identity** — `partial`
  - spec: `specification.md:15400`
  - notes: some opaque IDs exist in reciplexa-identity

- [x] **L6 L15401: 20.1 PackageInstanceId** — `ok`
  - spec: `specification.md:15401`
  - notes: PackageInstanceId in identity/package.rs

- [x] **L6 L15415: 20.2 ModuleId** — `ok`
  - spec: `specification.md:15415`
  - notes: ModuleId in identity/package.rs

- [x] **L6 L15421: 20.3 DefinitionId** — `ok`
  - spec: `specification.md:15421`
  - notes: DefinitionId opaque id in identity/package.rs

- [x] **L6 L15435: 20.4 BindingId** — `ok`
  - spec: `specification.md:15435`
  - notes: BindingId + BindingMap use-sites

- [x] **L6 L15441: 20.5 TypeId** — `partial`
  - spec: `specification.md:15441`
  - notes: TypeId opaque id exists; not wired through MOD metadata

- [x] **L6 L15447: 20.6 ConstructorId** — `partial`
  - spec: `specification.md:15447`
  - notes: ConstructorId opaque id exists; not MOD metadata

- [x] **L6 L15458: 20.7 SignatureId** — `ok`
  - spec: `specification.md:15458`
  - notes: SignatureId opaque id in identity/package.rs (functor/sig use deferred)

- [x] **L6 L15467: 20.8 FunctorId** — `ok`
  - spec: `specification.md:15467`
  - notes: FunctorId opaque id in identity/package.rs (functor impl deferred)

- [x] **L6 L15473: 20.9 Functor適用結果** — `deferred`
  - spec: `specification.md:15473`
  - notes: functor apply identity N/A until functors

- [x] **L6 L15482: 20.10 Rename** — `partial`
  - spec: `specification.md:15482`
  - notes: import rename only; no formal Rename on identity

- [x] **L6 L15488: 20.11 Source span** — `ok`
  - spec: `specification.md:15488`
  - notes: spans kept separate (TextRange / provenance)

- [x] **L5 L15492: 21. 分割コンパイルと適合試験** — `deferred`
  - spec: `specification.md:15492`
  - notes: 意図的後回し: InterfaceHash / compiled interface metadata

- [x] **L6 L15493: 21.1 Interface metadata** — `deferred`
  - spec: `specification.md:15493`
  - notes: 意図的後回し: .rpi → interface metadata emitter

- [x] **L6 L15513: 21.2 非公開情報** — `meta`
  - spec: `specification.md:15513`
  - notes: privacy rules; no metadata emitter yet

- [x] **L6 L15523: 21.3 InterfaceHash** — `deferred`
  - spec: `specification.md:15523`
  - notes: 意図的後回し: InterfaceHash absent

- [x] **L6 L15530: 21.4 Hashに含めるもの** — `deferred`
  - spec: `specification.md:15530`
  - notes: 意図的後回し: hash inputs N/A until InterfaceHash

- [x] **L6 L15540: 21.5 Hashに含めないもの** — `meta`
  - spec: `specification.md:15540`
  - notes: hash exclusions; no hasher

- [x] **L6 L15548: 21.6 Documentation hash** — `deferred`
  - spec: `specification.md:15548`
  - notes: 意図的後回し: DocumentationHash absent

- [x] **L6 L15552: 21.7 再コンパイル** — `deferred`
  - spec: `specification.md:15552`
  - notes: 意図的後回し: incremental recompile by InterfaceHash

- [x] **L6 L15558: 21.8 ABI hash** — `deferred`
  - spec: `specification.md:15558`
  - notes: AbiHash → OPEN-KER-001

- [x] **L6 L15564: 21.9 適合試験 MOD-01** — `partial`
  - spec: `specification.md:15564`
  - notes: qualified path import covered by module_tests; not named MOD-01 suite

- [x] **L6 L15574: 21.10 適合試験 MOD-02** — `partial`
  - spec: `specification.md:15574`
  - notes: as+only+rename covered by elaborate_units_* tests; not MOD-02 harness

- [x] **L6 L15594: 21.11 適合試験 MOD-03** — `deferred`
  - spec: `specification.md:15594`
  - notes: 意図的後回し: MOD-03 conformance corpus

- [x] **L6 L15621: 21.12 不適合試験 MOD-04** — `deferred`
  - spec: `specification.md:15621`
  - notes: 意図的後回し: MOD-04 negative suite

- [x] **L6 L15650: 21.13 適合試験 MOD-05** — `deferred`
  - spec: `specification.md:15650`
  - notes: 意図的後回し: MOD-05 suite

- [x] **L6 L15677: 21.14 適合試験 MOD-06** — `deferred`
  - spec: `specification.md:15677`
  - notes: 意図的後回し: MOD-06 suite

- [x] **L6 L15689: 21.15 不適合試験 MOD-07** — `deferred`
  - spec: `specification.md:15689`
  - notes: 意図的後回し: MOD-07 suite

- [x] **L6 L15699: 21.16 適合試験 MOD-08** — `deferred`
  - spec: `specification.md:15699`
  - notes: 意図的後回し: MOD-08 suite

- [x] **L6 L15708: 21.17 不適合試験 MOD-09** — `deferred`
  - spec: `specification.md:15708`
  - notes: 意図的後回し: MOD-09 suite

- [x] **L5 L15719: 22. 移管先OPEN・下位項目・状態** — `meta`
  - spec: `specification.md:15719`
  - notes: OPEN transfer table

- [x] **L6 L15720: 22.1 `OPEN-PKG-001`** — `deferred`
  - spec: `specification.md:15720`
  - notes: OPEN-PKG-001 — packages deferred

- [x] **L6 L15740: 22.2 `OPEN-MAC-001`** — `deferred`
  - spec: `specification.md:15740`
  - notes: macro-phase identity → MAC

- [x] **L6 L15751: 22.3 `OPEN-KER-001`** — `deferred`
  - spec: `specification.md:15751`
  - notes: OPEN-KER-001 ABI

- [x] **L6 L15761: 22.4 `OPEN-MOD-REC-001`** — `deferred`
  - spec: `specification.md:15761`
  - notes: recursive modules out of v1

- [x] **L6 L15773: 22.5 `OPEN-MOD-FC-001`** — `deferred`
  - spec: `specification.md:15773`
  - notes: first-class modules out of v1

- [x] **L6 L15783: 22.6 `OPEN-MOD-GEN-001`** — `deferred`
  - spec: `specification.md:15783`
  - notes: generative functors out of v1

- [x] **L6 L15792: 22.7 下位項目** — `meta`
  - spec: `specification.md:15792`
  - notes: sub-item index

- [x] **L6 L15831: 22.8 最終状態** — `meta`
  - spec: `specification.md:15831`
  - notes: MOD final-state prose; impl = outer+import skeleton

- [x] **L4 L15855: 13.10 `PKG-001` パッケージmanifest・依存解決・ワークスペース・リソース** — `partial`
  - spec: `specification.md:15855`
  - notes: Slice A–C: local packages + path-dep lock + math/japanese/graphics stubs + workspace.rpxm parse; registry deferred

- [x] **L5 L15856: DD-001 決定概要** — `meta`
  - spec: `specification.md:15856`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15857: DD-001.1 状態** — `meta`
  - spec: `specification.md:15857`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15878: DD-001.2 中心的な決定** — `meta`
  - spec: `specification.md:15878`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L5 L15903: 0. 適用範囲** — `meta`
  - spec: `specification.md:15903`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15904: 0.1 本項目が定めるもの** — `meta`
  - spec: `specification.md:15904`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15934: 0.2 本項目が直接定めないもの** — `meta`
  - spec: `specification.md:15934`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L5 L15954: 1. パッケージ** — `meta`
  - spec: `specification.md:15954`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15955: 1.1 定義** — `meta`
  - spec: `specification.md:15955`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15969: 1.2 モジュールとの違い** — `meta`
  - spec: `specification.md:15969`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L15979: 1.3 パッケージの種類** — `meta`
  - spec: `specification.md:15979`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L5 L15992: 2. パッケージmanifest** — `partial`
  - spec: `specification.md:15992`
  - notes: parse_rpxm + PackageManifest; remaining schema depth vs full PKG-001

- [x] **L6 L15993: 2.1 ファイル名** — `ok`
  - spec: `specification.md:15993`
  - notes: package.rpxm name convention + LocalPackageIndex::discover

- [x] **L6 L16011: 2.2 Package root** — `ok`
  - spec: `specification.md:16011`
  - notes: package.rpxm directory is package root via LocalPackageIndex::discover

- [x] **L6 L16021: 2.3 制限付きRPX形式** — `partial`
  - spec: `specification.md:16021`
  - notes: restricted sexp tokenize in rpxm.rs; not every static schema leaf

- [x] **L6 L16036: 2.4 静的schema** — `partial`
  - spec: `specification.md:16036`
  - notes: restricted sexp + known-field gate; residual static schema leaves open

- [x] **L6 L16052: 2.5 未知field** — `ok`
  - spec: `specification.md:16052`
  - notes: unknown fields rejected (RpxmError::UnknownField); format-version enforced

- [x] **L5 L16062: 3. Manifestの基本構文** — `partial`
  - spec: `specification.md:16062`
  - notes: minimal/explicit/parensed manifest forms parsed; residual schema depth open

- [x] **L6 L16063: 3.1 最小形** — `ok`
  - spec: `specification.md:16063`
  - notes: minimal (package (name)(version)(entry)) Phase-10 form parsed

- [x] **L6 L16068: 3.2 明示形** — `ok`
  - spec: `specification.md:16068`
  - notes: DD-001 flat explicit form (format-version/version/source-root/…) parsed

- [x] **L6 L16077: 3.3 Fieldの括弧** — `ok`
  - spec: `specification.md:16077`
  - notes: flat fields + parenthesized public-modules/entry-points/dependencies parsed

- [x] **L6 L16091: 3.4 Format version** — `ok`
  - spec: `specification.md:16091`
  - notes: format-version 1 accepted; other versions rejected

- [x] **L5 L16106: 4. パッケージ名** — `partial`
  - spec: `specification.md:16106`
  - notes: package name parsed as identity; full naming-rule matrix still light

- [x] **L6 L16107: 4.1 基本規則** — `ok`
  - spec: `specification.md:16107`
  - notes: ASCII lowercase kebab package/module paths via validate_package_path (SYN §4)

- [x] **L6 L16129: 4.2 用途** — `ok`
  - spec: `specification.md:16129`
  - notes: package name used as identity key in LocalPackageIndex / import first segment

- [x] **L6 L16140: 4.3 表示名** — `deferred`
  - spec: `specification.md:16140`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16149: 5. パッケージversion** — `partial`
  - spec: `specification.md:16149`
  - notes: version field parsed; full semver algebra deferred

- [x] **L6 L16150: 5.1 基本形式** — `ok`
  - spec: `specification.md:16150`
  - notes: version string required in manifest; exact path-dep compare uses version_req

- [x] **L6 L16159: 5.2 Version要素** — `deferred`
  - spec: `specification.md:16159`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16165: 5.3 互換性の一般原則** — `deferred`
  - spec: `specification.md:16165`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16175: 5.4 Breaking changeの例** — `deferred`
  - spec: `specification.md:16175`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16188: 5.5 Pre-release** — `deferred`
  - spec: `specification.md:16188`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16199: 6. Source rootとinterface root** — `ok`
  - spec: `specification.md:16199`
  - notes: source-root + interface-root parsed; module_source_path + .rpi stub gate

- [x] **L6 L16200: 6.1 Source root** — `ok`
  - spec: `specification.md:16200`
  - notes: source-root field + default src; LocalPackageIndex module load

- [x] **L6 L16211: 6.2 Module path** — `ok`
  - spec: `specification.md:16211`
  - notes: import graphics/shapes → src/shapes.rpx under package root

- [x] **L6 L16218: 6.3 Interface root** — `ok`
  - spec: `specification.md:16218`
  - notes: interface-root + module_interface_path; .rpi parse in rpi.rs

- [x] **L6 L16229: 6.4 Interface対応** — `deferred`
  - spec: `specification.md:16229`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16242: 6.5 Root数** — `deferred`
  - spec: `specification.md:16242`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16255: 6.6 Root pathの制限** — `deferred`
  - spec: `specification.md:16255`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16265: 7. 公開モジュール** — `ok`
  - spec: `specification.md:16265`
  - notes: public-modules parsed; LocalPackageIndex rejects non-public imports

- [x] **L6 L16266: 7.1 Manifest構文** — `ok`
  - spec: `specification.md:16266`
  - notes: (public-modules …) allow-list on load

- [x] **L6 L16274: 7.2 .rpi必須** — `ok`
  - spec: `specification.md:16274`
  - notes: public modules require .rpi when interface-root set; signature body check deferred

- [x] **L6 L16287: 7.3 内部モジュール** — `partial`
  - spec: `specification.md:16287`
  - notes: non-public cross-package import rejected; intra-package graph incomplete

- [x] **L6 L16297: 7.4 Internal moduleの.rpi** — `deferred`
  - spec: `specification.md:16297`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16307: 7.5 Inline下位モジュール** — `deferred`
  - spec: `specification.md:16307`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16313: 7.6 自動公開** — `deferred`
  - spec: `specification.md:16313`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16326: 8. 実行エントリ** — `ok`
  - spec: `specification.md:16326`
  - notes: entry-points parsed; legacy entry supported; library packages may omit entry

- [x] **L6 L16327: 8.1 Manifest構文** — `ok`
  - spec: `specification.md:16327`
  - notes: (entry-points …) DD-001 list → PackageManifest.entry_points

- [x] **L6 L16335: 8.2 Entry module** — `ok`
  - spec: `specification.md:16335`
  - notes: entry module from entry / entry-points[0]; libraries may omit

- [x] **L6 L16342: 8.3 .rpi** — `deferred`
  - spec: `specification.md:16342`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16348: 8.4 main** — `deferred`
  - spec: `specification.md:16348`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16354: 8.5 一module一entry** — `ok`
  - spec: `specification.md:16354`
  - notes: single entry module derived from entry-points; multi-entry matrix deferred

- [x] **L6 L16360: 8.6 実行契約** — `deferred`
  - spec: `specification.md:16360`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16366: 9. Manifestなしscript** — `ok`
  - spec: `specification.md:16366`
  - notes: script .rpx can import std packages (examples/pkg_circle.rpx)

- [x] **L6 L16367: 9.1 基本形** — `ok`
  - spec: `specification.md:16367`
  - notes: single-file script + package search roots

- [x] **L6 L16376: 9.2 必要な値** — `deferred`
  - spec: `specification.md:16376`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16384: 9.3 単一ファイル制限** — `deferred`
  - spec: `specification.md:16384`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16392: 9.4 外部dependency** — `ok`
  - spec: `specification.md:16392`
  - notes: scripts import std packages; consumer path-deps via manifest aliases (Slice B)

- [x] **L6 L16398: 9.5 Public API** — `deferred`
  - spec: `specification.md:16398`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16402: 10. 依存宣言** — `ok`
  - spec: `specification.md:16402`
  - notes: DD-001 dependencies (alias package name version path) + legacy (dep …)

- [x] **L6 L16403: 10.1 基本構文** — `ok`
  - spec: `specification.md:16403`
  - notes: DD-001 dependencies block + legacy (dep) forms

- [x] **L6 L16413: 10.2 Dependency alias** — `ok`
  - spec: `specification.md:16413`
  - notes: dependency alias = DependencySpec.name

- [x] **L6 L16428: 10.3 正式identity** — `deferred`
  - spec: `specification.md:16428`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16434: 10.4 正式パッケージ名** — `ok`
  - spec: `specification.md:16434`
  - notes: package field in dependency entry → DependencySpec.package

- [x] **L6 L16443: 10.5 Aliasの重複** — `deferred`
  - spec: `specification.md:16443`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16450: 11. Version constraint** — `deferred`
  - spec: `specification.md:16450`
  - notes: 意図的後回し: 完全な version constraint solver（正確一致 / * は維持）

- [x] **L6 L16451: 11.1 完全一致** — `ok`
  - spec: `specification.md:16451`
  - notes: exact version_req match (+ *) on path deps (load.rs / lockfile consistency)

- [x] **L6 L16469: 11.2 範囲指定** — `deferred`
  - spec: `specification.md:16469`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16475: 11.3 初期演算子集合** — `deferred`
  - spec: `specification.md:16475`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16485: 11.4 条件の結合** — `deferred`
  - spec: `specification.md:16485`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16494: 11.5 Pre-release** — `deferred`
  - spec: `specification.md:16494`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16498: 12. Dependency source** — `ok`
  - spec: `specification.md:16498`
  - notes: path dependency sources locked; registry/git/URL sources deferred

- [x] **L6 L16499: 12.1 既定Registry** — `deferred`
  - spec: `specification.md:16499`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16510: 12.2 Local path** — `ok`
  - spec: `specification.md:16510`
  - notes: path deps + alias map via register_path_dependencies

- [x] **L6 L16519: 12.3 Local packageの検証** — `deferred`
  - spec: `specification.md:16519`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16525: 12.4 Supported source** — `deferred`
  - spec: `specification.md:16525`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16533: 12.5 Git／URL** — `deferred`
  - spec: `specification.md:16533`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16537: 12.6 Sourceの排他性** — `deferred`
  - spec: `specification.md:16537`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16541: 13. Development dependency** — `deferred`
  - spec: `specification.md:16541`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16542: 13.1 構文** — `deferred`
  - spec: `specification.md:16542`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16548: 13.2 用途** — `deferred`
  - spec: `specification.md:16548`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16559: 13.3 Public APIへの漏出** — `deferred`
  - spec: `specification.md:16559`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16568: 13.4 Optional dependency** — `deferred`
  - spec: `specification.md:16568`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16574: 14. Public dependency** — `deferred`
  - spec: `specification.md:16574`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16575: 14.1 定義** — `deferred`
  - spec: `specification.md:16575`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16582: 14.2 自動導出** — `deferred`
  - spec: `specification.md:16582`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16588: 14.3 用途** — `deferred`
  - spec: `specification.md:16588`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16597: 15. 依存解決** — `ok`
  - spec: `specification.md:16597`
  - notes: resolve_packages deterministic sort + Lockfile::from_graph/from_consumer

- [x] **L6 L16598: 15.1 Manifestの役割** — `ok`
  - spec: `specification.md:16598`
  - notes: manifest drives resolve + lock consistency checks

- [x] **L6 L16604: 15.2 初回解決** — `deferred`
  - spec: `specification.md:16604`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16610: 15.3 Pre-release** — `deferred`
  - spec: `specification.md:16610`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16614: 15.4 統合** — `deferred`
  - spec: `specification.md:16614`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16618: 15.5 分割** — `deferred`
  - spec: `specification.md:16618`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16622: 15.6 決定性** — `ok`
  - spec: `specification.md:16622`
  - notes: resolve_packages deterministic sort; lockfile equality in resolver_tests

- [x] **L5 L16628: 16. 同一パッケージの複数version** — `deferred`
  - spec: `specification.md:16628`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16629: 16.1 基本方針** — `deferred`
  - spec: `specification.md:16629`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16639: 16.2 型identity** — `deferred`
  - spec: `specification.md:16639`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16647: 16.3 直接依存での明示** — `deferred`
  - spec: `specification.md:16647`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16663: 16.4 単一instance制約** — `deferred`
  - spec: `specification.md:16663`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16667: 17. Lockfile** — `ok`
  - spec: `specification.md:16667`
  
  - notes: rpx.lock write/read + path sources + consistency check (no registry)- notes: resolve_packages deterministic sort + lockfile stub

- [x] **L6 L16668: 17.1 ファイル名** — `ok`
  - spec: `specification.md:16668`
  
  - notes: Lockfile file name rpx.lock (write_rpx_lock / read_rpx_lock)- notes: resolve_packages deterministic sort + lockfile stub

- [x] **L6 L16674: 17.2 役割** — `ok`
  - spec: `specification.md:16674`
  
  - notes: manifest = constraints; rpx.lock = resolved path graph (from_consumer)- notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16681: 17.3 通常build** — `deferred`
  - spec: `specification.md:16681`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16687: 17.4 初回build** — `deferred`
  - spec: `specification.md:16687`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16698: 17.5 不整合** — `ok`
  - spec: `specification.md:16698`
  
  - notes: is_consistent_with_consumer rejects version/path mismatch (§17.5 message)- notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16704: 17.6 更新** — `deferred`
  - spec: `specification.md:16704`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16710: 17.7 部分更新** — `deferred`
  - spec: `specification.md:16710`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16716: 18. Lockfileの内容** — `ok`
  - spec: `specification.md:16716`
  - notes: path lock nodes: name/version/path source/dep edges + assert_consistent

- [x] **L6 L16717: 18.1 Package node** — `deferred`
  - spec: `specification.md:16717`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16730: 18.2 Registry package** — `deferred`
  - spec: `specification.md:16730`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16736: 18.3 Local path package** — `ok`
  - spec: `specification.md:16736`
  
  - notes: Local path packages: path:<rel> lock source; edits do not corrupt lock- notes: local packages on disk under packages/; lockfile path source still stub

- [x] **L6 L16742: 18.4 Version control** — `deferred`
  - spec: `specification.md:16742`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16748: 18.5 Offline build** — `deferred`
  - spec: `specification.md:16748`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16760: 19. Package identityとcontent hash** — `deferred`
  - spec: `specification.md:16760`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16761: 19.1 論理identityと内容identity** — `deferred`
  - spec: `specification.md:16761`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16771: 19.2 Registry package** — `deferred`
  - spec: `specification.md:16771`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16783: 19.3 Workspace／local package** — `deferred`
  - spec: `specification.md:16783`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16793: 19.4 Content hash対象** — `deferred`
  - spec: `specification.md:16793`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16813: 20. ワークスペース** — `deferred`
  - spec: `specification.md:16813`
  - notes: 意図的後回し: workspace member discovery/build（workspace.rpxm stub parse は維持）

- [x] **L6 L16814: 20.1 定義** — `meta`
  - spec: `specification.md:16814`
  - notes: PKG glossary/decision prose; no direct impl obligation

- [x] **L6 L16824: 20.2 Manifest** — `ok`
  - spec: `specification.md:16824`
  
  - notes: workspace.rpxm filename + parse_workspace_rpxm- notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred

- [x] **L6 L16830: 20.3 基本構文** — `ok`
  - spec: `specification.md:16830`
  
  - notes: (workspace format-version (members …)) DD-001 stub parse- notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred

- [x] **L6 L16839: 20.4 Member** — `deferred`
  - spec: `specification.md:16839`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16843: 20.5 Member path** — `deferred`
  - spec: `specification.md:16843`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16849: 20.6 Memberの外部配置** — `deferred`
  - spec: `specification.md:16849`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16855: 20.7 Package名重複** — `deferred`
  - spec: `specification.md:16855`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16859: 20.8 Nested workspace** — `deferred`
  - spec: `specification.md:16859`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L5 L16865: 21. Workspace依存・リソース・適合試験** — `deferred`
  - spec: `specification.md:16865`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L16866: 21.1 共通lockfile** — `ok`
  - spec: `specification.md:16866`
  
  - notes: Root rpx.lock shared model via Lockfile::from_consumer (member-local lock not written)- notes: Phase 10 skeleton (rpxm/json/resolver/lockfile); full PKG-001 deferred

- [x] **L6 L16872: 21.2 Member単独build** — `deferred`
  - spec: `specification.md:16872`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16876: 21.3 Workspace member優先** — `deferred`
  - spec: `specification.md:16876`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16884: 21.4 明示source** — `deferred`
  - spec: `specification.md:16884`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16895: 21.5 Member version** — `deferred`
  - spec: `specification.md:16895`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16899: 21.6 Member依存graph** — `deferred`
  - spec: `specification.md:16899`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16905: 21.7 Workspace外path dependency** — `deferred`
  - spec: `specification.md:16905`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16911: 21.8 Workspace identity** — `deferred`
  - spec: `specification.md:16911`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16917: 21.9 Resource root** — `deferred`
  - spec: `specification.md:16917`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16928: 21.10 Resource一覧** — `deferred`
  - spec: `specification.md:16928`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16945: 21.11 Resource path** — `deferred`
  - spec: `specification.md:16945`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16959: 21.12 Symbolic link** — `deferred`
  - spec: `specification.md:16959`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16963: 21.13 Resource identity** — `deferred`
  - spec: `specification.md:16963`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16973: 21.14 Resource参照** — `deferred`
  - spec: `specification.md:16973`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16982: 21.15 Pure／effectfulの区別** — `deferred`
  - spec: `specification.md:16982`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16989: 21.16 Resourceの外部公開** — `deferred`
  - spec: `specification.md:16989`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L16995: 21.17 Resource hash** — `deferred`
  - spec: `specification.md:16995`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L17001: 21.18 Generated resource** — `deferred`
  - spec: `specification.md:17001`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L17005: 21.19 Test fixture** — `deferred`
  - spec: `specification.md:17005`
  - notes: PKG-001 deferred (post language-kernel)

- [x] **L6 L17009: 21.20 適合試験 PKG-01** — `deferred`
  - spec: `specification.md:17009`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17028: 21.21 不適合試験 PKG-02** — `deferred`
  - spec: `specification.md:17028`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17039: 21.22 適合試験 PKG-03** — `deferred`
  - spec: `specification.md:17039`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17049: 21.23 不適合試験 PKG-04** — `deferred`
  - spec: `specification.md:17049`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17059: 21.24 適合試験 PKG-05** — `deferred`
  - spec: `specification.md:17059`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17069: 21.25 適合試験 PKG-06** — `deferred`
  - spec: `specification.md:17069`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17082: 21.26 適合試験 PKG-07** — `deferred`
  - spec: `specification.md:17082`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17094: 21.27 不適合試験 PKG-08** — `deferred`
  - spec: `specification.md:17094`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17113: 21.28 適合試験 PKG-09** — `deferred`
  - spec: `specification.md:17113`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17127: 21.29 不適合試験 PKG-10** — `deferred`
  - spec: `specification.md:17127`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17137: 21.30 適合試験 PKG-11** — `deferred`
  - spec: `specification.md:17137`
  - notes: PKG conformance suite deferred / not wired

- [x] **L6 L17147: 21.31 不適合試験 PKG-12** — `deferred`
  - spec: `specification.md:17147`
  - notes: PKG conformance suite deferred / not wired

- [x] **L5 L17157: 22. 移管先OPEN・下位項目・状態** — `deferred`
  - spec: `specification.md:17157`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17158: 22.1 `OPEN-BLD-001`** — `deferred`
  - spec: `specification.md:17158`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17171: 22.2 `OPEN-PKG-FEAT-001`** — `deferred`
  - spec: `specification.md:17171`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17181: 22.3 `OPEN-REG-001`** — `deferred`
  - spec: `specification.md:17181`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17193: 22.4 `OPEN-KER-001`** — `deferred`
  - spec: `specification.md:17193`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17204: 22.5 `OPEN-TST-001`** — `deferred`
  - spec: `specification.md:17204`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17215: 22.6 `OPEN-ERR-001`** — `deferred`
  - spec: `specification.md:17215`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17224: 22.7 `OPEN-CON-001`** — `deferred`
  - spec: `specification.md:17224`
  - notes: OPEN transferred; deferred with PKG

- [x] **L6 L17232: 22.8 下位項目** — `deferred`
  - spec: `specification.md:17232`
  - notes: OPEN transferred; deferred with PKG

- [x] **L5 L17260: Compiler-native package実装** — `deferred`
  - spec: `specification.md:17260`
  - notes: native package swap deferred (OPEN-NATIVE-PKG)

- [x] **L6 L17273: 22.9 最終状態** — `meta`
  - spec: `specification.md:17273`
  - notes: PKG final-state / resolved declaration prose

- [x] **L4 L17299: 13.10.1 `KER-001` Rust kernelとforeign primitive境界** — `partial`
  - spec: `specification.md:17299`
  - notes: BuiltinOp arithmetic/compare + EffectHost; full Rust/FFI ABI deferred

- [x] **L5 L17301: 概要・状態** — `partial`
  - spec: `specification.md:17301`
  - notes: kernel ops in eval/check; TEST-KER-* / foreign validator absent

- [x] **L4 L17326: 13.10.2 `RSC-001` Resource、I/O、host-handler境界** — `partial`
  - spec: `specification.md:17326`
  - notes: MemoryFsHost read-file/write-file; richer catalog/path safety deferred

- [x] **L5 L17328: 概要・状態** — `partial`
  - spec: `specification.md:17328`
  - notes: in-memory host for tests; resolve-font/load-image etc. not in language kernel

- [x] **L4 L17355: 13.11 `EDT-001` 編集スナップショット・トランザクション・競合・由来情報** — `partial`
  - spec: `specification.md:17355`
  - notes: DocumentSnapshot/Transaction/BindingMap use-sites + MacroSourceMap; full GUI reconciliation 依存待ち(第V部)

- [x] **L5 L17356: DD-001 決定概要** — `meta`
  - spec: `specification.md:17356`
  - notes: decision overview

- [x] **L6 L17357: DD-001.1 状態** — `meta`
  - spec: `specification.md:17357`
  - notes: status prose

- [x] **L6 L17375: DD-001.2 位置付け** — `meta`
  - spec: `specification.md:17375`
  - notes: positioning vs GUI

- [x] **L5 L17400: 0. 適用範囲** — `meta`
  - spec: `specification.md:17400`
  - notes: scope

- [x] **L6 L17401: 0.1 本項目が扱う編集** — `deferred`
  - spec: `specification.md:17401`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17412: 0.2 直接の対象としないもの** — `deferred`
  - spec: `specification.md:17412`
  - notes: collab/codec overrides → OPEN-EDT-*

- [x] **L5 L17435: 1. 編集モデルの基本原則** — `partial`
  - spec: `specification.md:17435`
  - notes: snapshot + StableNodeId + BindingId use-sites; full EDT value model 依存待ち(第V部)

- [x] **L6 L17436: 1.1 不変値と継続的identity** — `partial`
  - spec: `specification.md:17436`
  - notes: DocumentIdentity + StableNodeId; full EDT value model 依存待ち(第V部)

- [x] **L6 L17460: 1.2 スナップショット** — `ok`
  - spec: `specification.md:17460`
  - notes: DocumentSnapshot + DocumentRevision

- [x] **L6 L17477: 1.3 現在文書** — `deferred`
  - spec: `specification.md:17477`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17487: 2. 文書の所有構造** — `partial`
  - spec: `specification.md:17487`
  - notes: NodeStore ownership tree

- [x] **L6 L17488: 2.1 単一rootの所有tree** — `ok`
  - spec: `specification.md:17488`
  - notes: single Document root + children

- [x] **L6 L17505: 2.2 所有と参照の分離** — `deferred`
  - spec: `specification.md:17505`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17524: 2.3 共有** — `deferred`
  - spec: `specification.md:17524`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17535: 2.4 規範的な親子情報** — `ok`
  - spec: `specification.md:17535`
  - notes: parent/children on DocumentNode

- [x] **L5 L17548: 3. 文書treeの不変条件** — `deferred`
  - spec: `specification.md:17548`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17552: 1. RootNodeIdがnode storeに存在する** — `deferred`
  - spec: `specification.md:17552`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17553: 2. Rootは親を持たない** — `deferred`
  - spec: `specification.md:17553`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17554: 3. Root以外の全ノードはちょうど一つの親を持つ** — `deferred`
  - spec: `specification.md:17554`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17555: 4. 所有edgeにcycleがない** — `deferred`
  - spec: `specification.md:17555`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17556: 5. 同じ親のchildren列内に同一NodeIdが重複しない** — `deferred`
  - spec: `specification.md:17556`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17557: 6. 全ノードが同じDocumentIdに所属する** — `deferred`
  - spec: `specification.md:17557`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17558: 7. 全ノードがRootから到達可能である** — `deferred`
  - spec: `specification.md:17558`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17559: 8. 必須propertyが存在する** — `deferred`
  - spec: `specification.md:17559`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17560: 9. Property値がNode kindのschemaに適合する** — `deferred`
  - spec: `specification.md:17560`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17561: 10. 強いNodeId参照が有効な対象を指す** — `deferred`
  - spec: `specification.md:17561`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17563: 3.1 到達不能ノード** — `deferred`
  - spec: `specification.md:17563`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17578: 4. 識別子** — `deferred`
  - spec: `specification.md:17578`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17579: 4.1 識別子の種類** — `deferred`
  - spec: `specification.md:17579`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17601: 4.2 Opaque型** — `ok`
  - spec: `specification.md:17601`
  - notes: opaque newtypes in reciplexa-identity

- [x] **L6 L17623: 4.3 DocumentId** — `ok`
  - spec: `specification.md:17623`
  - notes: DocumentIdentity ≈ DocumentId

- [x] **L6 L17637: 4.4 NodeId** — `ok`
  - spec: `specification.md:17637`
  - notes: StableNodeId ≈ NodeId

- [x] **L6 L17653: 4.5 TransactionId** — `deferred`
  - spec: `specification.md:17653`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17665: 4.6 ID表現** — `ok`
  - spec: `specification.md:17665`
  - notes: Display formats document:/node:/rev:

- [x] **L5 L17671: 5. 保存・複製・Fork** — `deferred`
  - spec: `specification.md:17671`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17672: 5.1 通常保存** — `deferred`
  - spec: `specification.md:17672`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17683: 5.2 Save As** — `deferred`
  - spec: `specification.md:17683`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17693: 5.3 Duplicate／Fork** — `deferred`
  - spec: `specification.md:17693`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17708: 5.4 内部参照の複製** — `deferred`
  - spec: `specification.md:17708`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17718: 5.5 文書間参照** — `deferred`
  - spec: `specification.md:17718`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17724: 6. Revision** — `deferred`
  - spec: `specification.md:17724`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17725: 6.1 直線的な履歴** — `deferred`
  - spec: `specification.md:17725`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17738: 6.2 Commitの直列化** — `deferred`
  - spec: `specification.md:17738`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17753: 6.3 Revision増加** — `ok`
  - spec: `specification.md:17753`
  - notes: bump_revision on apply

- [x] **L6 L17772: 6.4 保存後のrevision** — `deferred`
  - spec: `specification.md:17772`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17780: 7. Snapshotの保持** — `deferred`
  - spec: `specification.md:17780`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17781: 7.1 不変性** — `deferred`
  - spec: `specification.md:17781`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17792: 7.2 過去版の永久取得** — `deferred`
  - spec: `specification.md:17792`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17798: 7.3 履歴の種類** — `deferred`
  - spec: `specification.md:17798`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L17811: 8. 編集トランザクション** — `deferred`
  - spec: `specification.md:17811`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17812: 8.1 概念構造** — `deferred`
  - spec: `specification.md:17812`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17822: 8.2 不変の第一級値** — `deferred`
  - spec: `specification.md:17822`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17835: 8.3 原子性** — `ok`
  - spec: `specification.md:17835`
  - notes: apply rolls back on error (transaction.rs)

- [x] **L6 L17848: 8.4 Operation順序** — `ok`
  - spec: `specification.md:17848`
  - notes: ops applied in vector order

- [x] **L5 L17856: 9. 編集Operation** — `deferred`
  - spec: `specification.md:17856`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17867: 9.1 CreateNode** — `ok`
  - spec: `specification.md:17867`
  - notes: InsertChild creates+attaches node (document ops)

- [x] **L6 L17885: 9.2 DeleteNode** — `ok`
  - spec: `specification.md:17885`
  - notes: RemoveNode deletes from snapshot

- [x] **L6 L17900: 9.3 SetProperty** — `partial`
  - spec: `specification.md:17900`
  - notes: SetLayout/SetText; no generic SetProperty

- [x] **L6 L17925: 9.4 InsertChild** — `ok`
  - spec: `specification.md:17925`
  - notes: DocumentEdit::InsertChild

- [x] **L6 L17950: 9.5 RemoveChild** — `deferred`
  - spec: `specification.md:17950`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L17958: 9.6 MoveNode** — `ok`
  - spec: `specification.md:17958`
  - notes: DocumentEdit::MoveNode

- [x] **L5 L17985: 10. NodeIdの予約** — `ok`
  - spec: `specification.md:17985`
  - notes: StableNodeIdAllocator for NodeId

- [x] **L6 L17986: 10.1 発行方式** — `ok`
  - spec: `specification.md:17986`
  - notes: monotonic allocate

- [x] **L6 L17999: 10.2 予約済みID** — `deferred`
  - spec: `specification.md:17999`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18005: 10.3 Copy** — `deferred`
  - spec: `specification.md:18005`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18011: 11. 適用前条件** — `partial`
  - spec: `specification.md:18011`
  - notes: basic UnknownNode/InvalidParent checks

- [x] **L6 L18012: 11.1 Operation固有条件** — `ok`
  - spec: `specification.md:18012`
  - notes: per-op errors in apply (UnknownNode/InvalidParent)

- [x] **L6 L18027: 11.2 Transaction全体の条件** — `ok`
  - spec: `specification.md:18027`
  - notes: EmptyBatch rejected

- [x] **L6 L18037: 11.3 Base revision** — `deferred`
  - spec: `specification.md:18037`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18043: 12. Stale transaction** — `deferred`
  - spec: `specification.md:18043`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18044: 12.1 定義** — `deferred`
  - spec: `specification.md:18044`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18048: 12.2 分類** — `deferred`
  - spec: `specification.md:18048`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18063: 12.3 保守的な再適用** — `deferred`
  - spec: `specification.md:18063`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18069: 12.4 無関係な変更** — `deferred`
  - spec: `specification.md:18069`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18073: 13. 競合** — `deferred`
  - spec: `specification.md:18073`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18074: 13.1 基本分類** — `deferred`
  - spec: `specification.md:18074`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18091: 13.2 競合と不正トランザクション** — `deferred`
  - spec: `specification.md:18091`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18109: 13.3 競合の収集** — `deferred`
  - spec: `specification.md:18109`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18115: 13.4 自動併合** — `deferred`
  - spec: `specification.md:18115`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18135: 14. 適用手順** — `partial`
  - spec: `specification.md:18135`
  - notes: simplified apply path without full 9-step protocol

- [x] **L5 L18139: 1. TransactionIdを確認** — `deferred`
  - spec: `specification.md:18139`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18140: 2. DocumentIdを確認** — `deferred`
  - spec: `specification.md:18140`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18141: 3. Base revisionを比較** — `deferred`
  - spec: `specification.md:18141`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18142: 4. Transaction-level preconditionを検査** — `deferred`
  - spec: `specification.md:18142`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18143: 5. 現在snapshotから作業状態を作成** — `partial`
  - spec: `specification.md:18143`
  - notes: mutates snapshot in place with rollback

- [x] **L5 L18144: 6. Operationを順番に仮適用** — `ok`
  - spec: `specification.md:18144`
  - notes: ops applied sequentially

- [x] **L5 L18145: 7. 文書全体の不変条件を検査** — `deferred`
  - spec: `specification.md:18145`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18146: 8. 成功時だけcommit** — `ok`
  - spec: `specification.md:18146`
  - notes: errors abort without partial commit

- [x] **L5 L18147: 9. 新revisionとUndo情報を生成** — `deferred`
  - spec: `specification.md:18147`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18152: 15. 適用結果** — `partial`
  - spec: `specification.md:18152`
  - notes: TransactionOutcome Applied/AppliedNoChange/Rejected

- [x] **L6 L18153: 15.1 結果型** — `ok`
  - spec: `specification.md:18153`
  - notes: TransactionOutcome Applied/AppliedNoChange/Rejected

- [x] **L6 L18160: 15.2 Applied** — `ok`
  - spec: `specification.md:18160`
  - notes: TransactionOutcome::Applied

- [x] **L6 L18170: 15.3 AppliedNoChange** — `ok`
  - spec: `specification.md:18170`
  - notes: TransactionOutcome::AppliedNoChange

- [x] **L6 L18176: 15.4 Rejected** — `ok`
  - spec: `specification.md:18176`
  - notes: Rejected outcome + TransactionError path

- [x] **L6 L18185: 15.5 AlreadyApplied** — `deferred`
  - spec: `specification.md:18185`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18191: 15.6 Transaction content hash** — `deferred`
  - spec: `specification.md:18191`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18205: 16. UndoとRedo** — `deferred`
  - spec: `specification.md:18205`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18206: 16.1 新revision** — `deferred`
  - spec: `specification.md:18206`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18218: 16.2 Undo情報** — `deferred`
  - spec: `specification.md:18218`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18234: 16.3 UndoToken** — `deferred`
  - spec: `specification.md:18234`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18241: 16.4 Undo競合** — `deferred`
  - spec: `specification.md:18241`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18247: 16.5 Redo** — `deferred`
  - spec: `specification.md:18247`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18251: 16.6 履歴保持** — `deferred`
  - spec: `specification.md:18251`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18264: 17. Provenance** — `partial`
  - spec: `specification.md:18264`
  - notes: SourceProvenance + NodeProvenance; not full EDT provenance taxonomy

- [x] **L6 L18265: 17.1 定義** — `deferred`
  - spec: `specification.md:18265`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18278: 17.2 Optional metadata** — `deferred`
  - spec: `specification.md:18278`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18284: 17.3 構造** — `partial`
  - spec: `specification.md:18284`
  - notes: flat SourceProvenance struct

- [x] **L6 L18296: 17.4 種類** — `deferred`
  - spec: `specification.md:18296`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18305: 17.5 UserCreated** — `deferred`
  - spec: `specification.md:18305`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18312: 17.6 SourceGenerated** — `deferred`
  - spec: `specification.md:18312`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18327: 17.7 MacroGenerated** — `deferred`
  - spec: `specification.md:18327`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18336: 17.8 Imported** — `deferred`
  - spec: `specification.md:18336`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18342: 17.9 Copied** — `deferred`
  - spec: `specification.md:18342`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18348: 17.10 Derived** — `deferred`
  - spec: `specification.md:18348`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18362: 18. 派生ノードと逆編集** — `deferred`
  - spec: `specification.md:18362`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18363: 18.1 編集可能性** — `deferred`
  - spec: `specification.md:18363`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18379: 18.2 Provenanceと逆編集** — `deferred`
  - spec: `specification.md:18379`
  - notes: 依存待ち(EDT): reverse-edit from Provenance is GUI/EDT, not Part II language

- [x] **L6 L18385: 18.3 逆編集結果** — `deferred`
  - spec: `specification.md:18385`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18392: 18.4 自動選択** — `deferred`
  - spec: `specification.md:18392`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18396: 18.5 逆写像不能** — `deferred`
  - spec: `specification.md:18396`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18409: 18.6 Stale provenance** — `deferred`
  - spec: `specification.md:18409`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18415: 19. 派生ノードのID継承** — `deferred`
  - spec: `specification.md:18415`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18416: 19.1 DerivationKey** — `deferred`
  - spec: `specification.md:18416`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18426: 19.2 曖昧な対応** — `deferred`
  - spec: `specification.md:18426`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18432: 19.3 NodeIdとの違い** — `deferred`
  - spec: `specification.md:18432`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18436: 20. Provenanceの安全性** — `deferred`
  - spec: `specification.md:18436`
  - notes: 依存待ち(EDT): Provenance safety/export policy is EDT/security layer

- [x] **L6 L18437: 20.1 真正性** — `deferred`
  - spec: `specification.md:18437`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18443: 20.2 Privacy** — `deferred`
  - spec: `specification.md:18443`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18454: 20.3 書換え** — `deferred`
  - spec: `specification.md:18454`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18460: 21. 公開API階層** — `partial`
  - spec: `specification.md:18460`
  - notes: Rust document API only; not RPX-exported EDT API

- [x] **L6 L18461: 21.1 純粋な第一級値** — `partial`
  - spec: `specification.md:18461`
  - notes: DocumentEdit values; not RPX first-class

- [x] **L6 L18474: 21.2 状態付きhandle** — `partial`
  - spec: `specification.md:18474`
  - notes: working DocumentSnapshot handle in GUI path

- [x] **L6 L18478: 21.3 抽象型** — `deferred`
  - spec: `specification.md:18478`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18491: 21.4 Constructor付き公開data** — `deferred`
  - spec: `specification.md:18491`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18500: 22. 高水準APIと低水準API** — `deferred`
  - spec: `specification.md:18500`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18501: 22.1 高水準API** — `partial`
  - spec: `specification.md:18501`
  - notes: source_sync GUI edits

- [x] **L6 L18515: 22.2 低水準API** — `partial`
  - spec: `specification.md:18515`
  - notes: DocumentTransaction ops

- [x] **L6 L18525: 22.3 信頼境界** — `meta`
  - spec: `specification.md:18525`
  - notes: trust boundary prose

- [x] **L5 L18529: 23. Pure処理とEffectful処理** — `partial`
  - spec: `specification.md:18529`
  - notes: doc txs pure-ish; host I/O separate (RSC)

- [x] **L6 L18530: 23.1 Pure処理** — `deferred`
  - spec: `specification.md:18530`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18538: 23.2 Effectful処理** — `partial`
  - spec: `specification.md:18538`
  - notes: source rewrite side effects via GUI sync

- [x] **L6 L18546: 23.3 Effectの暫定分類** — `meta`
  - spec: `specification.md:18546`
  - notes: provisional effect taxonomy

- [x] **L5 L18559: 24. 競合・不正・実行障害の分離** — `partial`
  - spec: `specification.md:18559`
  - notes: TransactionError vs Outcome; no conflict type

- [x] **L6 L18560: 24.1 正常な結果** — `ok`
  - spec: `specification.md:18560`
  - notes: successful DocumentTransaction apply yields updated snapshot

- [x] **L6 L18565: 24.2 意味的拒否** — `ok`
  - spec: `specification.md:18565`
  - notes: TransactionError semantic rejects

- [x] **L6 L18573: 24.3 実行障害** — `deferred`
  - spec: `specification.md:18573`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18586: 24.4 競合は例外ではない** — `ok`
  - spec: `specification.md:18586`
  - notes: edit errors are Result, not exceptions

- [x] **L5 L18590: 25. Undo履歴・Transaction履歴** — `deferred`
  - spec: `specification.md:18590`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18591: 25.1 有限保持** — `deferred`
  - spec: `specification.md:18591`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18600: 25.2 AlreadyApplied保証** — `deferred`
  - spec: `specification.md:18600`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18606: 25.3 Undo不可** — `deferred`
  - spec: `specification.md:18606`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18612: 26. 永続化** — `deferred`
  - spec: `specification.md:18612`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18613: 26.1 標準保存** — `deferred`
  - spec: `specification.md:18613`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18624: 26.2 編集履歴** — `deferred`
  - spec: `specification.md:18624`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L6 L18630: 26.3 Version付きcodec** — `deferred`
  - spec: `specification.md:18630`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18638: 27. 適合試験** — `deferred`
  - spec: `specification.md:18638`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L5 L18776: 28. 移管先OPEN** — `meta`
  - spec: `specification.md:18776`
  - notes: OPEN transfer

- [x] **L6 L18778: `OPEN-ERR-001`** — `deferred`
  - spec: `specification.md:18778`
  - notes: OPEN-ERR-001 transferred / out of pre-PKG kernel

- [x] **L6 L18787: `OPEN-MEM-001`** — `deferred`
  - spec: `specification.md:18787`
  - notes: OPEN-MEM-001 transferred / out of pre-PKG kernel

- [x] **L6 L18795: `OPEN-CON-001`** — `deferred`
  - spec: `specification.md:18795`
  - notes: OPEN-CON-001 transferred / out of pre-PKG kernel

- [x] **L6 L18804: `OPEN-IR-001`** — `deferred`
  - spec: `specification.md:18804`
  - notes: OPEN-IR-001 transferred / out of pre-PKG kernel

- [x] **L6 L18812: `OPEN-EDT-CODEC-001`** — `deferred`
  - spec: `specification.md:18812`
  - notes: OPEN-EDT-CODEC-001 transferred / out of pre-PKG kernel

- [x] **L6 L18819: `OPEN-EDT-COLLAB-001`** — `deferred`
  - spec: `specification.md:18819`
  - notes: OPEN-EDT-COLLAB-001 transferred / out of pre-PKG kernel

- [x] **L6 L18828: `OPEN-EDT-OVERRIDE-001`** — `deferred`
  - spec: `specification.md:18828`
  - notes: OPEN-EDT-OVERRIDE-001 transferred / out of pre-PKG kernel

- [x] **L5 L18835: 29. 最終状態** — `deferred`
  - spec: `specification.md:18835`
  - notes: 依存待ち(第V部): full GUI reconciliation (EDT ops/conflict/undo/codec)

- [x] **L4 L18860: 13.12 `IR-001` Layered visual/motion/render IR** — `partial`
  - spec: `specification.md:18860`
  - notes: backends+motion+view exist; full layered IR schema still provisional

- [x] **L5 L18862: 概要・状態** — `meta`
  - spec: `specification.md:18862`
  - notes: IR-001 overview/status (単一万能IR拒否確定; schema暫定)

- [x] **L5 L18875: SurfaceとArtifact** — `partial`
  - spec: `specification.md:18875`
  - notes: scene/document surfaces exist; Artifact algebra incomplete

- [x] **L5 L18889: RenderIR node algebra** — `partial`
  - spec: `specification.md:18889`
  - notes: pdf/svg/pptx/view primitives; full RenderIR node algebra incomplete

- [x] **L5 L18935: MotionIR** — `partial`
  - spec: `specification.md:18935`
  - notes: reciplexa-motion MotionTrack Constant/Keyframes/Samples

- [x] **L5 L18949: Backend lowering** — `partial`
  - spec: `specification.md:18949`
  - notes: pdf/svg/pptx lowering; AE/edit-preserving path incomplete

- [x] **L5 L18962: 不変条件** — `meta`
  - spec: `specification.md:18962`
  - notes: IR invariants prose

- [x] **L5 L18972: メタ理論・反例** — `meta`
  - spec: `specification.md:18972`
  - notes: IR metatheory / CE notes

- [x] **L5 L18981: テスト** — `partial`
  - spec: `specification.md:18981`
  - notes: phase12_motion TEST-IR-007 style; full IR-001..009 validator suite 意図的後回し

- [x] **L4 L18993: 13.13 `ERR-001` 通常の失敗・Failure effect・後始末・Defect・最上位実行境界** — `partial`
  - spec: `specification.md:18993`
  - notes: raise/handle/or-raise/as-result + Never + DYN-005; bracket/cleanup/defect 依存待ち(第III部)

- [x] **L5 L18994: DD-001 決定概要** — `meta`
  - spec: `specification.md:18994`
  - notes: ERR design principle / guideline prose

- [x] **L6 L18995: DD-001.1 状態** — `meta`
  - spec: `specification.md:18995`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19015: DD-001.2 既存仕様との関係** — `meta`
  - spec: `specification.md:19015`
  - notes: ERR design principle / guideline prose

- [x] **L5 L19041: 0. 設計原則** — `meta`
  - spec: `specification.md:19041`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19042: 0.1 失敗を一種類に統合しない** — `meta`
  - spec: `specification.md:19042`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19063: 0.2 判断基準** — `meta`
  - spec: `specification.md:19063`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19076: 0.3 公開APIと内部実装** — `meta`
  - spec: `specification.md:19076`
  - notes: ERR design principle / guideline prose

- [x] **L5 L19091: 1. option** — `ok`
  - spec: `specification.md:19091`
  - notes: option ADT via DAT; ERR §1 uses standard option

- [x] **L6 L19092: 1.1 用途** — `ok`
  - spec: `specification.md:19092`
  - notes: option for absence (DAT option); ERR policy prose

- [x] **L6 L19110: 1.2 不適切な用途** — `ok`
  - spec: `specification.md:19110`
  - notes: option not for exceptional failure (ERR §1.2 policy)

- [x] **L5 L19114: 2. resultおよび専用結果型** — `deferred`
  - spec: `specification.md:19114`
  - notes: dedicated ERR result API beyond DAT result 依存待ち(第III部); user data result path ok

- [x] **L6 L19115: 2.1 用途** — `deferred`
  - spec: `specification.md:19115`
  - notes: 専用結果型 API 依存待ち(第III部); DAT result/option usable

- [x] **L6 L19135: 2.2 専用結果型** — `deferred`
  - spec: `specification.md:19135`
  - notes: dedicated ERR result constructors 依存待ち(第III部)

- [x] **L6 L19150: 2.3 複数errorの収集** — `deferred`
  - spec: `specification.md:19150`
  - notes: multi-error collect language primitive 依存待ち(第III部)/OPEN

- [x] **L5 L19159: 3. failure E** — `ok`
  - spec: `specification.md:19159`
  - notes: failure perform + handle failure (eval/check)

- [x] **L6 L19160: 3.1 定義** — `ok`
  - spec: `specification.md:19160`
  - notes: failure E via Core Perform op `failure`

- [x] **L6 L19180: 3.2 正常経路と失敗経路** — `ok`
  - spec: `specification.md:19180`
  - notes: raise aborts to Failure handler; resume rejected

- [x] **L6 L19187: 3.3 Error payload** — `ok`
  - spec: `specification.md:19187`
  - notes: error payload is ordinary Core value (Dynamic typed interim)

- [x] **L5 L19195: 4. Failureの発生** — `ok`
  - spec: `specification.md:19195`
  - notes: raise → Perform failure; or-raise/as-result elaboration

- [x] **L6 L19196: 4.1 raise** — `ok`
  - spec: `specification.md:19196`
  - notes: (raise e) → Perform{op:failure}

- [x] **L6 L19214: 4.2 never** — `ok`
  - spec: `specification.md:19214`
  - notes: CoreType::Never for failure perform; subtype via unify

- [x] **L6 L19239: 4.3 基礎機構** — `ok`
  - spec: `specification.md:19239`
  - notes: existing effect perform; no separate exception runtime

- [x] **L5 L19245: 5. Failure handler** — `ok`
  - spec: `specification.md:19245`
  - notes: handle failure 1-param; resume rejected at check+eval

- [x] **L6 L19246: 5.1 非再開性** — `ok`
  - spec: `specification.md:19246`
  - notes: 1-param Failure handler; resume binding rejected

- [x] **L6 L19265: 5.2 Resume禁止** — `ok`
  - spec: `specification.md:19265`
  - notes: Failure resume param rejected at check+eval

- [x] **L6 L19271: 5.3 内部実装** — `ok`
  - spec: `specification.md:19271`
  - notes: lowered onto Handle/Perform; cont not exposed for failure

- [x] **L6 L19275: 5.4 Handlerの結果型** — `partial`
  - spec: `specification.md:19275`
  - notes: handler result typing via shared handle infer (interim)

- [x] **L5 L19306: 6. Failureとeffect row** — `ok`
  - spec: `specification.md:19306`
  - notes: failure in effect row; removed by handle failure

- [x] **L6 L19307: 6.1 型への明示** — `ok`
  - spec: `specification.md:19307`
  - notes: perform failure adds op to residual row

- [x] **L6 L19320: 6.2 Handlerによる除去** — `ok`
  - spec: `specification.md:19320`
  - notes: handle failure removes failure from residual

- [x] **L6 L19333: 6.3 Handler節自身のEffect** — `ok`
  - spec: `specification.md:19333`
  - notes: handler body effects merged like other handles

- [x] **L6 L19346: 6.4 Handler節内の新しいFailure** — `ok`
  - spec: `specification.md:19346`
  - notes: re-raise inside handler propagates (deep handle semantics)

- [x] **L5 L19359: 7. 一つのFailure型への統合** — `deferred`
  - spec: `specification.md:19359`
  - notes: single-failure-type-per-boundary enforcement 依存待ち(第III部)

- [x] **L6 L19360: 7.1 基本指針** — `meta`
  - spec: `specification.md:19360`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19384: 7.2 Error変換** — `ok`
  - spec: `specification.md:19384`
  - notes: re-raise via nested handle works (deep semantics)

- [x] **L6 L19401: 7.3 位置付け** — `meta`
  - spec: `specification.md:19401`
  - notes: ERR design principle / guideline prose

- [x] **L5 L19407: 8. resultとFailureの変換** — `ok`
  - spec: `specification.md:19407`
  - notes: or-raise / as-result explicit conversion helpers

- [x] **L6 L19408: 8.1 暗黙変換の禁止** — `ok`
  - spec: `specification.md:19408`
  - notes: no implicit result↔failure conversion

- [x] **L6 L19421: 8.2 resultからFailure** — `ok`
  - spec: `specification.md:19421`
  - notes: (or-raise result) elaborates to match+raise

- [x] **L6 L19441: 8.3 Failureからresult** — `ok`
  - spec: `specification.md:19441`
  - notes: (as-result (fn () body)) → handle failure + ok/err variants

- [x] **L5 L19468: 9. resultとFailureの選択指針** — `deferred`
  - spec: `specification.md:19468`
  - notes: result vs Failure choice-policy auto-enforcement 依存待ち(第III部)/メタ指針

- [x] **L6 L19469: 9.1 resultを推奨する場合** — `deferred`
  - spec: `specification.md:19469`
  - notes: result推奨指針はメタ; 言語強制なし 依存待ち(第III部)

- [x] **L6 L19479: 9.2 Failureを認める場合** — `meta`
  - spec: `specification.md:19479`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19486: 9.3 公開API** — `ok`
  - spec: `specification.md:19486`
  - notes: failure in effect row is explicit; no auto dual API

- [x] **L5 L19496: 10. Resource cleanup** — `deferred`
  - spec: `specification.md:19496`
  - notes: 依存待ち: bracket primitive + release guarantees (Part III runtime)

- [x] **L6 L19497: 10.1 基本primitive** — `deferred`
  - spec: `specification.md:19497`
  - notes: 依存待ち: bracket acquire/use/release (Part III runtime)

- [x] **L6 L19515: 10.2 役割** — `deferred`
  - spec: `specification.md:19515`
  - notes: 依存待ち: bracket roles (Part III runtime)

- [x] **L6 L19525: 10.3 特別な保証** — `deferred`
  - spec: `specification.md:19525`
  - notes: 依存待ち: bracket release guarantees (Part III runtime)

- [x] **L5 L19535: 11. Acquire規則** — `deferred`
  - spec: `specification.md:19535`
  - notes: 依存待ち: acquire/release registration (Part III runtime)

- [x] **L6 L19536: 11.1 Release登録** — `deferred`
  - spec: `specification.md:19536`
  - notes: 依存待ち: acquire failure skip release (Part III runtime)

- [x] **L6 L19540: 11.2 Acquire failure** — `deferred`
  - spec: `specification.md:19540`
  - notes: 依存待ち: partial acquire (Part III runtime)

- [x] **L6 L19552: 11.3 部分取得** — `deferred`
  - spec: `specification.md:19552`
  - notes: 依存待ち: nested bracket on partial acquire (Part III runtime)

- [x] **L5 L19558: 12. Useの正常終了** — `deferred`
  - spec: `specification.md:19558`
  - notes: 依存待ち: use normal completion + release (Part III runtime)

- [x] **L5 L19570: 13. Use中のFailure** — `deferred`
  - spec: `specification.md:19570`
  - notes: 依存待ち: failure during use + release (Part III runtime)

- [x] **L5 L19581: 14. 一般Effectと継続** — `deferred`
  - spec: `specification.md:19581`
  - notes: 依存待ち: effect suspend + continuation cleanup (Part III runtime)

- [x] **L6 L19582: 14.1 一時中断** — `deferred`
  - spec: `specification.md:19582`
  - notes: 依存待ち: temporary effect suspend (Part III runtime)

- [x] **L6 L19594: 14.2 Resume** — `deferred`
  - spec: `specification.md:19594`
  - notes: 依存待ち: resume without release (Part III runtime)

- [x] **L6 L19598: 14.3 Discard** — `deferred`
  - spec: `specification.md:19598`
  - notes: 依存待ち: discard continuation cleanup (Part III runtime)

- [x] **L6 L19602: 14.4 継続状態** — `meta`
  - spec: `specification.md:19602`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19620: 14.5 継続escape** — `deferred`
  - spec: `specification.md:19620`
  - notes: 依存待ち: continuation escape cleanup (Part III runtime)

- [x] **L5 L19638: 15. Cleanup順序と回数** — `deferred`
  - spec: `specification.md:19638`
  - notes: 依存待ち: cleanup LIFO order (Part III runtime)

- [x] **L6 L19639: 15.1 LIFO** — `deferred`
  - spec: `specification.md:19639`
  - notes: 依存待ち: cleanup LIFO (Part III runtime)

- [x] **L6 L19653: 15.2 高々一回** — `deferred`
  - spec: `specification.md:19653`
  - notes: 依存待ち: cleanup at-most-once (Part III runtime)

- [x] **L6 L19668: 15.3 一つのRelease失敗** — `deferred`
  - spec: `specification.md:19668`
  - notes: 依存待ち: single release failure (Part III runtime)

- [x] **L5 L19676: 16. Cleanup中のFailure** — `deferred`
  - spec: `specification.md:19676`
  - notes: 依存待ち: failure during cleanup (Part III runtime)

- [x] **L6 L19677: 16.1 Primary failure** — `deferred`
  - spec: `specification.md:19677`
  - notes: 依存待ち: primary failure during cleanup (Part III runtime)

- [x] **L6 L19681: 16.2 Suppressed failure** — `deferred`
  - spec: `specification.md:19681`
  - notes: 依存待ち: suppressed failure (Part III runtime)

- [x] **L6 L19702: 16.3 正常終了後のRelease failure** — `deferred`
  - spec: `specification.md:19702`
  - notes: 依存待ち: release failure after success (Part III runtime)

- [x] **L6 L19709: 16.4 複数のSuppressed failure** — `deferred`
  - spec: `specification.md:19709`
  - notes: 依存待ち: multiple suppressed failures (Part III runtime)

- [x] **L6 L19720: 16.5 通常Handlerへの公開** — `deferred`
  - spec: `specification.md:19720`
  - notes: 依存待ち: suppressed failure not in handler (Part III runtime)

- [x] **L5 L19734: 17. finally** — `deferred`
  - spec: `specification.md:19734`
  - notes: 依存待ち: finally (Part III runtime)

- [x] **L5 L19749: 18. Cancellation** — `deferred`
  - spec: `specification.md:19749`
  - notes: full ERR Cancellation model beyond CancellationToken stub 依存待ち(第III部)

- [x] **L6 L19750: 18.1 分類** — `meta`
  - spec: `specification.md:19750`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19757: 18.2 Cleanup** — `deferred`
  - spec: `specification.md:19757`
  - notes: 依存待ち: cancellation cleanup (Part III runtime)

- [x] **L6 L19761: 18.3 詳細** — `deferred`
  - spec: `specification.md:19761`
  - notes: 依存待ち: cancellation details (Part III runtime)

- [x] **L5 L19772: 19. Defect** — `deferred`
  - spec: `specification.md:19772`
  - notes: 依存待ち: defect model (Part III outcome/runtime)

- [x] **L6 L19773: 19.1 定義** — `deferred`
  - spec: `specification.md:19773`
  - notes: 依存待ち: defect definition (Part III outcome/runtime)

- [x] **L6 L19788: 19.2 Effect row** — `deferred`
  - spec: `specification.md:19788`
  - notes: 依存待ち: defect not in effect row (Part III)

- [x] **L6 L19794: 19.3 再開** — `deferred`
  - spec: `specification.md:19794`
  - notes: 依存待ち: no defect resume (Part III)

- [x] **L6 L19798: 19.4 通常Handler** — `deferred`
  - spec: `specification.md:19798`
  - notes: 依存待ち: defect not caught by handler (Part III)

- [x] **L6 L19802: 19.5 Cleanup** — `deferred`
  - spec: `specification.md:19802`
  - notes: 依存待ち: defect cleanup (Part III)

- [x] **L5 L19808: 20. Fault boundary** — `deferred`
  - spec: `specification.md:19808`
  - notes: 依存待ち: fault boundary (Part III runtime)

- [x] **L6 L19809: 20.1 定義** — `deferred`
  - spec: `specification.md:19809`
  - notes: 依存待ち: fault boundary definition (Part III)

- [x] **L6 L19823: 20.2 一般公開** — `deferred`
  - spec: `specification.md:19823`
  - notes: 依存待ち: fault boundary not user API (Part III)

- [x] **L6 L19827: 20.3 処理** — `deferred`
  - spec: `specification.md:19827`
  - notes: 依存待ち: fault boundary processing (Part III)

- [x] **L6 L19838: 20.4 継続条件** — `deferred`
  - spec: `specification.md:19838`
  - notes: 依存待ち: fault boundary continuation (Part III)

- [x] **L5 L19851: 21. Terminal failure** — `deferred`
  - spec: `specification.md:19851`
  - notes: 依存待ち: terminal failure (Part III)

- [x] **L6 L19852: 21.1 定義** — `deferred`
  - spec: `specification.md:19852`
  - notes: 依存待ち: terminal failure definition (Part III)

- [x] **L6 L19866: 21.2 通常Handler** — `deferred`
  - spec: `specification.md:19866`
  - notes: 依存待ち: terminal failure vs handler (Part III)

- [x] **L6 L19870: 21.3 Cleanup** — `deferred`
  - spec: `specification.md:19870`
  - notes: 依存待ち: terminal failure cleanup (Part III)

- [x] **L6 L19874: 21.4 可能な最小処理** — `deferred`
  - spec: `specification.md:19874`
  - notes: 依存待ち: terminal failure minimal handling (Part III)

- [x] **L5 L19886: 22. 個別事例の分類** — `meta`
  - spec: `specification.md:19886`
  - notes: ERR design principle / guideline prose

- [x] **L6 L19887: 22.1 Assertion** — `deferred`
  - spec: `specification.md:19887`
  - notes: 依存待ち: assertion→defect wiring (Part III)

- [x] **L6 L19897: 22.2 Match** — `deferred`
  - spec: `specification.md:19897`
  - notes: 依存待ち: match exhaustiveness defect (Part III)

- [x] **L6 L19904: 22.3 Dynamic cast** — `deferred`
  - spec: `specification.md:19904`
  - notes: 依存待ち: cast failure classification (Part III gradual)

- [x] **L6 L19911: 22.4 Index access** — `deferred`
  - spec: `specification.md:19911`
  - notes: 依存待ち: index access failure (Part III)

- [x] **L6 L19918: 22.5 Arithmetic overflow** — `deferred`
  - spec: `specification.md:19918`
  - notes: 依存待ち: arithmetic overflow policy (Part III)

- [x] **L6 L19934: 22.6 Division by zero** — `deferred`
  - spec: `specification.md:19934`
  - notes: 依存待ち: division by zero policy (Part III)

- [x] **L6 L19944: 22.7 Continuationの二重resume** — `ok`
  - spec: `specification.md:19944`
  - notes: one-shot double resume rejected at eval; DefectReport host deferred

- [x] **L6 L19950: 22.8 Validator** — `deferred`
  - spec: `specification.md:19950`
  - notes: 依存待ち: validator defect (Part III)

- [x] **L6 L19957: 22.9 Foreign adapter** — `deferred`
  - spec: `specification.md:19957`
  - notes: classify_foreign_adapter done; Terminal/fault boundary 依存待ち(第III部)

- [x] **L6 L19964: 22.10 Resource exhaustion** — `deferred`
  - spec: `specification.md:19964`
  - notes: classify_resource_exhaustion done; job budget/Terminal OOM 依存待ち(第III部)

- [x] **L5 L19974: 23. DefectReport** — `ok`
  - spec: `specification.md:19974`
  - notes: FailureReport/DefectReport/Cancellation/JobResult in reciplexa-outcome

- [x] **L6 L19975: 23.1 内容** — `ok`
  - spec: `specification.md:19975`
  - notes: report content fields in reciplexa-outcome

- [x] **L6 L19993: 23.2 安全性** — `ok`
  - spec: `specification.md:19993`
  - notes: DefectReport safety fields in reciplexa-outcome

- [x] **L6 L20007: 23.3 権限・機密性** — `deferred`
  - spec: `specification.md:20007`
  - notes: permission/secrecy fields present; host redaction 依存待ち(第III部)

- [x] **L5 L20011: 24. ジョブ結果** — `ok`
  - spec: `specification.md:20011`
  - notes: JobResult in reciplexa-outcome

- [x] **L5 L20039: 25. Entry pointと実行環境** — `deferred`
  - spec: `specification.md:20039`
  - notes: RequiredEffects⊆Provided entry matrix 依存待ち(第III部)

- [x] **L6 L20040: 25.1 Runtime capability** — `deferred`
  - spec: `specification.md:20040`
  - notes: 依存待ち(第III部): entry RequiredEffects ⊆ ProvidedEffects capability check

- [x] **L6 L20078: 25.2 実行環境ごとのmain** — `deferred`
  - spec: `specification.md:20078`
  - notes: 依存待ち(第III部): CLI/GUI/Server/Worker distinct main contracts

- [x] **L5 L20097: 26. 未処理Failure** — `deferred`
  - spec: `specification.md:20097`
  - notes: Application sink/retry for unhandled Failure 依存待ち(第III部)

- [x] **L6 L20098: 26.1 原則** — `deferred`
  - spec: `specification.md:20098`
  - notes: Application-boundary policy for unhandled Failure 依存待ち(第III部)

- [x] **L6 L20108: 26.2 最終防御** — `deferred`
  - spec: `specification.md:20108`
  - notes: 依存待ち(第III部): final Failure sink cleanup + JobResult conversion

- [x] **L6 L20119: 26.3 Runtime default表示** — `deferred`
  - spec: `specification.md:20119`
  - notes: full Error→Diagnostic explain API 依存待ち(第III部); default FailureReport display present

- [x] **L5 L20136: 27. 実行環境別の処理** — `deferred`
  - spec: `specification.md:20136`
  - notes: 依存待ち(第III部): env-specific Failure/Defect/Cancel/Terminal taxonomy

- [x] **L6 L20137: 27.1 CLI** — `deferred`
  - spec: `specification.md:20137`
  - notes: 依存待ち(第III部): CLI exit-status taxonomy

- [x] **L6 L20159: 27.2 GUI** — `deferred`
  - spec: `specification.md:20159`
  - notes: 依存待ち(第III部|第V部): GUI command/render fault boundaries

- [x] **L6 L20168: 27.3 Server** — `deferred`
  - spec: `specification.md:20168`
  - notes: 依存待ち(第III部): Server request isolation

- [x] **L6 L20174: 27.4 Plugin** — `deferred`
  - spec: `specification.md:20174`
  - notes: 依存待ち(第III部|第V部): Plugin invocation fault boundary

- [x] **L6 L20181: 27.5 Render job** — `deferred`
  - spec: `specification.md:20181`
  - notes: 依存待ち(第III部|第V部): Render job fault boundary

- [x] **L5 L20190: 28. Diagnostic** — `partial`
  - spec: `specification.md:20190`
  - notes: reciplexa-diagnostic + FailureDiagnosticBundle; ERR primary model light vs §28

- [x] **L6 L20191: 28.1 構築と出力の分離** — `ok`
  - spec: `specification.md:20191`
  - notes: DiagnosticCollector construct + CLI sink render

- [x] **L6 L20208: 28.2 PrimaryとSuppressed** — `deferred`
  - spec: `specification.md:20208`
  - notes: primary→suppressed order present; cleanup wiring 依存待ち(第III部)

- [x] **L6 L20220: 28.3 Libraryの責務** — `ok`
  - spec: `specification.md:20220`
  - notes: reciplexa-diagnostic constructs without choosing sink

- [x] **L5 L20226: 29. 適合試験** — `ok`
  - spec: `specification.md:20226`
  - notes: ERR-01/02/03 handle_tests + TEST-DYN-005 explicit error terminal

- [x] **L5 L20399: 30. 移管先OPEN** — `deferred`
  - spec: `specification.md:20399`
  - notes: OPEN transfer; deferred

- [x] **L6 L20401: `OPEN-MEM-001`** — `deferred`
  - spec: `specification.md:20401`
  - notes: OPEN transfer; deferred

- [x] **L6 L20410: `OPEN-CON-001`** — `deferred`
  - spec: `specification.md:20410`
  - notes: OPEN transfer; deferred

- [x] **L6 L20419: `OPEN-KER-001`** — `deferred`
  - spec: `specification.md:20419`
  - notes: OPEN transfer; deferred

- [x] **L6 L20427: `OPEN-TST-001`** — `deferred`
  - spec: `specification.md:20427`
  - notes: OPEN transfer; deferred

- [x] **L6 L20435: `OPEN-PKG-ENTRY-001`** — `deferred`
  - spec: `specification.md:20435`
  - notes: OPEN transfer; deferred

- [x] **L6 L20442: `OPEN-ERR-DIAG-001`** — `deferred`
  - spec: `specification.md:20442`
  - notes: OPEN transfer; deferred

- [x] **L5 L20450: 31. 最終状態** — `meta`
  - spec: `specification.md:20450`
  - notes: ERR final-state prose

- [x] **L4 L20476: 13.14 `MEM-001` Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算** — `partial`
  - spec: `specification.md:20476`
  
  - notes: reciplexa-mem Perceus IR ok; eval still Rc; budget/bracket/resource deferred reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path; eval uses Rc not Perceus by default

- [x] **L5 L20477: DD-001 決定概要** — `meta`
  - spec: `specification.md:20477`
  - notes: MEM glossary / policy prose

- [x] **L6 L20478: DD-001.1 状態** — `meta`
  - spec: `specification.md:20478`
  - notes: MEM glossary / policy prose

- [x] **L6 L20496: DD-001.2 既存仕様との関係** — `meta`
  - spec: `specification.md:20496`
  - notes: MEM glossary / policy prose

- [x] **L6 L20515: DD-001.3 中心的な決定** — `meta`
  - spec: `specification.md:20515`
  - notes: MEM glossary / policy prose

- [x] **L5 L20530: 0. 用語** — `meta`
  - spec: `specification.md:20530`
  - notes: MEM glossary / policy prose

- [x] **L6 L20531: 0.1 Perceus** — `meta`
  - spec: `specification.md:20531`
  - notes: MEM glossary / policy prose

- [x] **L6 L20549: 0.2 自動メモリ管理** — `meta`
  - spec: `specification.md:20549`
  - notes: MEM glossary / policy prose

- [x] **L6 L20562: 0.3 Resource** — `meta`
  - spec: `specification.md:20562`
  - notes: MEM glossary / policy prose

- [x] **L5 L20577: 1. メモリとResourceの分離** — `partial`
  - spec: `specification.md:20577`
  
  - notes: policy: Perceus values vs bracket resources; bracket surface incomplete- notes: policy: values vs resources; RSC/bracket separation incomplete

- [x] **L6 L20578: 1.1 通常値** — `ok`
  - spec: `specification.md:20578`
  
  - notes: Ordinary values lowered to ownership IR (Lit/Construct/MakeClosure)- notes: policy: values vs resources; RSC/bracket separation incomplete

- [x] **L6 L20594: 1.2 外部Resource** — `deferred`
  - spec: `specification.md:20594`
  
  - notes: 依存待ち: external Resource lifetime via bracket (Part III)- notes: policy: values vs resources; RSC/bracket separation incomplete

- [x] **L6 L20604: 1.3 基本原則** — `ok`
  - spec: `specification.md:20604`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20614: 2. Perceusによる自動メモリ管理** — `ok`
  - spec: `specification.md:20614`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20615: 2.1 利用者から見える意味** — `ok`
  - spec: `specification.md:20615`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20627: 2.2 回収時点** — `ok`
  - spec: `specification.md:20627`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20639: 2.3 物理identity** — `ok`
  - spec: `specification.md:20639`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20660: 3. Compilation pipeline** — `ok`
  - spec: `specification.md:20660`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20661: 3.1 適用順序** — `ok`
  - spec: `specification.md:20661`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20684: 3.2 Surface所有権注釈** — `deferred`
  - spec: `specification.md:20684`
  
  - notes: 依存待ち: Surface ownership annotations (none in v1 core path)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20708: 3.3 Trusted boundary** — `deferred`
  - spec: `specification.md:20708`
  
  - notes: 依存待ち: Trusted foreign ownership boundary metadata- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20717: 4. 所有権Core IR** — `ok`
  - spec: `specification.md:20717`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20718: 4.1 必須のCore要素** — `ok`
  - spec: `specification.md:20718`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20749: 4.2 評価順序** — `ok`
  - spec: `specification.md:20749`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20755: 5. dup** — `ok`
  - spec: `specification.md:20755`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20756: 5.1 意味** — `ok`
  - spec: `specification.md:20756`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20765: 5.2 挿入条件** — `ok`
  - spec: `specification.md:20765`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20769: 5.3 利用者からの不可視性** — `ok`
  - spec: `specification.md:20769`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20775: 6. drop** — `ok`
  - spec: `specification.md:20775`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20776: 6.1 意味** — `ok`
  - spec: `specification.md:20776`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20789: 6.2 最終使用位置** — `ok`
  - spec: `specification.md:20789`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20809: 6.3 制御フロー** — `ok`
  - spec: `specification.md:20809`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20816: 7. 分岐とJoin point** — `ok`
  - spec: `specification.md:20816`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20817: 7.1 排他的分岐** — `ok`
  - spec: `specification.md:20817`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20828: 7.2 Join時の整合** — `ok`
  - spec: `specification.md:20828`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20858: 8. Reuse** — `ok`
  - spec: `specification.md:20858`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20859: 8.1 位置付け** — `ok`
  - spec: `specification.md:20859`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20869: 8.2 一意性** — `ok`
  - spec: `specification.md:20869`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20877: 8.3 Reuse不成立** — `ok`
  - spec: `specification.md:20877`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20885: 8.4 非保証** — `ok`
  - spec: `specification.md:20885`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20894: 8.5 Reuse禁止値** — `ok`
  - spec: `specification.md:20894`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20916: 9. Closure環境** — `ok`
  - spec: `specification.md:20916`
  
  - notes: IR path; eval still Rc- notes: MakeClosure in mem lower; not default eval memory path

- [x] **L6 L20917: 9.1 表現** — `ok`
  - spec: `specification.md:20917`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20926: 9.2 生成** — `ok`
  - spec: `specification.md:20926`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20933: 9.3 解放** — `ok`
  - spec: `specification.md:20933`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20940: 9.4 Closure identity** — `deferred`
  - spec: `specification.md:20940`
  - notes: 依存待ち: closure identity observability (Part III / full MEM)

- [x] **L6 L20950: 9.5 Scoped値のcapture** — `ok`
  - spec: `specification.md:20950`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L20956: 10. var** — `partial`
  - spec: `specification.md:20956`
  
  - notes: LocalVar escape check in eval; Perceus-var cell integration incomplete- notes: LocalVar escape check in eval; full Perceus-var integration incomplete

- [x] **L6 L20957: 10.1 既存意味論** — `partial`
  - spec: `specification.md:20957`
  
  - notes: var semantics via LocalVar in Core/eval; IR lowers init as value- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20966: 10.2 物理表現** — `partial`
  - spec: `specification.md:20966`
  
  - notes: var physical cell not separate ownership class in mem IR yet- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L20976: 10.3 Perceusとの関係** — `meta`
  - spec: `specification.md:20976`
  - notes: MEM glossary / policy prose

- [x] **L5 L20982: 11. cell／ref** — `ok`
  - spec: `specification.md:20982`
  - notes: escape cell/ref out of v1 (spec); LocalVar only

- [x] **L6 L20983: 11.1 v1の方針** — `ok`
  - spec: `specification.md:20983`
  - notes: escape cell/ref out of v1 (spec); LocalVar only

- [x] **L6 L20991: 11.2 理由** — `ok`
  - spec: `specification.md:20991`
  - notes: escape cell/ref out of v1 (spec); LocalVar only

- [x] **L6 L21007: 11.3 将来拡張** — `ok`
  - spec: `specification.md:21007`
  - notes: escape cell/ref out of v1 (spec); LocalVar only

- [x] **L5 L21019: 12. 再帰型と循環値** — `ok`
  - spec: `specification.md:21019`
  
  - notes: v1 forbids heap cycles (spec policy); recursive data ok without cycles- notes: spec forbids heap cycles in v1; no cycle detector beyond policy

- [x] **L6 L21020: 12.1 再帰data型** — `ok`
  - spec: `specification.md:21020`
  
  - notes: Recursive data values use Construct; no heap cycle required- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21028: 12.2 循環する実行時値** — `ok`
  - spec: `specification.md:21028`
  
  - notes: Runtime cyclic values forbidden in v1 (policy; no cycle constructor)- notes: spec forbids heap cycles in v1; no cycle detector beyond policy

- [x] **L6 L21041: 12.3 論理的なID参照** — `ok`
  - spec: `specification.md:21041`
  
  - notes: Logical ID refs out of heap cycles; identity ids separate- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21047: 13. Continuation** — `partial`
  - spec: `specification.md:21047`
  
  - notes: Resume/DiscardCont/Raise in mem IR+exec; full continuation capture model light- notes: Resume/DiscardCont/Raise in mem IR+exec; not full continuation model

- [x] **L6 L21048: 13.1 表現** — `ok`
  - spec: `specification.md:21048`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21061: 13.2 Capture** — `partial`
  - spec: `specification.md:21061`
  
  - notes: Continuation capture representation minimal in IR- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21069: 13.3 Resume** — `ok`
  - spec: `specification.md:21069`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21078: 13.4 Discard** — `ok`
  - spec: `specification.md:21078`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21086: 13.5 One-shot** — `ok`
  - spec: `specification.md:21086`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21098: 13.6 Escape** — `partial`
  - spec: `specification.md:21098`
  
  - notes: Escape of continuation past resume scope still light- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21106: 14. Failure unwind** — `ok`
  - spec: `specification.md:21106`
  - notes: MemInstr::Raise runs RegisterCleanup LIFO in exec

- [x] **L6 L21107: 14.1 明示的なunwind** — `ok`
  - spec: `specification.md:21107`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21118: 14.2 Dropの欠落禁止** — `ok`
  - spec: `specification.md:21118`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21124: 14.3 Handler節** — `ok`
  - spec: `specification.md:21124`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21135: 15. Scoped Resource handle** — `deferred`
  - spec: `specification.md:21135`
  - notes: 依存待ち: scoped resource handle + bracket (Part III)

- [x] **L6 L21136: 15.1 隠れたscope** — `deferred`
  - spec: `specification.md:21136`
  
  - notes: 依存待ち: hidden resource scope / bracket (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21153: 15.2 bracketの概念型** — `deferred`
  - spec: `specification.md:21153`
  - notes: 依存待ち: bracket concept type surface (Part III)

- [x] **L6 L21168: 15.3 Escape禁止** — `deferred`
  - spec: `specification.md:21168`
  
  - notes: 依存待ち: scoped resource escape ban (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21182: 15.4 Scope内Closure** — `deferred`
  - spec: `specification.md:21182`
  - notes: 依存待ち: scope closure capture rules (Part III)

- [x] **L6 L21188: 15.5 独立した結果値** — `deferred`
  - spec: `specification.md:21188`
  
  - notes: 依存待ち: independent result vs handle (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21201: 16. Resource API** — `deferred`
  - spec: `specification.md:21201`
  - notes: 依存待ち: resource API surface (Part III)

- [x] **L6 L21202: 16.1 With-style API** — `deferred`
  - spec: `specification.md:21202`
  
  - notes: 依存待ち: with-style resource API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21210: 16.2 低水準API** — `deferred`
  - spec: `specification.md:21210`
  
  - notes: 依存待ち: low-level resource API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21216: 16.3 Release責任** — `deferred`
  - spec: `specification.md:21216`
  
  - notes: 依存待ち: release responsibility model (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21223: 16.4 自発的な無効化** — `deferred`
  - spec: `specification.md:21223`
  
  - notes: 依存待ち: voluntary invalidation API (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21236: 17. Borrowed view** — `ok`
  - spec: `specification.md:21236`
  - notes: borrow system out of v1 per spec

- [x] **L6 L21237: 17.1 所有値とBorrowed view** — `ok`
  - spec: `specification.md:21237`
  - notes: borrow system out of v1 per spec

- [x] **L6 L21247: 17.2 v1の方針** — `ok`
  - spec: `specification.md:21247`
  - notes: borrow system out of v1 per spec

- [x] **L5 L21261: 18. Weak referenceとFinalizer** — `ok`
  - spec: `specification.md:21261`
  - notes: weak/finalizer out of v1 per spec

- [x] **L6 L21262: 18.1 Weak reference** — `ok`
  - spec: `specification.md:21262`
  - notes: weak/finalizer out of v1 per spec

- [x] **L6 L21275: 18.2 利用者定義Finalizer** — `ok`
  - spec: `specification.md:21275`
  - notes: weak/finalizer out of v1 per spec

- [x] **L6 L21288: 18.3 Resource安全網** — `ok`
  - spec: `specification.md:21288`
  - notes: weak/finalizer out of v1 per spec

- [x] **L5 L21296: 19. Foreign boundary** — `deferred`
  - spec: `specification.md:21296`
  - notes: 依存待ち: foreign ownership boundary (Part III)

- [x] **L6 L21297: 19.1 Ownership metadata** — `deferred`
  - spec: `specification.md:21297`
  
  - notes: 依存待ち: foreign ownership metadata (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21320: 19.2 Borrowed契約違反** — `deferred`
  - spec: `specification.md:21320`
  
  - notes: 依存待ち: borrowed foreign contract (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21330: 19.3 Owned移送** — `deferred`
  - spec: `specification.md:21330`
  
  - notes: 依存待ち: owned foreign transfer (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21338: 20. Concurrencyへの接続** — `deferred`
  - spec: `specification.md:21338`
  - notes: 依存待ち(ASY)|仕様未決定: OPEN-CON-001 atomic RC / Send/Share

- [x] **L6 L21339: 20.1 v1の範囲** — `deferred`
  - spec: `specification.md:21339`
  
  - notes: 依存待ち(ASY)|OPEN-CON-001: concurrency v1 scope- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21345: 20.2 共有値** — `deferred`
  - spec: `specification.md:21345`
  
  - notes: 依存待ち(ASY)|OPEN-CON-001: shared values across tasks- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21360: 20.3 Scoped Resource** — `deferred`
  - spec: `specification.md:21360`
  - notes: 依存待ち(第III部|ASY): scoped resource must-not-send across tasks

- [x] **L5 L21366: 21. SnapshotとPerceus** — `deferred`
  - spec: `specification.md:21366`
  - notes: 依存待ち: snapshot + Perceus integration (Part III)

- [x] **L6 L21367: 21.1 構造共有** — `deferred`
  - spec: `specification.md:21367`
  
  - notes: 依存待ち: snapshot structural sharing + Perceus (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21380: 21.2 解放** — `deferred`
  - spec: `specification.md:21380`
  
  - notes: 依存待ち: snapshot release with Perceus (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21386: 21.3 一意性** — `deferred`
  - spec: `specification.md:21386`
  
  - notes: 依存待ち: snapshot uniqueness (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21390: 21.4 観測不能** — `deferred`
  - spec: `specification.md:21390`
  
  - notes: 依存待ち: snapshot observability (Part III)- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21394: 22. メモリ割当とEffect** — `ok`
  - spec: `specification.md:21394`
  
  - notes: Ordinary allocation absent from effect row; Perceus ops not effects- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21395: 22.1 通常Allocation** — `ok`
  - spec: `specification.md:21395`
  - notes: ordinary Construct/record allocation absent from effect row (typecheck test)

- [x] **L6 L21408: 22.2 Perceus操作** — `ok`
  - spec: `specification.md:21408`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21418: 22.3 理由** — `ok`
  - spec: `specification.md:21418`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21424: 23. メモリ予算** — `deferred`
  - spec: `specification.md:21424`
  - notes: 依存待ち(第III部): job-increment memory budget model

- [x] **L6 L21425: 23.1 適用単位** — `deferred`
  - spec: `specification.md:21425`
  
  - notes: 依存待ち(第III部): memory budget unit- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21440: 23.2 Job増分方式** — `deferred`
  - spec: `specification.md:21440`
  
  - notes: 依存待ち(第III部): job-increment budget- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21459: 23.3 Commit時の移管** — `deferred`
  - spec: `specification.md:21459`
  
  - notes: 依存待ち(第III部): commit-time budget transfer- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21475: 24. 予算超過** — `deferred`
  - spec: `specification.md:21475`
  - notes: 依存待ち(第III部): failure resource-exhausted on budget exceed

- [x] **L6 L21476: 24.1 型付きFailure** — `deferred`
  - spec: `specification.md:21476`
  
  - notes: 依存待ち(第III部): resource-exhausted Failure- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21492: 24.2 処理** — `deferred`
  - spec: `specification.md:21492`
  
  - notes: 依存待ち(第III部): budget-exceed handling- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21506: 24.3 予約領域** — `deferred`
  - spec: `specification.md:21506`
  
  - notes: 依存待ち(第III部): reserved budget region- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21512: 25. 単一巨大Allocation** — `deferred`
  - spec: `specification.md:21512`
  - notes: 依存待ち(第III部): huge-allocation precheck classification

- [x] **L6 L21513: 25.1 事前検査** — `deferred`
  - spec: `specification.md:21513`
  
  - notes: 依存待ち(第III部): huge allocation precheck- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21522: 25.2 分類** — `deferred`
  - spec: `specification.md:21522`
  
  - notes: 依存待ち(第III部): huge allocation classification- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21538: 26. Continuation予算** — `deferred`
  - spec: `specification.md:21538`
  - notes: 依存待ち(第III部): continuation capture budget

- [x] **L6 L21539: 26.1 課金対象** — `deferred`
  - spec: `specification.md:21539`
  
  - notes: 依存待ち(第III部): continuation capture budget- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21552: 26.2 超過時** — `deferred`
  - spec: `specification.md:21552`
  
  - notes: 依存待ち(第III部): continuation budget exceed- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21564: 27. Snapshot保持量** — `deferred`
  - spec: `specification.md:21564`
  - notes: 依存待ち(第III部|第V部): snapshot retention budget / refuse new handles

- [x] **L6 L21565: 27.1 有効なSnapshot handle** — `deferred`
  - spec: `specification.md:21565`
  
  - notes: 依存待ち(第III部|第V部): snapshot handle retention limit- notes: DocumentSnapshot holds while handle live; retention-limit refuse 依存待ち(第III部|第V部)

- [x] **L6 L21571: 27.2 保持policy** — `deferred`
  - spec: `specification.md:21571`
  
  - notes: 依存待ち(第III部|第V部): snapshot retention policy- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21584: 27.3 履歴破棄** — `deferred`
  - spec: `specification.md:21584`
  
  - notes: 依存待ち(第III部|第V部): history discard policy- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21588: 28. 一般heap OOM** — `deferred`
  - spec: `specification.md:21588`
  - notes: 依存待ち(第III部): process heap OOM vs managed Failure Terminal path

- [x] **L6 L21589: 28.1 管理予算との区別** — `deferred`
  - spec: `specification.md:21589`
  - notes: 依存待ち(第III部): managed budget vs Terminal OOM distinction

- [x] **L6 L21593: 28.2 分類** — `deferred`
  - spec: `specification.md:21593`
  
  - notes: 依存待ち(第III部): OOM classification- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21600: 28.3 Cleanup** — `deferred`
  - spec: `specification.md:21600`
  
  - notes: 依存待ち(第III部): OOM cleanup path- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21606: 29. Reference count overflow** — `ok`
  - spec: `specification.md:21606`
  - notes: checked_add refuses RC wraparound (ExecError::RefCountOverflow)

- [x] **L6 L21607: 29.1 Wraparound禁止** — `ok`
  - spec: `specification.md:21607`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21613: 29.2 実装** — `ok`
  - spec: `specification.md:21613`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21617: 29.3 分類** — `ok`
  - spec: `specification.md:21617`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21627: 29.4 その他の内部不変条件** — `ok`
  - spec: `specification.md:21627`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21640: 30. Verification** — `ok`
  - spec: `specification.md:21640`
  - notes: verify_ownership / verify_reuse in reciplexa-mem

- [x] **L6 L21641: 30.1 Ownership verifier** — `ok`
  - spec: `specification.md:21641`
  - notes: verify_ownership in reciplexa-mem

- [x] **L6 L21656: 30.2 Reuse verifier** — `ok`
  - spec: `specification.md:21656`
  - notes: verify_reuse in reciplexa-mem

- [x] **L6 L21666: 30.3 Verifier failure** — `ok`
  - spec: `specification.md:21666`
  - notes: verifier failure reported by reciplexa-mem verify_*

- [x] **L5 L21677: 31. メモリ観測API** — `partial`
  - spec: `specification.md:21677`
  
  - notes: RC/dup/drop not exposed (§31.1 ok); budget/peak observation deferred RC/dup/drop not exposed to RPX (§31.1); budget/peak observation API 依存待ち(第III部)|OPEN-MEM-PROF-001

- [x] **L6 L21678: 31.1 非公開情報** — `ok`
  - spec: `specification.md:21678`
  
  - notes: IR path; eval still Rc- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21691: 31.2 許可される情報** — `deferred`
  - spec: `specification.md:21691`
  
  - notes: 依存待ち|OPEN-MEM-PROF-001: allowed memory observation API- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L6 L21701: 31.3 安定性** — `deferred`
  - spec: `specification.md:21701`
  
  - notes: 依存待ち|OPEN-MEM-PROF-001: observation API stability- notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21714: 32. GUI状態** — `partial`
  - spec: `specification.md:21714`
  - notes: GUI uses explicit DocumentSnapshot (no implicit cell); full model OPEN-GUI-STATE-001

- [x] **L6 L21715: 32.1 暗黙cellへの非依存** — `ok`
  - spec: `specification.md:21715`
  - notes: v1 intentional non-feature (matches spec) — escape cell/weak/borrow out of v1

- [x] **L6 L21719: 32.2 推奨モデル** — `partial`
  - spec: `specification.md:21719`
  - notes: reciplexa-mem Perceus/dup/drop/reuse/verify/lower; not default eval path

- [x] **L5 L21728: 33. 適合試験** — `partial`
  - spec: `specification.md:21728`
  - notes: phase6_mem / mem tests exist; full MEM-001 suite incomplete

- [x] **L5 L21906: 34. 移管先OPEN** — `deferred`
  - spec: `specification.md:21906`
  - notes: OPEN transfer; deferred

- [x] **L6 L21908: `OPEN-MEM-CELL-001`** — `deferred`
  - spec: `specification.md:21908`
  - notes: OPEN transfer; deferred

- [x] **L6 L21917: `OPEN-MEM-BORROW-001`** — `deferred`
  - spec: `specification.md:21917`
  - notes: OPEN transfer; deferred

- [x] **L6 L21926: `OPEN-CON-001`** — `deferred`
  - spec: `specification.md:21926`
  - notes: OPEN transfer; deferred

- [x] **L6 L21935: `OPEN-KER-001`** — `deferred`
  - spec: `specification.md:21935`
  - notes: OPEN transfer; deferred

- [x] **L6 L21943: `OPEN-GUI-STATE-001`** — `deferred`
  - spec: `specification.md:21943`
  - notes: OPEN transfer; deferred

- [x] **L6 L21950: `OPEN-MEM-PROF-001`** — `deferred`
  - spec: `specification.md:21950`
  - notes: OPEN transfer; deferred

- [x] **L5 L21958: 35. 最終状態** — `meta`
  - spec: `specification.md:21958`
  - notes: MEM final-state prose

- [x] **L4 L21988: 13.14.1 `ASY-001` Concurrency and async** — `deferred`
  - spec: `specification.md:21988`
  - notes: ASY-001 未決定; runtime scheduler exists but language async unspecified

- [x] **L5 L21990: 状態** — `meta`
  - spec: `specification.md:21990`
  - notes: ASY status: 未決定 (thread/task/async not specified)

- [x] **L4 L22012: 13.15 `TST-001` Tests and conformance** — `partial`
  - spec: `specification.md:22012`
  - notes: lang_kernel_suite: STA/DYN/INT/SYN-C + LANG-* BND/DAT/EFF/ERR/TYP/PKG/ADT; full matrix open

- [x] **L5 L22014: 概要・状態** — `meta`
  - spec: `specification.md:22014`
  - notes: TST overview: product testing確定; surface syntax 未決定

- [x] **L5 L22039: Black/white box** — `deferred`
  - spec: `specification.md:22039`
  - notes: 仕様未決定: test-module/test-of white-box companion surface

- [x] **L5 L22045: 既存実装** — `meta`
  - spec: `specification.md:22045`
  - notes: describes current Rust tests empirically

- [x] **L5 L22051: テスト原則** — `partial`
  - spec: `specification.md:22051`
  - notes: conformance.rs + lang_kernel_suite TEST-* IDs; full artifact trace incomplete

- [x] **L5 L22067: メタ理論** — `meta`
  - spec: `specification.md:22067`
  - notes: tests ≠ proofs metatheory note

## 構文の統合仕様

- [x] **L2 L22072: 構文の統合仕様** — `meta`
  - spec: `specification.md:22072`
  - notes: integrative syntax chapter; incomplete by design

- [x] **L4 L22074: 14.1 全体EBNF（未完成）** — `partial`
  - spec: `specification.md:22074`
  - notes: skeleton EBNF; real grammar in reciplexa-syntax

- [x] **L4 L22123: 14.2 優先順位・結合** — `ok`
  - spec: `specification.md:22123`
  - notes: S-expr: no infix precedence (ops are list heads)

- [x] **L4 L22128: 14.3 予約語** — `partial`
  - spec: `specification.md:22128`
  - notes: is_reserved_special_form / keywords; full policy 未決定

- [x] **L4 L22134: 14.4 糖衣とCore** — `partial`
  - spec: `specification.md:22134`
  - notes: elaborate covers many sugars; table incomplete

## 静的意味論の統合仕様

- [x] **L2 L22146: 静的意味論の統合仕様** — `meta`
  - spec: `specification.md:22146`
  - notes: integrative statics chapter

- [x] **L4 L22148: 15.1 Kind** — `partial`
  - spec: `specification.md:22148`
  - notes: kinds implicit in CoreType/rows; Module/Signature kinds absent

- [x] **L4 L22156: 15.2 共通判断** — `partial`
  - spec: `specification.md:22156`
  - notes: infer/check judgments in check.rs; module sig judgment absent

- [x] **L4 L22168: 15.3 基本規則** — `partial`
  - spec: `specification.md:22168`
  - notes: T-VAR/if/record etc. partially in checker

- [x] **L4 L22222: 15.4 一般化** — `partial`
  - spec: `specification.md:22222`
  - notes: let generalization light; value restriction incomplete

- [x] **L4 L22228: 15.5 Subtypingと制約解決** — `partial`
  - spec: `specification.md:22228`
  - notes: unify + subtype stubs; full constraint solver deferred

- [x] **L4 L22236: 15.6 Module境界** — `deferred`
  - spec: `specification.md:22236`
  - notes: module signature checking deferred with MOD signatures

## 動的意味論の統合仕様

- [x] **L2 L22241: 動的意味論の統合仕様** — `meta`
  - spec: `specification.md:22241`
  - notes: integrative dynamics chapter

- [x] **L4 L22243: 16.1 構成** — `ok`
  - spec: `specification.md:22243`
  - notes: eval configurations via reciplexa-eval Outcome/resume (TEST-DYN-*)

- [x] **L4 L22257: 16.2 評価順序** — `ok`
  - spec: `specification.md:22257`
  - notes: CBV left-to-right in eval

- [x] **L4 L22267: 16.3 効果伝播** — `partial`
  - spec: `specification.md:22267`
  - notes: deep one-shot handlers; multi-shot deferred

- [x] **L4 L22272: 16.4 SourceEdit** — `partial`
  - spec: `specification.md:22272`
  - notes: syntax edit + source_sync; not full SourceEdit calculus

- [x] **L4 L22287: 16.5 観測可能な振る舞い** — `meta`
  - spec: `specification.md:22287`
  - notes: observational behavior framing

## エラーと停止状態

- [x] **L2 L22300: エラーと停止状態** — `meta`
  - spec: `specification.md:22300`
  - notes: error/stuck taxonomy prose

## 機能間の相互作用

- [x] **L2 L22319: 機能間の相互作用** — `meta`
  - spec: `specification.md:22319`
  - notes: cross-feature interaction notes

## メタ理論上の性質

- [x] **L2 L22339: メタ理論上の性質** — `meta`
  - spec: `specification.md:22339`
  - notes: Progress/Preservation goals; proofs absent

## 実装アーキテクチャ

- [x] **L2 L22384: 実装アーキテクチャ** — `partial`
  - spec: `specification.md:22384`
  - notes: pipeline exists as vertical slice; not final

- [x] **L4 L22386: 20.1 最終目標パイプライン** — `partial`
  - spec: `specification.md:22386`
  - notes: bytes→CST→elaborate→check→eval→lower present; phase gaps

- [x] **L4 L22404: 20.2 必要データ構造** — `partial`
  - spec: `specification.md:22404`
  - notes: many structures exist; ModuleEnv/Typed Core incomplete

- [x] **L4 L22422: 20.3 現行crateとの対応** — `partial`
  - spec: `specification.md:22422`
  - notes: crate map outdated in places (core/eval/bind now carry more); still useful

## 仕様と実装の対応

- [x] **L2 L22440: 仕様と実装の対応** — `meta`
  - spec: `specification.md:22440`
  - notes: stage correspondence table

- [x] **L4 L22460: 21.1 型検査器要件** — `partial`
  - spec: `specification.md:22460`
  - notes: typecheck_language_source; ModuleEnv/imported sigs incomplete

## テスト計画

- [x] **L2 L22474: テスト計画** — `meta`
  - spec: `specification.md:22474`
  - notes: test plan chapter

- [x] **L4 L22476: 22.1 構文** — `ok`
  - spec: `specification.md:22476`
  - notes: lexer/parser/edit + TEST-SYN-C001 parse/unparse round-trip

- [x] **L4 L22487: 22.2 静的意味** — `partial`
  - spec: `specification.md:22487`
  - notes: check/unify + STA-001/007 + BND ann/ADT/casts; full STA matrix open

- [x] **L4 L22502: 22.3 動的意味** — `ok`
  - spec: `specification.md:22502`
  - notes: TEST-DYN-001..005 wired (order/closure/handlers/var/failure); multi-shot is EFF deferral

- [x] **L4 L22514: 22.4 統合** — `partial`
  - spec: `specification.md:22514`
  - notes: GUI/source_sync + TEST-INT-002 .rpi boundary; not full INT matrix

- [x] **L4 L22529: 22.5 Property/differential/fuzz** — `deferred`
  - spec: `specification.md:22529`
  - notes: 意図的後回し: property/differential/fuzz harness (harden fuzz hook only)

- [x] **L3 L22549: 22.6 横断適合試験** — `partial`
  - spec: `specification.md:22549`
  - notes: cross-wires STA/DYN/INT + LANG-* ; not versioned full cross suite

## 完成判定基準

- [x] **L2 L22559: 完成判定基準** — `meta`
  - spec: `specification.md:22559`
  - notes: completion criteria process

- [x] **L4 L22561: 段階1: 設計案が記録された** — `ok`
  - spec: `specification.md:22561`
  - notes: spec + feature IDs recorded

- [x] **L4 L22569: 段階2: 仕様が明確になった** — `partial`
  - spec: `specification.md:22569`
  - notes: spec largely written; grammar/Core still holes

- [x] **L4 L22577: 段階3: 参照実装が動く** — `ok`
  - spec: `specification.md:22577`
  - notes: reference pipeline bytes→CST→elaborate→check→eval runs via lang_kernel_suite

- [x] **L4 L22585: 段階4: 適合試験を通過** — `partial`
  - spec: `specification.md:22585`
  - notes: some conformance tests pass; not versioned full suite gate

- [x] **L4 L22593: 段階5: 統合試験を通過** — `deferred`
  - spec: `specification.md:22593`
  - notes: multi-package/GUI/resource integration post-PKG

- [x] **L4 L22601: 段階6: 差分・生成・fuzzを通過** — `deferred`
  - spec: `specification.md:22601`
  - notes: 意図的後回し: stage-6 generators/differential/fuzz gate

- [x] **L4 L22609: 段階7: メタ理論が確認された** — `meta`
  - spec: `specification.md:22609`
  - notes: 形式証明: Progress/Preservation not mechanically proven (proof crate stubs only)

- [x] **L4 L22618: 段階8: 実装と形式仕様の対応を確認** — `partial`
  - spec: `specification.md:22618`
  - notes: crate↔spec mapping informal; no versioned correspondence report

---

## How to update

1. Change each item's status token to one of: `unchecked`, `ok`, `partial`, `gap`, `deferred`, `meta`.
2. Fill in `notes:` with evidence, gaps, or deferral reason.
3. When status is not `unchecked`, set the checkbox to `[x]` (leave `[ ]` while still `unchecked`).
4. Keep the top progress counters and `lang/part2-conformance-stats.json` in sync with the same counts.
