全然関係ないけど、基本的にはパッケージで実装されているけどよく使うのでどうせコンパイラでネイティブに実装されるようなところについて、どれがいいかを考えといてください。そしてそれは、そのネイティブ実装されるものだけが原則として単独の単独パッケージになっていて、それを読み込む形にしてください。この辺の議論は途中で思いついたので話す機会がありませんでしたが、もしこれをspecification.mdに統合するとなったときに、もう一度議論の俎上に上げてください。

# openの解決

## `OPEN-SYN-001` Code mode、doc mode、`src`の廃止

### 状態

`解決済み`

本決定は、RPX v1における次の事項を固定する。

- RPX source fileの既定reader mode
- code modeからdoc modeへの移行
- doc modeからcode modeへの移行
- reader modeの入れ子
- doc commandの名前解決
- reader expansion後の処理
- `src` wrapperの廃止
- 現行実装から最終仕様への移行方針

次の詳細は`OPEN-SYN-002`へ移管する。

- `@` escapeの完全字句文法
- command argumentのbracket構文
- doc bodyのbrace構文
- literal `@`のescape
- whitespaceおよび改行の正規化
- delimiter error recovery
- doc text chunkのtokenization
- identifierおよびliteralの完全字句文法
- reader formをlexically reservedにするか
- doc readerが生成するsyntax objectの正確な形

---

### `DD-SYN-001`: RPX source fileの既定mode

`確定`

RPX source fileの既定reader modeをcode modeとする。

top-level sourceには、通常のS式およびdeclarationを直接記述する。

```lisp
(type title str)

(val title "Report")

(doc
  @heading{title})
````

ファイル全体を暗黙のdoc modeとして扱わない。

RPXは文書専用のマークアップ形式ではなく、文書、図形、スライド、
アニメーションその他の表現モデルを、型付き関数およびpackageによって構築する
プログラミング言語である。

したがって、module、package、型注釈、値bindingおよび通常の計算を直接記述できる
code modeを、基本的なfile semanticsとする。

文書中心のsource profileまたはdoc-first形式を将来導入する場合は、
明示的なreader declarationまたは別のsource形式として設計する。

それをRPX v1の通常source fileへ暗黙に適用してはならない。

***

### `DD-SYN-002`: code mode

`確定`

code modeは通常のS式を読み取るreader modeである。

code modeでは、少なくとも次を通常のformとして読み取る。

* identifier
* literal
* list
* reader form
* trivia
* error tokenまたはerror node

概念的な部分文法を次に示す。

```ebnf
source    = trivia*, form*, trivia* ;
form      = identifier
          | literal
          | list
          | reader-form ;
list      = "(", trivia*, { form, trivia* }, ")" ;
```

特殊形式、macroおよび通常の関数適用は、具象parserが文字列だけを見て
最終決定するものではない。

すべてを基本的にはlistとしてparseした後、phase-awareな名前解決および
macro expansionによって、次のいずれかに分類する。

```text
Core special form
macro application
ordinary function application
invalid form
```

***

### `DD-SYN-003`: `doc`によるdoc modeへの移行

`確定`

code modeでは、`doc` reader formによってdoc modeへ入る。

概念例：

```lisp
(doc
  これは通常の文章です。)
```

`doc`という名称をRPX v1のreader formとして採用する。

`doc`は、組込みの`document` runtime型を直接生成する特殊な評価項ではない。

`doc`の役割は、doc modeで読み取ったtext、commandおよびembedded codeを、
通常のsyntax objectへreader expansionすることである。

reader expansion後のsyntax objectが持つ意味および結果型は、
importされたdoc packageおよび通常の名前解決によって決まる。

したがって、次を区別する。

```text
doc:
reader modeを切り替えるreader form

document:
packageが定義し得る通常の型または値
```

`doc`を使用したことだけを理由に、特定の`document`型、`block`型または
`inline`型を組込みで割り当ててはならない。

***

### `DD-SYN-004`: doc mode

`確定`

doc modeは、通常のtextを読み取り、`@`によってcommandまたはcode formへ移行する
reader modeである。

doc modeの入力は、概念的に次の要素から構成される。

```text
text chunk
named command
embedded code form
nested doc body
triviaまたはdelimiter
```

doc modeは独立したruntime言語ではない。

doc modeで読み取られた要素はsyntax objectへ変換され、その後は通常のRPX pipelineへ渡す。

```text
doc source
→ reader expansion
→ syntax object
→ macro expansion
→ name resolution
→ type／effect checking
→ typed Core
→ evaluation
```

doc mode専用のruntime evaluator、hidden global stateまたは
名前解決を迂回する特別な実行機構を設けない。

***

### `DD-SYN-005`: doc modeからcode modeへの移行

`確定`

doc modeでは、少なくとも次の二種類のcode entryを持つ。

#### Named command

```text
@identifier
```

named doc commandを開始する。

概念例：

```lisp
(doc
  @heading{Introduction})
```

#### Embedded code form

```text
@(form)
```

単一のcode formを埋め込む。

概念例：

```lisp
(doc
  半径は @(show radius) です。)
```

`@identifier`に続くargument、optionおよびbodyを表すbracket／brace構文の完全な規則は、
`OPEN-SYN-002`で確定する。

次も`OPEN-SYN-002`へ移管する。

* command argumentを省略できる条件
* 複数argumentの表現
* bracketとbraceの役割
* literal `@`
* nested brace
* delimiter recovery
* embedded code内のcomment
* doc textの改行処理

***

### `DD-SYN-006`: reader mode stack

`確定`

readerはcode modeとdoc modeのmode stackを持つ。

有限深さで、次の入れ子を許す。

```text
code
→ doc
→ code
→ doc
→ ...
```

例：

```lisp
(doc
  外側の文章

  @(render-fragment
     (doc
       内側の文章)))
```

各mode遷移は、対応するdelimiterによって終了しなければならない。

readerは、現在のmode、開始delimiter、開始spanおよび親modeを保持する。

概念構造：

```text
ReaderModeFrame {
  mode,
  opening-delimiter,
  opening-span,
  parent-mode
}
```

mode stackの深さによって、同じsyntaxの意味が不必要に変化してはならない。

不正なdelimiter、閉じ忘れおよびmode不一致は、対応する開始spanを含む
reader diagnosticとして報告する。

malformed inputをlossless CSTとしてどこまで保持するかは、`LEX-001`および
`OPEN-SYN-002`で確定する。

***

### `DD-SYN-007`: doc reader expansion

`確定`

doc readerは、doc itemを通常のsyntax objectへ変換する。

概念例：

```lisp
(doc
  Hello @em{world})
```

は、概念的に次のようなsyntax objectへ展開できる。

```lisp
(doc-node
  (text "Hello ")
  (call em
    (doc-node
      (text "world"))))
```

`doc-node`、`text`および`call`は説明用の名称であり、実際のconstructor名または
function名は`OPEN-SYN-002`およびdoc package仕様で決定する。

reader expansionは次を保存しなければならない。

* source order
* source span
* mode transition
* text chunkのorigin
* command identifierのorigin
* embedded code formのorigin
* nested doc bodyのorigin
* macroおよび診断に必要なprovenance

reader expansionによって生成されたidentifierは、単なる文字列ではなく、
通常のsyntax objectとしてscope、phaseおよびorigin情報を持たなければならない。

***

### `DD-SYN-008`: doc commandの名前解決

`確定`

doc reader自身は、command identifierの意味を決定しない。

readerは次をsyntax objectとして生成する。

```text
command identifier
command arguments
command body
source origin
```

その後、通常のphase-aware名前解決によってcommand bindingを解決する。

```text
doc reader
→ syntax object
→ macro expansion
→ name resolution
```

commandは、importされたpackageの通常のvalue、syntax bindingまたは
doc packageが定義する適切なbindingへ解決される。

正確にどのnamespaceを参照するかは、macro phaseおよびdoc commandの種類に応じて
`OPEN-MAC-001`および`OPEN-SYN-002`で確定する。

未束縛commandは静的なname errorとする。

```lisp
(doc
  @unknown{text})
```

について、`unknown`へ対応するbindingが存在しなければ拒否する。

診断例：

```text
unbound doc command:
  unknown

source:
  @unknown{text}
```

未知commandを暗黙にplain textへflattenしてはならない。

未知command fallbackをpackageごとに暗黙設定する機能はRPX v1へ導入しない。

必要な場合は、明示的なcommand、macro、reader extensionまたは通常関数として設計する。

***

### `DD-SYN-009`: doc commandの評価順序

`確定`

doc reader expansionはsource orderを保存する。

reader expansion後のdoc itemが通常の関数適用またはconstructor applicationとして
評価される場合、その評価順序はCoreの通常規則に従う。

```text
operator first
arguments left-to-right
body items source order
```

doc modeであることを理由に、通常の評価順序を変更してはならない。

doc commandのbodyが複数itemを持つ場合、そのitemをどのcollectionまたは
domain valueへ変換するかはdoc packageが決定する。

ただし、reader expansionおよび評価によってsource orderを失ってはならない。

***

### `DD-SYN-010`: `src`の廃止

`確定`

RPX source fileはcode modeを既定とするため、`src` wrapperをRPX v1の
Surface構文へ含めない。

次のsourceはRPX v1の正規構文ではない。

```lisp
(src
  (val x 1)
  (val y 2))
```

正しくは、declarationをtop-levelへ直接記述する。

```lisp
(val x 1)
(val y 2)
```

legacy `src`を受理するcompatibility syntax、migration period、
deprecation warningおよび自動fix-itを規範仕様に設けない。

`src`という名前が通常のbindingとして定義されていない場合、上記形式は通常の
名前解決規則によって未束縛identifierまたは不正applicationとして拒否される。

現行実装において`src`内部だけを特別に型検査・評価する挙動は、
最終RPX仕様へ継承しない。

既存sourceを移行する必要がある場合は、言語処理系の互換意味論ではなく、
独立した一回限りのsource変換toolによって対応してよい。

その変換toolは最終RPX仕様の一部ではない。

***

### `DD-SYN-011`: `src`のscopeおよび意味を継承しない

`確定`

現行実装の`src` wrapperが持つ可能性のある次の挙動を、RPX v1へ継承しない。

* `src`内部だけを型検査する
* `src`内部だけを評価する
* `src`外部をplain textとして扱う
* `src`内部だけeffect operationを許す
* `src`内部だけ異なる名前解決を使う
* `src`境界をmoduleまたはlexical scopeとして扱う
* `src`境界をmacro phase境界として扱う

top-level codeは通常のdeclaration groupとして処理する。

```text
source file
→ top-level declaration collection
→ reader／macro expansion
→ name resolution
→ type／effect checking
→ module elaboration
```

`src`に由来する暗黙scopeを設けない。

***

### `DD-SYN-012`: reader formとbinding identity

`確定`

`doc`を単なる通常関数名として認識するか、readerが特別に認識するreader formとするかは、
具象構文上の循環を避けるため区別する。

RPX v1では、code modeからdoc modeへ切り替えるための`doc`をreader formとして認識する。

readerは、doc bodyを読取る前にmodeを切り替える必要があるため、通常の関数適用として
全formを読んだ後では遅い。

ただし、reader expansion後に生成されるdoc constructorおよびcommandは、
通常のbinding identityによって解決する。

したがって次を区別する。

```text
reader-level `doc`:
mode切替に必要な構文

expanded doc constructors／commands:
通常の名前解決対象
```

`doc`のshadowingを許可するか、reader-level nameとして予約するかの最終字句規則は
`OPEN-SYN-002`で確定する。

RPX v1の基本方針として、reader mode切替に必要な`doc`を通常bindingによって
偶発的に無効化してはならない。

***

### `DD-SYN-013`: file、module、packageとの関係

`確定`

source fileの既定がcode modeであることと、fileがそのままmoduleであることを
同一視しない。

次は別々の設計事項である。

```text
reader mode:
source bytesをどのようなformとして読むか

declaration group:
読み取られたdeclarationのscopeと対応関係

module:
type／value／syntaxの抽象化およびvisibility

package:
配布、version、dependencyおよびroot module
```

`OPEN-SYN-001`はreader modeだけを固定する。

一つのfileが一つのmoduleを形成するか、複数fileでmoduleを形成するか、
file内に明示的な`module` declarationを必要とするかは`OPEN-MOD-001`で決定する。

### 現行状態

現行実装では、ファイル全体のreader処理と`src`内部の限定的な型検査・effect処理が
結び付いている場合がある。

### 最終仕様

RPX v1では次の構造とする。

```text
file default:
code mode

doc entry:
(doc ...)

legacy src:
なし
```

### 必要な実装移行

* parser／readerから`src`固有modeを除去する。
* `src`内部だけを処理する限定pipelineを廃止する。
* top-level declarationを通常pipelineへ送る。
* `doc`だけを明示的なreader mode切替として保持する。
* doc reader expansion後のformを通常のresolver／checkerへ送る。
* 未知doc commandのplain-text flattenを停止する。
* `src`を前提とするfixtureおよびexampleは正規sourceへ書き換える。
* `src`互換層は実装しない。

既存exampleの移行はsourceの一括変換として行ってよいが、その変換機構を
runtimeまたはparserの恒久機能にしてはならない。

***

### 適合試験

#### Code modeが既定

```lisp
(val x 1)
(val y 2)
(+ x y)
```

期待：

```text
top-level codeとして読まれる
`src` wrapperを要求しない
```

***

#### Doc modeへの移行

```lisp
(doc
  Hello)
```

期待：

```text
`doc` bodyだけをdoc modeで読む
```

***

#### Embedded code

```lisp
(val radius 40mm)

(doc
  半径は @(show radius) です。)
```

期待：

```text
`(show radius)`をcode formとして読む
source orderとoriginを保持する
```

***

#### Named command

```lisp
(doc
  @em{important})
```

期待：

```text
`em`をcommand identifierとしてreader expansionする
通常の名前解決でbindingを解決する
```

***

#### Unknown command

```lisp
(doc
  @unknown{text})
```

`unknown`が未束縛の場合の期待：

```text
静的name error
plain textへflattenしない
```

***

#### Nested mode

```lisp
(doc
  outer
  @(render
     (doc
       inner)))
```

期待：

```text
code → doc → code → docのmode stackを正しく処理する
```

***

#### `src`の拒否

```lisp
(src
  (val x 1))
```

`src`が通常bindingとして定義されていない場合の期待：

```text
静的エラー
compatibility warningではない
```

***

#### `src`固有scopeを持たない

```lisp
(val x 1)
(val y (+ x 1))
```

期待：

```text
通常のtop-level declaration groupとして処理する
`src`境界を必要としない
```

***

#### Reader expansionのsource order

```lisp
(doc
  first
  @command{second}
  third)
```

期待：

```text
first
command
third
```

の順序をsyntax objectおよびprovenanceで保持する。

***

#### Delimiter error

```lisp
(doc
  @em{missing)
```

期待：

```text
reader error
doc modeの開始位置と不一致delimiterの位置を診断する
```

error recoveryとlossless CST保持の正確な範囲は`OPEN-SYN-002`へ移管する。

***

### 未解決事項の移管

#### `OPEN-SYN-002`

次を移管する。

* UTF-8、BOM、invalid byte sequence
* whitespaceおよびnewline
* comment
* Unicode identifier
* identifier normalization
* operator identifier
* qualified name
* reserved word
* literal
* string escape
* raw string
* number
* unit suffix
* symbol／keyword
* `doc` commandのbracket／brace文法
* literal `@`
* nested delimiter
* doc text chunkの正規化
* reader error recovery
* top-levelおよび局所declaration group
* 型構文
* `doc`をlexically reservedにするか

#### `OPEN-MAC-001`

次を移管する。

* reader expansionとmacro expansionの完全なphase順序
* reader-generated syntax objectのscope
* doc commandがsyntax bindingを参照する場合のphase
* reader macroの権限
* intentional capture
* expansion fuel

#### `OPEN-MOD-001`

次を移管する。

* fileとmoduleの対応
* module内のdoc form
* doc packageのimport
* command visibility
* separate compilation時のreader dependency

#### `OPEN-DAT-001`

次を移管する。

* doc readerが生成するdata／variant表現
* doc itemのpattern match
* text／command nodeのconstructor
* doc fragment型の標準表現

***

### 解決後の基本原則

```text
file default:
code mode
```

```text
doc entry:
(doc ...)
```

```text
docからcode:
@identifier
@(form)
```

```text
reader:
mode stackを持つ
source orderとoriginを保持する
```

```text
doc command:
readerでは意味を解決しない
通常の名前解決へ送る
未束縛なら静的エラー
```

```text
src:
RPX v1では廃止
互換構文なし
migration periodなし
warning／fix-itなし
```

```text
現行src sourceの移行:
必要なら独立した一回限りの変換toolを使用
言語仕様またはruntime互換機能にはしない
```

以上により、RPX v1のfile既定mode、`doc` reader form、mode stack、
doc commandの名前解決および`src`の完全廃止が特定されたため、
`OPEN-SYN-001`を解決済みとする。

## `OPEN-SYN-002` 字句・Surface構文の完全化

### 0. 結論

`OPEN-SYN-002`は、以下の方針で**解決済み**とする。

この項目で確定する範囲は次のとおり。

* source encoding、改行、BOM、shebang
* 空白と構造化コメント
* 識別子、予約形式、演算子名
* 数値・文字列・真偽値・unit・bytes
* symbol／keywordを導入しない方針
* 宣言グループ
* 関数・list・tuple・recordの基本構文
* 型構文
* 汎用的な構造化文章領域である`markup`
* 構文エラー回復
* lossless CSTへのsource情報の保存

単位、色、画像、URL、markup commandなどの**意味と型はpackageが定義する**。ただし、それらを記述するために必要な一般構文は本項目で確定する。

***

### 1. Source fileと文字コード

#### 1.1 文字コード

RPX sourceはUTF-8だけを受理する。

```text
source encoding:
UTF-8
```

不正なUTF-8はlexerより前のsource decoding errorとする。

```text
invalid UTF-8:
source decoding error
```

通常の文字列も、妥当なUTF-8として表現可能なUnicode scalar value列だけを保持する。

***

#### 1.2 BOM

UTF-8 BOMはsource先頭に限って許可する。

```text
- byte offset 0に限る
- lossless CSTへ保持する
- RPXの意味には影響しない
- shebangとは併用できない
```

BOMとshebangが同時に存在した場合はsource errorとする。

***

#### 1.3 改行

次の改行形式を受理する。

```text
LF
CRLF
CR
```

行番号の計算上はいずれも一改行として扱う。

一方、lossless CSTではsourceに書かれた元の改行bytesを保持する。

***

#### 1.4 Source span

内部のsource spanはUTF-8 byte offsetによる半開区間とする。

```text
[start, end)
```

行・列番号は診断表示用にsource mapから算出する。

Unicode code point offsetや表示上のcolumn幅を、規範的なsource identityには使用しない。

***

#### 1.5 Shebang

Unix系環境での直接実行用に、任意のshebangを許可する。

```text
#!/usr/bin/env rpx
```

規則：

```text
- byte offset 0から始まる
- 第一行に限る
- RPXの型、module、effect、評価結果には影響しない
- lossless CSTへ保持する
- BOMとは併用できない
```

***

### 2. 空白とコメント

#### 2.1 Code modeの空白

code modeでは、少なくとも次を空白として認識する。

```text
space
tab
form feed
newline
```

空白はtoken境界を作るが、通常はsemantic ASTには渡さない。

lossless CSTではすべて保持する。

***

#### 2.2 構造化コメント

コメント構文は次の一種類とする。

```lisp
(// コメント)
```

複数行も許可する。

```lisp
(//
  この範囲はコメントです。

  (val old-value 10)

  (render old-value))
```

コメント内部の内容は次の処理対象にならない。

```text
- macro展開
- 名前解決
- 型検査
- effect推論
- 評価
- 通常の未使用warning
```

***

#### 2.3 コメントの入れ子

構造化コメントは入れ子可能とする。

```lisp
(//
  外側のコメント

  (//
    内側のコメント)

  外側の続き)
```

readerはコメント終了位置を判断するため、コメント内部で次だけを追跡する。

```text
- 丸括弧の対応
- 文字列の開始と終了
- 文字列delimiter
- 入れ子の構造化コメント
```

コメント内の通常コードをparse、名前解決、型検査してはならない。

***

#### 2.4 コメント内部の括弧

コメント内部でも括弧対応を必須とする。

```lisp
(//
  (val x
    (+ 1 2)))
```

は有効。

括弧対応が壊れた範囲を任意に囲めるraw commentは、v1では導入しない。

必要性が確認された場合に、別のraw comment形式として将来検討する。

***

#### 2.5 採用しないコメント

次は導入しない。

```text
// 行末コメント
; 行末コメント
(* ... *)
```

裸の`//`を採用しないことで、`/`や演算子構文との競合を避ける。

コメントは原則として独立したformとして配置する。

***

#### 2.6 CSTでの保持

コメントは削除せず、専用nodeとしてlossless CSTへ保存する。

概念的には次の情報を持つ。

```text
StructuredComment {
  opening-span,
  content-span,
  closing-span,
  raw-bytes
}
```

通常ASTやtyped Coreからは除外する。

***

### 3. 識別子

#### 3.1 Unicode識別子

通常識別子にはUnicodeを許可する。

```lisp
(val radius 40mm)

(val 半径 40mm)

(val 面積
  (* 幅 高さ))
```

識別子の基礎規則にはUnicodeの次の分類を使う。

```text
XID_Start
XID_Continue
```

日本語の漢字、ひらがな、カタカナなどは使用可能とする。

***

#### 3.2 Unicode正規化

識別子は名前解決前にNFC正規化する。

```text
元の綴り:
lossless CSTへ保持

NFC正規化後:
名前解決、重複判定、BindingId生成に使用
```

同じscopeで、異なるraw spellingがNFC正規化後に一致する場合はduplicate binderとする。

通常文字列は自動的にNFC正規化しない。

***

#### 3.3 先頭文字

caseを持つ文字では、小文字開始だけを許可する。

```text
report        許可
report-title  許可
半径          許可
レポート      許可

Report        拒否
REPORT        拒否
42-pages      拒否
-page         拒否
_report       拒否
```

caseを持たない日本語文字等は識別子の先頭へ使用できる。

型、constructor、moduleなども、大文字開始によって分類しない。名前のcategoryはnamespace、binding kind、BindingIdによって区別する。

***

#### 3.4 Kebab-case

複数語の区切りにはhyphenを使う。

```text
report-title
last-good-render
source-edit
```

制約：

```text
- 先頭に置けない
- 末尾に置けない
- 連続させられない
```

単独の`-`はoperator identifierとして別に扱う。

***

#### 3.5 Underscore

単独の`_`だけをwildcardとして予約する。

```lisp
(fn (_ value)
  value)
```

`_`はbindingを作らず、参照もできない。

通常識別子の一部には使用しない。

```text
report_title  拒否
_value        拒否
value_        拒否
```

数値separatorの`_`は別規則として許可する。

***

#### 3.6 `?`と`!`

通常識別子の末尾に限り、`?`または`!`を一つ付けられる。

```text
empty?
valid?
commit!
flush!
```

途中や重複は認めない。

```text
is?empty   拒否
value??    拒否
commit!!   拒否
valid?!    拒否
```

`?`と`!`は命名慣習であり、型やeffectの意味を直接持たない。

***

#### 3.7 不可視文字

識別子では次を禁止する。

```text
- bidi control
- zero-width space
- zero-width joiner
- zero-width non-joiner
- その他の不可視format文字
```

見た目が紛らわしい識別子や、不自然なscript混在はwarning対象にできる。

***

### 4. Package名とmodule path component

通常の局所識別子にはUnicodeを許可するが、package名とmodule path componentはASCII lowercase kebab-caseに限定する。

```text
許可:
report
report-layout
japanese-typesetting
graphics2

拒否:
Report
日本語組版
report_layout
-report
report-
report--layout
```

`/`は通常識別子には含めず、module path用に予約する。

`.`も通常識別子には含めず、module仕様で用途を決定する。

***

### 5. Operator identifier

v1ではoperator identifierを固定一覧とする。

```text
+
-
*
/
=
<
>
<=
>=
!=
```

`and`、`or`、`not`は通常識別子として提供する。

自由な記号列による利用者定義operatorは導入しない。

RPXにはinfix構文がないため、operatorもprefix applicationとして使う。

```lisp
(+ 1 2)
```

```lisp
(<= x y)
```

***

### 6. Core特殊形式

初期のCore特殊形式として次を予約する。

```text
markup
fn
val
type
type-alias
local
rec
let
letrec
if
seq
var
set
handle
with
```

Core特殊形式はshadowing禁止とする。

```lisp
(val if value)
```

は静的エラー。

通常のpackage関数や利用者bindingは、通常のscope規則に従ってshadowing可能とする。

将来使用する可能性がある名前を、必要になる前から大量に予約しない。

***

### 7. 数値リテラル

#### 7.1 整数

`int`は任意精度の符号付き整数とする。

```lisp
0
42
123456789012345678901234567890
```

基数：

```text
42       10進
0b101010 2進
0o52     8進
0x2a     16進
```

基数prefixは小文字だけを正式構文とする。

16進digitの`a`〜`f`は大小文字を許可する。

```lisp
0xff
0xFF
0xFf
```

***

#### 7.2 先頭ゼロ

不要な先頭ゼロを持つ10進整数は拒否する。

```text
許可:
0
7
70
0o7

拒否:
00
007
0123
```

固定桁の識別文字列等は文字列で表す。

***

#### 7.3 負数

負号の直後に空白なしで数字が続く場合、負の数値literalとして読める。

```lisp
-42
-3.5
-0xff
```

通常の符号反転も別に許可する。

```lisp
(- value)
```

したがって次は構文木上で異なる。

```text
-42:
負数literal

(- 42):
符号反転application
```

***

#### 7.4 数字separator

数字間のunderscoreをseparatorとして許可する。

```lisp
1_000_000
0b1010_1100
0xff_ff_ff
3.141_592
```

underscoreは値へ影響しない。

次は拒否する。

```text
_1000
1000_
1__000
0x_ff
1_.5
1._5
1e_10
```

***

#### 7.5 `f64`

次を`f64` literalとする。

```lisp
0.0
3.14
-2.5
1e10
1e+10
1e-10
1.5e3
```

小数点の前後には数字を要求する。

```text
許可:
0.5
1.0

拒否:
.5
1.
```

指数記号は小文字`e`を正式構文とする。

16進浮動小数点literalはv1では導入しない。

***

#### 7.6 非有限値

NaNとInfinityをliteralとして提供しない。

```text
nan:
literalではない

infinity:
literalではない
```

`f64` literalが有限範囲を超える場合は静的エラーとする。

ゼロへunderflowする場合はwarning対象とする。

`-0.0`は正のゼロへ勝手に正規化せず保持する。

***

### 8. 文字列

#### 8.1 基本方針

RPXの文字列literalでは、backslash escapeを使用しない。

```text
\:
通常の文字
```

すべての文字列はUnicode `str`値を作る。

通常文字列：

```lisp
"短い文字列"
```

長い文字列、引用符を含む文字列、複数行文字列：

```lisp
"""
複数行の文字列
"""
```

***

#### 8.2 文字列delimiter

一個の`"`は短い一行文字列を囲む。

```lisp
"Report"
```

三個以上の`"`は可変長delimiterとして使用できる。

```lisp
"""She said "hello"."""
```

内容に三個連続する引用符がある場合は、外側のdelimiterを増やせる。

```text
開始delimiterと終了delimiterでは、
同数の引用符を使用する。
```

二個の引用符は、空文字列との曖昧性を避けるためdelimiter長として使用しない。

```lisp
""
```

は空文字列。

***

#### 8.3 複数行文字列

三個以上の引用符で囲んだ文字列には物理改行を含められる。

```lisp
"""
第一段落です。

第二段落です。
"""
```

規則：

```text
- 開始delimiter直後の改行は値へ含めない
- 終了delimiter直前の改行は値へ含めない
- 内容中の物理改行はruntimeではLFへ正規化する
- 元の改行bytesはlossless CSTへ保持する
```

終了delimiterのindentationをbaselineとして、各非空行から同じprefixを除去する。

baselineより浅い非空行は字句エラーとする。

***

#### 8.4 特殊文字

文字列escapeは導入しない。

特殊文字は、通常の値や関数で生成する。

```lisp
newline
tab
carriage-return
nul
```

任意のUnicode scalar valueは次で生成できる。

```lisp
(unicode 0x3002)
```

`unicode`は有効なUnicode scalar valueなら一文字分の`str`を返す。

実行時値の場合は不正code pointを`result`で表す。

compile-time定数が明らかに不正なら静的エラーにできる。

***

#### 8.5 `str`の意味

`str`は次の意味を持つ。

```text
- 妥当なUTF-8
- 不変
- 所有された値として振る舞う
- NUL終端ではない
- 内部NULを保持可能
- bytesとは別型
- 自動Unicode正規化を行わない
```

具体的な物理表現は言語仕様へ固定しない。

Rust実装では`Arc<str>`等を利用できるが、将来のsmall-string optimization、interning、rope等を妨げない。

任意の整数による暗黙文字indexは提供しない。

```text
byte
Unicode scalar value
grapheme cluster
```

を必要に応じて明示的に区別する。

***

### 9. Boolとunit

真偽値literal：

```lisp
true
false
```

型関係：

```text
true  <: bool
false <: bool

bool ≃ union(true, false)
```

unit値：

```lisp
unit
```

空括弧`()`をunit値にはしない。

```text
unit : unit
```

何も返さない関数やbodyでも、必要なら`unit`を明示する。

***

### 10. Symbolとkeyword

runtime symbol literalとkeyword literalはv1では導入しない。

```text
'circle:
不採用

:circle:
不採用
```

役割分担：

```text
型付きの列挙値:
variant constructor

外部名・metadata:
strまたは専用nominal型

record field:
静的field label

macro:
syntax object

named argument:
必要になった時点で別途設計
```

`circle`、`heading`、`portrait`等は言語組込みではなく、packageがconstructorとして定義する。

***

### 11. Bytes

`bytes`は、0〜255の値からなる不変byte列とする。

```text
str:
valid UTF-8

bytes:
任意のbyte列
```

暗黙変換は行わない。

```lisp
(encode-utf8 text)
```

```text
str -> bytes
```

```lisp
(decode-utf8 data)
```

```text
bytes -> result<str, utf8-decode-error>
```

小さいbyte列は専用字句literalではなく、constructorで作る。

```lisp
(bytes 0x00 0xff 0x2a)
```

範囲外の静的要素はcompile-time error。

大きなbinaryはsourceへ埋め込まず、resource systemを使用する。

```text
resource:
外部データへの参照・authority

bytes:
実際に読み込まれた内容
```

***

### 12. 単位と色

#### 12.1 単位

RPXは、数値直後の空白なしsuffix構文を提供できる。

```lisp
40mm
2s
30deg
```

概念的には型付きunit constructorの適用である。

```lisp
(mm 40)
```

具体的な単位、結果型、変換規則はpackageが定義する。

```text
mm:
length package

s:
time package

deg:
angle package

fps:
motion package
```

一般的なdimension algebraはv1の言語核へ導入しない。

`%`、`em`、device pixel等の文脈依存量は固定倍率の絶対単位と同一視しない。

物理表現、fixed-point精度、canonical unit等は単位package／runtime仕様で引き続き確定する。

***

#### 12.2 色

色を言語組込みliteralにしない。

```text
#ff0000:
RPX組込みliteralではない
```

color packageがconstructorを提供する。

```lisp
(srgb8 255 0 0)
```

```lisp
(srgb 1.0 0.0 0.0)
```

```lisp
(oklch lightness chroma hue)
```

CSS形式が必要ならpackage parserを使う。

```lisp
(parse-css-color "#ff0000")
```

色空間、alpha、ICC profile、gamut mapping、補間空間等はcolor packageとrender pipelineの責任とする。

***

### 13. Source fileの宣言グループ

#### 13.1 Top-level

通常のsource fileのtop-levelは、順序付き宣言グループとする。

top-levelへ自由な実行式を置かない。

```lisp
(type title str)

(val title "Report")
```

effectfulな処理は、明示的な関数またはentry pointへ置く。

package import時に隠れたI/Oを発生させない。

***

#### 13.2 型注釈

値bindingへの型注釈は独立した宣言として書く。

```lisp
(type title str)

(val title "Report")
```

binder内の型注釈構文は導入しない。

```text
type annotation:
対応bindingより前

隣接:
必須ではない

同一declaration group:
必須

重複注釈:
禁止
```

通常implementation groupでは、対応するbindingを持たない型注釈はエラー。

***

#### 13.3 `val`

通常の`val`は逐次scopeを持つ。

```lisp
(val x 1)

(val y
  (+ x 1))
```

後方参照、自己参照は許可しない。

```lisp
(val y
  (+ x 1))

(val x 1)
```

は`x`未束縛。

注釈なしの`val`は許可し、型推論とvalue restrictionに従う。

***

#### 13.4 `rec`

自己再帰・相互再帰には明示的な`rec`宣言グループを使う。

```lisp
(rec
  (type even?
    (fn int bool))

  (type odd?
    (fn int bool))

  (val even?
    (fn (n)
      (if (= n 0)
          true
          (odd? (- n 1)))))

  (val odd?
    (fn (n)
      (if (= n 0)
          false
          (even? (- n 1))))))
```

規則：

```text
- group内のbindingは相互可視
- val右辺はelaboration後にfnでなければならない
- group検査中は単相
- polymorphic recursionは導入しない
- 検査完了後にvalue restrictionに従ってgeneralize可能
```

***

#### 13.5 `local`

局所宣言グループには`local`を使う。

```lisp
(local
  (val x 1)
  (val y (+ x 1))
  (+ x y))
```

文法：

```text
(local declaration* result-expression)
```

最後の式は必須。

内部的にはsequential `let`へ変換できる。

局所`rec`も許可する。

***

#### 13.6 重複binding

同じscopeでのduplicate binderは禁止。

```lisp
(val value 1)
(val value 2)
```

内側scopeでの通常shadowingは許可する。

top-level `var`は禁止する。

top-level `val` initializerはpureでなければならない。

***

### 14. 関数

#### 14.1 値構文

関数値はparameter listを明示する。

```lisp
(fn (x y)
  (+ x y))
```

0引数：

```lisp
(fn ()
  unit)
```

***

#### 14.2 関数型

関数型には、引数型を囲う追加の括弧を置かない。

```lisp
(fn int int)
```

```text
引数:
int

戻り値:
int
```

```lisp
(fn int int int)
```

```text
引数:
int, int

戻り値:
int
```

effect付き：

```lisp
(fn str unit
  (effects console))
```

pure functionでは`effects`節を省略する。

***

#### 14.3 Fixed arity

```lisp
(fn int int int)
```

は2引数のfixed-arity functionであり、curried functionではない。

Curried functionは明示的に入れ子にする。

```lisp
(fn int
  (fn int int))
```

自動部分適用はv1では導入しない。

```text
引数不足:
arity error

引数過剰:
arity error
```

部分適用が必要なら明示的な`fn`を作る。

***

### 15. List、tuple、record

#### 15.1 List

値：

```lisp
(list 1 2 3)
```

型：

```lisp
(list int)
```

空list：

```lisp
(list)
```

List値は不変。

異種要素から期待型なしで自動的なunion型を生成しない。異種listが必要なら型注釈を要求する。

***

#### 15.2 Tuple

値：

```lisp
(tuple 1 "title" true)
```

型：

```lisp
(tuple int str bool)
```

Tuple arityは2以上とする。

```text
0要素:
unitを使用

1要素:
元の型をそのまま使用
```

Tuple位置参照はcompile-timeに確定するindexを使う。

***

#### 15.3 Record

値：

```lisp
(record
  (title "Report")
  (page-count 10))
```

型：

```lisp
(record
  (title str)
  (page-count int))
```

field labelは通常変数ではなく、構造的なLabelIdとして扱う。

同じrecord内のduplicate fieldは禁止。

fieldのsource順序はCSTへ保持するが、recordのsemantic equalityはfield順序に依存しない。

***

#### 15.4 Field access

```lisp
(field report title)
```

`title`は評価される式ではなく静的field label。

***

#### 15.5 Updateとextension

既存fieldだけを書き換える。

```lisp
(record-update report
  (title "Updated"))
```

存在しないfieldだけを追加する。

```lisp
(record-extend report
  (author "Unknown"))
```

暗黙の上書きや、更新とextensionを混ぜるspread構文は導入しない。

***

### 16. 型構文

#### 16.1 型適用

```lisp
(list int)
```

```lisp
(option str)
```

```lisp
(result str utf8-decode-error)
```

一般形：

```text
(type-constructor type-argument ...)
```

***

#### 16.2 `forall`

```lisp
(forall ((a type))
  (fn a a))
```

kind：

```text
type
record-row
effect-row
```

例：

```lisp
(forall ((a type)
         (e effect-row))
  (fn a a
    (effects e)))
```

***

#### 16.3 集合論的型

```lisp
(union int str)
```

```lisp
(intersect
  (fn int int)
  (fn str str))
```

```lisp
(not int)
```

```lisp
(diff number int)
```

基本的な正規化：

```text
(union)        ≃ never
(intersect)    ≃ any
(union t)      ≃ t
(intersect t)  ≃ t
```

***

#### 16.4 Dynamic

境界付きdynamic型を使用する。

```lisp
(dynamic any)
```

```lisp
(dynamic number)
```

裸の`dynamic`は正式型として必須にせず、必要ならaliasとして提供する。

***

#### 16.5 Open record

Closed record：

```lisp
(record
  (title str))
```

Open record：

```lisp
(record
  (title str)
  (row r))
```

`row`は最後に一つだけ置ける。

Optional field：

```lisp
(record
  (title str)
  (optional subtitle str))
```

これは次とは異なる。

```lisp
(record
  (title str)
  (subtitle (option str)))
```

***

#### 16.6 Effect row

```lisp
(fn str unit
  (effects console resource))
```

Effect application：

```lisp
(fn int int
  (effects (state int)))
```

Effect-row変数：

```lisp
(forall ((e effect-row))
  (fn str unit
    (effects console e)))
```

Surface上の同一effectの重複記述は禁止する。

空effect rowは省略する。明示的な`(effects)`は受理してもよいが、formatterは省略形へ正規化できる。

***

#### 16.7 型alias

```lisp
(type-alias report
  (record
    (title str)
    (page-count int)))
```

`type-alias`はtransparent aliasであり、新しいnominal identityを作らない。

Recursive type aliasはv1では禁止する。再帰dataは`data` declarationで定義する。

***

### 17. `markup` reader

#### 17.1 位置づけ

文書専用の`doc`特殊形式は導入しない。

代わりに、構造化された文章・字幕・スライド本文・UI text等を記述する汎用reader形式として`markup`を導入する。

```lisp
(markup
  これは構造化された文章です。)
```

動画や図形全体を`markup`で記述する必要はない。

動画内の字幕、テロップ、title、説明文等で必要に応じて利用できる。

***

#### 17.2 Coreとpackageの分担

Core readerが知るもの：

```text
- markup領域の開始と終了
- text
- horizontal whitespace
- soft break
- paragraph break
- markup command
- code argument
- nested markup body
- embedded code
- source span
```

Core readerが知らないもの：

```text
- strongが太字か
- linkがURLか
- imageが何を表示するか
- headingが文書見出しか
- captionが動画字幕か
```

commandの意味と型はpackageが定義する。

***

#### 17.3 Markup command

基本形：

```lisp
@name
```

```lisp
@name[code-expression]
```

```lisp
@name(markup-body)
```

```lisp
@namemarkup-body
```

例：

```lisp
@page-number
```

```lisp
@space[20mm]
```

```lisp
@strong(重要)
```

```lisp
@link公式サイト
```

```lisp
@imageキャプション
```

***

#### 17.4 `[]`

角括弧内には通常のRPX code expressionをちょうど一つ書く。

```lisp
@image[logo]
```

```lisp
@link外部サイト
```

```lisp
@image[
  (record
    (source logo)
    (width 40mm))
](プロジェクトのロゴ)
```

空の`[]`は禁止。

複数の値が必要ならtupleまたはrecordへまとめる。

code引数は最大一つとする。

***

#### 17.5 `()`

`@name(...)`の丸括弧内は入れ子可能なmarkup bodyとして読む。

```lisp
@strong(
  この中には@term(専門用語)があります。
)
```

markup bodyは最大一つとする。

空bodyは許可する。

```lisp
@at()
```

本文なしの`@name`とは区別する。

***

#### 17.6 任意のcode埋込み

通常RPX codeを埋め込む場合は次を使う。

```lisp
@(show page-count)
```

```lisp
@(if detailed?
     detailed-content
     short-content)
```

`@(...)`内は通常のRPX S式として読む。

***

#### 17.7 糖衣展開

```lisp
@space[20mm]
```

は概念的に：

```lisp
(space 20mm)
```

へ変換される。

```lisp
@strong(重要)
```

は概念的に：

```lisp
(strong
  <markup-fragment "重要">)
```

へ変換される。

```lisp
@imageキャプション
```

は概念的に：

```lisp
(image
  path
  <markup-fragment "キャプション">)
```

へ変換される。

`@(space 20mm)`も意味上は同じ通常applicationを挿入できる。

正準styleでは、単純なmarkup commandは`@name[...]`形式を推奨する。

***

#### 17.8 `@at()`

文字としての`@`は、標準markup commandである`at`を使う。

```lisp
(markup
  user@at()example.com)
```

`@at()`は形式上、空のmarkup bodyを受け取る通常commandである。

reader専用の特殊escapeにはしない。

```text
@@:
不採用
```

`at`は標準markup packageまたはmarkup preludeが提供する。

```lisp
@code["@"]
```

は、`@`をcode表記として装飾する別用途である。

***

#### 17.9 丸括弧

Markup body内ではASCII丸括弧の対応を追跡する。

```lisp
@strong(
  RPX (prototype language) is statically typed.
)
```

対応の取れた括弧は通常textとして保持できる。

対応しない丸括弧を文章として示す必要がある場合、Core専用escapeは追加せず、`code`や`verbatim`等のpackage commandを使用する。

```lisp
@code[")"]
```

日本語の全角括弧`（ ）`は通常文字であり、delimiterではない。

***

#### 17.10 改行と空白

単一の物理改行は`SoftBreak`として保持する。

```lisp
(markup
  これは
  日本語です。)
```

Core readerは、改行をspaceへ変換するか削除するかを決めない。

空行は`ParagraphBreak`として保持する。

```lisp
(markup
  第一段落です。

  第二段落です。)
```

Horizontal whitespaceも早期に一つへ潰さず、範囲を保持する。

言語、用途、prose／verbatim等に応じた最終処理はpackageが行う。

***

#### 17.11 Markupの型

`markup`はexpected typeに応じてelaborate可能なsyntax formとする。

期待型がある場合：

```text
document-inline
caption-content
slide-heading
UI text
```

等のpackage固有型へelaborateできる。

期待型がない場合は、汎用的な`markup-fragment`型を既定とする。

Markup commandは通常のvalue bindingとして名前解決し、そのparameter型からmarkup bodyの期待型を得る。

***

### 18. 構文エラー回復

#### 18.1 基本原則

構文エラー時にもlossless CSTを返す。

使用する回復node：

```text
MissingToken
UnexpectedToken
ErrorNode
```

parserは意味のある式や値を勝手に補わない。

局所的で確実な閉じdelimiterだけを仮想挿入できる。

***

#### 18.2 不足した閉じ括弧

EOF等で`)`が不足している場合、仮想`MissingToken`)\`を挿入する。

複数不足している場合はopen delimiter stackに基づいて補う。

診断は可能な限り一つにまとめ、各開始位置を関連情報として示す。

***

#### 18.3 余分な閉じ括弧

対応する`(`を持たない`)`は削除せず、`UnexpectedToken`として保存する。

そのtokenを意味解析から無視し、後続formの解析を続ける。

***

#### 18.4 未終了文字列

一行文字列が閉じられないまま改行へ到達した場合、行末へ仮想`"`を挿入する。

複数行文字列が未終了の場合は、EOFまでを文字列として保持し、EOFに仮想終了delimiterを置く。

複数行文字列の途中で通常codeへ勝手に復帰しない。

***

#### 18.5 未終了コメント

構造化コメントが未終了の場合、EOFまでをコメントとして保持し、EOFに仮想`)`を置く。

コメント内部に見える宣言へ勝手に復帰しない。

***

#### 18.6 Markupの`[]`

一つのcode expressionを正常に読み終えた後、`]`が不足していると判断できる場合は、その位置へ仮想`]`を挿入できる。

`[]`内に複数のcode expressionがある場合は、argument全体を`ErrorNode`にする。

最初の式だけを採用して残りを捨ててはならない。

診断では、recordまたはtupleへまとめる修正を提案できる。

***

#### 18.7 Markup body

Markup bodyの`)`が不足している場合、delimiter stackに従って最も内側の未終了formを処理する。

EOFまたは確実な外側同期点で仮想`)`を挿入する。

***

#### 18.8 不正な`@`

`@`の後にcommand名または`(`がない場合、`@`だけを`UnexpectedToken`として保持する。

後続textの解析は継続する。

診断例：

```text
expected a markup command name or `(` after `@`

examples:
  @strong(text)
  @(show value)

to write the `@` character:
  @at()
```

***

#### 18.9 未知command

```lisp
@strog(重要)
```

は構文としては正常。

未知command名はparser errorではなく、名前解決errorとして扱う。

```text
unbound markup command:
  strog

did you mean:
  strong
```

***

#### 18.10 型検査のcascade抑制

式位置の`ErrorNode`には内部専用のerror typeを与える。

error typeは任意の期待型へ暫定適合できるが、正常な型として外部へ公開しない。

これにより、一つのsyntax errorから大量の二次的型エラーが発生することを防ぐ。

***

#### 18.11 Formatter

構文エラーを含むsourceについて、formatterは仮想tokenを勝手に実体化しない。

```text
正常node:
format可能

ErrorNode:
元bytesを保持

MissingToken:
sourceへ自動挿入しない
```

修正は明示的なcode actionとして提供できる。

```text
Quick fix:
Insert missing `)`
```

***

### 19. 適合例

```lisp
#!/usr/bin/env rpx

(// Report definition)

(type report-title str)

(val report-title
  "進捗報告")

(type make-heading
  (fn str markup-fragment))

(val make-heading
  (fn (title)
    (markup
      @strong(@(show title)))))

(type report-options
  (record
    (title str)
    (page-size page-size)
    (optional subtitle str)))

(val options
  (record
    (title report-title)
    (page-size a4)))

(type body markup-fragment)

(val body
  (markup
    @heading(概要)

    この文書は@strong(型付き)の制作言語で生成されます。

    詳細は@link仕様書を参照してください。

    @image[
      (record
        (source architecture-diagram)
        (width 120mm))
    ](RPXの処理pipeline)

    連絡先はuser@at()example.comです。))
```

***

### 20. 不適合例

#### 不要な先頭ゼロ

```lisp
(val value 007)
```

#### 大文字開始識別子

```lisp
(val Report "...")
```

#### Underscoreを含む識別子

```lisp
(val report_title "...")
```

#### 重複field

```lisp
(record
  (title "First")
  (title "Second"))
```

#### 1要素tuple

```lisp
(tuple 1)
```

#### 自動部分適用

```lisp
(val increment
  (add 1))
```

`add`が2引数fixed-arity関数ならarity error。

#### Markup引数内の複数式

```lisp
@imageキャプション
```

#### 文字としての`@`に`@@`を使用

```lisp
user@@example.com
```

`@@`は正式escapeではない。次を使う。

```lisp
user@at()example.com
```

***

### 21. 本項目で意図的に確定しない事項

`OPEN-SYN-002`の外へ回す事項は次のとおり。

```text
- data／variantの最終構文
- patternとmatch
- constructor pattern
- exhaustiveness
- module、signature、functor
- import／export
- package manifest
- named／optional function argument
- variadic function
-一般的な部分適用糖衣
- 利用者定義operator
- 一般的なdimension algebra
- 単位の正確な内部表現
- fixed-point精度
- color managementの詳細
- ICC profile
- stream API
- resource declarationの最終構文
- markup package固有command
- markup-fragmentから各domain型への具体的elaboration protocol
```

***

### 22. 状態

```text
OPEN-SYN-002:
RESOLVED
```

下位項目：

```text
OPEN-SYN-002A
文字コード・空白・コメント
→ RESOLVED

OPEN-SYN-002B
識別子・予約形式
→ RESOLVED

OPEN-SYN-002C
リテラル
→ RESOLVED

OPEN-SYN-002D
宣言グループ
→ RESOLVED

OPEN-SYN-002E
型構文
→ RESOLVED

OPEN-SYN-002F
markup reader
→ RESOLVED

OPEN-SYN-002G
構文エラー回復
→ RESOLVED
```

## OPEN-DAT-001 代数的データ型・constructor・pattern・`match`

### DD-001 決定概要

#### DD-001.1 状態

```text
Status:
RESOLVED

Scope:
代数的データ型
値constructor
constructor固有型
pattern
match
網羅性検査
再帰data型
相互再帰data型
抽象化境界
```

#### DD-001.2 中心的な決定

```text
- 代数的データ型の宣言には`data`を使う
- `type`は値bindingへの型注釈として維持する
- `type-alias`はtransparent aliasに使う
- data型はsealedである
- constructorは通常のvalue namespaceに置く
- constructor固有型を型namespaceに生成する
- constructor適用はconstructor固有型を返す
- constructor固有型は親data型のsubtypeである
- match caseは`pattern -> expression`形式で書く
- 非網羅matchと到達不能caseは静的エラーにする
- 再帰data型にはstrict positivityを要求する
- 通常のdata値は不変、strict、非循環である
```

#### DD-001.3 宣言名

通常のADTには`data`を使用する。

```lisp
(data option
  ((a type))

  none
  (some a))
```

`adt`は予約語にしない。GADTを導入する場合も、原則として`data`宣言の拡張として設計する。

```text
type:
値bindingへの型注釈

type-alias:
transparentな型alias

data:
sealed nominal ADT
```

***

### 0. 適用範囲

#### 0.1 本項目が定めるもの

本項目は次を規定する。

```text
- data宣言のSurface構文
- 型parameter
- value constructor
- constructor固有型
- constructorの型parameter推論
- recursive data
- mutually recursive data
- patternの種類
- matchの構文
- patternによるbinding
- 型精緻化
- exhaustiveness
- unreachable case
- sealed constructor集合
- abstract export時の意味
- data値の評価と不変性
```

#### 0.2 本項目が定めないもの

次は別のOPEN項目へ移管する。

```text
- module signatureの最終構文
- import／export構文
- GADT
- existential constructor
- open variant
- derive
- 自動等値性
- serialization
- runtime reflection
- proper tail call
- ownership／borrow
- lazy value
- mutable cyclic graph
```

***

### 1. `data`宣言

#### 1.1 Parameterを持たないdata型

```lisp
(data alignment
  left
  center
  right
  justify)
```

これは新しいnominal型`alignment`と、4つのconstructorを定義する。

#### 1.2 Parameterを持つdata型

```lisp
(data option
  ((a type))

  none
  (some a))
```

Parameter節では、parameter名とkindを組にする。

```text
(a type)
```

複数parameterの例：

```lisp
(data result
  ((value-type type)
   (error-type type))

  (ok value-type)
  (err error-type))
```

#### 1.3 Parameterなしの場合

Parameterがないdata宣言では、空のparameter節を書かない。

適合：

```lisp
(data alignment
  left
  center
  right)
```

不適合：

```lisp
(data alignment
  ()

  left
  center
  right)
```

#### 1.4 Constructor payload

Constructorは0個以上の位置payload型を持てる。

```lisp
(data shape
  (circle point length)
  (rectangle point size)
  (path path-data))
```

複雑なpayloadについては、1個のrecord型をpayloadとして使用できる。

```lisp
(data shape
  (circle
    (record
      (center point)
      (radius length))))
```

***

### 2. `data`宣言が生成するbinding

#### 2.1 親type constructor

```lisp
(data option
  ((a type))

  none
  (some a))
```

は型constructor`option`を生成する。

```text
option:
type -> type
```

具体型：

```lisp
(option int)
```

```lisp
(option str)
```

#### 2.2 Value constructor

値namespaceには次を生成する。

```text
none
some
```

概念型：

```text
none:
forall a.
option<a>

some:
forall a.
a -> option<a>
```

#### 2.3 Constructor固有型

型namespaceにはconstructorごとのcase typeを生成する。

```lisp
(none int)
```

```lisp
(some int)
```

関係：

```text
(none int) <: (option int)
(some int) <: (option int)
```

#### 2.4 Sealed constructor集合

`data`宣言は、網羅性検査に使用するsealed constructor集合を生成する。

```text
option<a>
=
none<a>
または
some<a>
```

他packageはconstructorを追加できない。

#### 2.5 Runtime reflection

`data`宣言は、通常のRPX値としてruntime type descriptorを自動生成しない。

次のような機能は別項目とする。

```lisp
(type-of-value value)
```

```lisp
(constructors-of option)
```

***

### 3. 名前とnamespace

#### 3.1 型namespaceと値namespace

Constructor名は値位置と型位置で異なる意味を持つ。

値位置：

```lisp
(some 42)
```

型位置：

```lisp
(some int)
```

値位置ではconstructor application、型位置ではconstructor固有型である。

#### 3.2 型名とconstructor名の同名禁止

次は拒否する。

```lisp
(data box
  ((a type))

  (box a))
```

型位置の`(box int)`が親型とcase typeのどちらか判別できないためである。

代わりに異なる名前を使う。

```lisp
(data boxed
  ((a type))

  (box a))
```

#### 3.3 Constructor名の重複

同じdata宣言内で同じconstructor名を複数回定義できない。

```lisp
(data result
  (ok int)
  (ok str))
```

これはoverloadではなくduplicate constructor errorとする。

#### 3.4 Constructor identity

Constructorは文字列ではなく`ConstructorId`によって識別する。

異なるpackageの同名constructorは異なるidentityを持つ。

```text
graphics/circle
diagram/circle
```

***

### 4. Nullary constructor

#### 4.1 宣言

Payloadを持たないconstructorは裸のidentifierで宣言する。

```lisp
(data alignment
  left
  center
  right)
```

#### 4.2 値

Nullary constructorは値そのものである。

```lisp
left
```

次のようには書かない。

```lisp
(left)
```

`(left)`は0引数関数適用と解釈される。

#### 4.3 関数ではない

Nullary constructorは0引数関数ではない。

常に`none`を返す関数が必要なら明示的に書く。

```lisp
(fn (_)
  none)
```

***

### 5. Payload constructor

#### 5.1 値構築

```lisp
(circle center 20mm)
```

Constructorは通常のfixed-arity pure function valueとして使用できる。

#### 5.2 Constructorの高階利用

Payload constructorは高階関数へ渡せる。

```lisp
(map some values)
```

`values : list<int>`なら、`some`は使用位置で次へinstance化される。

```text
int -> option<int>
```

#### 5.3 Arity

宣言されたpayload数と適用時の引数数は一致しなければならない。

```lisp
(data shape
  (circle point length))
```

適合：

```lisp
(circle center radius)
```

不適合：

```lisp
(circle center)
```

```lisp
(circle center radius extra)
```

***

### 6. Constructor固有型

#### 6.1 精密な結果型

Constructor適用は親型ではなく、constructor固有型をprincipal typeとして返す。

```lisp
(some 42)
```

精密型：

```lisp
(some int)
```

親型：

```lisp
(option int)
```

#### 6.2 Subtyping

```text
some<int> <: option<int>
none<int> <: option<int>
```

したがって、親型を要求する場所へ明示変換なしで渡せる。

#### 6.3 親型とのsealed union関係

Constructorが可視なscopeでは、親型をconstructor固有型のsealed unionとして扱う。

```text
option<int>
≃
union(
  none<int>,
  some<int>)
```

#### 6.4 表示上の単純化

内部推論ではconstructor固有型を保持するが、通常のhoverや公開APIでは親型へ簡略表示できる。

```text
内部:
some<int>

通常表示:
option<int>

詳細表示:
option<int>, refined as `some`
```

***

### 7. 型parameter推論

#### 7.1 Payloadからの推論

```lisp
(some 42)
```

から：

```text
a = int
```

を推論する。

#### 7.2 期待型からの推論

```lisp
(type value
  (option str))

(val value
  none)
```

期待型から：

```text
none<str>
```

としてinstance化する。

#### 7.3 一部未確定のparameter

```lisp
(ok 42)
```

では成功値型は`int`と分かるが、error型は未確定である。

Pureな`val`なら未確定parameterをgeneralizeできる。

```lisp
(val success
  (ok 42))
```

概念型：

```text
forall error-type.
result<int, error-type>
```

#### 7.4 Value restriction

Mutableまたはgeneralize不能な位置では、未確定parameterを量化しない。

不適合：

```lisp
(local
  (var result
    (ok 42))

  result)
```

`error-type`を決定できないため、型注釈を要求する。

適合：

```lisp
(local
  (type result
    (result int render-error))

  (var result
    (ok 42))

  result)
```

#### 7.5 値位置の明示型argument

Constructorへ値位置で明示的な型argumentを渡す構文は導入しない。

型を指定する場合は、期待型または型注釈を使う。

***

### 8. Variance

#### 8.1 自動推論

Varianceは利用者が記述せず、constructor payloadにおける型parameterの出現位置から自動推論する。

#### 8.2 共変

```lisp
(data option
  ((a type))

  none
  (some a))
```

`a`は正の位置だけに現れるため共変である。

```text
cat <: animal
なら
option<cat> <: option<animal>
```

#### 8.3 反変

```lisp
(data consumer
  ((a type))

  (consumer
    (fn a unit)))
```

`a`は関数引数位置に現れるため反変である。

#### 8.4 不変

```lisp
(data channel
  ((a type))

  (channel
    (fn unit a)
    (fn a unit)))
```

`a`は正と負の両方へ現れるため不変である。

#### 8.5 Phantom parameter

Constructor payloadへ現れないparameterはphantomとして扱う。

```lisp
(data identifier
  ((domain type))

  (identifier int))
```

Phantom parameterは許可するがwarning対象とする。

***

### 9. 再帰data型

#### 9.1 自己再帰

```lisp
(data tree
  ((a type))

  empty

  (node
    a
    (tree a)
    (tree a)))
```

定義中の型名をconstructor payloadから参照できる。

#### 9.2 Strict positivity

定義中のdata型はstrictly positiveな位置にのみ出現できる。

適合：

```lisp
(data tree
  ((a type))

  empty
  (node a (tree a) (tree a)))
```

不適合：

```lisp
(data bad
  (bad
    (fn bad unit)))
```

`bad`が関数引数位置に現れるため、strict positivity違反である。

#### 9.3 関数戻り値位置

定義中の型が関数戻り値に現れることは、positivity上は許可できる。

```lisp
(data stream-node
  ((a type))

  (stream-node
    a
    (fn unit (stream-node a))))
```

ただし、serializationや自動deriveの可否は別に判断する。

***

### 10. 相互再帰data型

#### 10.1 `rec` group

相互再帰data型は、明示的な`rec` groupで宣言する。

```lisp
(rec
  (data expression
    (literal int)
    (sequence (list statement)))

  (data statement
    (evaluate expression)
    (return expression)))
```

#### 10.2 Group内の可視性

同じ`rec` group内のdata型名は相互に可視である。

#### 10.3 宣言kindの混在禁止

同じ`rec` groupへ`data`と`val`を混在させない。

適合：

```lisp
(rec
  (data first ...)
  (data second ...))
```

適合：

```lisp
(rec
  (val first
    (fn (...) ...))

  (val second
    (fn (...) ...)))
```

不適合：

```lisp
(rec
  (data first ...)
  (val second
    (fn (...) ...)))
```

#### 10.4 Group全体のpositivity

Mutually recursive groupでは、group内の全data型についてまとめてstrict positivityを検査する。

***

### 11. Data値の実行意味

#### 11.1 不変性

Data値は常に不変である。

Constructor payloadを後から破壊的に書き換えることはできない。

#### 11.2 Strict評価

Constructor payloadは、constructor値を構築する前に左から右へ評価する。

```lisp
(node
  (compute-root)
  (compute-left)
  (compute-right))
```

評価順序：

```text
1. compute-root
2. compute-left
3. compute-right
4. node構築
```

#### 11.3 Effect

Constructor自体はpureである。

Constructor適用式のeffectは、payload式のeffectから生じる。

#### 11.4 途中失敗

Payload評価が途中で失敗した場合、残りのpayloadを評価せず、data値も構築しない。

***

### 12. 非循環性とsharing

#### 12.1 Cyclic data値

通常のSurface codeから、自己参照するcyclic data値を構築できない。

不適合：

```lisp
(rec
  (val infinite
    (node 1 infinite infinite)))
```

Value-level `rec`の右辺は`fn`に限る。

#### 12.2 Structural sharing

完成済みの不変部分値は共有できる。

```lisp
(local
  (val subtree
    (node 5 empty empty))

  (node 10 subtree subtree))
```

#### 12.3 Sharingの非観測性

Structural sharingの有無をpointer identityとして観測できない。

一般的な`physical-equal?`をdata値へ提供しない。

#### 12.4 無限構造

無限stream等はcyclic dataではなく、thunkや関数を明示的に含む有限値として表現する。

```lisp
(data stream
  ((a type))

  (stream
    a
    (fn unit (stream a))))
```

***

### 13. Patternの適用場所

#### 13.1 v1の制限

Patternは`match` case内で使用する。

`val` binderや関数parameterへ一般patternを導入しない。

適合：

```lisp
(match pair
  (tuple first second ->
    (+ first second)))
```

不採用：

```lisp
(val (tuple first second)
  pair)
```

```lisp
(fn ((tuple first second))
  (+ first second))
```

#### 13.2 Binder

Constructor payload位置の裸identifierはbinderとして扱う。

```lisp
(some item ->
  item)
```

`item`はcase body内だけで有効である。

#### 13.3 Wildcard

```lisp
_
```

は任意値へ一致し、bindingを作らない。

#### 13.4 Duplicate binder

同じpattern内で同じ名前を複数回束縛できない。

```lisp
(tuple value value ->
  ...)
```

は静的エラー。

`_`は複数回使用可能である。

***

### 14. `match`構文

#### 14.1 基本構文

```lisp
(match subject-expression
  (pattern ->
    result-expression)

  (pattern ->
    result-expression))
```

#### 14.2 Payload constructor

```lisp
(match value
  (some item ->
    item)

  (none ->
    default))
```

#### 14.3 複数payload

```lisp
(match shape-value
  (circle center radius ->
    (render-circle center radius))

  (rectangle origin rectangle-size ->
    (render-rectangle origin rectangle-size))

  (path data ->
    (render-path data)))
```

#### 14.4 `->`

`->`はmatch branch専用の予約tokenである。

通常operatorではなく、値として参照できない。

#### 14.5 Case body

矢印の右側には一つの式だけを書く。

複数処理は`seq`または`local`でまとめる。

```lisp
(some item ->
  (seq
    (log item)
    (render item)))
```

***

### 15. Patternの種類

#### 15.1 Nullary constructor pattern

```lisp
(none ->
  default)
```

Case先頭の裸identifierはnullary constructorとして解決する。

#### 15.2 Payload constructor pattern

```lisp
(some item ->
  item)
```

Constructor headは`ConstructorId`として名前解決する。

#### 15.3 Wildcard pattern

```lisp
(_ ->
  fallback)
```

#### 15.4 Catch-all binder

対象値全体を束縛する場合は`bind` patternを使う。

```lisp
(bind value ->
  (handle-other value))
```

`bind`はpattern特殊形式とする。

#### 15.5 Literal pattern

v1では次を許可する。

```text
整数literal
文字列literal
true
false
unit
```

例：

```lisp
(match format
  ("pdf" ->
    render-pdf)

  ("svg" ->
    render-svg))
```

`f64` literal patternは導入しない。

***

### 16. Nested pattern

#### 16.1 Nested payload constructor

```lisp
(match value
  (some (some item) ->
    item)

  (some inner ->
    fallback)

  (none ->
    default))
```

#### 16.2 Nested tuple

```lisp
(match value
  (some (tuple first second) ->
    (+ first second))

  (none ->
    0))
```

#### 16.3 Nested nullary constructor

Constructor payload位置の裸identifierはbinderとして読むため、nested nullary constructor patternはv1では直接提供しない。

次のようにnested `match`を使用する。

```lisp
(match outer
  (some inner ->
    (match inner
      (none ->
        first-result)

      (some value ->
        second-result)))

  (none ->
    outer-default))
```

***

### 17. Tuple pattern

#### 17.1 基本形

```lisp
(match pair
  (tuple first second ->
    (+ first second)))
```

#### 17.2 Arity

Pattern arityは入力tuple型と一致しなければならない。

#### 17.3 Binding型

入力型：

```lisp
(tuple int str)
```

Pattern：

```lisp
tuple count title
```

Binding：

```text
count : int
title : str
```

#### 17.4 Refutability

Tuple自体の型が一致していても、内部にrefutable patternがあればtuple pattern全体もrefutableである。

***

### 18. Record pattern

#### 18.1 基本形

```lisp
(match report
  (record
    (title title)
    (page-count count)
  ->
    (render-summary title count)))
```

#### 18.2 Partial decomposition

指定したrequired fieldだけを分解する。

追加fieldは許可する。

```lisp
(record
  (title title))
```

は、`title`以外のfieldを持つrecordにも一致する。

#### 18.3 Closed pattern

Field集合の完全一致を要求するclosed record patternはv1では導入しない。

#### 18.4 Required fieldのみ

Patternへ書けるのは、入力型で存在と型が静的に保証されたrequired fieldだけである。

Optional fieldは直接分解しない。

#### 18.5 Optional field

Optional fieldはfield access結果を`option`としてmatchする。

```lisp
(match (field report subtitle)
  (none ->
    no-subtitle)

  (some subtitle ->
    (render-subtitle subtitle)))
```

#### 18.6 Unknown row field

Open row内にあるか不明なfieldをrecord patternへ書けない。

Binder型を静的に決定できないためである。

***

### 19. Pattern typing

#### 19.1 三つの結果

Pattern checkerは各patternから次を計算する。

```text
Covered:
そのpatternが覆う型

Bindings:
case bodyへ導入される名前と型

Residual:
一致しなかった値の型
```

#### 19.2 Constructor pattern

入力：

```lisp
(option int)
```

Pattern：

```lisp
some item
```

結果：

```text
Covered:
(some int)

Bindings:
item : int

Residual:
(none int)
```

#### 19.3 Literal pattern

入力：

```lisp
(union "pdf" "svg")
```

Pattern：

```lisp
"pdf"
```

結果：

```text
Covered:
"pdf"

Residual:
"svg"
```

#### 19.4 Wildcard

```text
Covered:
入力型全体

Bindings:
なし

Residual:
never
```

#### 19.5 Disjoint pattern

入力型とのintersectionが`never`なら、そのcaseは静的に到達不能である。

```text
this pattern cannot match the input type
```

***

### 20. `match`の型・effect・評価

#### 20.1 対象式

`match`対象式は一度だけ評価する。

#### 20.2 Case順序

Caseをsource orderで検査し、最初に一致したcaseを選択する。

#### 20.3 結果型

期待型がある場合、各到達可能case bodyをその型で検査する。

期待型がない場合、到達可能case body型のunionを結果型とする。

```lisp
(match value
  (none ->
    "unknown")

  (some item ->
    item))
```

`item : int`なら結果型は：

```lisp
(union str int)
```

#### 20.4 `never`

`never`を返すcaseは、他branchの結果型を汚染しない。

```text
union(never, t) ≃ t
```

#### 20.5 Effect

`match`全体のeffectは次の和である。

```text
対象式のeffect
+
全到達可能case bodyのeffect
```

Pattern照合自体はpureとする。

***

### 21. 網羅性・公開・適合試験

#### 21.1 網羅性

Case処理前の残余型を対象型とする。

各caseについて：

```text
case-space =
intersection(remaining, pattern-type)
```

処理後：

```text
remaining =
diff(remaining, pattern-type)
```

最後に：

```text
remaining = never
```

なら網羅的である。

#### 21.2 非網羅match

非網羅matchは静的エラー。

```lisp
(match value
  (some item ->
    item))
```

診断例：

```text
non-exhaustive match

missing case:
  none
```

#### 21.3 到達不能case

既に前のcaseで覆われたcaseは静的エラー。

```lisp
(match value
  (_ ->
    default)

  (some item ->
    item))
```

#### 21.4 空match

Caseを持たない`match`は、対象型が`never`の場合に限って許可する。

#### 21.5 Sealed data

通常の`data`はsealedとする。

他packageからconstructorを追加できない。

公開data型へのconstructor追加はbreaking changeとして扱う。

#### 21.6 Transparent export

型と全constructorを公開する。

外部利用者は構築、pattern match、網羅性検査を行える。

#### 21.7 Abstract export

親型identityだけを公開し、constructorをすべて隠せる。

外部利用者は内部constructorでmatchできない。

#### 21.8 一部constructor公開

一部constructorだけの公開はv1では導入しない。

#### 21.9 適合試験 ADT-01

```lisp
(data option
  ((a type))

  none
  (some a))

(type value
  (option int))

(val value
  (some 42))
```

期待結果：

```text
success
valueの内部精密型:
some<int>

指定型:
option<int>
```

#### 21.10 適合試験 ADT-02

```lisp
(match value
  (some item ->
    item)

  (none ->
    0))
```

期待結果：

```text
success
result type:
int
```

#### 21.11 適合試験 ADT-03

```lisp
(match value
  (some item ->
    item))
```

期待結果：

```text
static error:
non-exhaustive match

missing:
none
```

#### 21.12 適合試験 ADT-04

```lisp
(match value
  (_ ->
    0)

  (some item ->
    item))
```

期待結果：

```text
static error:
unreachable match case
```

#### 21.13 適合試験 ADT-05

```lisp
(data tree
  ((a type))

  empty
  (node a (tree a) (tree a)))
```

期待結果：

```text
success
recursive occurrence is strictly positive
```

#### 21.14 不適合試験 ADT-06

```lisp
(data bad
  (bad
    (fn bad unit)))
```

期待結果：

```text
static error:
recursive occurrence of `bad` is not strictly positive
```

#### 21.15 適合試験 ADT-07

```lisp
(val success
  (ok 42))
```

期待結果：

```text
success
generalized type:
forall e. result<int, e>
```

#### 21.16 不適合試験 ADT-08

```lisp
(local
  (var success
    (ok 42))

  success)
```

期待結果：

```text
static error:
cannot infer ungeneralized error type

suggestion:
add a type annotation
```

#### 21.17 適合試験 ADT-09

```lisp
(match report
  (record
    (title title)
  ->
    title))
```

入力型：

```lisp
(record
  (title str)
  (page-count int))
```

期待結果：

```text
success
title : str
```

#### 21.18 不適合試験 ADT-10

```lisp
(match report
  (record
    (subtitle subtitle)
  ->
    subtitle))
```

`subtitle`がoptional fieldの場合の期待結果：

```text
static error:
optional fields cannot be directly decomposed by record patterns

suggestion:
match the result of field access
```

***

### 22. 移管先OPEN・下位項目・状態

#### 22.1 `OPEN-SYN-002`への追補

`->`をmatch branch separator用の予約tokenとして追補する。

```text
->
```

は通常operator identifierではない。

#### 22.2 `OPEN-MOD-001`

次を移管する。

```text
- data型のexport構文
- transparent export
- abstract export
- constructor import
- package qualification
- signature内のabstract type
- module境界でのcase型可視性
```

#### 22.3 `OPEN-SEM-001`

次を移管する。

```text
- proper tail call
- 一般再帰のstack safety
- stack overflowのfailure分類
- 関数呼出しstackの観測性
```

#### 22.4 `OPEN-DERIVE-001`

次を移管する。

```text
- 構造的等値性の自動導出
- hash
- ordering
- debug表示
- serialization
- clone／copy能力
```

#### 22.5 `OPEN-GADT-001`

次を将来項目として移管する。

```text
- constructorごとのresult type
- GADT
- existential constructor parameter
- pattern matchによる型等式導入
- typed AST
```

宣言名として`gadt`を新設するか、`data`を拡張するかはこの項目で決める。

#### 22.6 `OPEN-DYNAMIC-001`

次を移管する。

```text
- anyからのruntime type test
- dynamic cast
- dynamic値へのconstructor pattern
- cast failure
- RuntimeCheckable制約
```

`match`自体へ暗黙のdynamic castを含めない。

#### 22.7 `OPEN-LAZY-001`

次を移管する。

```text
- lazy
- force
- memoized thunk
- 無限stream
- 内部mutationを伴う遅延評価
```

#### 22.8 `OPEN-GRAPH-001`

必要になった場合に次を移管する。

```text
- cyclic graph
- node identity
- graph serialization
- mutable graph
- cycle-aware traversal
```

通常data値の循環参照としては実装しない。

#### 22.9 下位項目

```text
OPEN-DAT-001A
data宣言と型parameter
→ RESOLVED

OPEN-DAT-001B
constructorとconstructor固有型
→ RESOLVED

OPEN-DAT-001C
varianceとphantom parameter
→ RESOLVED

OPEN-DAT-001D
recursive／mutually recursive data
→ RESOLVED

OPEN-DAT-001E
pattern構文とpattern typing
→ RESOLVED

OPEN-DAT-001F
match構文・型・effect
→ RESOLVED

OPEN-DAT-001G
exhaustivenessとunreachable case
→ RESOLVED

OPEN-DAT-001H
sealed／abstract data境界
→ RESOLVED

OPEN-DAT-001I
runtime data semantics
→ RESOLVED
```

#### 22.10 最終状態

```text
OPEN-DAT-001:
RESOLVED
```

本解決により、RPXは次を型安全に表現できる。

```text
- 列挙型
- option／result
- recursive tree
- mutually recursive AST
- constructor固有型
- constructorによる型精緻化
- 網羅的pattern matching
- abstract data representation
- immutable structural sharing
```

## OPEN-MOD-001 モジュール・シグネチャ・Functor・分割コンパイル
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
コンパイル単位
外側モジュール
下位モジュール
インポート
インターフェースファイル
シグネチャ
抽象型
シグネチャ指定
シグネチャの精緻化
型共有
Functor
再エクスポート
正式identity
分割コンパイル

#### DD-001.2 中心的な決定
- 一つの実装source fileを一つのcompilation unitとする
- 各compilation unitは一つの外側モジュールを定義する
- ファイル内にはinline下位モジュールを定義できる
- 下位モジュールを別ファイルから読み込む機能はv1では導入しない
- 通常のモジュール依存グラフはDAGとする
- importは通常形式、as、only、個別asを組み合わせられる
- aliasや再エクスポートは正式identityを変更しない
- .rpiを明示インターフェースファイルとする
- .rpiがない内部モジュールでは実装からシグネチャを推論する
- パッケージ外へ公開するモジュールには.rpiを要求する
- 名前付きシグネチャを導入する
- シグネチャ適合は構造的に判定する
- 抽象型のidentityはモジュールごとにnominalとする
- 適用的Functorを採用する
- 生成的Functor、再帰モジュール、第一級モジュールはv1では導入しない
- interface metadataとInterfaceHashを生成する

### 0. 適用範囲
#### 0.1 本項目が定めるもの

本項目は次を規定する。

- source fileとcompilation unitの関係
- 外側モジュールと下位モジュール
- モジュールpath
- importと名前解決
- .rpiインターフェース
- 推論シグネチャ
- 名前付きシグネチャ
- 抽象型とconstructor公開data
- シグネチャ指定
- 下位モジュール仕様
- シグネチャの精緻化
- 型共有
- Functor
- module-alias
- re-export
- signature include
- 正式identity
- interface metadata
- 分割コンパイル

#### 0.2 本項目が直接定めないもの

次は別のOPEN項目へ移管する。

- package manifestの具体的形式
- version constraint
- lockfile
- package feature
- resource root
- macro phase
- ABIの完全な規則
- trusted adapter ABI
- 再帰モジュール
- 第一級モジュール
- 生成的Functor
- destructive signature substitution
- open variant

### 1. コンパイル単位
#### 1.1 基本単位

一つの実装source fileは、一つのコンパイル単位を定義する。

compilation unit
=
個別に解析・型検査・コンパイル可能な単位


各コンパイル単位は、一つの外側モジュールとして振る舞う。

#### 1.2 外側モジュールのwrapper

Source file内には、外側モジュールの明示wrapperを書かない。

次のようにはしない。

(module graphics/color
  ...)


ファイル自身が外側モジュールの本体である。

#### 1.3 一ファイル内の外側モジュール数

一つのsource fileに複数の独立した外側モジュールを定義できない。

次のような構成は不採用とする。

(module graphics/color
  ...)

(module graphics/image
  ...)

#### 1.4 一ファイル一モジュールとの違い

正確な設計は次である。

一ファイル:
一つのコンパイル単位
一つの外側モジュール

ファイル内部:
複数のinline下位モジュールを定義可能

### 2. モジュールpathとファイルpath
#### 2.1 Source root

パッケージは一つ以上のsource rootを宣言する。

具体的なmanifest構文はOPEN-PKG-001へ移管する。

#### 2.2 Module path

Source rootからの相対pathをモジュールpathとする。

src/color/conversion.rpx


は概念的に次のモジュールpathを持つ。

color/conversion

#### 2.3 Interface path

対応する実装ファイルとインターフェースファイルは、同じモジュールpathを持つ。

実装:
src/color/conversion.rpx

interface:
interface/color/conversion.rpi


両者は別モジュールではなく、同じModuleIdの実装部分と仕様部分である。

#### 2.4 Path変更

モジュールpathを変更した場合、v1では新しいModuleIdを生成する。

color/conversion
→ rendering/color-conversion


は、原則として公開API上のモジュール移動・改名である。

Identityを維持する必要がある場合は、旧pathにmodule aliasを残す。

### 3. 下位モジュール
#### 3.1 基本構文

ファイル内にはinline下位モジュールを定義できる。

(module parsing
  (data state
    initial
    reading
    failed)

  (type parse
    (fn str parse-result))

  (val parse
    (fn (source)
      ...)))

#### 3.2 下位モジュールのpath

外側モジュールがdocument/readerなら、この下位モジュールは概念的に次のpathを持つ。

document/reader/parsing

#### 3.3 下位モジュールの外部ファイル化

次のように、本文なしのmodule宣言によって別fileを探索する機能はv1では導入しない。

(module parsing)


下位モジュールはinlineのみとする。

大きくなった場合は、独立したコンパイル単位へ移動する。

#### 3.4 Module body

下位モジュールの本体は、通常の順序付き宣言グループである。

置けるものには次が含まれる。

- type
- type-alias
- val
- data
- rec
- nested module
- signature
- functor
- 将来のeffect宣言

### 4. 下位モジュールのscopeと純粋性
#### 4.1 Importの位置

importはコンパイル単位の先頭import領域だけに置ける。

下位モジュール内のimportは認めない。

不適合：

(module parsing
  (import text/utf8)

  ...)


診断例：

imports are only permitted in the compilation-unit import section

#### 4.2 親scopeの参照

下位モジュールは、定義位置より前にある親モジュールのbindingを参照できる。

適合：

(val default-limit 100)

(module parsing
  (val limit
    default-limit))

#### 4.3 後方参照

定義位置より後ろにある親bindingは参照できない。

不適合：

(module parsing
  (val limit
    default-limit))

(val default-limit 100)

#### 4.4 新しいscope

下位モジュールは新しい宣言scopeを作る。

内側bindingは、通常のscope規則に従って外側bindingをshadowできる。

#### 4.5 Top-level effect

外側モジュールと下位モジュールのtop-level値initializerはpureでなければならない。

不適合：

(module configuration
  (val current
    (read-resource config-file)))


Effectfulな処理は関数内へ置く。

#### 4.6 モジュールは通常値ではない

通常モジュールをvalへ格納したり、通常関数へ渡したりできない。

不適合：

(val selected
  parsing)

### 5. 修飾参照
#### 5.1 区切り記号

モジュールおよび構成要素の修飾には/を使う。

parsing/parse

graphics/color/srgb8

parsing/state

#### 5.2 除算との区別

除算operatorは単独の/である。

(/ left right)


修飾名は複数componentからなる名前pathとして解析する。

#### 5.3 内部表現

graphics/color/srgb8を一つの文字列識別子として扱わず、概念的に次のpath nodeとして保持する。

QualifiedName {
  graphics,
  color,
  srgb8
}

#### 5.4 .

.はモジュール修飾には使用しない。

小数点以外の一般的な.用途は引き続き未割当とする。

### 6. Import
#### 6.1 通常import
(import graphics/color)


これはgraphics/colorへの依存を宣言し、正式pathによる修飾参照を可能にする。

graphics/color/black


暗黙のcolor別名は生成しない。

#### 6.2 モジュール別名
(import graphics/color as color)


利用：

color/black


colorは現在のコンパイル単位内だけで有効なmodule aliasである。

#### 6.3 選択的インポート
(import graphics/color only black white srgb8)


利用：

black
white
(srgb8 255 0 0)

#### 6.4 個別項目の改名
(import graphics/color only black as blk)


導入される局所名はblkである。

blk


blackという無修飾名は導入されない。

#### 6.5 asとonlyの併用

モジュール別名と選択的インポートは同時に使用できる。

(import graphics/color as color only
  black as blk
  white)


利用可能な参照：

color/red
color/black
blk
white

#### 6.6 Import item

概念文法：

import-item :=
  name
  | name as local-name

#### 6.7 自動再公開

Importは構成要素を自動的に再公開しない。

import:
内部利用

re-export:
外部への再公開

### 7. Importと正式identity
#### 7.1 Aliasの効果

Module aliasは新しいModuleIdを作らない。

graphics/color
color


が同じresolved moduleを指すなら、正式identityは同じである。

#### 7.2 選択的import

選択的importは新しい値や型を作らない。

black
color/black
graphics/color/black


が同じ定義を指すなら、すべて同じBindingIdへ解決する。

#### 7.3 Source spelling

利用者が書いたpath、alias、局所名、source spanはCSTとprovenanceへ保持する。

意味解析:
正式identityを使用

編集・診断:
元の綴りとsource spanを使用

#### 7.4 同じidentityの重複import

同じ正式identityを複数経路からimportした場合は、意味上同じ定義として扱う。

異なるidentityが同じ局所名へ入る場合は衝突エラーとする。

### 8. .rpiインターフェース
#### 8.1 役割

.rpiは、対応する.rpxモジュールの公開シグネチャを記述する。

.rpx:
実装

.rpi:
外部契約

#### 8.2 Wrapper

.rpi全体を暗黙のシグネチャ本体とする。

次のようなwrapperは不要である。

(signature module-name
  ...)

#### 8.3 .rpi内のimport

.rpi内でもimportを許可する。

(import graphics/color as color)

(type render
  (fn color/color bytes))


Interface importは型・シグネチャ・effectなどの名前解決に使われ、実行時初期化を発生させない。

#### 8.4 Public module

パッケージ外へ公開されるモジュールには.rpiを要求する。

公開モジュールの指定方法はOPEN-PKG-001へ移管する。

#### 8.5 Internal module

パッケージ内部だけで使うモジュールでは.rpiを省略できる。

.rpiがない場合、実装から内部シグネチャを推論する。

#### 8.6 Script／実行entry

Scriptおよびexecutable entryには.rpiを要求しない。

### 9. 推論シグネチャと抽象化境界
#### 9.1 .rpiなしの内部モジュール

.rpiがない内部モジュールでは、全top-level構成要素からシグネチャを推論し、同じパッケージ内から利用可能にする。

#### 9.2 .rpiありのモジュール

.rpiが存在する場合、同じパッケージの別モジュールからも.rpi外の構成要素へアクセスできない。

.rpiはパッケージ外向けの飾りではなく、実際の抽象化境界である。

#### 9.3 Interface追加

既存の内部モジュールに後から.rpiを追加した場合、.rpiに記載されていない構成要素は他モジュールから見えなくなる。

これは意図的な抽象化強化であり、内部参照に対してbreakingになり得る。

### 10. .rpiの値仕様
#### 10.1 type

RPXのtypeは、新しい型を定義するものではなく、値bindingの型契約である。

(type parse-url
  (fn str
    (result url url-error)))


.rpiでは次を意味する。

parse-urlという値を公開し、その型は指定された型である。

#### 10.2 実装

.rpxには対応するvalを置く。

(val parse-url
  (fn (source)
    ...))

#### 10.3 型注釈の省略

.rpiに型がある公開値について、.rpx側の重複したtype宣言は省略できる。

Compilerは.rpiの型を実装の期待型として使用する。

#### 10.4 実装側にも型がある場合

.rpxにもtype宣言を書くことはできる。

この場合、.rpiと.rpxの型は同値でなければならない。

### 11. 型の公開方法
#### 11.1 抽象型仕様
(abstract-type url)


外部へ公開されるもの：

- urlという型identity
- parameter数
- parameter kind


外部へ公開されないもの：

- constructor
- record表現
- alias先
- runtime layout

#### 11.2 Parameter付き抽象型
(abstract-type collection
  ((a type)))


外部では次のように使える。

(collection int)

#### 11.3 Constructor公開data仕様

.rpiにdataを記載した場合、型と全constructorを公開する。

(data shape
  (circle point length)
  (rectangle point size)
  (path path-data))


外部利用者は値の構築、pattern match、網羅性検査を行える。

#### 11.4 実装との一致

Constructor公開data仕様と実装のdataは、次について一致しなければならない。

- 型名
- parameter数
- parameter kind
- constructor集合
- constructor名
- payload数
- payload型
- 再帰構造

#### 11.5 type-alias

Interface内の：

(type-alias title str)


は、titleとstrの透明な型等式を外部へ公開する。

title ≃ str


別のnominal型identityが必要なら、実装でdataを定義し、interfaceではabstract-typeとして公開する。

### 12. 名前付きシグネチャ
#### 12.1 基本構文
(signature ordered
  (abstract-type element)

  (type compare
    (fn element element int)))

#### 12.2 日本語上の意味
signature:
モジュール仕様
モジュール契約

named signature:
名前付きシグネチャ
再利用可能なモジュール契約

#### 12.3 .rpiとの違い
.rpi:
特定のコンパイル単位に対応する匿名シグネチャ

(signature name ...):
複数のモジュールやFunctorで再利用できる名前付きシグネチャ

#### 12.4 Signature identity

名前付きシグネチャには正式なSignatureIdを与える。

ただし、シグネチャ適合はSignatureIdの一致ではなく、要求構造に基づいて判定する。

### 13. シグネチャ指定
#### 13.1 基本構文
(module integer-order
  implements ordered

  ...)


日本語では次のように説明する。

signature ascription:
シグネチャ指定
モジュール契約の適用

#### 13.2 適合判定

シグネチャ適合は構造的に行う。

モジュールは、シグネチャが要求する型・値・下位モジュールを適合する形で提供しなければならない。

#### 13.3 追加構成要素

実装側は、シグネチャにない追加の構成要素を持てる。

ただし、シグネチャ指定後は、それらを外部から隠す。

#### 13.4 抽象型identity

同じシグネチャを満たす別モジュールの抽象型は、原則として別identityを持つ。

module-a/element
≠
module-b/element

#### 13.5 実装内部のalias

実装内部で：

(type-alias element int)


としていても、適用シグネチャが：

(abstract-type element)


なら、外部からelement = intだとは分からない。

### 14. 下位モジュール仕様
#### 14.1 名前付きシグネチャ
(signature parser-api
  (type parse
    (fn str parse-result)))


親シグネチャ：

(signature document-api
  (module parser
    implements parser-api))


これは、parser-apiを満たすparser下位モジュールを要求する。

#### 14.2 インラインシグネチャ

短い、その場限りの仕様には無名シグネチャを使える。

(module metadata
  (signature
    (type author
      (fn document str))))

#### 14.3 下位モジュール型の参照

公開された下位モジュールの型componentは修飾名で参照できる。

parser/state

#### 14.4 抽象型の独立性

同じparser-apiを満たしても、異なるモジュールの抽象型は同一ではない。

fast-parser/state
≠
strict-parser/state

#### 14.5 非公開型の漏出

公開シグネチャから、非公開モジュールまたは非公開型を参照できない。

public signature exposes a private type


として静的エラーにする。

### 15. シグネチャの精緻化と型共有
#### 15.1 基本構文
(refine signature-expression
  refinement ...)

#### 15.2 抽象型の具体化
(refine set-api
  (type-alias element int))


これは、set-apiのelement型がintと同一であるという型等式を追加する。

#### 15.3 別モジュールとの型共有
(refine set-api
  (type-alias element ordering/element))


意味：

結果signatureのelement
=
ordering/element

#### 15.4 日本語用語
signature refinement:
シグネチャの精緻化
シグネチャの具体化

type sharing:
型共有

type equality constraint:
型同一性制約

#### 15.5 許可される精緻化

v1では、抽象型componentへの型等式追加だけを許可する。

#### 15.6 禁止される変更

refineで次を行えない。

- 値の型を別の型へ変更する
- constructorを追加・削除する
- 具体型を矛盾する別型へ変更する
- 仕様項目を削除する

#### 15.7 Parameter付き型constructor

Parameter数とkindが一致する場合、型constructor同士の等式を認める。

(refine collection-api
  (type-alias collection list))

#### 15.8 下位モジュール内の型

修飾型pathも精緻化対象にできる。

(refine storage-api
  (type-alias key/element str))

### 16. Functor
#### 16.1 概念

Functorは、モジュールを受け取り、新しいモジュールを生成するモジュールレベルの関数である。

通常関数:
値 → 値

Functor:
モジュール → モジュール

#### 16.2 基本構文
(functor make-set
  (ordering implements ordered)

  returns
    (refine set-api
      (type-alias element ordering/element))

  declaration ...)

#### 16.3 Parameter

Functor parameterはモジュールparameterであり、名前と入力シグネチャを持つ。

Functor本体では次のように参照する。

ordering/element

ordering/compare

#### 16.4 結果シグネチャ

Functorは結果シグネチャを明示する。

Functor本体に追加helperがあっても、結果シグネチャにないものは適用結果の外部から見えない。

#### 16.5 Fixed arity

Functorはfixed arityとする。

引数不足および引数過剰は静的エラーである。

#### 16.6 通常値ではない

Functorをvalへ保存したり、通常関数へ渡したりできない。

高階Functorはv1では導入しない。

### 17. Functor適用
#### 17.1 基本構文
(module integer-set
  apply make-set integer-order)


これはmake-setをinteger-orderへ適用し、結果モジュールをinteger-setへ束縛する。

#### 17.2 複数引数
(module result
  apply make-map key-order value-api)


各引数は対応する入力シグネチャを満たさなければならない。

#### 17.3 名前付きモジュールのみ

Functor引数は、正式ModuleIdを持つ名前付きモジュールpathに限定する。

無名inline moduleや第一級モジュール値は渡せない。

#### 17.4 適用結果の再利用

Functor適用結果を別Functorへ渡す場合は、一度名前付きモジュールへ束縛する。

適合：

(module integer-set
  apply make-set integer-order)

(module serialized-set
  apply make-serializer integer-set)

### 18. 適用的Functor
#### 18.1 基本規則

v1のFunctorは適用的である。

同じFunctorId
+
同じ引数ModuleId列
→
同じ結果型identity

#### 18.2 同一入力
(module set-a
  apply make-set integer-order)

(module set-b
  apply make-set integer-order)


では、対応する生成型は同一である。

set-a/set
=
set-b/set

#### 18.3 異なる入力
(module integer-set
  apply make-set integer-order)

(module string-set
  apply make-set string-order)


では、結果型は異なる。

integer-set/set
≠
string-set/set

#### 18.4 Aliasの影響

Import aliasや再エクスポートpathはFunctor結果identityに影響しない。

解決済みFunctorIdとModuleIdを使用する。

#### 18.5 生成的Functor

適用ごとに新しい型identityを作る生成的Functorはv1では導入しない。

### 19. 再公開とシグネチャ合成
#### 19.1 module-alias

既存モジュールを下位モジュールとして公開する。

(module-alias color graphics/color)


新しいModuleIdを作らず、元モジュールのidentityを維持する。

#### 19.2 re-export

既存モジュールの構成要素を現在のモジュール直下で再公開する。

(re-export graphics/color only
  black
  white
  srgb8 as make-srgb8)

#### 19.3 Identity

再公開は新しい値・型・constructorを作らない。

追加されるのは公開pathだけである。

#### 19.4 Dataの原子性

Data型に関係する公開は、次を一つのgroupとして扱う。

- 親data型
- 全constructor
- constructor固有型


個別constructorだけを再エクスポートする機能はv1では導入しない。

#### 19.5 include

シグネチャ内で別シグネチャの構成要素を取り込める。

(signature extended-api
  (include base-api)

  ...)

#### 19.6 衝突

異なるidentityを持つ同名componentがincludeまたはre-exportで衝突した場合は静的エラー。

同じ正式identityなら統合可能である。

#### 19.7 実装moduleのinclude

実装モジュールへ別モジュールの全構成要素を無修飾で取り込む一般的なincludeはv1では導入しない。

通常は修飾importを使う。

### 20. 正式identity
#### 20.1 PackageInstanceId

依存解決後の特定のパッケージ実体を識別する。

概念的には次を含む。

- package名
- 解決済みversion
- source identity
- resolutionまたはcontent hash


具体形式はOPEN-PKG-001へ移管する。

#### 20.2 ModuleId
ModuleId =
PackageInstanceId
+
正規化済みmodule path

#### 20.3 DefinitionId

Named declarationの正式identityは概念的に次から作る。

DefinitionId =
ModuleId
+
namespace
+
正規化済み宣言名


Source byte offsetは使用しない。

#### 20.4 BindingId

値bindingの正式identityである。

Import aliasや再エクスポートで変化しない。

#### 20.5 TypeId

Nominal型定義の正式identityである。

type-aliasは新しいTypeIdを生成しない。

#### 20.6 ConstructorId
ConstructorId =
親TypeId
+
constructor名


Constructor順序だけには依存させない。

Runtime tag番号とは区別する。

#### 20.7 SignatureId
SignatureId =
ModuleId
+
signature名


適合判定そのものは構造的に行う。

#### 20.8 FunctorId
FunctorId =
ModuleId
+
functor名

#### 20.9 Functor適用結果
FunctorApplicationKey =
FunctorId
+
引数ModuleId列


結果内の型identityは、これに結果definition pathを加えて決める。

#### 20.10 Rename

公開宣言のrenameは、原則として新しいidentityを生成する。

互換性を維持する場合は、旧名をaliasまたは再エクスポートとして残す。

#### 20.11 Source span

Source spanは正式identityへ含めず、provenanceとして別に保持する。

### 21. 分割コンパイルと適合試験
#### 21.1 Interface metadata

.rpiまたは推論シグネチャからcompiled interface metadataを生成する。

最低限、次を含む。

- ModuleId
- 公開BindingIdと型
- 公開TypeId
- 抽象型のkindとparameter
- 公開dataのconstructor集合
- ConstructorIdとpayload型
- 公開下位モジュール
- 公開シグネチャ
- 公開Functor
- effect row
- 型共有制約
- 再エクスポートpath
- metadata format version

#### 21.2 非公開情報

外部metadataへ次を含めない。

- private binding
- private constructor
- abstract typeの内部表現
- private nested module
- private helper

#### 21.3 InterfaceHash

正規化済みの公開シグネチャから意味上のinterface hashを生成する。

InterfaceHash =
hash(normalized public signature)

#### 21.4 Hashに含めるもの
- 正式identity
- 公開型
- 公開effect
- constructor集合
- abstract／constructor公開の区別
- 下位モジュール仕様
- Functor signature
- 型共有制約

#### 21.5 Hashに含めないもの
- whitespace
- comment
- indentation
- private implementation
- import alias spelling
- 公開APIに影響しないsource span

#### 21.6 Documentation hash

型検査用のSemanticInterfaceHashと、文書生成用のDocumentationHashを分離できる。

#### 21.7 再コンパイル

依存先の実装が変わってもInterfaceHashが同じなら、依存モジュールの再型検査を省略できる。

InterfaceHashが変わった場合、依存モジュールを再型検査する。

#### 21.8 ABI hash

Binary representationやcalling conventionに関するAbiHashは、semantic interface hashと分離する。

詳細はOPEN-KER-001およびbackend仕様へ移管する。

#### 21.9 適合試験 MOD-01
(import graphics/color)


期待結果：

graphics/colorが依存moduleとして解決される
暗黙aliasは生成されない
graphics/color/blackで参照可能

#### 21.10 適合試験 MOD-02
(import graphics/color as color only
  black as blk
  white)


期待結果：

color/black:
有効

blk:
有効

white:
有効

black:
無修飾では未束縛

#### 21.11 適合試験 MOD-03
// url.rpi

(abstract-type url)

(type parse-url
  (fn str
    (result url url-error)))

// url.rpx

(data url
  (validated-url internal-url-data))

(val parse-url
  (fn (source)
    ...))


期待結果：

success

外部からurl型を参照可能
validated-urlは不可視
.rpiの型をval実装の期待型として使用

#### 21.12 不適合試験 MOD-04

.rpi：

(data option
  ((a type))

  none
  (some a))


.rpx：

(data option
  ((a type))

  none
  (some a)
  unknown)


期待結果：

static error:
implementation data representation does not match interface

extra constructor:
unknown

#### 21.13 適合試験 MOD-05
(signature ordered
  (abstract-type element)

  (type compare
    (fn element element int)))

(module integer-order
  implements ordered

  (type-alias element int)

  (val compare
    (fn (left right)
      ...))

  (val helper
    ...))


期待結果：

integer-orderはorderedを満たす
helperはmodule内部で利用可能
helperは外部から不可視
elementは外部では抽象型

#### 21.14 適合試験 MOD-06
(module set-a
  apply make-set integer-order)

(module set-b
  apply make-set integer-order)


期待結果：

set-a/setとset-b/setは同じTypeId

#### 21.15 不適合試験 MOD-07
module-a imports module-b
module-b imports module-a


期待結果：

static error:
cyclic module dependency

#### 21.16 適合試験 MOD-08
(module-alias palette graphics/color)


期待結果：

paletteはgraphics/colorと同じModuleIdを参照
内部TypeId・BindingId・ConstructorIdを維持

#### 21.17 不適合試験 MOD-09
(signature public-api
  (type expose
    (fn private-module/internal-type str)))


private-moduleまたはinternal-typeが非公開なら、期待結果：

static error:
public signature exposes a private type

### 22. 移管先OPEN・下位項目・状態
#### 22.1 `OPEN-PKG-001`

次を移管する。

- package manifest
- package名とversion
- dependency alias
- version constraint
- package source
- lockfile
- PackageInstanceIdの具体形式
- source root
- interface root
- public module一覧
- executable entry
- script package
- local path package
- 複数version共存
- package feature

#### 22.2 `OPEN-MAC-001`

次を移管する。

- macro phase
- macro import
- macro生成DefinitionId
- hygiene
- stable generated identity
- intentional capture

#### 22.3 `OPEN-KER-001`

次を移管する。

- ABI hash
- ForeignValue
- trusted adapter ABI
- runtime metadata
- validator boundary

#### 22.4 `OPEN-MOD-REC-001`

将来項目として次を移管する。

- recursive module
- signature-only recursion
- module cycle initialization
- cyclic TypeId group


v1のmodule dependency graphはDAGとする。

#### 22.5 `OPEN-MOD-FC-001`

将来項目として次を移管する。

- first-class module
- module pack／unpack
- existential package
- runtime module selection
- first-class moduleとFunctorの相互作用

#### 22.6 `OPEN-MOD-GEN-001`

将来項目として次を移管する。

- generative Functor
- 適用ごとのfresh TypeId
- capability生成
- anonymous module argument

#### 22.7 下位項目

```text
OPEN-MOD-001A
コンパイル単位・外側module・下位module
→ RESOLVED

OPEN-MOD-001B
import・alias・選択的import・修飾参照
→ RESOLVED

OPEN-MOD-001C
.rpi・推論signature・抽象型
→ RESOLVED

OPEN-MOD-001D
名前付きsignature・signature ascription
→ RESOLVED

OPEN-MOD-001E
下位module仕様・signature refinement・型共有
→ RESOLVED

OPEN-MOD-001F
Functor
→ RESOLVED

OPEN-MOD-001G
module-alias・re-export・signature include
→ RESOLVED

OPEN-MOD-001H
正式identity・separate compilation
→ RESOLVED

OPEN-MOD-001I
recursive／first-class moduleのv1範囲判定
→ RESOLVED
```
#### 22.8 最終状態

```text
OPEN-MOD-001:
RESOLVED
```

本解決により、RPXは次を提供する。

- ファイル単位の分割コンパイル
- inline下位モジュール
- 明示的で柔軟なimport
- 人間向けの.rpi契約
- 実装からのシグネチャ推論
- 抽象型による表現隠蔽
- constructor公開data仕様
- 再利用可能な名前付きシグネチャ
- 構造的なシグネチャ適合
- 型共有
- 適用的Functor
- identityを維持する再公開
- 安定した型・値・constructor identity
- interface hashによる差分コンパイル


## OPEN-PKG-001 パッケージmanifest・依存解決・ワークスペース・リソース
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
パッケージmanifest
パッケージ名とversion
source／interface root
公開モジュール
実行エントリ
スクリプト
依存宣言
依存先source
version制約
依存解決
lockfile
PackageInstanceId
複数version
ワークスペース
パッケージリソース

#### DD-001.2 中心的な決定
- パッケージmanifestにはpackage.rpxmを使用する
- Manifestは評価されない制限付きRPXデータ形式とする
- パッケージ名はASCII lowercase kebab-caseとする
- パッケージversionはmajor.minor.patchを基礎とする
- source rootは一つとし、既定値を"src"とする
- interface rootは省略可能な一つとし、既定値を"interface"とする
- 公開モジュールはmanifestで明示する
- 公開モジュールには.rpiを要求する
- 実行エントリはmanifestで明示し、.rpiを要求しない
- Manifestなしのscriptは単一ファイルに限定する
- 依存には局所的な依存別名を与える
- Version単独指定は完全一致として扱う
- Registry依存とlocal path依存をv1で扱う
- Lockfileにはrpx.lockを使用する
- 通常buildはlockfileを暗黙更新しない
- 通常の初回解決では最高の安定versionを選ぶ
- 必要な場合は同じパッケージの複数versionを共存させる
- ワークスペースにはworkspace.rpxmを使用する
- ワークスペース全体で一つのrpx.lockを共有する
- リソースはパッケージ内の論理identityとして扱う
- Resource rootの既定値を"resources"とする
- 配布対象resourceはmanifestで明示する
- 生成source・生成resource・任意build stepは別項目へ移管する

### 0. 適用範囲
#### 0.1 本項目が定めるもの

本項目は次を規定する。

- パッケージとモジュールの関係
- package.rpxmの形式
- パッケージ名
- パッケージversion
- source root
- interface root
- public module
- internal module
- executable entry point
- manifestなしscript
- dependency alias
- dependency source
- version constraint
- development dependency
- dependency resolution
- rpx.lock
- PackageInstanceId
- 複数version共存
- workspace.rpxm
- workspace member
- workspace共通lockfile
- package resource
- resource root
- resource identity
- 配布resource一覧

#### 0.2 本項目が直接定めないもの

次は別のOPEN項目へ移管する。

- Registry protocol
- パッケージ公開・署名・失効
- Git／任意URL dependency
- optional dependency
- feature system
- platform条件付きdependency
- mainの最終的な型
- test discoveryとtest fixture
- 任意build script
- generated source
- generated resource
- ABI hash
- trusted adapter
- resource decoderのtrusted boundary
- runtime resource lifetime

### 1. パッケージ
#### 1.1 定義

パッケージは、複数のモジュール、外部依存、リソースおよび実行エントリをまとめる配布単位である。

package
├─ package manifest
├─ source modules
├─ interface files
├─ public modules
├─ internal modules
├─ entry points
├─ resources
└─ dependencies

#### 1.2 モジュールとの違い
モジュール:
名前空間、型抽象化、分割コンパイルの単位

パッケージ:
配布、version付け、依存解決の単位


一つのパッケージは複数の外側モジュールを含められる。

#### 1.3 パッケージの種類

パッケージを排他的なlibraryまたはexecutableへ分類しない。

public moduleがある:
libraryとして利用可能

entry pointがある:
実行programを生成可能


一つのパッケージは両方を持てる。

### 2. パッケージmanifest
#### 2.1 ファイル名

パッケージmanifestのファイル名は次とする。

package.rpxm


拡張子の役割：

.rpx:
RPX実装source

.rpi:
RPX interface

.rpxm:
RPX manifest

#### 2.2 Package root

package.rpxmが存在するdirectoryをpackage rootとする。

package-root/
├─ package.rpxm
├─ src/
├─ interface/
└─ resources/

#### 2.3 制限付きRPX形式

ManifestはRPXに似たS式を使用するが、RPXプログラムとして評価しない。

Manifestでは次を禁止する。

- 関数適用
- val／data等の通常宣言
- 名前解決
- macro展開
- effect実行
- 外部ファイルの動的読込み
- 環境変数の暗黙参照
- 条件分岐による構成変更

#### 2.4 静的schema

Manifest parserは規定されたfieldとliteralだけを受理する。

不適合：

(package document
  (val version
    "1.0.0"))


適合：

(package document
  version "1.0.0")

#### 2.5 未知field

未知fieldは無視せず静的エラーにする。

unknown package field:
  soruce-root

did you mean:
  source-root

### 3. Manifestの基本構文
#### 3.1 最小形
(package document
  format-version 1
  version "1.0.0")

#### 3.2 明示形
(package document
  format-version 1
  version "1.0.0"

  source-root "src"
  interface-root "interface"
  resource-root "resources")

#### 3.3 Fieldの括弧

単一値fieldは括弧で囲まない。

version "1.0.0"
source-root "src"


複数項目または入れ子構造を持つfieldは括弧でまとめる。

(public-modules
  document
  document/query)

#### 3.4 Format version
format-version 1


はmanifest schemaのversionであり、パッケージrelease versionとは異なる。

format-version:
manifest形式のversion

version:
パッケージreleaseのversion


未対応format versionは明確に拒否する。

### 4. パッケージ名
#### 4.1 基本規則

パッケージ名はASCII lowercase kebab-caseとする。

適合：

document
graphics-kit
japanese-typesetting
render2
layout-engine


不適合：

Document
graphics_kit
日本語組版
-graphics
graphics-
graphics--kit

#### 4.2 用途

正式パッケージ名は次に使用される。

- Registry identity
- Dependency宣言
- Lockfile
- Cache
- PackageInstanceId
- CLI表示

#### 4.3 表示名

Unicodeを含む人間向け表示名は、将来の任意fieldとして追加できる。

display-name "日本語組版"


表示名は正式identityに影響しない。

### 5. パッケージversion
#### 5.1 基本形式

Versionはmajor.minor.patchを基礎とする。

version "1.2.3"


文字列literalとして記述し、専用version parserで検証する。

#### 5.2 Version要素
1.2.3
│ │ └─ patch
│ └─── minor
└───── major

#### 5.3 互換性の一般原則
major:
互換性を壊す変更

minor:
後方互換な機能追加

patch:
後方互換な修正

#### 5.4 Breaking changeの例

少なくとも次は公開APIのbreaking changeである。

- 公開bindingの削除またはrename
- 公開値の非互換な型変更
- 公開effect rowの拡大
- 公開dataへのconstructor追加・削除
- constructor payloadの変更
- 公開abstract TypeIdの変更
- Functor signatureの非互換変更
- public module pathの削除または変更

#### 5.5 Pre-release

次はpre-release versionである。

2.0.0-alpha.1
2.0.0-beta.2
2.0.0-rc.1


Pre-releaseは依存条件で明示的に要求された場合だけ自動選択対象とする。

### 6. Source rootとinterface root
#### 6.1 Source root

実装sourceを探索する基準directoryを一つ指定できる。

source-root "src"


省略時の既定値：

src

#### 6.2 Module path

Source rootからの相対pathがmodule pathになる。

src/document/parser.rpx
→ document/parser

#### 6.3 Interface root

.rpiを探索する基準directoryを省略可能な一つとして指定できる。

interface-root "interface"


省略時の既定値：

interface

#### 6.4 Interface対応
src/document/parser.rpx

interface/document/parser.rpi


は同じmodule path：

document/parser


に対応する。

#### 6.5 Root数

v1では次に限定する。

source root:
一つ

interface root:
省略可能な一つ


複数rootと探索順序は導入しない。

#### 6.6 Root pathの制限

Root pathはpackage root内の相対pathでなければならない。

禁止：

- 絶対path
- `..` component
- symbolic linkによるpackage外脱出

### 7. 公開モジュール
#### 7.1 Manifest構文
(public-modules
  document
  document/query)


public-modulesに書かれた外側モジュールだけが、別パッケージからimportできる。

#### 7.2 .rpi必須

公開モジュールには、対応する実装とinterfaceの両方を要求する。

src/document.rpx
interface/document.rpi


Interfaceがない場合：

public module has no interface file:
  document

#### 7.3 内部モジュール

public-modulesに書かれていないコンパイル単位は内部モジュールである。

同じパッケージ内:
利用可能

別パッケージ:
import不可

#### 7.4 Internal moduleの.rpi

内部モジュールでは.rpiは任意である。

.rpiなし:
実装から内部シグネチャを推論

.rpiあり:
.rpiを同一パッケージ内でも抽象化境界として適用

#### 7.5 Inline下位モジュール

Manifestのpublic-modulesが対象とするのは、ファイルに対応する外側モジュールである。

Inline下位モジュールの公開範囲は、その外側モジュールの.rpiが決める。

#### 7.6 自動公開

.rpiが存在するだけでは自動的にpublic moduleにしない。

.rpi:
モジュールの見せ方

public-modules:
パッケージ外へ見せるか


を分離する。

### 8. 実行エントリ
#### 8.1 Manifest構文
(entry-points
  document-cli
  preview-server)


一つのパッケージは複数の実行エントリを持てる。

#### 8.2 Entry module

各entry pointは、source root内の外側モジュールpathを表す。

document-cli
→ src/document-cli.rpx

#### 8.3 .rpi

Entry moduleには.rpiを要求しない。

同じmoduleがpublic moduleも兼ねる場合は、public moduleとして.rpiが必要である。

#### 8.4 main

Entry moduleはmainという値を提供しなければならない。

mainの正確な型、許容effect、引数、exit statusおよびasync対応は実行意味論・エラー・並行性の各項目へ移管する。

#### 8.5 一module一entry

v1では、一つのentry moduleに一つのmainを対応させる。

複数programが必要な場合は、薄いentry moduleを複数作る。

#### 8.6 実行契約

Entry contractはlibrary向け.rpiとは別に検査する。

同じmoduleがpublicでも、mainを.rpiへ書くことは要求しない。

### 9. Manifestなしscript
#### 9.1 基本形

単一の.rpxファイルは、manifestなしでscriptとして実行できる。

report.rpx


Compilerは一時的な暗黙script packageとして扱う。

#### 9.2 必要な値

Scriptはmainを提供する。

(val main
  (fn ()
    ...))

#### 9.3 単一ファイル制限

Manifestなしscriptは単一ファイルに限定する。

別の.rpxファイルを暗黙に同一packageとして探索しない。

複数ファイルが必要になった場合はpackage.rpxmを作成する。

#### 9.4 外部dependency

Standalone scriptは、標準library以外の外部dependencyを持たない。

外部dependencyを使用する場合は、packageまたはworkspaceのmanifestとlockfileを使用する。

#### 9.5 Public API

Manifestなしscriptはpublic moduleを持たず、.rpiを要求しない。

### 10. 依存宣言
#### 10.1 基本構文
(dependencies
  (graphics
    package graphics-kit
    version ">=1.2.0 <2.0.0")

  (markup
    package rpx-markup
    version "2.1.0"))

#### 10.2 Dependency alias

各依存項目の先頭名は、現在のパッケージ内で使う依存別名である。

graphics:
依存別名

graphics-kit:
正式パッケージ名


Source：

(import graphics/color as color)

#### 10.3 正式identity

依存別名はPackageInstanceIdに含めない。

同じresolved packageを別aliasで参照しても、正式identityは同じである。

#### 10.4 正式パッケージ名

package fieldには正式パッケージ名を書く。

package graphics-kit


パッケージ名は専用name grammarを持つため、文字列literalにしない。

#### 10.5 Aliasの重複

同じdependency aliasを複数回定義できない。

duplicate dependency alias:
  graphics

### 11. Version constraint
#### 11.1 完全一致

単一versionは完全一致として扱う。

version "1.2.3"


許可：

1.2.3


拒否：

1.2.4
1.3.0
2.0.0

#### 11.2 範囲指定

明示的な比較演算子で範囲を記述する。

version ">=1.2.0 <2.0.0"

#### 11.3 初期演算子集合

v1では次を許可する。

=
>
>=
<
<=

#### 11.4 条件の結合

空白で並べた複数条件はANDとする。

>=1.2.0 <2.0.0


OR条件はv1では導入しない。

#### 11.5 Pre-release

Pre-releaseは、constraintがpre-release versionを明示的に含む場合だけ候補にする。

### 12. Dependency source
#### 12.1 既定Registry

pathまたは明示sourceがないdependencyは、既定registryから解決する。

(graphics
  package graphics-kit
  version ">=1.2.0 <2.0.0")


実際に使用したregistry identityはlockfileへ記録する。

#### 12.2 Local path
(theme
  package document-theme
  version "0.4.0"
  path "../document-theme")


Pathは現在のpackage.rpxmがあるdirectoryからの相対pathとして解釈する。

#### 12.3 Local packageの検証

Path先のmanifestにある正式パッケージ名が、dependencyのpackage fieldと一致しなければならない。

Version条件がある場合、path先のversionも条件を満たさなければならない。

#### 12.4 Supported source

v1で正式に扱う依存sourceは次とする。

- 既定registry
- local path
- workspace member

#### 12.5 Git／URL

Git repository、branch、commitおよび任意URL dependencyはv1では導入しない。

#### 12.6 Sourceの排他性

一つの依存項目で複数のsourceを同時指定できない。

### 13. Development dependency
#### 13.1 構文
(development-dependencies
  (testing
    package rpx-testing
    version "1.0.0"))

#### 13.2 用途

Development dependencyは次に使用する。

- test
- benchmark
- 開発tool


通常のlibrary利用者の依存graphには含めない。

#### 13.3 Public APIへの漏出

Development dependencyの型、effect、signatureまたはFunctorを公開.rpiへ露出できない。

public interface depends on a development-only package


として静的エラーにする。

#### 13.4 Optional dependency

Optional dependencyおよびfeature連動依存はv1では導入しない。

将来OPEN-PKG-FEAT-001へ移管する。

### 14. Public dependency
#### 14.1 定義

依存先の型その他のidentityが、自パッケージの公開.rpiに現れる場合、その依存は公開依存である。

(type create
  (fn markup/fragment document))

#### 14.2 自動導出

Public dependencyをmanifestへ手動指定させない。

Compilerが.rpiから自動的に導出し、interface metadataへ記録する。

#### 14.3 用途

Public dependency情報は次に利用できる。

- 互換性検査
- Documentation
- Registry metadata
- Dependency更新の影響分析

### 15. 依存解決
#### 15.1 Manifestの役割

Manifestは受理可能なversion・source条件を記述する。

具体的な依存graphはlockfileで固定する。

#### 15.2 初回解決

Lockfileがない場合、条件を満たす候補から依存graphを解決する。

通常は、条件を満たす最高の安定versionを選ぶ。

#### 15.3 Pre-release

Pre-releaseは明示的に要求された場合だけ選択する。

#### 15.4 統合

同じ正式パッケージ名、同じsourceおよび互換な条件について、一つのversionで全要求を満たせる場合は、一つのpackage instanceへ統合する。

#### 15.5 分割

一つのversionで全条件を満たせない場合、通常のpure packageでは複数のPackageInstanceIdへ分割できる。

#### 15.6 決定性

同じmanifest、registry状態およびlockfile状態からは、同じ解決結果を得なければならない。

候補の選択およびtie-break規則を決定的にする。

### 16. 同一パッケージの複数version
#### 16.1 基本方針

依存graph内で、同じ正式パッケージ名の複数version共存を許可する。

graphics-kit 1.5
graphics-kit 2.1


は別のPackageInstanceIdを持つ。

#### 16.2 型identity

同じmodule path・型名を持っていても、package instanceが異なれば別の型である。

graphics-kit@1/color
≠
graphics-kit@2/color

#### 16.3 直接依存での明示
(dependencies
  (graphics-v1
    package graphics-kit
    version "1.5.0")

  (graphics-v2
    package graphics-kit
    version "2.1.0"))


Source：

(import graphics-v1/color as old-color)
(import graphics-v2/color as new-color)

#### 16.4 単一instance制約

Trusted adapter、process-global plugin等で複数instanceを禁止する必要がある場合、その制約はOPEN-KER-001へ移管する。

### 17. Lockfile
#### 17.1 ファイル名

Lockfileのファイル名は次とする。

rpx.lock

#### 17.2 役割
package.rpxm:
依存の許容条件

rpx.lock:
完全な解決済み依存graph

#### 17.3 通常build

Lockfileが存在し、manifestと整合する場合、記録済みgraphをそのまま使用する。

通常buildはlockfileを変更しない。

#### 17.4 初回build

Lockfileがない場合：

依存解決
→ lockfile生成
→ build


を行う。

#### 17.5 不整合

Lockfileのversionがmanifest条件を満たさない場合、通常buildでは自動更新せずエラーにする。

lockfile is not consistent with the package manifest

#### 17.6 更新

Version変更は明示的なdependency update操作によってのみ行う。

CLIの具体的な構文は別項目へ移管する。

#### 17.7 部分更新

一つの依存だけを更新する操作を許容する。

Resolverは、固定可能なlock entryを維持し、必要な範囲だけ再解決する。

### 18. Lockfileの内容
#### 18.1 Package node

各解決済みpackage nodeについて、少なくとも次を記録する。

- 正式package名
- exact version
- source identity
- artifact checksum
- package content hash
- dependency edges
- dependency alias
- PackageInstanceId生成に必要な情報

#### 18.2 Registry package

Registry packageではartifact checksumを必須とする。

Checksum不一致は重大な完全性エラーである。

#### 18.3 Local path package

Local path packageは開発中の可変sourceとして扱う。

Source編集によってlockfile破損とはしないが、再コンパイル対象にはする。

#### 18.4 Version control

Workspaceのrpx.lockはversion controlへ含めることを推奨する。

Library作者のlockfileは、そのlibraryを利用する別workspaceの解決graphへ強制適用しない。

#### 18.5 Offline build

Offline buildでは：

- networkへ接続しない
- registry indexを更新しない
- lockfileを変更しない
- cache内artifactだけを使う


必要artifactがなければ明示的に失敗する。

### 19. Package identityとcontent hash
#### 19.1 論理identityと内容identity

次を分離する。

LogicalPackageInstanceId:
依存graph上のpackage nodeの同一性

PackageContentHash:
現在のpackage内容の完全性とcache identity

#### 19.2 Registry package

概念的には次を使用する。

PackageInstanceId:
正式package名
+ exact version
+ source identity

PackageContentHash:
registry artifactの内容hash

#### 19.3 Workspace／local package

Workspace内のsource編集だけで、全TypeIdを毎回変更しない。

PackageInstanceId:
workspace dependency graph上の論理node

PackageContentHash:
編集に応じて変更

#### 19.4 Content hash対象

少なくとも次を含む。

- package.rpxm
- .rpx
- .rpi
- 配布resource
- 必要なpackage metadata


次は含めない。

- file mtime
- 所有者情報
- 絶対path
- editor temporary file
- build output
- OS固有metadata

### 20. ワークスペース
#### 20.1 定義

ワークスペースは、複数のローカルパッケージを一つの開発単位として扱う仕組みである。

パッケージ:
配布・version付け・依存の単位

ワークスペース:
複数パッケージの共同開発単位

#### 20.2 Manifest

ワークスペースmanifestのファイル名は次とする。

workspace.rpxm

#### 20.3 基本構文
(workspace
  format-version 1

  (members
    "document-core"
    "document-render"
    "document-cli"))

#### 20.4 Member

各memberは独自のpackage.rpxmを持たなければならない。

#### 20.5 Member path

Member pathはworkspace rootからの明示的な相対pathとする。

Directory globはv1では導入しない。

#### 20.6 Memberの外部配置

原則としてworkspace root内のpackageだけをmemberにできる。

Workspace外packageはpath dependencyとして扱う。

#### 20.7 Package名重複

同一workspace内で正式パッケージ名を重複させられない。

#### 20.8 Nested workspace

ワークスペースの入れ子を禁止する。

一つのpackageは高々一つのworkspaceに所属する。

### 21. Workspace依存・リソース・適合試験
#### 21.1 共通lockfile

Workspace全体でrootのrpx.lockを共有する。

Member directory内に個別lockfileを置かない。

#### 21.2 Member単独build

一つのmemberだけをbuildする場合も、workspace rootのlockfileを使用する。

#### 21.3 Workspace member優先

通常dependencyの正式package名とversion条件に適合するworkspace memberが存在する場合、source指定がなければworkspace memberを優先する。

(document
  package document-core
  version ">=1.2.0 <2.0.0")

#### 21.4 明示source
source workspace


はworkspace memberを必須にする。

source registry


はworkspace memberを無視し、registryから解決する。

#### 21.5 Member version

Workspace memberもpackage versionを持ち、依存側のversion条件を満たさなければならない。

#### 21.6 Member依存graph

Workspace member間の依存graphもDAGでなければならない。

Cycleは静的構成エラーとする。

#### 21.7 Workspace外path dependency

開発時には許可できるが、package publication時には禁止する。

Release／reproducible modeで警告またはエラーにできる。

#### 21.8 Workspace identity

v1では明示的な永続Workspace UUIDを導入しない。

絶対filesystem pathをPackageInstanceIdへ直接含めない。

#### 21.9 Resource root

パッケージは一つのresource rootを持てる。

resource-root "resources"


省略時の既定値：

resources

#### 21.10 Resource一覧

配布対象resourceはmanifestで明示する。

(resources
  "styles/default.css"
  "images"
  "fonts/body.woff2"
  "locale")


File指定はそのfileだけを含める。

Directory指定は、その配下の通常fileを再帰的に含める。

Globはv1では導入しない。

#### 21.11 Resource path

Resource pathはresource rootからの正規化済み相対pathである。

禁止：

- 絶対path
- `..`
- 空component
- package root外への脱出


規範的separatorは/とする。

#### 21.12 Symbolic link

Resourceとしてsymbolic linkを禁止する。

#### 21.13 Resource identity
PackageResourceId
=
PackageInstanceId
+
normalized resource path


異なるpackageの同じpathは別resourceである。

#### 21.14 Resource参照

Sourceからは次の特殊形式で参照する。

(resource "styles/default.css")


これは文字列やOS pathではなく、package-resource型の静的handleを生成する。

#### 21.15 Pure／effectfulの区別
(resource "path"):
pureなresource identityの構築

resource内容の読込み:
resource effectを要求

#### 21.16 Resourceの外部公開

別パッケージのresource pathを直接参照できない。

必要な場合、所有パッケージが.rpiを通じてpackage-resource値を公開する。

#### 21.17 Resource hash

ResourceはPackageContentHashへ含める。

各resourceのcontent hashもcompiled metadataへ保存できる。

#### 21.18 Generated resource

生成resource、生成sourceおよび任意build stepはOPEN-BLD-001へ移管する。

#### 21.19 Test fixture

Test fixtureはOPEN-TST-001へ移管する。

#### 21.20 適合試験 PKG-01
(package document
  format-version 1
  version "1.0.0")


期待結果：

success

source-root:
src

interface-root:
interface

resource-root:
resources

#### 21.21 不適合試験 PKG-02
(package Document
  format-version 1
  version "1.0.0")


期待結果：

static manifest error:
package name must be ASCII lowercase kebab-case

#### 21.22 適合試験 PKG-03
(public-modules
  document
  document/query)


対応する.rpxと.rpiが存在する場合：

success

#### 21.23 不適合試験 PKG-04
(public-modules
  document/query)


interface/document/query.rpiが存在しない場合：

static manifest error:
public module has no interface file

#### 21.24 適合試験 PKG-05
(entry-points
  document-cli
  preview-server)


両moduleが存在し、mainを提供する場合：

success

#### 21.25 適合試験 PKG-06
(dependencies
  (graphics
    package graphics-kit
    version ">=1.2.0 <2.0.0"))


期待結果：

graphicsをdependency aliasとして登録
条件を満たす最高の安定versionを初回解決
resolved packageをlockfileへ記録

#### 21.26 適合試験 PKG-07
(dependencies
  (theme
    package document-theme
    version "0.4.0"
    path "../document-theme"))


Path先の名前とversionが一致する場合：

success

#### 21.27 不適合試験 PKG-08

Lockfile：

graphics-kit 1.7.3


Manifest条件：

>=2.0.0 <3.0.0


期待結果：

build error:
lockfile is not consistent with the package manifest

lockfileは自動更新しない

#### 21.28 適合試験 PKG-09
(workspace
  format-version 1

  (members
    "document-core"
    "document-render"))


両directoryに有効なpackage.rpxmがあり、package名が重複しない場合：

success
workspace rootのrpx.lockを共有

#### 21.29 不適合試験 PKG-10
(resources
  "../secret.txt")


期待結果：

static manifest error:
resource path escapes the package resource root

#### 21.30 適合試験 PKG-11
(val logo
  (resource "images/logo.png"))


Resourceがmanifest一覧に含まれ、実在する場合：

success
logo : package-resource

#### 21.31 不適合試験 PKG-12
(val logo
  (resource "images/missing.png"))


期待結果：

build error:
package resource does not exist

### 22. 移管先OPEN・下位項目・状態
#### 22.1 `OPEN-BLD-001`

次を移管する。

- 宣言的build step
- generated source
- generated resource
- code generator
- build sandbox
- build input／output declaration
- 非決定的buildの拒否
- build cache

#### 22.2 `OPEN-PKG-FEAT-001`

次を将来項目として移管する。

- optional dependency
- package feature
- feature unification
- platform条件
- feature集合とPackageInstanceId

#### 22.3 `OPEN-REG-001`

次を移管する。

- Registry protocol
- package upload
- package署名
- owner／namespace
- package yanking／失効
- checksum配布
- immutable release

#### 22.4 `OPEN-KER-001`

次を移管する。

- trusted adapter package
- process-global package制約
- 複数instance禁止条件
- ABI hash
- resource validator
- ForeignValue

#### 22.5 `OPEN-TST-001`

次を移管する。

- test entry
- test-only module
- development dependencyの可視性
- test fixture
- test resource
- 許容effect row

#### 22.6 `OPEN-ERR-001`

次を移管する。

- mainのfailure型
- exit status
- terminal failure
- cleanup

#### 22.7 `OPEN-CON-001`

次を移管する。

- async main
- cancellation
- concurrent entry point

#### 22.8 下位項目

```text
OPEN-PKG-001A
manifest・package名・version・root
→ RESOLVED

OPEN-PKG-001B
公開module・entry point・script
→ RESOLVED

OPEN-PKG-001C
dependency宣言・source・version条件
→ RESOLVED

OPEN-PKG-001D
dependency resolution・lockfile・PackageInstanceId
→ RESOLVED

OPEN-PKG-001E
workspace
→ RESOLVED

OPEN-PKG-001F
package resource
→ RESOLVED
```

#### 22.9 最終状態

```text
OPEN-PKG-001:
RESOLVED
```



本解決により、RPXは次を提供する。

- 静的で評価されないpackage manifest
- 明示的なpublic module境界
- 軽量なinternal module開発
- 複数entry point
- 単一ファイルscript
- 明示的なdependency alias
- Registry／local／workspace dependency
- 厳密なversion constraint
- 再現可能なlockfile build
- 複数package versionの共存
- 共有lockfileを持つworkspace
- 安定したPackageInstanceId
- パッケージ所有のresource identity
- Resourceの明示的な配布範囲
- 生成処理を通常package解決から分離する安全な基盤

## OPEN-MAC-001 最小式マクロ・展開・衛生性
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
利用者定義マクロ
式マクロ
マクロ展開段階
マクロパターン
末尾反復
名前衝突防止
展開順序
展開上限
マクロのscope
展開結果の由来情報

#### DD-001.2 設計原則

Reciplexaでは、通常の抽象化に第一級関数を優先する。

値の受渡しで表現できる処理:
通常関数を使用する

評価順序・binding・構文構造の変更が必要な処理:
必要な場合に限りマクロを使用する


マクロは言語の中心的な抽象化機能ではなく、関数では表現できない限定的な構文糖衣に使用する。

#### DD-001.3 中心的な決定
- v1では式マクロだけを提供する
- マクロは同一コンパイル単位内だけで使用できる
- 一つのマクロは一つの入力patternだけを持つ
- マクロpattern変数には`$`を付ける
- 固定arityを基本とする
- 必要な場合のみ末尾の一個以上反復`...+`を許可する
- マクロは定義より後ろでのみ使用できる
- 自己再帰・相互再帰マクロを禁止する
- マクロ展開は型検査前に行う
- マクロは型情報を参照しない
- マクロは既定で衛生的である
- 意図的な名前capture APIは提供しない
- 展開には有限の予算を適用する
- マクロの別ファイル・別パッケージへの公開はv1では行わない
- 宣言マクロ、型マクロ、patternマクロ、procedural macroはv1では行わない

### 0. 適用範囲
#### 0.1 本項目が定めるもの

本項目は次を規定する。

- マクロと通常関数の役割分担
- マクロ宣言のSurface構文
- マクロ呼出し
- マクロpattern変数
- 固定arity
- 末尾反復
- テンプレートへの構文挿入
- マクロのscope
- マクロ展開の段階
- 展開順序
- 名前衝突防止
- 展開予算
- エラー診断
- 展開構文のprovenance

#### 0.2 本項目が定めないもの

次は将来項目へ移管する。

- 宣言マクロ
- 型位置マクロ
- pattern位置マクロ
- module構成要素を生成するマクロ
- 複数rule
- 0個以上の反復
- procedural macro
- typed macro
- マクロのパッケージ間公開
- macro dependency
- import-macro
- .rpiのmacro仕様
- 意図的capture
- compiler plugin

### 1. マクロと通常関数
#### 1.1 通常関数の優先

通常関数で表現できる処理にマクロを使用しない。

次のような処理は通常関数として定義する。

(clamp value minimum maximum)

(render-with-style document style)

(map transform values)


通常関数には次の利点がある。

- 引数と戻り値を直接型検査できる
- 第一級値として受け渡せる
- 高階関数へ渡せる
- 通常の関数合成を利用できる
- 実行時の呼出しとして追跡できる
- 展開によるコード膨張がない

#### 1.2 マクロが適する場合

マクロは、通常関数では同じ意味を保持できない場合に使用する。

典型例は条件付き評価である。

(when ready?
  (render document))


これをstrictな通常関数として実装すると、ready?の結果にかかわらず(render document)が先に評価される。

マクロなら次へ展開できる。

(if ready?
    (render document)
    unit)


したがって、マクロの主な用途は次とする。

- 条件付き評価
- 短絡評価
- 局所bindingを伴う式糖衣
- 本文の評価を遅延・制御する式糖衣

### 2. マクロの基本構文
#### 2.1 宣言形式
(macro macro-name
  (macro-pattern ...)
  ->
  expression-template)

#### 2.2 固定arityの例
(macro unless
  ($condition $expression)
  ->
  (if $condition
      unit
      $expression))


利用：

(unless failed?
  (continue))


展開：

(if failed?
    unit
    (continue))

#### 2.3 可変長本文の例
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))


利用：

(when ready?
  (prepare)
  (render document))


展開：

(if ready?
    (seq
      (prepare)
      (render document))
    unit)

#### 2.4 ->

->はマクロ入力patternと展開templateの区切りとして使用する。

左側:
呼出しに一致させる構文pattern

右側:
一致時に生成する式template


->は通常の値operatorではない。

### 3. 式マクロ
#### 3.1 使用可能な位置

利用者定義マクロは式位置でのみ使用できる。

適合：

(val result
  (when ready?
    (render document)))


適合：

(fn (value)
  (unless invalid?
    (process value)))

#### 3.2 展開結果

式マクロは、ちょうど一つのRPX式へ展開しなければならない。

複数処理を生成する場合はseqまたはlocalで一つの式へまとめる。

(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

#### 3.3 宣言位置

次のように、マクロによってtype、val、dataなどを生成する機能はv1では提供しない。

(define-enum-with-name
  alignment
  left
  center
  right)

#### 3.4 型位置

次のような型生成マクロはv1では認めない。

(type value
  (generated-type ...))

#### 3.5 Pattern位置

次のようなpattern生成マクロはv1では認めない。

(match value
  ((generated-pattern ...) ->
    result))

#### 3.6 Interfaceおよびmanifest

利用者定義マクロは次では使用できない。

- .rpi
- package.rpxm
- workspace.rpxm
- rpx.lock


公開interfaceとパッケージ構成は、マクロ展開に依存させず明示的に記述する。

### 4. マクロpattern変数
#### 4.1 基本表記

マクロ呼出しから構文を受け取る変数には$を付ける。

$condition

$expression

$body

#### 4.2 通常identifierとの区別
$name:
マクロpattern変数

name:
通常のRPX identifierまたはtemplate由来identifier


通常のRPX identifierは$で開始できない。

これにより、マクロ変数と通常の名前を字句上区別する。

#### 4.3 受け取るもの

マクロpattern変数は実行時の値ではなく、入力構文を受け取る。

呼出し：

(unless failed?
  (continue))


では、次の構文が束縛される。

$condition:
failed?という構文

$expression:
(continue)という構文


この時点では、どちらも評価されない。

#### 4.4 Pattern変数の重複

同一pattern内で、同じマクロpattern変数を複数回束縛できない。

不適合：

(macro same
  ($value $value)
  ->
  $value)


診断例：

duplicate macro pattern variable:
  $value

#### 4.5 Template内での再利用

Patternで一度束縛した変数は、template内で複数回使用できる。

(macro duplicate
  ($expression)
  ->
  (tuple $expression $expression))


ただし、同じ入力式を複数回挿入すると実行時にも複数回評価され得る。

一度だけ評価する必要がある場合は、衛生的な局所bindingを生成する。

### 5. 固定arity
#### 5.1 基本方針

固定数の入力で足りるマクロは、固定arityとする。

(macro unless
  ($condition $expression)
  ->
  ...)

#### 5.2 引数不足
(unless failed?)


は静的エラーである。

macro `unless` expects 2 arguments, but received 1

#### 5.3 引数過剰
(unless failed?
  first-expression
  second-expression)


も静的エラーである。

macro `unless` expects 2 arguments, but received 3

#### 5.4 通常関数との整合

マクロも通常関数と同様に、明示されたarityを持つ。

ただし、マクロの入力は値ではなく構文である。

### 6. 末尾反復
#### 6.1 一個以上の入力

一個以上の末尾構文を受け取る場合は...+を使用する。

$body ...+

#### 6.2 Templateでの展開

Patternで反復束縛した構文列は、template内で...を使って展開する。

(seq $body ...)

#### 6.3 使用例
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))


呼出し：

(when ready?
  first-expression
  second-expression)


対応：

$condition:
ready?

$body:
first-expression
second-expression

#### 6.4 反復位置

可変長反復はpattern引数列の末尾にだけ置ける。

適合：

($condition $body ...+)


不適合：

($prefix ...+ $last)

#### 6.5 反復数

一つのマクロpatternに含められる可変長反復は一つだけである。

#### 6.6 0個以上の反復

0個以上を表すpatternの...はv1では提供しない。

本文が不要な場合は、固定arityの別マクロを定義する。

### 7. Template
#### 7.1 Template変数

Template内で使用できる$nameは、同じマクロpatternで束縛されたものだけである。

不適合：

(macro example
  ($input)
  ->
  (+ $input $missing))


診断例：

unbound macro pattern variable:
  $missing

#### 7.2 通常identifier

Template内の$を持たないidentifierは、通常のRPX構文を生成する。

(if $condition
    $expression
    unit)


ここでifとunitはマクロpattern変数ではない。

#### 7.3 Templateの構文妥当性

展開template自体が構文として不正な場合、マクロ定義位置で静的エラーにする。

入力挿入後にのみ判明する構文不整合は、マクロ呼出し位置を主位置として診断する。

#### 7.4 評価回数

Templateへ同じ入力構文を複数回挿入すると、展開後の実行時評価も複数回発生し得る。

(macro twice
  ($expression)
  ->
  (+ $expression $expression))


Effectfulな式を一度だけ評価する必要がある場合は、局所bindingを生成する。

### 8. Scope
#### 8.1 コンパイル単位内限定

v1のマクロは、定義された同一コンパイル単位内だけで使用できる。

- 別ファイルからimportできない
- 別パッケージへ公開できない
- .rpiから公開できない

#### 8.2 宣言順序

マクロは定義位置より後ろでのみ使用できる。

適合：

(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

(val result
  (when ready?
    (render document)))


不適合：

(val result
  (when ready?
    (render document)))

(macro when
  ...)

#### 8.3 下位モジュール

下位モジュールは、定義位置より前にある親モジュールのマクロを使用できる。

(macro when
  ($condition $body ...+)
  ->
  ...)

(module rendering
  (val render-if-ready
    (fn (document)
      (when ready?
        (render document)))))

#### 8.4 子scopeから親scope

マクロscopeは通常の逐次的な字句scopeに従う。

後続宣言や兄弟下位モジュールでまだ定義されていないマクロは参照できない。

### 9. マクロ名と呼出し
#### 9.1 呼出し構文

マクロ呼出しは、通常のS式適用と同じ表面形を持つ。

(when condition
  expression)

#### 9.2 判別

呼出しheadがマクロbindingへ解決された場合、そのformをマクロ呼出しとして展開する。

呼出しheadが通常の値bindingへ解決された場合、通常関数適用として扱う。

#### 9.3 名前衝突

同一scopeで、マクロと通常値が同じlocal spellingを持ち、呼出しが曖昧になることを禁止する。

ambiguous call head:
  when

both a macro and a value are visible

#### 9.4 名前変更

マクロ名のrenameは、同一コンパイル単位内の対応する呼出しを通常のrename refactoring対象とする。

マクロは外部公開されないため、v1では公開API identityを持たない。

### 10. 展開段階
#### 10.1 処理順序

概念的なコンパイル順序は次とする。

### 1. Sourceをreaderで読む
### 2. Lossless CSTを構築する
### 3. 逐次scopeに従ってマクロ定義を認識する
### 4. 式マクロを展開する
### 5. 通常の名前解決を行う
### 6. 型検査する
### 7. IRへ変換する
### 8. 実行可能artifactを生成する

#### 10.2 型検査前展開

マクロは通常の型検査より前に展開する。

展開後の式全体を通常の型checkerが検査する。

#### 10.3 型情報

マクロは次を参照できない。

- 入力式の推論型
- 期待型
- overload解決結果
- effect推論結果
- pattern網羅性情報

#### 10.4 Typed macro

型検査結果を参照して構文生成するtyped macroはv1では導入しない。

### 11. 展開順序
#### 11.1 Source order

マクロ定義と通常宣言の可視性はsource orderに従う。

#### 11.2 外側からの展開

マクロ呼出しを検出した場合、その呼出しを展開し、展開結果を同じ位置で再度検査する。

展開結果に先行定義済みの別マクロ呼出しが含まれる場合、さらに展開できる。

#### 11.3 決定性

展開順序をhash mapの反復順、thread schedulingまたはfilesystem順へ依存させない。

同じsourceと同じ処理系versionからは、同じ展開結果を得なければならない。

#### 11.4 展開後の再検査

展開結果が再びマクロ呼出しである場合、通常式になるまで展開を繰り返す。

ただし、展開予算を超えてはならない。

### 12. 再帰マクロ
#### 12.1 自己再帰

マクロが直接自分自身を展開結果へ含めることを禁止する。

不適合：

(macro forever
  ($value)
  ->
  (forever $value))

#### 12.2 相互再帰

複数マクロによる相互再帰も禁止する。

macro-a
→ macro-b

macro-b
→ macro-a

#### 12.3 先行マクロの利用

後から定義されたマクロが、既に定義済みの別マクロを利用することはできる。

(macro unless
  ($condition $body ...+)
  ->
  ...)

(macro unless-ready
  ($body ...+)
  ->
  (unless ready?
    $body ...))

#### 12.4 参照graph

マクロ参照graphはDAGでなければならない。

DAG:
有向非巡回graph
循環のない依存関係

### 13. 衛生性
#### 13.1 定義

マクロの衛生性とは、マクロが生成した名前と利用側の名前が、偶然同じ綴りであるだけで衝突しない性質である。

通常の説明では、次の語を使用する。

マクロの衛生性
マクロ展開時の名前衝突防止

#### 13.2 基本保証
- 入力構文のidentifierは利用側contextを維持する
- Templateが新しく導入したbinderはfresh identityを持つ
- Template内の定義側参照は定義側bindingへ解決する
- 利用側の同名bindingを偶然取り込まない

#### 13.3 Fresh identity

マクロが生成する局所binderは、展開ごとに異なる内部identityを持つ。

Source上の綴りが同じでも、別展開で生成されたbinderは別bindingである。

#### 13.4 入力構文の再挿入

Pattern変数から受け取ったidentifierをtemplateへ挿入する場合、元の名前解決contextを維持する。

利用側で別bindingを意味していたidentifierを、マクロ定義側の同名bindingへ変更してはならない。

### 14. 衛生性の例
#### 14.1 マクロ定義
(macro evaluate-once
  ($expression)
  ->
  (local
    (val temporary
      $expression)

    temporary))

#### 14.2 利用
(local
  (val temporary 100)

  (evaluate-once
    (+ temporary 1)))

#### 14.3 概念的なidentity
利用側:
temporary#user

マクロ生成:
temporary#expansion


展開後の意味は概念的に次となる。

(local
  (val temporary#user 100)

  (local
    (val temporary#expansion
      (+ temporary#user 1))

    temporary#expansion))


#user等の表記は説明用であり、利用者がsourceへ書くものではない。

### 15. 意図的capture
#### 15.1 v1の方針

利用側の名前を綴りから検索して意図的に取り込む一般APIは提供しない。

intentional capture:
意図的な名前取り込み

v1:
不採用

#### 15.2 Binder名の明示

マクロが利用側で使われるbinder名を必要とする場合、呼出し側から明示的に受け取る。

概念例：

(with-context context
  (render context))


隠れたcontextを生成するのではなく、入力構文として受け取ったidentifierをbinder位置で使用する。

#### 15.3 理由
- 呼出しだけから導入名が分かる
- 名前衝突が予測可能
- Refactoringが容易
- 隠れたbinding依存を避けられる

### 16. 純粋性
#### 16.1 展開入力

マクロ展開結果は、次だけから決まる。

- マクロ定義
- マクロ呼出しの入力構文
- 先行して定義されたマクロ環境
- 言語およびマクロ仕様version

#### 16.2 禁止される外部入力

マクロ展開から次へアクセスできない。

- filesystem
- package resource
- network
- 環境変数
- 時刻
- 乱数
- process
- ForeignValue
- editor state
- compiler内部の非公開情報

#### 16.3 通常値

通常のvalや関数を、マクロ展開中に実行できない。

Macro phaseとruntime phaseは分離する。

#### 16.4 再現性

同じ入力に対する展開は、環境や実行machineによらず同じ構文結果を生成しなければならない。

### 17. 展開予算
#### 17.1 基本原則

Compilerはマクロ展開へ有限の展開予算を適用する。

仕様本文では「展開予算」または「展開上限」と呼ぶ。

#### 17.2 監視対象

実装は少なくとも次を制限できる。

- 展開回数
- 展開の入れ子深さ
- 生成構文node数

#### 17.3 具体値

具体的な上限値は言語仕様へ固定せず、toolchain policyとして管理する。

ただし、準拠処理系は有限上限を持たなければならない。

#### 17.4 上限超過

上限超過は静的エラーとする。

macro expansion limit exceeded


診断には、可能な範囲で次を含める。

- マクロ名
- 呼出し位置
- 展開chain
- 超過した制限種別

#### 17.5 Sourceからの無制限化

通常RPX sourceから展開予算を無制限に変更できない。

悪意あるsourceまたは依存関係が制限を無効化できてはならない。

### 18. エラー診断
#### 18.1 Pattern不一致

固定arityまたは末尾反復の条件を満たさない場合、マクロ呼出し位置で診断する。

macro invocation does not match its pattern

#### 18.2 引数数
macro:
  unless

expected:
  2 arguments

received:
  1 argument

#### 18.3 ...+

一個以上必要な末尾入力が空の場合：

macro `when` requires at least one body expression

#### 18.4 未束縛pattern変数

Template内で未束縛の$nameを参照した場合、マクロ定義位置でエラーにする。

#### 18.5 不正な展開結果

展開結果が式として不正な場合、主診断位置をマクロ呼出しにし、マクロ定義およびtemplate位置を補助情報として示す。

#### 18.6 型エラー

展開後の式に型エラーがある場合、通常の型診断に加えて、その式がどのマクロ呼出しから生成されたかを示す。

### 19. Provenance
#### 19.1 保持する情報

生成構文には次の由来情報を保持する。

- マクロ定義位置
- マクロ呼出し位置
- 入力構文のsource span
- 展開templateのsource span
- 展開chain

#### 19.2 診断例
type error in syntax generated by macro `when`

macro invocation:
  current-file.rpx:42

macro definition:
  current-file.rpx:10

#### 19.3 Source map

Formatter、IDE、定義移動およびエラー表示のため、生成構文から元sourceへの対応を失わない。

#### 19.4 正式identity

v1では宣言マクロを導入しないため、マクロが公開DefinitionIdを生成することはない。

局所binderについてのみ、展開ごとのfresh identityを割り当てる。

### 20. マクロの外部公開
#### 20.1 v1の制限

マクロを別コンパイル単位または別パッケージへ公開できない。

- macro-dependenciesなし
- import-macroなし
- .rpiのmacro仕様なし
- compiled macro artifactなし

#### 20.2 標準構文

複数パッケージで共通に必要な必須構文は、次のいずれかとして提供する。

- Core特殊形式
- 通常関数
- 標準libraryの通常値

#### 20.3 将来拡張

再利用可能なマクロpackageが必要になった場合、macro dependencyとinterfaceの設計を別項目で追加する。

### 21. 適合試験
#### 21.1 適合試験 MAC-01：固定arity
(macro unless
  ($condition $expression)
  ->
  (if $condition
      unit
      $expression))

(val result
  (unless failed?
    (continue)))


期待結果：

success

展開:
(if failed?
    unit
    (continue))

#### 21.2 不適合試験 MAC-02：引数不足
(unless failed?)


期待結果：

static error:
macro `unless` expects 2 arguments, but received 1

#### 21.3 不適合試験 MAC-03：引数過剰
(unless failed?
  first
  second)


期待結果：

static error:
macro `unless` expects 2 arguments, but received 3

#### 21.4 適合試験 MAC-04：末尾反復
(macro when
  ($condition $body ...+)
  ->
  (if $condition
      (seq $body ...)
      unit))

(when ready?
  (prepare)
  (render document))


期待結果：

success

展開:
(if ready?
    (seq
      (prepare)
      (render document))
    unit)

#### 21.5 不適合試験 MAC-05：空の...+
(when ready?)


期待結果：

static error:
macro `when` requires at least one body expression

#### 21.6 不適合試験 MAC-06：反復が末尾以外
(macro invalid
  ($prefix ...+ $last)
  ->
  $last)


期待結果：

static error:
variable-length macro repetition must appear at the end of the pattern

#### 21.7 不適合試験 MAC-07：重複pattern変数
(macro duplicated
  ($value $value)
  ->
  $value)


期待結果：

static error:
duplicate macro pattern variable:
  $value

#### 21.8 不適合試験 MAC-08：未束縛template変数
(macro invalid
  ($input)
  ->
  (+ $input $missing))


期待結果：

static error:
unbound macro pattern variable:
  $missing

#### 21.9 適合試験 MAC-09：衛生的binder
(macro evaluate-once
  ($expression)
  ->
  (local
    (val temporary
      $expression)

    temporary))

(local
  (val temporary 100)

  (evaluate-once
    (+ temporary 1)))


期待結果：

success

利用側temporaryと、
マクロ生成temporaryは別binding

#### 21.10 不適合試験 MAC-10：定義前使用
(val result
  (when ready?
    (render document)))

(macro when
  ($condition $body ...+)
  ->
  ...)


期待結果：

static error:
macro is not defined at this source position:
  when

#### 21.11 不適合試験 MAC-11：自己再帰
(macro forever
  ($value)
  ->
  (forever $value))


期待結果：

static error:
recursive macro expansion is not permitted

#### 21.12 不適合試験 MAC-12：宣言位置
(define-values first second)


define-valuesが式マクロとして定義されており、宣言位置で使用された場合：

static error:
expression macro cannot be used in declaration position

#### 21.13 不適合試験 MAC-13：Interface
// module.rpi

(when condition
  specification)


期待結果：

static interface error:
user-defined macros are not permitted in interface files

#### 21.14 不適合試験 MAC-14：展開上限

展開chainが処理系の有限上限を超えた場合：

static error:
macro expansion limit exceeded

### 22. 移管先OPEN・下位項目・状態
#### 22.1 `OPEN-MAC-EXT-001`

次を将来項目として移管する。

- 複数rule
- 0個以上の反復
- 複数反復
- 構文分類付きpattern変数
- 宣言マクロ
- 型マクロ
- patternマクロ
- module itemマクロ

#### 22.2 `OPEN-MAC-PKG-001`

次を将来項目として移管する。

- マクロのパッケージ間公開
- macro-dependencies
- import-macro
- .rpiのmacro仕様
- compiled macro artifact
- macro packageのinterface hash

#### 22.3 `OPEN-MAC-PROC-001`

次を将来項目として移管する。

- procedural macro
- syntax object操作API
- 独自診断API
- procedural macro sandbox
- 実行step・memory・time制限

#### 22.4 `OPEN-MAC-TYPED-001`

次を将来項目として移管する。

- typed macro
- 型検査後のcode generation
- derive
- 型情報を利用した展開

#### 22.5 `OPEN-MAC-CAP-001`

次を将来項目として移管する。

- 意図的capture
- 非衛生的identifier生成
- 利用側scopeの明示操作

#### 22.6 `OPEN-EDT-001`

次を移管する。

- マクロ呼出しsiteのstable provenance
- 展開結果とsource editの対応
- 展開構文に対するrename
- stale transaction

#### 22.7 下位項目

```text
OPEN-MAC-001A
マクロの最小Surface構文
→ RESOLVED

OPEN-MAC-001B
Pattern変数・固定arity・末尾反復
→ RESOLVED

OPEN-MAC-001C
Scope・宣言順序・再帰禁止
→ RESOLVED

OPEN-MAC-001D
展開段階・展開順序
→ RESOLVED

OPEN-MAC-001E
衛生性・名前衝突防止
→ RESOLVED

OPEN-MAC-001F
展開予算・診断
→ RESOLVED

OPEN-MAC-001G
Identity・provenance
→ RESOLVED

OPEN-MAC-001H
外部公開範囲
→ RESOLVED
```

#### 22.8 最終状態

```text
OPEN-MAC-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 第一級関数を中心とする通常の抽象化
- 関数では表現できない評価制御向けの限定的な式マクロ
- 単純な一pattern構文
- 固定arityと末尾反復
- 型検査前の決定的な展開
- 自動的な名前衝突防止
- 隠れた名前captureを持たない安全な展開
- 再帰しない有限な展開
- マクロ依存をパッケージ境界へ持ち込まない単純なv1設計


## OPEN-EDT-001 編集スナップショット・トランザクション・競合・由来情報
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
編集可能な文書モデル
不変スナップショット
文書・ノード・トランザクションの識別
編集トランザクション
適用前条件
旧版トランザクション
競合検出
Undo／Redo
由来情報
派生ノード
逆編集
編集エンジンの公開API

#### DD-001.2 位置付け

OPEN-EDT-001は、主としてRPX言語の構文仕様ではなく、Reciplexa編集エンジンの規範的なソフトウェア仕様である。

本項目では、異なる実装でも揃えるべき意味論と公開APIを定める。一方、内部データ構造やIDのbit表現などは実装依存とする。

規範的に定めるもの:
- スナップショットの不変性
- ノード識別子の継続性
- トランザクションの原子性
- 旧版編集の再検証
- 競合分類
- Undoの意味
- 由来情報と逆編集の基本原則
- 公開API上の型と結果分類

実装依存とするもの:
- IDの具体的なbit表現
- スナップショットの内部データ構造
- Hash algorithm
- 履歴の具体的な保持上限
- 永続形式
- Network wire protocol
- Cacheおよびindex構造

### 0. 適用範囲
#### 0.1 本項目が扱う編集

本項目が正規の編集対象とするのは、永続的なノード識別子を持つ編集可能な文書・シーンモデルである。

対象:
- 文書ノード
- シーンノード
- ノードのproperty
- 所有関係
- 明示的なノード参照

#### 0.2 直接の対象としないもの

次は本項目の正規編集対象ではない。

- RPXソーステキスト
- Lossless CST
- 型検査済みAST
- コンパイラ内部の一時IR
- Backend artifact


ソースコードのrenameやimport追加などは、IDE・refactoring仕様へ移管する。

ただし、次の識別情報は共有できる。

- PackageInstanceId
- ModuleId
- DefinitionId
- BindingId
- TypeId
- SyntaxNodeId
- マクロ展開provenance

### 1. 編集モデルの基本原則
#### 1.1 不変値と継続的identity

通常の文書ノード値は不変とする。

一方、内容が変化しても同じ段落・図形・セクションとして追跡する必要があるため、各ノードは安定したNodeIdを持つ。

NodeValue:
不変値

NodeId:
版をまたいで同じ編集対象を追跡する識別子


例：

revision 10:
P -> Paragraph("Hello")

revision 11:
P -> Paragraph("Hello world")


値は置き換わっているが、PというNodeIdは維持される。

#### 1.2 スナップショット

文書状態は、不変のDocumentSnapshotとして表す。

概念的には次の構成を持つ。

DocumentSnapshot {
  document-id,
  revision,
  root-node-id,
  node-store,
  provenance-index
}


スナップショット取得後に現在文書が更新されても、取得済みスナップショットの論理内容は変わらない。

#### 1.3 現在文書

変化していく現在文書はDocumentHandleによって表す。

DocumentSnapshot:
特定revisionの不変状態

DocumentHandle:
現在revisionへ接続された状態付きhandle

### 2. 文書の所有構造
#### 2.1 単一rootの所有tree

文書の親子関係は、単一rootを持つ所有treeに限定する。

- Rootは一つ
- Rootは親を持たない
- Root以外の各到達可能ノードは、ちょうど一つの親を持つ
- 所有関係のcycleは禁止


例：

document
└─ page
   ├─ paragraph
   └─ image

#### 2.2 所有と参照の分離

ノードの構造的な包含関係は所有edgeで表す。

構造上の親子ではない関連は、property内の明示的なNodeId参照として表す。

例：

TableOfContentsEntry {
  target: HeadingNodeId
}

Annotation {
  target: ParagraphNodeId
}


参照先ノードを所有しているわけではない。

#### 2.3 共有

同じノードを複数の親が所有することは禁止する。

共有が必要なものは次のいずれかで表す。

- 不変値の共有
- package-resource等のhandle共有
- NodeIdによる明示参照
- 別々のinstance nodeから共有定義を参照

#### 2.4 規範的な親子情報

所有関係の正本は、親ノードのchildren列とする。

規範データ:
親から子への所有edge

派生index:
子から親への逆index


実装は性能のために親indexを保持できるが、意味上の正本ではない。

### 3. 文書treeの不変条件

確定済みスナップショットは、次をすべて満たさなければならない。

### 1. RootNodeIdがnode storeに存在する
### 2. Rootは親を持たない
### 3. Root以外の全ノードはちょうど一つの親を持つ
### 4. 所有edgeにcycleがない
### 5. 同じ親のchildren列内に同一NodeIdが重複しない
### 6. 全ノードが同じDocumentIdに所属する
### 7. 全ノードがRootから到達可能である
### 8. 必須propertyが存在する
### 9. Property値がNode kindのschemaに適合する
### 10. 強いNodeId参照が有効な対象を指す

#### 3.1 到達不能ノード

確定済みスナップショットでは、Rootから到達不能なノードを認めない。

ただし、トランザクションの仮適用中は、一時的なdetached nodeを許可する。

Transaction適用中:
一時的detached nodeを許可

Commit時:
全ノードの到達可能性を要求


Clipboardや一時作業領域は、文書スナップショットとは別の所有領域にする。

### 4. 識別子
#### 4.1 識別子の種類
DocumentId:
一つの文書系列を識別する

NodeId:
文書系列内の一つのノードを識別する

TransactionId:
一つの編集要求を識別する

Revision:
文書系列内の版を識別する

PropertyId:
ノードschema内のpropertyを識別する


意味上のノードaddressは次である。

NodeAddress =
DocumentId + NodeId

#### 4.2 Opaque型

各IDは内部表現を隠したopaque型とする。

許可される基本操作：

- 等値比較
- Hashing
- 検証済みserialization
- Debug表示


許可しない操作：

- 数値演算
- 内部componentの分解
- 順序へ意味を持たせる比較
- 自由な文字列からの無検証生成


IDを知っていることは、編集権限を持つことを意味しない。

#### 4.3 DocumentId

新規文書作成時に発行する。

NewDocument
→ 新DocumentId
→ revision 0
→ Root NodeId発行


保存pathやファイル名とは独立したidentityである。

通常の保存、ファイル名変更、保存先変更では維持する。

#### 4.4 NodeId

必須保証：

- 同一文書系列内で一意
- 内容変更で維持
- Moveで維持
- Copyで新規発行
- 削除後に再利用しない
- Serialize／deserializeで維持


Node kindもNodeIdの生存期間中は不変とする。

Kind変更は旧ノードの削除と新ノードの作成として表す。

#### 4.5 TransactionId

トランザクション作成者が発行する。

同じ要求のnetwork retry等では、同じTransactionIdを再利用する。

同一ID + 同一content hash:
AlreadyApplied

同一ID + 異なるcontent hash:
TransactionIdentityConflict

#### 4.6 ID表現

UUID、中央連番、ActorIdとsequenceなどの具体形式は固定しない。

各IDは、将来のoffline生成を妨げないopaque表現とする。

### 5. 保存・複製・Fork
#### 5.1 通常保存

通常保存では、次を維持する。

- DocumentId
- NodeId
- 現在revision


再読込後も同じ文書系列として扱う。

#### 5.2 Save As

通常のSave Asは、同じ文書identityを別の保存先へ書く操作とする。

DocumentId:
維持

保存path:
変更

#### 5.3 Duplicate／Fork

独立した文書を作る場合は、明示的なDuplicateまたはFork操作を使う。

- 新しいDocumentId
- 全NodeIdを新規発行
- 新しいrevision系列


対応表を結果として返せる。

CloneMap {
  old-node-id -> new-node-id
}

#### 5.4 内部参照の複製

複製元文書内のNodeId参照は、対応する新NodeIdへ書き換える。

元:
Link L1 -> Heading H1

複製:
Link L2 -> Heading H2

#### 5.5 文書間参照

v1の規範モデルでは、通常のNodeId参照を同じDocumentId内に限定する。

文書間参照は、将来の外部anchor仕様へ移管する。

### 6. Revision
#### 6.1 直線的な履歴

一つの文書は、一本の単調増加するrevision列を持つ。

revision 0
    ↓ T1
revision 1
    ↓ T2
revision 2


Branchとmergeはv1では導入しない。

#### 6.2 Commitの直列化

複数actorは同じsnapshotからトランザクションを並行生成できる。

ただし、同一DocumentIdへのcommitは一つずつ直列化する。

Transaction生成:
並行可能

Transaction適用:
直列化

Transaction内部:
原子的

#### 6.3 Revision増加

意味上の変更を伴うトランザクションが成功した場合だけ、revisionを増加させる。

Applied:
revision + 1

Rejected:
変更なし

AlreadyApplied:
変更なし

AppliedNoChange:
revision変更なし


Revisionはwraparoundしてはならない。

#### 6.4 保存後のrevision

通常保存・再読込ではrevisionを維持する。

履歴がcompactされても、revision番号を巻き戻さない。

Revision番号だけで適用可能性を判断せず、操作ごとの適用前条件を再検証する。

### 7. Snapshotの保持
#### 7.1 不変性

有効なsnapshot handleが存在する間、そのスナップショットの論理内容を参照できなければならない。

内部実装は次を自由に選べる。

- 永続データ構造
- Copy-on-write
- Checkpointと差分
- Operation logからの再構築

#### 7.2 過去版の永久取得

Revision番号だけを指定して、任意の過去snapshotを永久に取得できることは保証しない。

明示的に保持されたsnapshot handleだけが、その寿命中の利用を保証される。

#### 7.3 履歴の種類

次の履歴は別々の保持policyを持てる。

- Snapshot history
- Transaction history
- Undo history
- Tombstone metadata
- Audit log


具体的な保持上限は実装依存とする。

### 8. 編集トランザクション
#### 8.1 概念構造
EditTransaction {
  transaction-id,
  document-id,
  base-revision,
  transaction-preconditions,
  ordered-operations,
  origin
}

#### 8.2 不変の第一級値

EditTransactionは不変の第一級値とする。

- 関数へ渡せる
- 関数から返せる
- Previewできる
- 保存・送信の対象にできる
- Commit前に検査できる


ただし、内部表現は公開せず、抽象型として提供する。

#### 8.3 原子性

トランザクションは全体が一括して成功または失敗する。

全操作成功:
すべて適用

一つでも失敗:
何も適用しない


部分成功を認めない。

#### 8.4 Operation順序

Operationは記載順に作業スナップショットへ仮適用する。

各操作後の一時状態が最終不変条件を満たす必要はない。

トランザクション末尾で文書全体の不変条件を検査する。

### 9. 編集Operation

v1の基本Operationは次とする。

CreateNode
DeleteNode
SetProperty
InsertChild
RemoveChild
MoveNode

#### 9.1 CreateNode
CreateNode {
  node-id,
  kind,
  initial-properties
}


検査内容：

- NodeIdが未使用
- Node kindが有効
- 必須propertyが存在
- Property値がschemaへ適合


作成直後のdetached状態は、同一トランザクション内に限り認める。

#### 9.2 DeleteNode
DeleteNode {
  node-id,
  expected-parent,
  expected-position,
  expected-subtree-fingerprint?
}


対象ノードと所有subtree全体を削除する。

Rootは削除できない。

Subtree外から強い参照がある場合、同一トランザクション内で参照を解消しない限り拒否する。

#### 9.3 SetProperty
SetProperty {
  node-id,
  property-id,
  expected-old-value,
  replacement
}


意味：

none -> some:
property追加

some -> some:
property変更

some -> none:
property削除


必須propertyは削除できない。

期待旧値と現在値が異なれば競合とする。

#### 9.4 InsertChild
InsertChild {
  parent-id,
  child-id,
  position
}


子位置は整数indexではなくanchor方式を使う。

ChildPosition :=
  First
  | Last
  | Before(NodeId)
  | After(NodeId)


検査内容：

- 親と子が存在
- 子が未所有
- Anchorが有効
- 親schemaが子kindを許可
- Cycleを生成しない

#### 9.5 RemoveChild

親子関係を一時的に解除する低水準Operationとする。

通常の高水準APIではMoveNodeを優先する。

確定時にdetached nodeが残る場合、トランザクションを拒否する。

#### 9.6 MoveNode
MoveNode {
  node-id,
  expected-old-parent,
  expected-old-position,
  new-parent,
  new-position
}


Moveでは次を維持する。

- NodeId
- Node kind
- Property
- 所有subtree
- 非所有参照
- Provenance


禁止：

- Rootの移動
- 自身の子孫への移動
- 異なるDocumentIdへの直接移動
- Schemaに適合しない親への移動

### 10. NodeIdの予約
#### 10.1 発行方式

正式なNodeIdは、文書編集contextに属するallocatorから、トランザクション構築前に予約する。

ID予約:
effectful

Transaction組立て:
pure

Commit:
effectful

#### 10.2 予約済みID

トランザクションが失敗または破棄された場合でも、予約済みNodeIdを再利用しない。

欠番は問題にしない。

#### 10.3 Copy

Copyは既存NodeIdを再利用せず、新しいNodeIdを予約して内容を複製する。

MoveとCopyは明確に分ける。

### 11. 適用前条件
#### 11.1 Operation固有条件

対象固有の条件はOperationへ直接含める。

例：

SetProperty:
期待旧値

MoveNode:
期待旧親・旧位置

DeleteNode:
期待親・位置・任意fingerprint

#### 11.2 Transaction全体の条件

複数Operationに共通する大域条件だけを、transaction-level preconditionとして持つ。

例：

- 文書schema version
- 特定の大域状態
- 外部resource revision

#### 11.3 Base revision

base-revisionは、トランザクションがどのsnapshotから作られたかを示す。

Base revisionが現在revisionと同じであることを、適用の絶対条件にはしない。

### 12. Stale transaction
#### 12.1 定義

現在revisionより古いrevisionを基に作成されたトランザクションを、旧版トランザクションまたはstale transactionと呼ぶ。

#### 12.2 分類
Fresh:
base = current

StaleApplicable:
base < current
かつ全ての適用前条件が成立

StaleConflicted:
base < current
かつ一つ以上の条件が不成立

InvalidFuture:
base > current

#### 12.3 保守的な再適用

Stale transactionを別の内容へ自動変換する高度なmergeは行わない。

同じOperationを現在状態へ再検証し、そのまま適用できる場合だけ適用する。

#### 12.4 無関係な変更

別ノードまたは独立propertyへの変更だけが行われている場合、旧版トランザクションを適用できる。

### 13. 競合
#### 13.1 基本分類
EditConflict =
  DocumentMismatch
  | FutureRevision
  | TargetMissing
  | TargetAlreadyExists
  | NodeKindMismatch
  | PropertyValueMismatch
  | ParentMismatch
  | PositionMismatch
  | AnchorMissing
  | OwnershipConflict
  | CycleWouldBeCreated
  | NodeStillReferenced
  | SubtreeChanged
  | TransactionIdentityConflict

#### 13.2 競合と不正トランザクション

適用不成立を次の二種類に分ける。

Conflict:
作成時には妥当だったが、
現在状態の変化により適用不能

InvalidTransaction:
形式、型、schemaまたは基本制約に違反


概念的な拒否理由：

EditRejection =
  Conflicted(List<EditConflict>)
  | Invalid(EditValidationError)

#### 13.3 競合の収集

独立に検査可能な競合は、可能な限り一度に収集する。

先行Operationの失敗によって後続Operationが意味を失う場合、派生的な失敗は抑制する。

#### 13.4 自動併合

自動併合する範囲：

- 異なるNodeIdへの独立変更
- 同一ノードの独立propertyへの変更
- 同一propertyを同じ値へ変更する冗長操作


競合として拒否する範囲：

- 同一propertyへの異なる変更
- 削除済みノードへの操作
- 同一ノードの異なる親への移動
- 同一anchor・同一側への順序依存挿入
- 親子構造が両立しない変更


CRDTや高度な共同編集mergeはv1では導入しない。

### 14. 適用手順

トランザクションは次の順序で処理する。

### 1. TransactionIdを確認
### 2. DocumentIdを確認
### 3. Base revisionを比較
### 4. Transaction-level preconditionを検査
### 5. 現在snapshotから作業状態を作成
### 6. Operationを順番に仮適用
### 7. 文書全体の不変条件を検査
### 8. 成功時だけcommit
### 9. 新revisionとUndo情報を生成


一つでも拒否理由があれば、現在スナップショットを変更しない。

### 15. 適用結果
#### 15.1 結果型
ApplyResult =
  Applied
  | AppliedNoChange
  | Rejected
  | AlreadyApplied

#### 15.2 Applied

次を含む。

- TransactionId
- 旧revision
- 新revision
- 新snapshot
- UndoToken

#### 15.3 AppliedNoChange

意味上の変更がなかったことを表す。

Revisionは増加しない。

#### 15.4 Rejected

次を含む。

- TransactionId
- Base revision
- Current revision
- EditRejection

#### 15.5 AlreadyApplied

同じTransactionIdと同じ内容のトランザクションが既に適用済みであることを表す。

編集を再適用しない。

#### 15.6 Transaction content hash

TransactionIdの再送判定では、意味に影響する内容からhashを作る。

- DocumentId
- Base revision
- 適用前条件
- Operation列
- Operation順序
- NodeId
- PropertyId
- 旧値・新値
- Anchor

### 16. UndoとRedo
#### 16.1 新revision

Undoは過去revisionへ巻き戻す処理ではない。

逆トランザクションを現在状態へ適用し、新revisionを作る。

T1:
revision 10 -> 11

Undo T1:
revision 11 -> 12

#### 16.2 Undo情報

適用成功時に、逆操作に必要な情報を内部記録する。

CreateNode:
作成NodeId

DeleteNode:
削除subtree

SetProperty:
旧値

MoveNode:
旧親と旧位置

#### 16.3 UndoToken

標準公開APIでは、内部の逆トランザクションそのものではなく、opaqueなUndoTokenを返す。

UndoToken:
特定の適用結果をundoする権利・参照

#### 16.4 Undo競合

Undoも通常のトランザクションと同じ適用前条件を検査する。

他の変更を暗黙に消すような強制Undoは行わない。

#### 16.5 Redo

Redoも新しいTransactionIdを持つ新トランザクションとして実行する。

#### 16.6 履歴保持

Undo履歴は有限にできる。

- 最大件数
- 最大容量
- Checkpoint以前の破棄


具体値は実装依存とする。

Undo可能性は照会できなければならない。

### 17. Provenance
#### 17.1 定義

Provenanceは、ノードがどこから生成されたかを示す由来情報である。

NodeId:
ノードの同一性

Provenance:
ノードの生成元・導出経路


Provenanceを変更してもNodeIdは変わらない。

#### 17.2 Optional metadata

Provenanceはoptionalとする。

欠落していてもノードは有効だが、生成元への移動や逆編集は利用できない。

#### 17.3 構造

Provenanceは共有可能なDAGとして管理できる。

NodeId
→ ProvenanceId
→ ProvenanceRecord
→ parent ProvenanceId


Provenance graphにcycleを認めない。

#### 17.4 種類
Provenance =
  UserCreated
  | SourceGenerated
  | MacroGenerated
  | Imported
  | Copied
  | Derived

#### 17.5 UserCreated

利用者操作によって直接作成されたノード。

- 作成TransactionId
- Actor等の任意補助情報

#### 17.6 SourceGenerated

RPX sourceの評価から生成されたノード。

記録候補：

- PackageInstanceId
- ModuleId
- DefinitionId
- SyntaxNodeId
- Source revision


絶対filesystem pathは規範的provenanceへ保存しない。

#### 17.7 MacroGenerated

次を区別して記録する。

- マクロ呼出し位置
- マクロ定義位置
- Template内構造path
- 入力由来構文かtemplate由来構文か

#### 17.8 Imported

別文書または外部データから取り込まれたノード。

Import後は新しいNodeIdを持ち、通常は独立編集可能とする。

#### 17.9 Copied

同一文書内のcopyまたは文書forkによって作られたノード。

元NodeIdとの関係を記録するが、identityは共有しない。

#### 17.10 Derived

Layout、filter、geometry処理等から派生したノード。

DerivedOrigin {
  producer-id,
  input-node-addresses,
  input-revisions,
  derivation-key?
}


複数入力を持てる。

### 18. 派生ノードと逆編集
#### 18.1 編集可能性

ノードまたはノード層を次のように分類できる。

Editable:
直接編集可能

SourceMapped:
逆写像できる編集だけ可能

DerivedReadOnly:
参照可能だが直接編集不可

Ephemeral:
内部処理専用

#### 18.2 Provenanceと逆編集

Provenanceが存在するだけでは、逆編集可能とはみなさない。

生成結果への編集を上位モデルへ変換するには、明示的な逆編集mapperが必要である。

#### 18.3 逆編集結果
ReverseEditResult =
  Mapped
  | NotRepresentable
  | StaleOrigin
  | Ambiguous

#### 18.4 自動選択

複数の逆写像候補がある場合、v1では自動選択しない。

#### 18.5 逆写像不能

v1では次の二つだけを採用する。

逆編集可能:
上位modelへのEditTransactionへ変換

逆編集不能:
編集を拒否


OverrideとDetachは将来項目とする。

#### 18.6 Stale provenance

派生結果が古い入力revisionから生成されている場合、逆編集を拒否する。

再生成後に再試行する必要がある。

### 19. 派生ノードのID継承
#### 19.1 DerivationKey

派生処理が一意かつ安定したDerivationKeyを提供できる場合、再生成後の対応ノードにNodeIdを継承できる。

DerivationKey {
  producer-id,
  source-node-id,
  local-role
}

#### 19.2 曖昧な対応

再生成前後の対応が曖昧な場合、新しいNodeIdを発行する。

誤った同一視より、ID変更を選ぶ。

#### 19.3 NodeIdとの違い

DerivationKeyはNodeIdそのものではなく、再生成時の対応候補を探すための補助keyである。

### 20. Provenanceの安全性
#### 20.1 真正性

Provenanceは追跡・説明用metadataであり、デジタル署名や権限証明ではない。

外部入力のprovenanceは偽造されている可能性がある。

#### 20.2 Privacy

Provenance export時には次のpolicyを選択できる。

- Preserve
- Summarize
- Remove


ローカル絶対pathや秘密の内部情報を外部文書へ漏らしてはならない。

#### 20.3 書換え

一般利用者が任意のProvenanceを自由に作成・変更するAPIは提供しない。

Provenanceは編集エンジンおよび検証済みimporterが管理する。

### 21. 公開API階層
#### 21.1 純粋な第一級値
DocumentId
NodeId
TransactionId
Revision
DocumentSnapshot
EditTransaction
EditConflict
EditRejection
ApplyResult
Provenance
ChildPosition

#### 21.2 状態付きhandle
DocumentHandle
UndoToken

#### 21.3 抽象型

次は内部表現を公開しない。

- document-snapshot
- document-handle
- edit-transaction
- document-id
- node-id
- transaction-id
- provenance
- undo-token

#### 21.4 Constructor付き公開data

利用者が分岐処理する必要があるため、次はconstructorを公開する。

- apply-result
- edit-rejection
- edit-conflict
- reverse-edit-result

### 22. 高水準APIと低水準API
#### 22.1 高水準API

通常利用者には型付き編集関数を優先提供する。

例：

change-paragraph-text
move-section
replace-image-resource
insert-paragraph


これらはスナップショットから現在値を読み、適切な適用前条件を自動設定する。

#### 22.2 低水準API

Generic inspector、importer、plugin等のために低水準builderを提供できる。

ただし、EditTransactionの内部recordを直接構築させない。

- Builder APIを利用
- 作成時に検証
- Commit時にも完全再検証

#### 22.3 信頼境界

Network、plugin、serialization等から受け取ったトランザクションは信用せず、commit境界で完全に再検証する。

### 23. Pure処理とEffectful処理
#### 23.1 Pure処理
- Snapshotの照会
- Nodeの照会
- EditTransactionの構築
- Transactionの事前検証
- Conflictの解析
- Provenanceの照会

#### 23.2 Effectful処理
- 現在snapshotの取得
- NodeIdの予約
- Transactionのcommit
- Undo／Redo
- Save／Load
- 履歴compaction

#### 23.3 Effectの暫定分類
document-read:
現在文書の観測

document-edit:
ID予約、Commit、Undo、Redo

storage:
永続化


最終的なeffect名とresource lifetimeはOPEN-MEM-001およびOPEN-ERR-001へ移管する。

### 24. 競合・不正・実行障害の分離
#### 24.1 正常な結果
- Applied
- AppliedNoChange
- AlreadyApplied

#### 24.2 意味的拒否
ApplyResult.Rejected
├─ Conflicted
└─ Invalid


これは通常値として返す。

#### 24.3 実行障害

次はApplyResultへ混ぜない。

- Storage failure
- Permission failure
- Cancellation
- Resource exhaustion
- Internal fault


これらはeffect failureとしてOPEN-ERR-001で規定する。

#### 24.4 競合は例外ではない

旧版編集や同一propertyへの並行変更は通常運用で発生し得るため、例外やterminal failureとして扱わない。

### 25. Undo履歴・Transaction履歴
#### 25.1 有限保持

次の履歴は有限保持を許可する。

- Undo情報
- 適用済みTransactionId
- Tombstone metadata
- 過去snapshot

#### 25.2 AlreadyApplied保証

Document transaction serviceがTransactionIdを既知として保持している範囲では、同じトランザクションを二重適用しない。

永久的なexactly-once保証はv1では要求しない。

#### 25.3 Undo不可

Undo情報が破棄済みの場合、明示的なUndoUnavailableを返す。

部分的なUndoを暗黙実行しない。

### 26. 永続化
#### 26.1 標準保存

標準的な文書保存では、少なくとも次を保存する。

- DocumentId
- 現在revision
- Root NodeId
- NodeId
- Node内容
- 必要なschema version

#### 26.2 編集履歴

Transaction履歴、Undo履歴、TombstoneおよびAudit logは任意の別journalとして保存できる。

標準文書形式へ必須で埋め込まない。

#### 26.3 Version付きcodec

DocumentSnapshotやEditTransactionの内部表現を汎用serializationへ直接公開しない。

永続化にはversion付きの専用codecを使用する。

具体的なbinary・text形式は別項目へ移管する。

### 27. 適合試験
EDT-01：ノード内容の変更
Revision 10:
Node P = Paragraph("Hello")

Transaction:
P.textを"Hello world"へ変更


期待結果：

- NodeId Pを維持
- Revision 11を生成
- 旧snapshotは不変

EDT-02：Move
PをSection AからSection Bへ移動


期待結果：

- PのNodeIdを維持
- Pのsubtreeを維持
- 旧親と新親のchildrenを更新

EDT-03：Copy
Pを複製


期待結果：

- 新しいNodeId Qを発行
- PとQは独立編集可能
- QのprovenanceにP由来を記録可能

EDT-04：Cycle
Node Aの子孫Bの下へAを移動


期待結果：

Rejected:
CycleWouldBeCreated

EDT-05：旧版だが独立
T1:
Revision 10を基にP.textを変更

先行T2:
Q.colorだけを変更


期待結果：

T1のpreconditionが成立するため、
Revision 11へ再検証して適用可能

EDT-06：Property競合
T1:
P.textを"A"から"B"へ変更

先行T2:
P.textを"A"から"C"へ変更


期待結果：

Rejected:
PropertyValueMismatch

EDT-07：原子性

複数Operationのうち一つが競合した場合：

- どのOperationも確定snapshotへ反映しない
- Revisionを増加させない

EDT-08：Transaction再送

同じTransactionIdと同じ内容を再送：

AlreadyApplied


同じTransactionIdと異なる内容：

Rejected:
TransactionIdentityConflict

EDT-09：参照中ノードの削除

別ノードから強く参照されるPを、参照を処理せず削除：

Rejected:
NodeStillReferenced

EDT-10：Undo競合
T1:
A -> B

T2:
B -> C

Undo T1:
B -> Aを試行


期待結果：

現在値はCなので、
PropertyValueMismatchとして拒否

EDT-11：文書複製

文書D1をD2へDuplicate：

- 新しいDocumentId
- 全NodeIdを新規発行
- 内部NodeId参照を再対応
- CloneMapを生成可能

EDT-12：Stale provenance

Revision 18から生成された派生ノードを、元文書がRevision 20の時点で逆編集：

StaleOrigin

EDT-13：逆写像不能

一つの生成元から多数のノードが生成され、特定ノードだけの編集を一意に元へ戻せない場合：

NotRepresentable
または
Ambiguous


自動的なsource変更を行わない。

### 28. 移管先OPEN

#### `OPEN-ERR-001`

- Storage failure
- Permission failure
- Cancellation
- Terminal failure
- Cleanup
- Commit failureの伝播

#### `OPEN-MEM-001`

- DocumentHandleのlifetime
- SnapshotHandleの保持
- document-read／document-edit effect
- ID allocator
- 履歴dataの解放

#### `OPEN-CON-001`

- Commit queueの公平性
- 複数actor
- Network retry
- 永続的な重複排除
- Offline collaboration
- CRDT／OT

#### `OPEN-IR-001`

- Node kindとProperty schema
- Document modelとcanonical IRの境界
- 派生IR
- ProducerId
- DerivationKey

#### `OPEN-EDT-CODEC-001`

- 文書保存形式
- Transaction wire format
- Schema migration
- Provenance export policy

#### `OPEN-EDT-COLLAB-001`

- Branch
- Merge
- ActorId
- Offline transaction
- 同位置挿入の決定的順序
- 共同編集履歴

#### `OPEN-EDT-OVERRIDE-001`

- 派生ノードoverride
- Detach
- Override再適用
- 再生成時のoverride追跡

### 29. 最終状態

```text
OPEN-EDT-001:
RESOLVED
```


本解決により、Reciplexa編集エンジンは次を提供する。

- 不変スナップショット
- 単一rootの所有tree
- 安定した文書・ノードidentity
- 直線的なrevision履歴
- 原子的な編集トランザクション
- 旧版編集の保守的な再適用
- 構造化された競合結果
- 二重適用の防止
- 新revisionとしてのUndo／Redo
- 生成元を追跡するprovenance
- 明示的な逆編集可能性
- 派生ノードへの安全な編集制限
- 第一級関数から扱える不変Transaction
- 状態変更と意味的競合と実行障害の分離

## OPEN-ERR-001 通常の失敗・Failure effect・後始末・Defect・最上位実行境界
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
optionとresult
型付きFailure effect
一般の再開可能Effectとの区別
Failure handler
resultとの明示的変換
Resource cleanup
bracket
継続破棄時の後始末
Primary／suppressed failure
Defect
Terminal failure
Fault boundary
Entry pointの最上位失敗処理
Diagnostic

#### DD-001.2 既存仕様との関係

本項目は、既に決定されているエフェクト・ハンドラの基礎仕様を変更しない。

既存仕様から、少なくとも次を前提とする。

- Effectはeffect rowによって型へ記録される
- Handlerはdeep handlerである
- 継続はone-shotである
- Operation探索規則が存在する
- Handlerには正常終了を扱うreturn clauseがある
- Handlerによって処理されたEffectはeffect rowから除かれる
- 一般Effectの継続は高々一回だけresumeできる


本項目で新たに定めるのは、既存のエフェクト機構上に構築する次の規則である。

- 非再開型のFailure effect
- resultとの使い分け
- 後始末保証
- DefectとTerminal failureの分類
- 実行境界での最終処理


現行の設計文書でも、deep handler、one-shot continuation、operation探索、return clause、effect rowなどは既決定の基礎部分として整理され、例外・cleanup・fault分類が未決定事項として残されていた。

### 0. 設計原則
#### 0.1 失敗を一種類に統合しない

Reciplexaでは、すべての不成功を単一の「例外」として扱わない。

次の五種類を区別する。

1. option
   理由を必要としない通常の欠如

2. resultまたは専用data型
   理由を伴う通常の不成功

3. failure E
   現在の計算経路を中断する、型付きで回復可能な失敗

4. defect
   プログラムまたは処理系の論理的不変条件違反

5. terminal failure
   Runtime全体を安全に継続できない致命的障害

#### 0.2 判断基準
失敗を通常のデータとして扱う:
option／result／専用data型

処理境界まで非局所的に脱出する:
failure E

本来成立すべき内部前提が破られた:
defect

Runtimeの健全性を保証できない:
terminal failure

#### 0.3 公開APIと内部実装

resultとfailure Eの選択は、公開APIか内部実装かだけでは決めない。

次の意味上の違いによって判断する。

result:
失敗を値として返し、呼出し側が通常分岐として扱う

failure E:
現在の計算を中断し、外側の処理境界へ制御を移す


公開APIではresultを既定として推奨するが、処理全体の中断がAPIの本質である場合は、failure Eを公開してよい。

### 1. option
#### 1.1 用途

optionは、理由を伴わない通常の欠如に使う。

例：

(type find-node
  (fn document-snapshot node-id
    (option document-node)))


適する例：

- 検索対象が存在しない
- 任意propertyが未設定
- Cacheに値がない
- 最初の一致がない

#### 1.2 不適切な用途

利用者が失敗理由を必要とする場合、optionではなくresultまたは専用data型を使用する。

### 2. resultおよび専用結果型
#### 2.1 用途

resultは、失敗を通常値として保存・変換・分岐したい場合に使用する。

(type parse
  (fn str
    (result document parse-error)))


適する用途：

- Parsing
- Validation
- Dynamic cast
- Checked index access
- Checked arithmetic
- 複数errorの蓄積
- Retry候補の提示
- 利用者がその場で処理する不成功

#### 2.2 専用結果型

成功・失敗の二分だけでは表現不足の場合、専用data型を定義する。

編集トランザクションの結果は、その例である。

ApplyResult =
  Applied
  | AppliedNoChange
  | Rejected
  | AlreadyApplied


編集競合は通常運用で発生し得るため、failure EではなくApplyResult.Rejectedとして返す。

#### 2.3 複数errorの収集

最初の一件で中断せず複数の問題を集めたい場合、resultまたは専用validation型を使用する。

(type validate-document
  (fn document-snapshot
    (result validated-document
            (non-empty-list validation-error))))

### 3. failure E
#### 3.1 定義

failure Eは、型Eの値を伴って現在の計算経路を中断する、組込みの型parameter付きEffectである。

failure E:
E型のerrorによって計算を中断できるEffect


例：

(data decode-error
  invalid-header
  unexpected-end
  (invalid-byte int))

(type decode-document
  (fn bytes document
    (effects
      (failure decode-error))))

#### 3.2 正常経路と失敗経路
正常終了:
宣言された戻り値を返す

Failure:
戻り値を返さず、外側のFailure handlerへ制御を移す

#### 3.3 Error payload

Eは通常のRPX型である。

特別な例外基底classや、全errorを統合する動的な例外objectは導入しない。

Error型は通常のdata等で定義する。

### 4. Failureの発生
#### 4.1 raise

Failureを発生させる標準operationをraiseとする。

概念型：

(type raise
  (forall ((e type))
    (fn e never
      (effects
        (failure e)))))


使用例：

(raise
  (invalid-byte value))

#### 4.2 never

neverは値を一つも持たない空の型である。

raise:
正常経路では戻らない

戻り型:
never


neverは任意の型のsubtypeとして扱える。

never <: T


したがって、次の式全体はdocument型を持てる。

(if valid?
    document
    (raise invalid-header))


ただし、式全体のeffect rowにはfailure decode-errorが残る。

#### 4.3 基礎機構

raiseは独立した例外Runtimeを導入せず、既存のエフェクトoperation発生機構を使用する。

表面上は通常の適用に近いが、意味上は組込みの非再開型operationである。

### 5. Failure handler
#### 5.1 非再開性

Failure handlerはerror値だけを受け取る。

(handle
  (decode-document input)

  (failure error ->
    fallback-document))


一般の再開可能Effectのhandler節とは異なり、Failure handlerには継続変数を渡さない。

一般Operation:
引数とone-shot continuationを受け取る

Failure:
errorだけを受け取り、continuationを公開しない

#### 5.2 Resume禁止

Failure発生地点からの再開は禁止する。

再開可能な通知や問い合わせが必要な場合は、failure Eではなく通常のEffect operationを定義する。

#### 5.3 内部実装

Runtime内部では既存のエフェクトハンドラ機構へloweringできるが、Failureの継続は利用者コードへ公開しない。

#### 5.4 Handlerの結果型

return節を省略する場合、正常終了値とFailure節の結果は共通の結果型へ適合しなければならない。

(handle
  (decode-document input)

  (failure error ->
    fallback-document))

正常結果:
document

Failure節:
document

handle式:
document


異なる型へ変換する場合は、既存のreturn節を明示する。

(handle
  (decode-document input)

  (return document ->
    (ok document))

  (failure error ->
    (err error)))

### 6. Failureとeffect row
#### 6.1 型への明示

処理されていないFailureは、関数のeffect rowへ必ず現れなければならない。

(type load-document
  (fn package-resource document
    (effects
      resource
      (failure document-load-error))))


暗黙例外を認めない。

#### 6.2 Handlerによる除去

Failure handlerがfailure Eを処理した場合、対応するEffectを結果rowから除去する。

処理前:
{resource, failure decode-error}

処理:
failure decode-error

処理後:
{resource}

#### 6.3 Handler節自身のEffect

Handler節の評価中に発生したEffectは、handle式全体のeffect rowへ残る。

(handle
  computation

  (failure error ->
    (log-error error)))


log-errorがconsole Effectを持つ場合、handle式もconsole Effectを持つ。

#### 6.4 Handler節内の新しいFailure

Handler節内で新たに発生したFailureは、現在のhandlerではなく外側のhandlerへ伝播する。

handle対象内のfailure:
現在handlerが処理

handler節内の新failure:
外側handlerへ伝播


これにより、回復処理中の失敗が同じhandlerへ無限再入することを防ぐ。

### 7. 一つのFailure型への統合
#### 7.1 基本指針

一つの処理領域では、原則として一つの公開Failure payload型へ統合する。

次のような型を乱用しない。

(effects
  (failure decode-error)
  (failure resource-error)
  (failure permission-error))


代わりに上位error型を定義する。

(data document-load-error
  (decode decode-error)
  (resource resource-error)
  (permission permission-error))

(type load-document
  (fn package-resource document
    (effects
      (failure document-load-error))))

#### 7.2 Error変換

下位のFailureは、境界で上位error型へ明示的に変換する。

(handle
  (decode-image image-bytes)

  (failure error ->
    (raise
      (image-failure error))))

処理前:
failure image-error

処理後:
failure document-load-error

#### 7.3 位置付け

この指針は、一般EffectRowの多重labelやnamed instanceの能力を禁止するものではない。

Failure APIの可読性・型合成・ハンドラ設計を単純にするための規範的指針である。

### 8. resultとFailureの変換
#### 8.1 暗黙変換の禁止

次を自動変換しない。

resultのerr
→ failure

failure
→ resultのerr


制御フローを変更する変換は、コード上で明示する。

#### 8.2 resultからFailure

標準補助操作として、概念的なor-raiseを提供する。

(type or-raise
  (forall ((a type)
           (e type))
    (fn (result a e) a
      (effects
        (failure e)))))


意味：

ok value:
valueを返す

err error:
raise error

#### 8.3 Failureからresult

既存のhandlerとreturn clauseを使用する。

(handle
  computation

  (return value ->
    (ok value))

  (failure error ->
    (err error)))


高階APIとして提供する場合は、CBVによる事前評価を防ぐため無引数関数を受け取る。

概念型：

(type as-result
  (forall ((a type)
           (e type))
    (fn
      (fn unit a
        (effects
          (failure e)))
      (result a e))))

### 9. resultとFailureの選択指針
#### 9.1 resultを推奨する場合
- 呼出し側がその場で分岐する
- 失敗を保存・変換する
- 複数errorを集積する
- 検索またはvalidation
- Dynamic cast
- Checked arithmetic
- 編集競合
- 利用者が修正して再試行する通常結果

#### 9.2 Failureを認める場合
- 深い呼出し階層から脱出する
- 処理領域全体を中断する
- 中間関数がerrorを転送するだけになる
- 外側handlerで一括したpolicyを適用する
- Loader、decoder、job等の処理単位を失敗させる

#### 9.3 公開API

公開APIでもFailureを使用できる。

ただし、Failure型はeffect rowへ明示し、利用者がhandlerを設置できるようにする。

すべてのAPIについてresult版とFailure版の両方を自動的に提供する必要はない。

正準APIを一つ決め、実需がある場合のみ変換用wrapperを追加する。

### 10. Resource cleanup
#### 10.1 基本primitive

後始末の基礎primitiveとしてbracketを採用する。

公開形は高階関数に見えるAPIとし、内部ではRuntimeまたは標準handlerのprimitiveとして実装する。

概念例：

(bracket
  (fn ()
    (open-resource path))

  (fn (handle)
    (process-resource handle))

  (fn (handle)
    (close-resource handle)))

#### 10.2 役割
acquire:
Resourceを取得する

use:
Resourceを利用する

release:
Resourceを解放する

#### 10.3 特別な保証

通常の高階関数とは異なり、次の経路でreleaseを保証する。

- useの正常終了
- failureによる中断
- 継続の破棄
- 構造化されたcancellation
- Unwind可能なdefect

### 11. Acquire規則
#### 11.1 Release登録

releaseはacquireが正常値を返した直後に登録する。

#### 11.2 Acquire failure

acquireが正常値を返す前にFailureを発生させた場合、対応するreleaseは呼ばない。

acquire開始
↓
failure
↓
resource未取得
↓
releaseしない

#### 11.3 部分取得

Acquire内部で複数段階のresource取得が必要な場合、Acquire自身が内側のbracketを使用する。

外側bracketは部分取得状態を推測しない。

### 12. Useの正常終了

useが正常終了した場合、次の順序を保証する。

1. useが結果を生成
2. releaseを実行
3. releaseが成功
4. useの結果を外側へ返す


bracketが正常に戻った時点で、resourceの解放は完了している。

### 13. Use中のFailure

useがFailureを発生させた場合、次を行う。

1. 元のFailureを一時保持
2. releaseを実行
3. release成功後、元のFailureを再伝播


元のFailure発生地点から処理を再開しない。

### 14. 一般Effectと継続
#### 14.1 一時中断

useの途中で再開可能な一般Effectが発生しても、その時点ではreleaseしない。

一般Effectをperform
↓
Handlerへ制御移動
↓
継続はSuspended
↓
Resourceは保持

#### 14.2 Resume

Handlerが継続をresumeした場合、bracket scopeとresource lifetimeを維持したまま計算を続ける。

#### 14.3 Discard

Handlerが継続を再開せず破棄した場合、中断された計算のbracket scopeを終了し、内側から外側へreleaseする。

#### 14.4 継続状態

概念的な状態：

Suspended
Resumed
Discarded
Consumed


許可される遷移：

Suspended -> Resumed -> Consumed
Suspended -> Discarded -> Consumed


Consumed後の再resumeまたは再discardは禁止する。

#### 14.5 継続escape

v1では、one-shot continuationをhandler節の外へescapeさせない。

禁止：

- Recordへの保存
- Heap cellへの格納
- Closureへ捕捉して返す
- Handler節外での遅延resume


Handler節を抜けるまでに、継続はresumeまたはdiscardされなければならない。

これによりcleanup時点を構造的に決定する。

既存仕様により、より厳しい継続所有規則が存在する場合は、その規則を優先する。

### 15. Cleanup順序と回数
#### 15.1 LIFO

Nested bracketは、resource取得順の逆順にreleaseする。

取得:
A
B
C

解放:
C
B
A

#### 15.2 高々一回

各release actionは高々一回だけ実行する。

概念状態：

Registered
Running
Completed

Registered -> Running -> Completed


Completed後に再実行しない。

#### 15.3 一つのRelease失敗

内側のreleaseが失敗しても、残る外側のreleaseを続行する。

release Cが失敗
→ release Bを試行
→ release Aを試行

### 16. Cleanup中のFailure
#### 16.1 Primary failure

計算を最初に中断させたFailureをprimary failureとする。

#### 16.2 Suppressed failure

Primary failureの後、後始末中に追加で発生したFailureをsuppressed failureとして記録する。

例：

Use:
decode-error

Release:
close-error


結果：

Primary:
decode-error

Suppressed:
close-error

#### 16.3 正常終了後のRelease failure

Useが正常終了し、Releaseだけが失敗した場合：

Release failureがprimary failureになる
Useの正常結果は返さない

#### 16.4 複数のSuppressed failure

Nested cleanupで複数のreleaseが失敗した場合、実際のcleanup実行順で記録する。

Primary:
E0

Suppressed:
E1
E2

#### 16.5 通常Handlerへの公開

v1の通常Failure handlerには、primary error Eだけを渡す。

Suppressed failureは構造化されたRuntime診断metadataとして保持し、次から観測可能にする。

- 最上位診断
- Log
- Debugger
- Defect／failure report


通常の回復ロジックから異種のcleanup failureを精密に処理したいAPIは、releaseを明示的なresultとして扱う別APIを提供できる。

### 17. finally

一般的な後始末用にfinally相当の補助APIを提供できる。

ただし、独立したCore primitiveにはせず、値を取得しないbracketとして構築する。

bracket:
取得したresourceをreleaseへ渡す

finally:
取得値のない後始末


Coreの後始末primitiveはbracket系へ集約する。

### 18. Cancellation
#### 18.1 分類

Cancellationは通常Failure、Defect、Terminal failureのいずれとも同一視しない。

Cancellation:
外部要求による構造化された計算中断

#### 18.2 Cleanup

Cancellationではbracket cleanupを実行する。

#### 18.3 詳細

Cancellationを一般Effect、専用signal、またはConcurrency Runtimeの操作として表すかはOPEN-CON-001へ移管する。

本項目では次だけを確定する。

- CancellationはDefectではない
- CancellationはTerminal failureではない
- Cancellationは構造化unwindを実行する
- Releaseを保証する

### 19. Defect
#### 19.1 定義

Defectは、本来成立すべきプログラム上または処理系上の論理的不変条件が破られた状態である。

例：

- Internal assertion違反
- 網羅的matchの実行時不一致
- 証明済みcastの失敗
- 証明済みindex条件の違反
- One-shot continuationの二重resume
- Validator自身の契約違反
- 型検査済みIRの内部矛盾
- 所有treeの不変条件破壊

#### 19.2 Effect row

Defectは通常のeffect rowへ現れない。

Defectを通常のFailure代わりに使用してはならない。

#### 19.3 再開

Defect発生地点からのresumeは禁止する。

#### 19.4 通常Handler

通常のhandleではDefectを捕捉できない。

#### 19.5 Cleanup

Runtimeとcleanup stackが健全であることを確認できるDefectでは、構造化unwindとbracket cleanupを行う。

健全性を確認できない場合はTerminal failureへ昇格する。

### 20. Fault boundary
#### 20.1 定義

Fault boundaryは、Defectの影響を一つの計算単位へ隔離する、Runtimeまたはtrusted hostが設置する境界である。

適用例：

- Entry point
- GUI command
- Render job
- Server request
- Worker
- Plugin invocation
- Test case

#### 20.2 一般公開

通常の利用者コードが任意位置へFault boundaryを設置し、Defectを通常errorとして扱う一般APIは提供しない。

#### 20.3 処理

Fault boundaryはDefect発生時に次を行う。

1. 中断された継続を破棄
2. Unwind可能ならcleanupを実行
3. 未commitの作業状態を破棄
4. DefectReportを生成
5. 該当jobをDefectedとして終了
6. 健全な外側Runtimeへ制御を戻す

#### 20.4 継続条件

外側処理を継続できるのは、少なくとも次を保証できる場合だけである。

- Runtime内部構造が健全
- Cleanup stackが健全
- 共有stateへの部分commitがない
- Memory safetyが破られていない
- Defect発生jobのresourceを隔離できる


保証できない場合はTerminal failureとする。

### 21. Terminal failure
#### 21.1 定義

Terminal failureは、Runtimeまたはprocessを安全に継続できない障害である。

例：

- Runtime memory構造の破損
- Stack／continuation表現の破損
- Cleanup stackの破損
- Kernelの致命的不変条件破壊
- 安全な回復不能のOut-of-memory
- Processの強制終了
- 電源断

#### 21.2 通常Handler

通常HandlerおよびFault boundaryから回復できない。

#### 21.3 Cleanup

完全なcleanupを保証しない。

#### 21.4 可能な最小処理

実装が安全に実行できる場合のみ、次を行う。

- 事前確保済み領域による最小診断
- Host／OSへの終了通知
- Watchdogへの報告
- ProcessまたはRuntime instanceの停止


通常の利用者定義formatting、文書保存、任意cleanup、通常JobResultの返却は保証しない。

### 22. 個別事例の分類
#### 22.1 Assertion
利用者入力の検査:
result／failure

内部不変条件の検査:
違反時はDefect


Assertionを公開入力validationの代替として使用しない。

#### 22.2 Match
静的に網羅的と証明されたmatchの不一致:
Defect

意図的な部分match:
default節または明示的なresult／failureを要求

#### 22.3 Dynamic cast
通常のcast不一致:
option／result

成功すると静的に保証された内部castの失敗:
Defect

#### 22.4 Index access
一般のchecked access:
option／result

内部で範囲内と保証されたaccessの違反:
Defect

#### 22.5 Arithmetic overflow
数学的int:
Overflowなし

固定幅checked演算:
result

固定幅wrap演算:
型の通常意味としてwrap

証明済み範囲条件の違反:
Defect


Build modeによってoverflow意味論を変えない。

#### 22.6 Division by zero
一般除算:
resultまたは明示Failure

non-zero型を受け取る除算:
正常値

non-zero保証の破壊:
Defect

#### 22.7 Continuationの二重resume
Defect


Runtime内部構造の健全性を失った場合はTerminal failureへ昇格する。

#### 22.8 Validator
外部入力が検証不合格:
result／failure

Validator自身の契約違反:
Defect

#### 22.9 Foreign adapter
安全に検出できる契約違反:
Defectとしてadapter jobを停止

Memory safetyまたはRuntime整合性の破壊:
Terminal failure

#### 22.10 Resource exhaustion
局所的quota不足:
failure resource-error

File descriptorやGPU buffer不足:
failure resource-error

一般heapの回復不能OOM:
Terminal failureを許容

### 23. DefectReport
#### 23.1 内容

Fault boundaryは構造化されたDefectReportを生成する。

概念的な内容：

DefectReport {
  kind,
  safe-message,
  source-origin,
  stack-trace,
  effect-handler-trace,
  transaction-id?,
  document-id?,
  suppressed-cleanup-failures,
  runtime-version
}

#### 23.2 安全性

Defect report生成は、利用者定義の一般showや複雑なformattingへ依存しない。

通常は次の最小情報を使用する。

- Defect kind
- 固定または検証済みmessage
- 正規化されたsource identity
- 制限されたstack情報


値の詳細dumpはdebug policyで明示的に有効化する。

#### 23.3 権限・機密性

Defect reportには機密情報が含まれ得るため、出力policyを実行環境が管理する。

### 24. ジョブ結果

Fault boundaryを持つ実行単位では、結果を次のように分類できる。

JobResult<A, E> =
  Completed(A)
  | Failed(E)
  | Cancelled
  | Defected(DefectReport)


意味：

Completed:
正常完了

Failed:
型付きの予想可能な失敗

Cancelled:
外部要求による中止

Defected:
プログラム上の欠陥


Terminal failureでは、通常JobResultを返すところまで到達しない。

### 25. Entry pointと実行環境
#### 25.1 Runtime capability

Entry pointのrequired effect rowは、実行環境が提供するEffectの部分集合でなければならない。

RequiredEffects(main)
⊆
ProvidedEffects(entry-environment)


例：

CLI environment:
console
resource
storage
clock
failure sink

GUI environment:
window
input
render
resource
storage
failure sink

Server environment:
network
storage
clock
request failure sink


未提供Effectが残る場合は静的エラーとする。

entry point requires an unsupported effect:
  window

#### 25.2 実行環境ごとのmain

すべてのEntry pointへ一種類のmain型を強制しない。

CLI entry:
exit statusを返し得る

GUI entry:
application lifecycleを開始する

Server entry:
serverまたはserviceを開始する

Worker entry:
job loopを開始する


最上位Failure処理は、各Entry environmentの契約に従う。

### 26. 未処理Failure
#### 26.1 原則

Application固有のFailureは、Application境界で処理することを推奨する。

- 利用者向けdiagnostic
- Retry policy
- Exit status
- GUI dialog
- HTTP response

#### 26.2 最終防御

Entry environmentが対応するFailure sinkを提供する場合、mainのeffect rowに未処理failure Eを残せる。

Runtimeは最終防御として次を行う。

- 安全な最小診断
- Entryまたはjobを失敗扱いにする
- Cleanup完了を待つ
- Entry environmentに対応する終了結果へ変換

#### 26.3 Runtime default表示

任意のError型へ複雑な自動文字列化を要求しない。

Runtime default handlerは、可能な範囲で次だけを使用する。

- Error型の正式identity
- 安全なconstructor名
- Source位置
- Effect trace


詳細な利用者向け説明は、Error型ごとの明示的な変換関数が提供する。

(type explain-document-error
  (fn document-error diagnostic))

### 27. 実行環境別の処理
#### 27.1 CLI
正常終了:
0相当

Application failure:
Applicationが明示した非ゼロstatus

未処理Failure:
Runtime標準の非ゼロstatus

Defect:
Failureとは区別された異常終了status

Cancellation:
Host規約または専用status

Terminal failure:
Runtime／OS依存の異常終了


具体的な整数値はtoolchainまたはhost規約へ移管する。

#### 27.2 GUI
- Command単位でFailureを通常結果へ変換
- Render job単位でFault boundaryを設置
- Plugin invocation単位でFault boundaryを設置
- 一操作の失敗でevent loop全体を終了しない


Runtime最終handlerは、処理漏れによるApplication全体の破損を防ぐ最後の境界とする。

#### 27.3 Server
- Request単位でFailureをresponseへ変換
- Request単位でDefectを隔離
- 他requestは健全なら継続
- Terminal failureではprocessを停止

#### 27.4 Plugin
- Plugin invocation単位でFault boundary
- 未commit transactionを破棄
- Plugin所有resourceをcleanup
- DefectReportを生成
- Hostが健全なら継続

#### 27.5 Render job

不変スナップショットを入力とし、出力をcommit前の一時成果物として扱う。

Render defect:
成果物を破棄
DefectReportを返す
Editor本体は健全なら継続

### 28. Diagnostic
#### 28.1 構築と出力の分離
Diagnostic construction:
構造化されたdiagnostic値を生成

Diagnostic sink:
表示・保存・送信先を決定


Sinkの例：

- Terminal stderr
- GUI notification
- Log file
- Test reporter
- HTTP response
- Telemetry

#### 28.2 PrimaryとSuppressed

Primary failureを最初に表示し、suppressed cleanup failureを追加情報として表示する。

例：

Document load failed:
  invalid document header

Additional failure during cleanup:
  failed to close resource

#### 28.3 Libraryの責務

Libraryは最終的な出力先や終了方法を決めない。

Errorの意味と、必要に応じてDiagnosticへの変換関数を提供する。

### 29. 適合試験
ERR-01：通常のresult

Dynamic castが不一致の場合：

result／optionとして返す
Defectにしない

ERR-02：Failureの型
(type decode
  (fn bytes document
    (effects
      (failure decode-error))))


Handlerなしで呼び出す関数は、同じFailureをeffect rowへ含めなければならない。

ERR-03：Failure handler
(handle
  (decode bytes)

  (failure error ->
    fallback))


期待結果：

- Failure節へ継続を渡さない
- failure decode-errorを結果rowから除去

ERR-04：Failureからresult
(handle
  computation

  (return value ->
    (ok value))

  (failure error ->
    (err error)))


期待結果：

result value-type error-type

ERR-05：Acquire failure

acquireが失敗した場合：

releaseを実行しない

ERR-06：Use failure

useがFailureを発生した場合：

- releaseを一回実行
- release成功後に元Failureを再伝播

ERR-07：Effectの一時中断

一般Effect発生後、継続がresumeされた場合：

resourceをreleaseせず、
bracket scopeを維持して続行

ERR-08：継続破棄

一般Effectの継続がdiscardされた場合：

継続内のbracketを内側から外側へrelease

ERR-09：Nested cleanup

取得順：

A
B
C


解放順：

C
B
A

ERR-10：Cleanup failure
Use:
E0

Release B:
E1

Release A:
E2


期待結果：

Primary:
E0

Suppressed:
E1, E2

Release Aまで実行

ERR-11：正常終了後のRelease failure
Use:
正常値R

Release:
E


期待結果：

EをPrimary failureとして伝播
Rは返さない

ERR-12：通常cast不一致
result／option

ERR-13：証明済みcastの失敗
Defect

ERR-14：One-shot継続の二重resume
Defect


Runtime健全性を保証できない場合：

Terminal failureへ昇格

ERR-15：Fault boundary

Plugin内でUnwind可能なDefectが発生：

- Plugin計算を中止
- Cleanupを実行
- 未commit状態を破棄
- DefectReportを返す
- Hostは健全なら継続

ERR-16：Terminal failure

Runtime memory構造が破損：

- 通常handlerで回復しない
- Cleanupを保証しない
- 最小診断後にRuntimeを停止

ERR-17：Entry effect検査

CLI Entry pointがwindow Effectを要求し、CLI環境が提供しない場合：

static error:
entry point requires an unsupported effect

ERR-18：編集競合

TransactionがPropertyValueMismatchで拒否された場合：

ApplyResult.Rejectedとして返す
failure effectにしない

ERR-19：Storage障害

Transaction commit中に保存装置へ接続できない場合：

ApplyResult.Rejectedではなく、
failure document-service-errorとして扱う

### 30. 移管先OPEN

#### `OPEN-MEM-001`

- bracketのRuntime表現
- Cleanup stack
- SnapshotHandleのlifetime
- Resource handleの所有権
- Continuationとresource lifetime
- Releaseのcleanup-safe Effect

#### `OPEN-CON-001`

- Cancellationの表現
- Structured concurrency
- Task境界
- Commit queue
- Continuationの非同期利用
- Fault boundaryとworker

#### `OPEN-KER-001`

- Trusted adapterの契約
- Validator boundary
- ForeignValue
- Adapter defectとterminal failureの境界
- Kernel crash containment

#### `OPEN-TST-001`

- Test caseごとのFault boundary
- Failureの期待
- Defectの期待
- Cleanup検査
- Entry environmentのtest double

#### `OPEN-PKG-ENTRY-001`

- CLI／GUI／Server／Worker entry profile
- mainの正確な型
- Exit status
- Entry environmentのProvidedEffects

#### `OPEN-ERR-DIAG-001`

- Diagnostic schema
- Source trace
- Effect trace
- Primary／suppressed表示
- Privacy policy

### 31. 最終状態

```text
OPEN-ERR-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- option／result／Failure／Defect／Terminal failureの明確な分類
- Effect rowへ現れる型付きFailure
- 再開不能なFailure handler
- resultとFailureの明示的変換
- 暗黙例外を持たない失敗型
- bracketによるResource cleanup
- 継続破棄時の後始末
- Nested cleanupのLIFO保証
- Primary／suppressed failure
- 通常Handlerから分離されたDefect
- Runtime管理のFault boundary
- Terminal failureの明確な限界
- Entry environmentによるEffect capability検査
- CLI／GUI／Server／Plugin／Render jobごとの障害隔離
- Diagnostic構築と出力先の分離

## OPEN-MEM-001 Perceusメモリ管理・スコープ付きリソース・継続・メモリ予算
### DD-001 決定概要
#### DD-001.1 状態
Status:
RESOLVED

Scope:
通常値の自動メモリ管理
Perceus方式の精密参照カウント
dup／drop／reuse
Closure環境
One-shot continuation
Failure unwind
Resource handle
bracketとの責務分離
メモリ予算
Allocation failure
Reference count overflow
GC時点の観測可能性

#### DD-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- 通常値は原則として不変
- valは不変binding
- varはescape不能な局所可変状態
- varの物理的な配置方法は観測不能
- 一般的なescape可能cell／refは未導入
- Effect handlerはdeep handler
- Continuationはone-shot
- Failureは再開不能
- bracketは正常終了・Failure・Cancellation・継続破棄時にcleanupする
- 文書Snapshotは不変値
- 文書の所有構造はtree


旧版の設計記録では、varのescape禁止は決定済みである一方、GC方式、closure環境、continuation、resource lifetime、escape可能なcell／ref、循環値、GUI再評価時の状態寿命などがOPEN-MEM-001へ残されていた。

#### DD-001.3 中心的な決定
- 通常値の自動メモリ管理にはPerceus方式を採用する
- Perceusは精密参照カウントとreuse解析を行う
- Effect／handler／bracketを明示的制御フローへloweringした後に適用する
- 利用者へdup、drop、reuse、参照カウントを公開しない
- 一般heap上の強参照cycleはv1では構築不能とする
- 一般的なescape可能cell／refはv1では導入しない
- varはescape不能な局所状態のままとする
- 外部resourceの意味的解放はPerceusではなくbracketが担当する
- Scoped resource handleはbracket外へescapeできない
- 一般的なborrow system、weak reference、利用者定義finalizerはv1では導入しない
- 通常allocationはeffect rowへ現さない
- 明示的な予算超過はfailure resource-exhaustedとする
- 回復不能な一般heap OOMはterminal failureを許容する

### 0. 用語
#### 0.1 Perceus

Perceusは、関数型Coreに対して精密な参照カウント命令を挿入し、参照の一意性を利用してメモリ領域の再利用を行うコンパイル方式である。

代表的な内部操作は次である。

dup:
参照を共有する

drop:
不要になった所有参照を解放する

reuse:
一意な不要領域を新しい値の構築へ再利用する


Perceusは、循環のないプログラムにおいて不要参照を保持しないことを目標とし、参照が一意なら不変データの領域を再利用して、純粋な関数型コードを内部的にin-place実行できる。

#### 0.2 自動メモリ管理

本仕様における自動メモリ管理は、Tracing GCではなく、Perceus方式の精密参照カウントを中心とする。

自動メモリ管理:
Perceus方式

Tracing GC:
標準方式として採用しない

明示的free:
利用者へ提供しない

#### 0.3 Resource

Resourceは、単なるheap上の値ではなく、意味的な取得・利用・解放時点を持つ外部対象である。

例：

- File handle
- Socket
- GPU buffer
- Window
- Database transaction
- Foreign library object
- OS process
- Device context

### 1. メモリとResourceの分離
#### 1.1 通常値

次はPerceusによる自動メモリ管理の対象である。

- data値
- record
- list
- str
- bytes
- Closure環境
- Handler環境
- Continuationの内部表現
- DocumentSnapshot
- Provenance DAG
- Resource handleのRPX側wrapper

#### 1.2 外部Resource

外部Resource本体の解放時点は、参照カウントが0になった時点ではなく、bracketのscope終了によって決める。

Perceus:
RPX値のmemoryを管理する

bracket:
外部Resourceの意味的lifetimeを管理する

#### 1.3 基本原則
Memory reclamation:
Perceus

Semantic resource cleanup:
bracket


File handleのwrapperが最後にdropされたことを、File closeの正規の意味的契機とはしない。

### 2. Perceusによる自動メモリ管理
#### 2.1 利用者から見える意味

通常のRPX値について、次を保証する。

- 利用者は明示的にfreeしない
- 到達不能または所有権が消費された値は自動的に解放可能
- 参照カウント値を観測できない
- 物理addressを観測できない
- 共有の有無を観測できない
- reuseの成立・不成立を観測できない
- 物理的なin-place更新によって意味が変わらない

#### 2.2 回収時点

Perceusは正確な最終使用位置でdropを挿入することを目標とする。

ただし、利用者が特定値の物理的な解放時点へ依存するAPIは提供しない。

意味上:
不要参照を保持しない

利用者API:
解放時点への直接依存を許さない

#### 2.3 物理identity

通常の不変値にpointer identityを公開しない。

同じ内容のrecord:
同じallocationかどうかを観測不能

同じlist:
構造共有されているか観測不能

同じstr:
internされているか観測不能


継続identityが必要な領域では、専用のopaque IDを使う。

- DocumentId
- NodeId
- TransactionId
- Resource固有のidentity

### 3. Compilation pipeline
#### 3.1 適用順序

Perceusは、Effectと制御移動を明示化した後のCore IRへ適用する。

1. Source解析
2. Macro展開
3. 名前解決
4. 型・Effect検査
5. 高水準構文のdesugar
6. Effect handler／Failure／bracketのlowering
7. Closure conversion
8. 明示的制御フローCoreの生成
9. 最終使用・共有解析
10. dup／drop挿入
11. Ownership verifier
12. Reuse解析
13. Reuse特殊化
14. Reuse verifier
15. Backend lowering


Kokaでも、Effect等を明示的な制御フローを持つ内部Coreへ変換した後にPerceusを適用する方式が用いられている。

#### 3.2 Surface所有権注釈

通常のRPXソースへ次の所有権注釈を要求しない。

owned
borrowed
shared
consume
dup
drop


通常利用者は、不変値と第一級関数を通常どおり使用する。

所有権はコンパイラが次から推論する。

- 変数の使用回数
- 最終使用位置
- 制御フロー
- Closure capture
- Continuation capture
- Effect lowering結果
- Foreign interface metadata

#### 3.3 Trusted boundary

所有権情報を明示的に記述できるのは、主に次のtrusted metadataである。

- Kernel primitive
- Foreign adapter
- Resource API
- Backend ABI

### 4. 所有権Core IR
#### 4.1 必須のCore要素

Perceus適用前のCoreは、少なくとも次を明示する。

Let
Call
TailCall
Construct
Deconstruct
Match
Closure
Branch
Join
Return
Continuation capture
Resume
Discard
Handler frame
RegisterCleanup
RunCleanup
Raise
ForeignCall


Perceus適用後は、さらに次を含む。

Dup
Drop
ReuseCandidate
ConstructReuse

#### 4.2 評価順序

既存仕様のCBVかつ左から右の評価順序を維持する。

Perceusによるdropやreuseの挿入は、利用者から観測できる評価順序およびEffect順序を変更してはならない。

### 5. dup
#### 5.1 意味

同じ値を複数の生存経路から使用する必要がある場合、コンパイラはdupを挿入する。

dup x


参照カウントを増加させ、複数の所有参照を成立させる。

#### 5.2 挿入条件

構文上の複数利用ごとに機械的に挿入するのではなく、所有権・borrow・最終使用解析に基づいて必要な場合だけ挿入する。

#### 5.3 利用者からの不可視性

dupの有無は利用者から観測できない。

参照カウント増加を明示的なEffectとして扱わない。

### 6. drop
#### 6.1 意味

値が以後どの制御経路からも使用されない地点で、コンパイラはdropを挿入する。

drop x


参照カウントが0になった場合、次を行う。

1. 値が保持するfieldをdrop
2. Closureなら捕捉環境をdrop
3. Constructor領域を解放またはreuse候補化

#### 6.2 最終使用位置

dropは字句scope末尾ではなく、正確な最終使用位置へ挿入できる。

(local
  (val large-value
    (build-large-value))

  (process large-value)

  (unrelated-work))


概念的には次となる。

build large-value
process large-value
drop large-value
unrelated-work

#### 6.3 制御フロー

各制御経路で、所有参照は次のいずれかを満たさなければならない。

- 別の処理へ移送される
- 最終的にdropされる

### 7. 分岐とJoin point
#### 7.1 排他的分岐

排他的分岐では、実際に通る経路が一つであるため、各branchへ所有権を移送できる。

condition
├─ true  -> xを左branchへ移送
└─ false -> xを右branchへ移送


分岐前に常にdupする必要はない。

#### 7.2 Join時の整合

Join pointへ到達する全経路は、各値について整合する所有状態を提供しなければならない。

適合：

Branch A:
owned x

Branch B:
owned x

Join:
owned x


適合：

Branch A:
xをdrop

Branch B:
xをdrop

Join:
xなし


所有状態が一致しない場合、コンパイラが必要なdup・dropを挿入するか、内部IR不正として拒否する。

### 8. Reuse
#### 8.1 位置付け

reuseは、参照カウントの正しさに必要な機構ではなく、意味保存最適化である。

dup／drop:
正しさに必要

reuse:
性能最適化

#### 8.2 一意性

不要になったconstructorが一意参照されている場合、その領域を同等のsize classを持つ新constructorへ再利用できる。

共有されている場合は通常allocationへfallbackする。

Perceusのreuse解析は、不変データの一意性を利用し、純粋な関数型プログラムを内部的にin-place実行できるようにする。

#### 8.3 Reuse不成立

Reuse候補が共有されていた場合：

- Failureにしない
- Defectにしない
- 通常の新規allocationへfallbackする

#### 8.4 非保証

次を言語仕様として保証しない。

- 特定の値が必ずin-place更新される
- 特定のallocationが必ず省略される
- Compiler versionを越えて同じreuse判断になる
- Sourceの小変更後も同じreuse判断になる

#### 8.5 Reuse禁止値

次は原則としてreuse対象外とする。

- Foreign resource wrapper
- Pinned memory
- 外部APIへaddressを渡した値
- Foreign-owned値
- Runtime metadata
- Continuation stack segment
- Cleanup実行中のframe
- ABI上固定されたmemory


概念的な分類：

ReuseClass =
  Reusable
  | NonReusable
  | Pinned
  | ForeignOwned

### 9. Closure環境
#### 9.1 表現

Closureは、codeと捕捉環境を持つ値として扱う。

Closure {
  code,
  environment
}

#### 9.2 生成

Closure生成時：

- 捕捉値をClosure環境へ移送する
- 外側でも必要な値にはdupを挿入する

#### 9.3 解放

Closureが不要になった場合：

- 捕捉環境内の値をdrop
- Closure領域を解放

#### 9.4 Closure identity

Closureの物理identityや等値比較を一般公開しない。

Closure equality:
提供しない

Closure address:
公開しない

#### 9.5 Scoped値のcapture

varまたはscoped resourceを捕捉するClosureは、そのscope内でのみ使用できる。

そのClosureがscope外へescapeする場合は静的エラーとする。

### 10. var
#### 10.1 既存意味論

varはescape不能な局所可変状態である。

- 変数scope外へ直接返せない
- EscapeするClosureへ捕捉できない
- Module stateへ保存できない
- 一般共有heap stateにならない

#### 10.2 物理表現

利用者から見える意味が同じなら、実装は次を選べる。

- Stack slot
- Handler-local state
- Mutable frame
- SSA変換
- Heap上の局所slot

#### 10.3 Perceusとの関係

通常の不変値とClosure環境はPerceusで管理する。

varの局所storageは、一般の共有参照カウント付きobjectである必要はない。

### 11. cell／ref
#### 11.1 v1の方針

一般的なescape可能cell／refはv1では導入しない。

- 共有可変heap stateなし
- Stateful closureの一般機構なし
- 任意の循環参照構築なし

#### 11.2 理由

一般cell／refは次を要求する。

- Heap identity
- Alias規則
- 共有可変状態
- Concurrency memory model
- Race規則
- Cycle処理
- Serialization規則
- GUI再評価時のidentity


Perceusの基本的なgarbage-free保証はcycle-freeなプログラムを前提とし、mutationで循環参照が作られる場合は別の処理が必要になる。

#### 11.3 将来拡張

将来導入する場合はOPEN-MEM-CELL-001で次を検討する。

- Cycleの静的禁止
- Region制約
- Weak reference
- Cycle collector
- Ownership制約
- Handler-local state限定
- 明示的cycle分断

### 12. 再帰型と循環値
#### 12.1 再帰data型

再帰data型は許可する。

(data tree
  leaf
  (branch tree tree))

#### 12.2 循環する実行時値

通常の不変値から、直接自己参照するheap cycleを構築する機能はv1では提供しない。

再帰型:
許可

有限の不変再帰値:
許可

一般heap上の強参照cycle:
v1では構築不能

#### 12.3 論理的なID参照

NodeId等のopaque IDによる論理参照cycleは、heap object間の強いruntime pointer cycleとは区別する。

文書の所有edgeはtreeであり、非所有参照のcycle可否は各schemaで規定する。

### 13. Continuation
#### 13.1 表現

One-shot continuationは内部的な所有値として表す。

概念構造：

Continuation {
  stack-segment,
  captured-values,
  handler-frames,
  cleanup-frames
}

#### 13.2 Capture

Continuation生成時：

- 再開に必要な値をContinuationへ移送
- Handler側でも必要な値にdupを挿入
- Cleanup frameをContinuationへ関連付ける

#### 13.3 Resume

Resume時：

- Continuationの所有権を再開先へ移送
- Captured valuesを復元
- Handler／cleanup frameを再接続
- Continuationを消費

#### 13.4 Discard

Discard時：

1. Cleanup frameをLIFOで実行
2. Captured valuesをdrop
3. Continuation objectを解放

#### 13.5 One-shot

Continuationは高々一回だけ消費できる。

Suspended -> Resumed
Suspended -> Discarded


二重resumeまたは二重discardはDefectである。

Runtimeの所有状態が破損している可能性がある場合はTerminal failureへ昇格する。

#### 13.6 Escape

v1ではContinuationをhandler節外へescapeさせない。

Handler節の終了時までに、Continuationはresumeまたはdiscardされなければならない。

未消費ならRuntimeがdiscardする。

### 14. Failure unwind
#### 14.1 明示的なunwind

raiseは通常の戻り経路を持たない。

Failure loweringでは次を明示する。

- どのframeを離れるか
- どのlocal値をdropするか
- どのcleanupを実行するか
- Error値をどのhandlerへ移送するか

#### 14.2 Dropの欠落禁止

Failure handlerへ直接jumpするだけではならない。

離脱する全frameについて、必要なdropとcleanupを行う。

#### 14.3 Handler節

Handler節では少なくとも次が生存する。

- Operation引数
- One-shot continuation
- Handlerの捕捉環境


Handler節のすべての制御経路で、Continuationが確実に消費されなければならない。

### 15. Scoped Resource handle
#### 15.1 隠れたscope

bracketは呼出しごとにfreshなresource scopesを生成する。

概念的なhandle型：

resource-handle<s, R>

s:
Resourceの有効scope

R:
File、Socket、GPU buffer等のResource種別


利用者へsを直接記述させない。

#### 15.2 bracketの概念型
bracket:
  acquire : unit -> resource<s, R>
  use     : resource<s, R> -> A
  release : resource<s, R> -> unit
  result  : A


制約：

freshなsは、bracketの結果Aへ現れてはならない


これは内部的にはrank-2相当のscope生成規則として扱える。

#### 15.3 Escape禁止

次を禁止する。

- Handleをbracketの結果として返す
- Handleを含むrecordやdataを返す
- Handleをlistやoptionへ包んで返す
- Handleをdynamicへ封入して逃がす
- 外側stateへ格納する
- Module-level bindingへ格納する
- EscapeするClosureへ捕捉する
- Effect operationのpayloadとして外側handlerへ送る
- 別taskへ送る

#### 15.4 Scope内Closure

Scoped handleをClosureへ捕捉すること自体は許可する。

ただし、そのClosureをresource scope外へescapeさせてはならない。

#### 15.5 独立した結果値

Resourceから生成された値がResource scopeへ依存しない場合は、外へ返せる。

例：

(with-file path
  (fn (file)
    (read-all file)))


read-allが独立した所有bytesを返すなら、結果はscope外で有効である。

### 16. Resource API
#### 16.1 With-style API

通常利用者向けには、Resource種別ごとのwith-style APIを優先する。

(with-file path
  (fn (file)
    (read-all file)))

#### 16.2 低水準API

Trusted・高度APIとして、汎用bracket、acquire、release pairを提供できる。

通常利用者に手動closeを要求するAPIを標準形としない。

#### 16.3 Release責任
Handle wrapper memory:
Perceus

External resource release:
bracket cleanup frame

#### 16.4 自発的な無効化

Scope内でも、外部要因でResourceが使用不能になることはある。

- Network切断
- GPU device loss
- 外部process終了


Resource操作は必要に応じてfailure resource-errorを持つ。

Scope型はuse-after-releaseを防止するが、外部障害まで排除しない。

### 17. Borrowed view
#### 17.1 所有値とBorrowed view

Resourceからcopy・decodeされた独立所有値はscope外へ返せる。

Resource内部memoryを直接参照するviewは、Resource scopeへ依存する。

概念型：

borrowed-bytes<s>

#### 17.2 v1の方針

一般利用者向けのborrow systemはv1では導入しない。

- Read borrow
- Mutable borrow
- Alias検査
- Borrowed Closure


は将来項目へ移管する。

Zero-copyが必要なKernel内部APIでは、trustedなscoped viewを使用できる。

### 18. Weak referenceとFinalizer
#### 18.1 Weak reference

一般利用者向けWeak referenceはv1では提供しない。

理由：

- 値の消滅がメモリ管理時点へ依存する
- プログラム挙動がPerceus内部判断へ依存する
- 将来cell／cycleと複雑に相互作用する


Runtime内部cacheで使用することはできるが、RPXプログラムの意味を変えてはならない。

#### 18.2 利用者定義Finalizer

利用者が任意コードをdrop時に実行するFinalizerは提供しない。

- 実行順序
- Failure
- Resource復活
- Cycle
- Foreign ownership


が複雑になるためである。

#### 18.3 Resource安全網

RuntimeがResource wrapperの最終drop時に安全網としてreleaseを試行することは許可する。

ただし、正しいプログラムはそれへ依存してはならない。

必須解放はbracketで行う。

### 19. Foreign boundary
#### 19.1 Ownership metadata

Foreign callの各引数は、trusted metadataによって少なくとも次に分類する。

Borrowed:
呼出し中だけ参照し、外部側は保持しない

Owned:
外部側へ所有権を移す

Shared:
外部側が呼出し後も参照を保持する

Copied:
外部表現へ複製する


戻り値も次のように分類する。

- Owned result
- Scoped borrowed view
- Foreign resource handle

#### 19.2 Borrowed契約違反

Foreign側がBorrowed値を呼出し後も保持した場合：

安全に検出可能:
Defect

Memory safetyを破壊:
Terminal failure

#### 19.3 Owned移送

Owned引数を外部側へ移した後、RPX側はその値をdropしてはならない。

外部側が最終的な解放責任を負う。

詳細はOPEN-KER-001へ移管する。

### 20. Concurrencyへの接続
#### 20.1 v1の範囲

通常のPerceus参照カウントは、単一thread内では非atomic操作を使用できる。

すべての値へ最初からatomic reference countを要求しない。

#### 20.2 共有値

将来、thread間共有を導入する場合は次が必要になる。

- Send規則
- Share規則
- Thread共有可能性
- Atomic reference count
- Shared値のreuse禁止または制約


Perceus関連の実装・研究でも、thread共有される値と共有されない値を区別し、必要な場合だけatomic参照カウントを使うことが重要になる。

詳細はOPEN-CON-001へ移管する。

#### 20.3 Scoped Resource

Scoped resource handleを別taskへ送信できないものとする。

Task共有可能なResourceは、専用のmanaged service handleとして別途設計する。

### 21. SnapshotとPerceus
#### 21.1 構造共有

不変DocumentSnapshotはrevision間で内部構造を共有できる。

Revision 10:
Root -> A, B, C

Revision 11:
Root -> A, B', C


AとCは複数snapshotから参照される。

#### 21.2 解放

古いsnapshotが不要になると、そのrootをdropする。

他snapshotから共有されている部分は参照カウントが残るため保持される。

#### 21.3 一意性

共有が解消され一意になった構造は、将来の更新処理でreuse候補になり得る。

#### 21.4 観測不能

Snapshotがどの程度構造共有されているかは利用者から観測できない。

### 22. メモリ割当とEffect
#### 22.1 通常Allocation

通常heap allocationをeffect rowへ記録しない。

Construct
Closure生成
String生成
Snapshot更新
Continuation生成


は、通常の計算意味論を実現するRuntime動作である。

#### 22.2 Perceus操作

次もeffect rowへ現れない。

- Reference count更新
- dup
- drop
- reuse
- Allocationからreuseへの置換

#### 22.3 理由

通常allocationをEffectにすると、ほぼすべての関数がallocation Effectを持ち、Effect rowの情報価値が低下する。

不変値の生成は、外部状態を変更する意味的Effectとは区別する。

### 23. メモリ予算
#### 23.1 適用単位

明示的なメモリ予算を、隔離可能な実行単位へ設定できる。

例：

- Render job
- Import job
- Export job
- Plugin invocation
- Server request
- Test case
- Document operation
- Worker

#### 23.2 Job増分方式

v1では、予算課金にjob増分方式を採用する。

Job開始前から存在する共有入力:
原則として課金しない

Jobが新規に割り当てた値:
Job予算へ課金

Job内で解放された値:
課金を戻す

Job結果としてcommitされた値:
永続所有領域へ課金を移管


この値は厳密なprocess heap使用量ではなく、そのJobが生み出した増分量を表す。

#### 23.3 Commit時の移管

Jobが生成した値をDocumentやCacheへcommitするとき、課金を移管する。

Commit前:
Job budget

Commit成功:
Document／cache budget

Commit失敗:
Job unwindでdrop


移管先の予算が不足する場合、commitを原子的に拒否する。

### 24. 予算超過
#### 24.1 型付きFailure

管理されたメモリ予算を超える場合は、型付きFailureとする。

failure resource-exhausted


概念的なerror型：

(data resource-exhausted
  (memory-budget-exceeded memory-budget-info)
  (allocation-too-large allocation-size-info)
  (continuation-budget-exceeded continuation-budget-info)
  (snapshot-retention-limit snapshot-limit-info)
  (external-resource-limit external-limit-info))

#### 24.2 処理

予算超過時：

1. 対象allocationを実行しない
2. resource-exhaustedを発生
3. Continuationをdiscard
4. bracket cleanupを実行
5. Job所有rootをdrop
6. JobをFailedとして終了


共有Runtimeおよび別Jobは健全なら継続できる。

#### 24.3 予約領域

予算超過の報告とcleanupを安全に行うため、Runtimeは最小予約領域を持てる。

予約領域も利用できない場合はTerminal failureである。

### 25. 単一巨大Allocation
#### 25.1 事前検査

巨大なallocationを実行する前に、次を検査する。

- Size計算overflow
- 実装上限
- Job予算
- Resource種別ごとの上限

#### 25.2 分類
外部入力による不正size:
resultまたはfailure

設定上限超過:
failure allocation-too-large

内部で証明済みのsize条件違反:
Defect

安全に報告不能なallocator failure:
Terminal failure


外部入力から読み取ったdimensionを、検証せずallocation sizeへ使用してはならない。

### 26. Continuation予算
#### 26.1 課金対象

Continuation生成時、次を予算へ課金する。

- Continuation object
- Captured stack segment
- Handler frame
- Cleanup frame
- 新たに割り当てた補助構造


既に同じJobへ課金済みの捕捉値本体を、物理allocationなしに二重課金しない。

#### 26.2 超過時

Continuation捕捉が予算を超える場合：

- 通常Continuation生成を中止
- 最小resource-exhaustedを構築
- 安全なFailure boundaryへ移動
- Cleanupをunwind


安全なFailure処理に必要な領域も確保できない場合はTerminal failureである。

### 27. Snapshot保持量
#### 27.1 有効なSnapshot handle

有効なSnapshot handleが存在する間、その論理内容を保持する。

予算不足を理由に、既存handleを暗黙に無効化しない。

#### 27.2 保持policy

次は所有serviceごとに個別の保持policyを持つ。

- Undo履歴
- Transaction履歴
- Preview snapshot
- Render job snapshot
- Plugin snapshot


予算不足時には、新しいhandleの発行、新しいJob開始、追加履歴保持を拒否できる。

#### 27.3 履歴破棄

保持policyによる過去履歴の破棄は、現在Snapshotの意味を変更しない。

### 28. 一般heap OOM
#### 28.1 管理予算との区別

Process全体のheapが枯渇し、Failure値やcleanup処理に必要な領域も確保できない場合、通常のfailure resource-exhaustedとしての回復を保証しない。

#### 28.2 分類
安全に隔離・報告可能:
failure resource-exhausted

安全なunwindを保証不能:
terminal failure

#### 28.3 Cleanup

Terminal OOMでは、完全なcleanupを保証しない。

事前確保領域による最小診断のみを試行できる。

### 29. Reference count overflow
#### 29.1 Wraparound禁止

参照カウントをwraparoundさせてはならない。

Wraparoundは生存中の値の早期解放につながり、memory safetyを破壊する。

#### 29.2 実装

十分広いcountを使用し、増加時にoverflowを検出する。

#### 29.3 分類
Objectの所有整合性を維持して隔離可能:
Defect

既にownership状態が不明:
Terminal failure


標準仕様としてsaturating countによる永久leakには移行しない。

#### 29.4 その他の内部不変条件

次もDefectまたはTerminal failureである。

- Countが負になる
- Drop済みobjectを再drop
- Reuse tokenの二重消費
- Shared objectを一意としてreuse
- Foreign ownership状態の矛盾


CompilerのOwnership verifierにより、可能な限り実行前に排除する。

### 30. Verification
#### 30.1 Ownership verifier

dup／drop挿入後に独立したVerifierを実行する。

検査事項：

1. 各所有参照が全経路でdropまたは移送される
2. 共有地点に必要なdupがある
3. drop後に使用されない
4. 移送後に元参照を使用しない
5. Branch mergeの所有状態が整合する
6. Continuationが高々一回消費される
7. Cleanup frameが高々一回実行される
8. Foreign ownership契約が守られる

#### 30.2 Reuse verifier

Reuse最適化後に次を検査する。

- Reuse対象が一意
- ReuseClassがReusable
- Size／layout条件を満たす
- Pinned／Foreign値を再利用しない
- Reuse前後で観測可能な意味を維持する

#### 30.3 Verifier failure

Verifier failureは利用者のfailure Eではない。

Compiler defect


としてコンパイルを中止する。

不正なBackend codeを生成してはならない。

### 31. メモリ観測API
#### 31.1 非公開情報

通常RPXコードへ次を公開しない。

- 手動dup／drop
- Reference count
- Reuse token
- Heap address
- 強制free
- 任意objectの厳密size
- Perceus内部queue
- 値の一意性

#### 31.2 許可される情報

次は診断・管理用途として提供できる。

- 設定済みbudget
- 概算残量
- Peak category
- 予算超過情報
- Debug／profiling統計

#### 31.3 安定性

Profiling値は診断用であり、プログラム意味論上の安定値とは保証しない。

- Compiler version
- Backend
- Reuse判断
- Alignment
- Header layout


によって変化し得る。

### 32. GUI状態
#### 32.1 暗黙cellへの非依存

GUI再評価をまたぐ状態を、Closure環境や暗黙cellの物理寿命へ依存させない。

#### 32.2 推奨モデル
- GUI modelは明示的な不変値
- Updateはmessageまたはtransaction
- 再評価時の状態継承はStable Key
- 一時Closure環境は再評価時に破棄可能


詳細はOPEN-GUI-STATE-001へ移管する。

### 33. 適合試験
MEM-01：基本的なdrop
(local
  (val value
    (build-large-value))

  (process value)

  (unrelated-work))


期待結果：

valueはprocess後に使用されないため、
unrelated-workより前にdrop可能

MEM-02：共有値
(tuple value value)


期待結果：

必要な共有参照についてdupを挿入
値を早期dropしない

MEM-03：排他的分岐
(if condition
    (consume-left value)
    (consume-right value))


期待結果：

実際に通るbranchへ所有権を移送
不要な事前dupを要求しない

MEM-04：Reuse成功

一意なconstructorを分解して同等sizeのconstructorを再構築する場合：

領域をreuse可能
観測可能な結果は通常allocationと同一

MEM-05：Reuse不成立

値が共有されている場合：

新規allocationへfallback
Failureにしない

MEM-06：Closure capture

不変値を捕捉するClosure：

Closure環境をPerceusで管理
Closureのdrop時に捕捉値をdrop

MEM-07：Scoped handle escape
(with-file path
  (fn (file)
    file))


期待結果：

static error:
resource handle escapes its bracket scope

MEM-08：Closure経由のescape
(with-file path
  (fn (file)
    (fn ()
      (read-all file))))


期待結果：

static error:
escaping closure captures a scoped resource

MEM-09：独立結果
(with-file path
  (fn (file)
    (read-all file)))


read-allが独立したbytesを返す場合：

success

MEM-10：Continuation resume

Resource scope内で一般Effectが発生し、Continuationがresumeされた場合：

- Resourceをreleaseしない
- Cleanup frameを復元
- Continuationを一回消費

MEM-11：Continuation discard

Continuationがdiscardされた場合：

- CleanupをLIFO実行
- Captured valuesをdrop
- Continuationを解放

MEM-12：Failure unwind

Resource scope内でFailureが発生：

- Cleanupを実行
- 離脱frameの通常値をdrop
- Error値をhandlerへ移送

MEM-13：Cycle

一般不変値を使って直接自己参照cycleを作ろうとする場合：

v1では表現不可

MEM-14：予算超過

Job予算を超えるallocation要求：

failure resource-exhausted
Cleanup後にJobを終了
別Jobは健全なら継続

MEM-15：Commit予算不足

Job結果をDocumentへcommitする際、Document budgetが不足：

- Commitを原子的に拒否
- 現在Snapshotを変更しない
- Job一時値をdrop

MEM-16：巨大Allocation

外部入力が実装上限を超える巨大bufferを要求：

failure allocation-too-large


Size計算自体が内部前提に反してoverflowした場合：

Defect

MEM-17：一般heap OOM

Failure値やcleanup用領域も確保できない場合：

Terminal failureを許容

MEM-18：Reference count overflow

参照カウントが実装上限を超える場合：

- Wraparoundしない
- 隔離可能ならDefect
- 所有整合性不明ならTerminal failure

MEM-19：Ownership verifier

drop後の値使用を含む内部Core：

Compiler defectとして拒否
Backend codeを生成しない

MEM-20：Foreign Borrowed違反

Foreign側がBorrowed引数を呼出し後も保持：

安全に検出可能:
Defect

Memory safety破壊:
Terminal failure

### 34. 移管先OPEN

#### `OPEN-MEM-CELL-001`

- Escape可能cell／ref
- Stateful closure
- Heap identity
- Cycle
- Cycle collector
- Shared mutable state

#### `OPEN-MEM-BORROW-001`

- 一般borrowed view
- Read borrow
- Mutable borrow
- Alias規則
- Borrowed Closure
- Zero-copy API

#### `OPEN-CON-001`

- Thread間Send
- Shared immutable value
- Atomic reference count
- Structured concurrency
- Cancellation
- Task間Resource共有

#### `OPEN-KER-001`

- Foreign ownership metadata
- Borrowed／Owned／Shared／Copied
- Pinned memory
- ForeignValue
- Adapter contract

#### `OPEN-GUI-STATE-001`

- Stable Key
- GUI再評価
- Model stateの継承
- Component lifetime

#### `OPEN-MEM-PROF-001`

- Profiling API
- Peak memory
- Allocation category
- Reuse統計
- Debug ownership trace

### 35. 最終状態

```text
OPEN-MEM-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- Perceus方式の精密参照カウント
- Effect lowering後のownership解析
- 自動的なdup／drop挿入
- 一意性に基づくreuse
- Functional But In-Place最適化
- 利用者から観測不能な物理共有と更新
- Closure環境の自動管理
- One-shot continuationの所有権管理
- Failure unwind時のdropとcleanup
- bracketとPerceusの責務分離
- Scope外へescapeできないResource handle
- v1での一般cell／ref・cycle・Weak referenceの不採用
- Job単位のメモリ予算
- 型付きの予算超過Failure
- 回復不能OOMのTerminal failure分類
- Ownership verifierとReuse verifier


次に検討する項目は、複数task、Cancellation、thread共有値、atomic reference countおよびFault boundaryを接続する **OPEN-CON-001「並行性・構造化Concurrency・Cancellation」**とする。
