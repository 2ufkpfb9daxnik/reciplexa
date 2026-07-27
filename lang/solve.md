# openを解決する

## `OPEN-EVAL-001` 最小Core calculus

### 状態

`解決済み`

本決定は、RPXの最小Core v1について、項、値、評価順序、逐次評価および
評価文脈を固定するものである。

`letrec`、pattern match、record、effect handler、局所可変状態、dynamic cast、
primitive固有の失敗は最小Core v1には含めず、それぞれ別の設計事項として扱う。

---

### `DD-EVAL-001`: 評価戦略と評価順序

`確定`

RPX Coreは、正格call-by-valueで評価する。

関数適用では、operatorを最初に評価し、その後、引数をsource orderで
左から右へ評価する。すべての引数が値になった後で関数本体を評価する。

以下の評価順序は、effect、diagnostic、provenanceおよび外部操作から
観測可能な言語仕様であり、実装によって変更してはならない。

1. 関数適用ではoperatorを最初に評価する。
2. 引数は左から右へ評価する。
3. `let`のinitializerをbodyより先に評価する。
4. `seq`の各式を左から右へ評価する。
5. `if`ではconditionを先に評価し、選択されたbranchだけを評価する。
6. 将来recordを追加する場合、field式はsource orderで評価する。
7. 将来handlerを追加する場合、handler式をbodyより先に評価する。

遅延評価はCoreの既定評価戦略にはしない。必要な場合は、明示的な関数、
データ型、effectまたはhygienic macroとして導入する。

---

### `DD-EVAL-002`: 最小Coreの項

`確定`

最小Core v1の項を次のように定める。

```text
e ::= x
    | literal
    | fn(x1, ..., xn, e)
    | app(e, e1, ..., en)
    | let(x, e1, e2)
    | if(e1, e2, e3)
    | seq(e1, ..., en)       where n >= 1
````

各項の意味は次のとおりである。

* `x`: lexical variable reference
* `literal`: Core primitive valueのリテラル
* `fn`: lexical closureを生成する関数式
* `app`: 関数適用
* `let`: 単一の不変局所束縛
* `if`: Bool条件による分岐
* `seq`: 一個以上の式の逐次評価

`val`は最小Coreの式ではない。

* Surfaceの局所`val`を許す場合はCore `let`へelaborateする。
* top-level `val`はfileまたはmodule declarationとして別途elaborateする。
* named function sugarは`val`と`fn`へelaborateする。

次のものは最小Core v1に含めない。

* `letrec`
* `match`およびpattern
* recordとvariant
* `var`およびassignment
* effect operationとhandler
* module
* dynamic cast
* resource operation固有の項

これらを言語から棄却するのではなく、安定した最小Coreに対する拡張として
別途定義する。

***

### `DD-EVAL-003`: `let`

`確定`

Core `let`は単一bindingだけを持つ。

```text
let(x, initializer, body)
```

`initializer`を現在のlexical environmentで評価し、その結果を`x`へ束縛した
環境で`body`を評価する。

`x`は自身の`initializer`から参照できない。

Surface `let`は一個以上のsequential bindingを持つことができる。

```lisp
(let ((x e1)
      (y e2)
      (z e3))
  body)
```

これは次の単一binding Core `let`の入れ子へ、左から右にelaborateする。

```lisp
(let ((x e1))
  (let ((y e2))
    (let ((z e3))
      body)))
```

したがって、先行するbindingは後続するinitializerおよびbodyから参照できる。

同一のSurface `let` binding list内では、同じ名前のbinderを重複してはならない。
意図的なshadowingは、明示的に入れ子にした`let`によって表現する。

parallel bindingは最初の言語仕様には導入しない。必要になった場合は、
通常の`let`とは異なる明示的構文として追加する。

Core `let`を関数適用へ完全には脱糖しない。これは、let-polymorphism、
value restriction、型・effect推論、diagnosticおよびprovenanceにおける
generalization pointを保持するためである。

***

### `DD-EVAL-004`: 複数式bodyと`seq`

`確定`

仕様でbodyを持つと定められたSurface構文では、一個以上のbody expressionを許す。

複数のbody expressionはsource orderで左から右へ評価する。最後以外の式の値は
破棄し、最後の式の値をbody全体の結果とする。

Coreの`fn`、`let`等のbodyは常に単一式とする。複数のSurface body expressionは
Core `seq`へelaborateする。

Surface:

```lisp
(fn (x)
  (log x)
  (+ x 1))
```

Core:

```lisp
(fn (x)
  (seq
    (log x)
    (+ x 1)))
```

bodyが一式だけの場合、不要な`seq`は生成しない。

`seq`は一個以上の式を持つ。空の`seq`および空のSurface bodyは認めない。
何もしないbodyは明示的な`Unit`値を返す。

途中式では任意の型の値を破棄できる。非`Unit`の純粋な値が未使用である場合、
処理系はwarningを出してよいが、これをCoreの型エラーとはしない。

`seq(e1, ..., en)`の結果型は最後の式`en`の型であり、effectはすべての式の
effectをsource orderに従って合成したものとする。

***

### `DD-EVAL-005`: `if`

`確定`

Core `if`は厳密に次の三式を持つ。

```text
if(condition, then_branch, else_branch)
```

`condition`は静的に`Bool`型でなければならない。

Number、String、Unit、record、collection、dynamicその他の値を、
暗黙にBoolへ変換しない。0、空文字列、空collection、null相当の値に
truthinessまたはfalsinessを与えない。

動的評価では、conditionを最初に評価する。

* 結果が`true`ならthen branchだけを評価する。
* 結果が`false`ならelse branchだけを評価する。
* 選択されなかったbranchは評価しない。

then branchの型が`T`、else branchの型が`U`の場合、`if`全体の型は
`union(T, U)`とする。

semantic subtypingによって`T <: U`なら、次が成立する。

```text
union(T, U) ≃ U
```

condition、then branchおよびelse branchのすべての静的effectを、
`if`式全体のeffectへ合成する。実行時には選択されたbranchのeffectだけが発生する。

***

### `DD-TYP-IF-001`: 条件分岐による型の絞り込み

`確定`

RPXは、処理系が認識可能な型判定によるoccurrence typingを採用する。

不変な局所変数`x`がstatic型`S`を持ち、conditionが`x`の型`T`への所属を
判定する場合、各branchで`x`を次のように扱う。

```text
then branch: intersect(S, T)
else branch: diff(S, T)
```

`diff(S, T)`はstatic type fragmentにおいて次と型等価である。

```text
intersect(S, not(T))
```

初期仕様では、直接参照された不変局所変数だけを絞り込み対象とする。

次は本決定の対象外とする。

* `var`等の可変変数
* 任意のBool戻り値関数
* effectful expression
* property path
* 計算式の再評価を必要とする絞り込み
* bounded dynamicに対するruntime narrowing

型判定の表面構文、predicate typeおよび利用者定義predicateへの拡張方法は、
型システムの別項目で決定する。

***

### 値

最小Core v1の値を次のように定める。

```text
v ::= primitive-value
    | closure(environment, parameters, body)
```

初期の`primitive-value`は少なくとも次を含む。

```text
Bool
Unit
Number
String
```

Numberの内部分類、精度、overflow、NaN、Infinity、単位付きquantityおよび
Stringのindexing単位は`OPEN-SYN-002`およびliteral/primitive仕様へ移管する。

将来のCore拡張により、次の値を追加できる。

```text
record-value
variant-value
handler-value
continuation/resumption
resource-handle
packed-module
dynamic value with evidence
```

ただし、追加時には対応する型付け規則、評価規則およびcanonical formsを
別途定義しなければならない。

***

### Closureとlexical scope

`fn(x1, ..., xn, body)`を評価すると、現在のlexical environmentを捕捉した
closureを生成する。

```text
closure(environment, parameters, body)
```

closure生成自体はbodyを評価せず、runtime effectを発生させない。
bodyのeffectは関数型へ記録する。

closure適用時には、closure生成時のenvironmentを基礎とし、仮引数へ実引数の値を
束縛した環境でbodyを評価する。

同名の内側bindingは外側bindingをshadowする。名前解決後のCoreでは、
識別子は文字列ではなく一意な`BindingId`によって参照する。

***

### 関数適用

関数適用`app(operator, argument1, ..., argumentN)`は次の順序で評価する。

1. operatorを値まで評価する。
2. argumentを左から右へ値まで評価する。
3. operatorの値が対応するarityのclosureであることを確認する。
4. 各parameterを対応するargument valueへ束縛する。
5. closureのbodyを評価する。

最小Coreでは固定arity関数だけを扱う。

arity mismatchは、通常のstatic fragmentでは型検査またはelaboration時に拒否する。
型検査済みのstatic Coreで、non-function applicationまたはarity mismatchが
動的に発生してはならない。

dynamic境界、foreign primitiveまたは不正なCoreに起因するarity mismatchの分類は、
`OPEN-TYP-001`、`OPEN-KER-001`または`OPEN-ERR-001`で決定する。

可変長引数、keyword引数、optional引数、partial applicationおよびcurryingの
表面構文は本決定に含めない。

***

### 評価文脈

最小Core v1の評価文脈を概念的に次のように定める。

```text
E ::= []
    | app(E, e1, ..., en)
    | app(v, v1, ..., vi-1, E, ei+1, ..., en)
    | let(x, E, e)
    | if(E, e1, e2)
    | seq(v1, ..., vi-1, E, ei+1, ..., en)
```

この評価文脈は、次を保証する。

* operator-first
* argument left-to-right
* initializer-before-body
* sequence left-to-right
* condition-before-branch
* unselected branch non-evaluation

ここで`seq`の最後以外の値は、次の式へ進む際に破棄する。

実装はCEK、CEKS、bytecode VMまたは直接インタプリタのいずれを使用してもよい。
ただし、上記の観測可能な評価順序を保存しなければならない。

***

### 最小Coreの終端状態

最小Core v1の評価は、次のいずれかとなる。

1. 値を返して正常終了する。
2. 無限にstepを続けて発散する。
3. 実装上のresource limitへ到達する。

well-typedでclosedな最小Core v1の項は、次の状態になってはならない。

* unbound variable
* non-function application
* arity mismatch
* non-Bool `if` condition
* unexplained stuck state
* undefined behavior

resource exhaustionのcatch可能性およびhost failureとしての分類は、
`OPEN-MEM-001`および`OPEN-ERR-001`へ移管する。

***

### 未決定事項の移管

`OPEN-EVAL-001`を閉じるため、最小Core v1に含まれない事項を次へ移管する。

* `letrec`のscopeと初期化制約:
  `OPEN-BND-001`
* `var`のescapeおよびstate-handler elaboration:
  `OPEN-BND-001`
* pattern、match、網羅性、guard:
  `OPEN-DAT-001`
* primitiveのdomain error:
  `OPEN-ERR-001`
* effect operation、handler、resumption:
  `OPEN-EFF-001`
* bounded dynamicによるruntime failure:
  `OPEN-TYP-001`
* record/variant値と評価規則:
  `OPEN-DAT-001`および`OPEN-ROW-001`
* literal、数値およびStringの詳細:
  `OPEN-SYN-002`
* runtime resource exhaustion:
  `OPEN-MEM-001`および`OPEN-ERR-001`

これらは`OPEN-EVAL-001`の未解決部分とは数えず、各拡張の導入時に
最小Coreへ追加する評価規則として追跡する。

***

### 適合試験

#### lexical closure

```lisp
(let ((x 10))
  (let ((f (fn (y) (+ x y))))
    (let ((x 20))
      (f 1))))
```

期待結果:

```text
11
```

#### sequential `let`

```lisp
(let ((x 1)
      (y (+ x 1)))
  y)
```

期待結果:

```text
2
```

#### duplicate binder

```lisp
(let ((x 1)
      (x 2))
  x)
```

期待結果:

```text
静的エラー: 同一Surface let内のduplicate binder
```

#### application order

```lisp
(f
  (log-and-return "a" 1)
  (log-and-return "b" 2))
```

期待される観測順序:

```text
a
b
```

#### sequence result

```lisp
(seq
  (log "first")
  (log "second")
  42)
```

期待される観測順序:

```text
first
second
```

期待結果:

```text
42
```

#### selected branch only

```lisp
(if true
    (log-and-return "then" 1)
    (log-and-return "else" 2))
```

期待される観測:

```text
then
```

期待結果:

```text
1
```

#### strict Bool condition

```lisp
(if 1
    10
    20)
```

期待結果:

```text
静的エラー: expected Bool, found Integer
```

#### union result

```lisp
(if condition
    42
    "unknown")
```

期待される型:

```text
union(Integer, String)
```

#### occurrence typing

```lisp
; x : union(Number, String)

(if (number? x)
    (+ x 1)
    (string-length x))
```

期待されるbranch環境:

```text
then: x : Number
else: x : String
```

***

### 解決後の最小Core

```text
e ::= x
    | literal
    | fn(x1, ..., xn, e)
    | app(e, e1, ..., en)
    | let(x, e1, e2)
    | if(e1, e2, e3)
    | seq(e1, ..., en)       where n >= 1

v ::= primitive-value
    | closure(environment, parameters, body)
```

この最小Coreについて、評価戦略、値、評価文脈および主要なstuck状態が
特定されたため、`OPEN-EVAL-001`を解決済みとする。

## `OPEN-EFF-001` Algebraic effect handler意味論

### 状態

`解決済み`

本決定は、RPX v1における次の事項を固定する。

- deep handler
- one-shot resumption
- operationの探索と自動伝播
- 明示的forward
- return clause
- handler単位の結果型変換
- duplicate labelを許すEffectRowの基本的意味
- ambient effectとnamed/scoped effect instance
- first-class handler value
- Core `handle`とSurface `with`
- 未処理effectの静的・動的な扱い

次の機能はRPX v1の基本handlerには含めず、将来拡張または別の
`OPEN-*`へ移管する。

- multi-shot resumption
- shallow handler
- first-class resumption
- 一般的なanswer-type modification
- 利用者向けの任意のeffect masking
- cleanup失敗を含む完全なfinalization意味論
- exception/error taxonomy
- 並行実行、cancellationおよびtask間resumption

---

### `DD-EFF-001`: deep handler

`確定`

RPXの通常のeffect handlerはdeep handlerとする。

operationがnearest matching handlerによって処理され、operation clauseが
resumptionを再開した場合、再開された計算は同じhandlerの動的scope内で
引き続き実行される。

再開後の計算が同じeffectのoperationを再び実行した場合、そのoperationは
同じhandlerによって再び処理される。ただし、再開後により内側のmatching
handlerが設置された場合は、nearest matching handler規則に従って内側の
handlerが処理する。

概念的には、次のように動作する。

```text
operation発生
→ nearest matching handlerのoperation clause
→ resume
→ 現在のdeep handlerを再設置
→ 元の計算の残りを実行
````

deep handlerが複数のoperationを処理することは、同一resumptionを複数回
使うことを意味しない。各operationはそれぞれ別個のresumptionを生成する。

shallow handlerはRPX v1の公開言語には含めない。将来導入する場合は、
通常のhandlerとは異なる明示的な構文および型を持つ別機能として定義する。

***

### `DD-EFF-002`: one-shot resumption

`確定`

RPXの通常のresumptionはone-shotとする。

正確には、resumptionは0回または1回だけ使用できるaffine capabilityである。

operation clauseはresumptionを次のいずれかの方法で扱う。

1. 一度だけresumeする。
2. 一度だけforwardする。
3. resumeもforwardもせず、捕捉された計算を破棄する。

同一resumptionを複数回resumeしてはならない。

同一resumptionをresumeした後にforwardしてはならない。また、forwardした後に
resumeまたは再度forwardしてはならない。

```lisp
; 許可
(op (argument resume)
  (resume value))
```

```lisp
; 許可
(op (argument resume)
  alternative-handler-result)
```

```lisp
; 拒否
(op (argument resume)
  (seq
    (resume first-value)
    (resume second-value)))
```

multi-shot resumptionは通常の`resume`の挙動として提供しない。

将来multi-shotを導入する場合は、少なくとも次を別途規定しなければならない。

* 継続を複製できる条件
* 局所状態を共有するか複製するか
* resource handleを含む継続を複製できるか
* `SourceEdit`等の外部effectを含む継続を複製できるか
* cleanupを何回実行するか
* 複製のmemory cost
* multi-shot性を型またはeffect rowで追跡する方法

multi-shotが別途導入されるまで、RPXの公開言語で利用できるresumptionは
one-shotだけとする。

***

### `DD-EFF-003`: resumptionの型とscope

`確定`

resumptionは通常の関数型ではなく、専用のaffine型を持つ。

operationが次の型を持つとする。

```text
op : P ->{L} R
```

handler全体の結果型を`B`、再開中に外側へ残り得るeffect rowを`E`とすると、
operation clause内のresumptionは概念的に次の型を持つ。

```text
Resume<R, B, E>
```

各型引数の意味は次のとおりである。

* `R`: operation呼出地点へ返す値の型
* `B`: handler全体の結果型
* `E`: 再開された計算から外側へ残り得るeffect row

resumptionの再開は、通常の関数適用とは異なるCore項として扱う。

```text
resume(k, value)
```

型付けの概念形は次のとおりである。

```text
k : Resume<R, B, E>
value : R
────────────────────────
resume(k, value) : B ! E
```

`resume`へ渡す値の型はoperation結果型`R`である。一方、`resume`式そのものの
結果型はhandler全体の結果型`B`である。

resumptionはoperation clause内だけの特殊binderとし、RPX v1では通常の
first-class valueにしない。

次を禁止する。

* recordまたはcollectionへの格納
* moduleからのexport
* operation clauseからの返却
* 通常関数への引渡し
* closureによる捕捉
* polymorphic valueとしての一般化
* serialization
* 複製
* handlerのscope外へのescape

`resume`はtail positionだけには限定しない。したがって、再開後の結果に対して
後処理を行うことができる。

```lisp
(op (argument resume)
  (let ((result (resume value)))
    (postprocess result)))
```

各実行経路でresumptionが高々一回だけ消費されることを静的に検査する。
runtimeも防御的にresumptionの消費状態を保持し、型検査済みCoreの外部から
二重resumeが行われないことを検証する。

first-class affine resumptionは将来拡張として保留する。

***

### `DD-EFF-004`: effect operationの呼出し

`確定`

effect operationは、通常の関数呼出しに近いSurface構文で呼び出す。

```lisp
(random)
(log "message")
(load-image resource-id)
```

`use`等の接頭辞は必須にしない。残す場合もSurface上の任意の糖衣とし、
Core operation項にはしない。

名前解決後のCoreでは、通常の関数適用とeffect operationを区別する。

```text
通常関数:
App(function, arguments)

effect operation:
Perform(operation_id, arguments)
```

operation identityは名前文字列ではなく、名前解決済みの一意な
`EffectId`および`OperationId`で識別する。

別moduleまたは別packageが同名のeffectやoperationを定義しても、identityが異なる
場合は別のoperationとして扱う。

***

### `DD-EFF-005`: nearest matching handlerと自動伝播

`確定`

effect operationが発生した場合、runtimeはhandler stackを内側から外側へ探索し、
operation identityに一致する最初のhandler frameへ制御を移す。

handlerが処理対象として宣言していないeffectのoperationは、利用者による
明示的な記述なしに外側へ自動伝播する。

```text
operation発生
→ 最内側のhandlerが非対応なら通過
→ 次の外側のhandlerが非対応なら通過
→ nearest matching handlerが処理
```

自動伝播はoperationの再評価を意味しない。発生済みのoperation requestと捕捉された
継続を外側へ渡す。

通常のhandlerは、処理対象として宣言したeffectに属するすべてのoperationについて
clauseを持たなければならない。

対象effectのoperation clauseが不足している場合、それを暗黙のforwardとはみなさず、
静的エラーにする。

この規則により、operation clauseの書き忘れと、意図的な外側への委譲を区別する。

***

### `DD-EFF-006`: 明示的forward

`確定`

handlerが現在処理中のoperationを外側へ委譲する場合、operation clause内で
明示的な`forward`を使用する。

```text
forward(k)
```

`forward`は次の意味を持つ。

1. 現在処理中のoperation requestを維持する。
2. 現在のhandler frameより外側から探索を再開する。
3. 次のnearest matching handlerへoperationとresumptionを委譲する。
4. resumptionを消費する。

`forward`は任意の送り先を指定できない。特定のhandlerやhost handlerへ直接飛ばしたり、
複数のmatching handlerを指定個数だけ飛び越えたりしてはならない。

```text
許可:
現在のhandler
→ 次の外側のmatching handler

禁止:
現在のhandler
→ 名前で指定した任意のhandler

禁止:
現在のhandler
→ matching handlerを二層飛び越す
```

forward後は同じresumptionをresumeまたは再度forwardできない。

handlerが新しいoperationを発生させることと、現在のoperationをforwardすることは
区別する。

```text
新しいoperationを呼ぶ:
新しいoperation requestと新しいresumptionを生成する。

forwardする:
現在のoperation requestと現在のresumptionを外側へ委譲する。
```

***

### `DD-EFF-007`: handler clauseの実行scope

`確定`

operation clauseおよびreturn clause自身は、現在のhandler frameの外側にある
effect環境で実行する。

したがって、clause自身が現在処理中のeffectと同じeffectのoperationを発生させた場合、
現在のhandler自身ではなく、その外側のnearest matching handlerが処理する。

```text
元のbodyのoperation
→ 現在のhandler

operation clause自身が発生させたoperation
→ 現在のhandlerより外側を探索
```

これは、handler clause内で同じoperationを再発行したときの無限自己再帰を防ぎ、
handlerを段階的に合成するためである。

一方、operation clauseがresumptionをresumeした場合は、deep handlerの意味に従って
現在のhandlerを再設置し、元の計算の残りを実行する。

```text
operation clause自身の計算:
現在のhandlerより外側

resumeされた元の計算:
現在のhandlerを再設置した内側
```

return clauseにはresumptionは存在しない。bodyはreturn clauseの実行前に既に
正常終了している。

operation clauseがresumptionを使用せずに値を返した場合、暗黙のresumeは行わない。
捕捉された計算の残りは破棄する。

***

### `DD-EFF-008`: return clause

`確定`

handlerは省略可能なreturn clauseを持つ。

return clauseは、handler対象bodyが正常終了したとき、その正常終了値を受け取って
handler全体の結果へ変換する。

bodyの正常終了型を`A`、handler全体の結果型を`B`、return clause自身のeffectを
`Hret`とすると、return clauseは概念的に次の型を持つ。

```text
A ->{Hret} B
```

return clauseを省略した場合は、次の恒等変換を補う。

```lisp
(return (value)
  value)
```

省略時は次を要求する。

```text
A ≃ B
```

handlerが結果型を変更する場合は、return clauseを明示しなければならない。

例として、Failureを`Result`へ変換するhandlerは次のように表せる。

```lisp
(handler Failure
  (return (value)
    (Ok value))

  (fail (error resume)
    (Err error)))
```

この場合、return clauseは概念的に次の型を持つ。

```text
A -> Result<A, Error>
```

operation clauseの結果型も、return clauseの結果型と同じ`B`でなければならない。

return clause自身はeffectを発生させてよい。そのeffectは現在のhandlerでは処理せず、
現在のhandlerより外側のhandlerへ伝播する。

***

### `DD-EFF-009`: handler単位の結果型変換

`確定`

RPX v1は、handler単位でbodyの正常終了型`A`をhandler全体の結果型`B`へ変換することを
許す。

概念的なhandler型は次の形を持つ。

```text
Handler<L, A, B, H>
```

各要素の意味は次のとおりである。

* `L`: handlerが処理するeffect label
* `A`: handler対象bodyの正常終了型
* `B`: handler適用後の結果型
* `H`: return clauseおよびoperation clause自身が要求するeffect row

たとえば、結果型を変更しないhandlerは次の形になる。

```text
Handler<Random, A, A, <>>
```

Failureを`Result`へ変換するhandlerは次の形になる。

```text
Handler<
  Failure<Error>,
  A,
  Result<A, Error>,
  <>
>
```

これはhandlerの入口と出口における一つの`A -> B`変換である。

任意の式またはcontrol operationが周囲のanswer typeを段階的に変更する、
一般的なanswer-type modificationはRPX v1には導入しない。

一般的answer-type modification、answer-type polymorphismおよびその型推論は
将来拡張として保留する。

***

### `DD-EFF-010`: handler valueのrank-1多相性

`確定`

handler valueは、通常の`val`一般化およびvalue restrictionに従ってrank-1多相に
なれる。

不変かつgeneralizableなhandler valueでは、body結果型、handler結果型および
effect-row変数を一般化できる。

結果型を変更しないRandom handlerは、概念的に次の型schemeを持ち得る。

```text
forall A E.
  Handler<Random, A, A, <>>
```

Failureを`Result`へ変換するhandlerは、概念的に次の型schemeを持ち得る。

```text
forall A E.
  Handler<
    Failure<Error>,
    A,
    Result<A, Error>,
    <>
  >
```

すべてのhandlerを無条件に一般化してはならない。

次はvalue restrictionの対象とする。

* effectfulな式によって生成されたhandler
* 局所可変状態を捕捉するhandler
* affine capabilityを捕捉するhandler
* scoped effect evidenceを不正にescapeさせるhandler
* generalizable valueに該当しないhandler生成式

handler一般化の最終的なalgorithmは、通常の`val`一般化および
`OPEN-BND-001`で決定するvalue restrictionと整合させる。

***

### `DD-EFF-011`: first-class handler value

`確定`

handlerはfirst-class valueとする。

handler valueについて次を許可する。

* `val`への束縛
* 関数引数としての受渡し
* 関数からの返却
* record fieldへの格納
* moduleおよびpackage APIからの公開
* 不変なlexical environmentの捕捉

handler valueと、実行時に設置されたhandler instanceを区別する。

```text
handler value:
effectの解釈方法を表す再利用可能な値

handler instance:
特定のhandle適用によって生成される実行時frame
```

同じhandler valueを複数回設置した場合、設置ごとにfresh handler frameを生成する。

```text
同じhandler value h
├─ handler frame 1
└─ handler frame 2
```

handler valueが捕捉した通常の不変値は、通常のclosure captureと同じ規則で
各instanceから参照できる。

instance固有の可変状態をhandler valueそのものの基本機構には組み込まない。
instance固有状態が必要な場合は、局所stateおよびrunner関数によって構築する。

runtimeは最適化として局所stateをhandler frameへ融合してよいが、この最適化は
観測可能な意味を変更してはならない。

handler valueについて、次は提供しない。

* 一般的な等値比較
* hash
* serialization
* 捕捉環境の観測
* private module representationの観測
* handler instance identityの通常値としての取得

handlerが処理するeffect identityは静的に型へ現れなければならない。
実行時の文字列名だけで任意のeffectを処理する動的handlerは、RPX v1には導入しない。

***

### `DD-EFF-012`: Core `handle`とSurface `with`

`確定`

Coreの`handle`は、handler valueと引数なし関数であるbody thunkを受け取る
専用の制御項とする。

```text
handle(handler_expression, body_thunk)
```

Surface上の概念形は次のとおりである。

```lisp
(handle handler-value
  (fn ()
    body))
```

RPXは正格call-by-valueであるため、bodyを通常引数として直接渡してはならない。
bodyを直接引数にすると、handler設置前にbodyが評価され、対象operationが未処理になる。

Surfaceでは`with`を糖衣構文として提供する。

```lisp
(with handler-value
  body-expression-1
  body-expression-2)
```

これはhygienicに次へelaborateする。

```lisp
(handle handler-value
  (fn ()
    (seq
      body-expression-1
      body-expression-2)))
```

Core `handle`の評価順序を次のように定める。

1. handler expressionを評価し、handler valueを得る。
2. body thunkを評価し、closureを得る。
3. handler valueからfresh handler frameを生成する。
4. handler frameを設置する。
5. body thunkを一度だけ起動する。
6. body中のoperationをhandler意味論に従って処理する。
7. bodyが正常終了した場合はreturn clauseを適用する。
8. handlerの動的scopeを終了する。
9. handler全体の結果を返す。

`handle`は通常の関数適用ではなく、Core専用項とする。

```text
App(function, arguments)
Handle(handler, body_thunk)
```

同じhandler valueを再利用することは、同じhandler instanceを再利用することを
意味しない。

***

### `DD-EFF-013`: ambient effectとnamed/scoped effect instance

`確定`

RPXは、次の二種類のeffect利用形態を持つ。

1. ambient effect
2. named/scoped effect instance

#### Ambient effect

通常のoperationはambient effectとして呼び出す。

```lisp
(log "message")
(random)
(load-image resource-id)
```

operationは動的scope内のnearest matching handlerへ送られる。

ambient effectは、通常次の用途に使用する。

* Log
* Random
* Resource
* FileRead／FileWrite
* Failure
* host service

#### Named/scoped effect instance

同じeffectの複数handlerから呼出側が送り先を選択する必要がある場合、
named/scoped effect instanceを使用する。

概念例：

```lisp
(read state-a)
(read state-b)
```

または：

```lisp
(state-a.read)
(state-b.read)
```

表面構文は別途決定する。

named handlerの設置時に、freshなinstance identityと型付きevidenceを生成する。

```text
install named handler
→ fresh scope identity s
→ Evidence<L, s>
→ handler frame for L<s>
```

異なる設置から生成されたinstanceは、同じhandler valueを使用していても
異なるidentityを持つ。

```text
s1 ≠ s2
```

operationの送り先はevidenceによって明示する。

scoped instanceのidentityは、そのscope外へescapeしてはならない。
特に局所`var`、局所resource、transaction等に使用するinstanceは、
scope identityが結果型、外部closure、module exportまたは永続データへ漏れないことを
型検査する。

duplicate effect labelとnamed instance identityは別の機構である。

```text
duplicate effect label:
row polymorphismとnested handlerの型付けに使用する。

named instance:
同種の複数handlerからoperationの送り先を区別するために使用する。
```

***

### `DD-EFF-014`: EffectRowの意味

`確定`

EffectRowはoperationの動的実行回数を数えるものではない。

次の計算が`log`を二回実行しても、通常必要なeffect要求は一層の`Log`である。

```lisp
(seq
  (log "a")
  (log "b"))
```

```text
実行回数:
2

effect要求:
<Log>
```

一つのdeep handlerが両operationを処理できる。

EffectRowは同一effect labelの重複を許す。

```text
<Log, Log | E>
```

重複は、operationの実行回数ではなく、row extensionの異なる由来、
nested handlerおよびeffect-polymorphicな計算を型付けするために保持する。

`<L | E>`は、effect row `E`へ一層の`L`を追加するrow extensionである。

`E`にも`L`が含まれている場合、重複を勝手に除去してはならない。

```text
E = <L, Resource>

<L | E>
= <L, L, Resource>
```

handlerは対象effectを一層だけ処理する。

```text
<L | E>
   ↓ handle L
E
```

重複がある場合も、一度ですべて除去してはならない。

```text
<L, L, Resource>
   ↓ handle L
<L, Resource>
```

次は不正である。

```text
<L, L, Resource>
   ↓ handle L
<Resource>
```

named instanceでは、instance identityをeffect labelの一部として扱う。

```text
State<s1> ≠ State<s2>
```

したがって、次は同一ラベルの重複ではなく、異なるeffect labelである。

```text
<State<s1>, State<s2>>
```

***

### `DD-EFF-015`: ambient effect rowと制約生成

`確定`

複数の部分式のeffectを単純な集合和またはmultiset加算によって合成してはならない。

各計算にはambient effect rowを割り当て、各部分式はそのrowに対する
effect要求制約を生成する。

operationが次の型を持つとする。

```text
op : P ->{L} R
```

operation呼出しは、ambient row `E`について次を要求する。

```text
L ∈ E
```

同じ計算内で同じoperationを複数回呼び出した場合、同じeffect層が複数の
membership要求を満たしてよい。

```text
L ∈ E
L ∈ E

最小の通常解:
E = <L>
```

これを機械的に次へ変換してはならない。

```text
E = <L, L>
```

複数の部分式については、それぞれのeffect要求を満たす共通のambient rowを求める。

```lisp
(seq
  (log "start")
  (random)
  (log "finish"))
```

制約：

```text
Log ∈ E
Random ∈ E
Log ∈ E
```

通常の最小解：

```text
E = <Log, Random>
```

一方、effect-polymorphicなrowへ明示的に一層を追加する場合はrow extensionを使用する。

```text
<Log | E>
```

`E`にも`Log`が含まれている場合、その重複は維持する。

規範仕様では、曖昧な`join(E1, E2)`を基本演算としない。

代わりに、次の関係および制約を用いる。

```text
Requires(L, E)
```

`E`がeffect label `L`を少なくとも一層含むことを要求する。

```text
Includes(E1, E2)
```

`E2`が`E1`のeffect要求を満たすことを要求する。

```text
HandlesOne(L, Ebody, Erest)
```

`Ebody`が`<L | Erest>`と単一化可能であり、handlerが`L`を一層処理することを表す。

正確なrow unification algorithm、principal solutionおよびalgorithmic completenessは
`OPEN-TYP-002`と接続して別途検証する。

***

### `DD-EFF-016`: handlerの型付け骨格

`確定`

effect operationを次のように表す。

```text
op : P ->{L} R
```

handler対象bodyの型を次のように表す。

```text
body : A ! <L | E>
```

Coreではbodyをthunkとして渡す。

```text
body_thunk : Unit ->{<L | E>} A
```

handler valueの概念型を次とする。

```text
Handler<L, A, B, H>
```

* `L`: 処理するeffect
* `A`: bodyの正常終了型
* `B`: handler全体の結果型
* `H`: handler clause自身のeffect要求

return clauseは概念的に次の型を持つ。

```text
A ->{Hret} B
```

各operation clauseは、operation引数とaffine resumptionを受け取る。

```text
argument : P
resume   : Resume<R, B, E>
```

operation clauseのbodyは次の型を持つ。

```text
B ! Hop
```

全clauseのeffect要求はhandler effect `H`によって満たされなければならない。

```text
Hret ⊑ H
Hop1 ⊑ H
...
HopN ⊑ H
```

handler適用の概念的な規則は次のとおりである。

```text
h    : Handler<L, A, B, H>
body : Unit ->{<L | E>} A

E ⊑ F
H ⊑ F
────────────────────────────────
handle(h, body) : B ! F
```

ここで`F`は、bodyから残るeffect `E`とhandler clause自身のeffect `H`の
両方を満たすambient rowである。

handlerはbodyの`L`を一層だけ処理する。

***

### `DD-EFF-017`: 利用者向けmaskの禁止

`確定`

RPX v1は、一般利用者向けの任意のeffect masking機能を提供しない。

利用者コードは、動的scopeに設置された任意のhandlerを一時的に無効化したり、
指定したmatching handlerを飛び越えたりしてはならない。

次のような一般的機能は提供しない。

```lisp
(mask Log
  body)
```

```lisp
(skip-handler audit-handler
  body)
```

理由は次のとおりである。

* sandboxまたは監査handlerの回避を防ぐ
* module sealingとprivate不変条件を保護する
* handlerの送り先を型およびscopeから予測可能にする
* resource、transaction、SourceEdit等の安全境界を維持する
* named instanceと重複する低水準機能を避ける

operation clauseおよびreturn clauseの実行中に現在のhandler frameを探索対象から
除外する動作は、handler意味論の内部規則であり、first-classなmask operationとして
公開しない。

特定のhandler instanceを使用する必要がある場合はnamed/scoped effect instanceを使う。

現在処理中のoperationを外側へ委譲する場合は、operation clause内だけで使用できる
`forward`を使う。

将来、runtime、schedulerまたはforeign interfaceの実装にmask相当の操作が必要になった
場合は、通常のsafe RPXから分離されたtrusted internal primitiveとして検討する。

***

### `DD-EFF-018`: residual effectと実行境界

`確定`

通常の関数および公開パッケージAPIは、残余effectを型に持つことができる。

```text
load-project :
Path ->{Resource, Failure} Project
```

このeffect rowは未処理runtime errorではなく、呼出側への型付き要求である。

private関数のeffectは、次のいずれかでなければならない。

1. パッケージ内部のhandlerで処理する。
2. 呼出側の関数型へ正確に伝播する。
3. effectを禁止する境界で静的に拒否する。

package初期化は原則としてempty effect rowを要求する。

```text
package initialization : <>
```

外部操作が必要なpackageは、import時に処理を実行せず、effectfulな公開関数として
提供する。

testは、許可されたtest handlerを適用した後にempty effect rowになることを要求する。

実行可能entry pointでは、次のいずれかを要求する。

```text
residual effects = <>
```

または：

```text
residual effects
⊆ 実行hostが明示的に提供するeffect
```

host handlerは実行前のlink段階で接続し、entry pointのeffect要求と照合する。

```text
typed entry point
+
typed host handlers
+
runtime/backend configuration
→ closed executable computation
```

必要なhost handlerが不足する場合、operationを実行するまで待たず、link errorとして
拒否する。

***

### `DD-EFF-019`: 未処理effectの動的分類

`確定`

well-typedかつwell-linkedなclosed computationでは、未処理effectは発生してはならない。

runtimeは防御的検査として、handler stackおよびhost boundaryのいずれにも
対応handlerが存在しない場合を検出する。

この状態を構造化されたruntime faultとして扱う。

```text
UnhandledEffect {
  effect_id,
  operation_id,
  effect_name,
  operation_name,
  source_span,
  module_path,
  operation_origin,
  active_handler_stack,
  expected_host_capability,
  runtime_version
}
```

`UnhandledEffect`は通常のRPX effectではない。通常のRPX handlerからcatchできない
runtime faultとする。

一般的な`UnhandledEffect` handlerを言語内に提供してはならない。これを許すと、
operationごとに異なる戻り値型を無視してeffect systemを迂回できるためである。

runtimeは未処理effectを次として扱ってはならない。

* Rust panic
* undefined behavior
* 説明のないstuck状態
* 不正な任意値の返却

CLIでは構造化診断を表示し、非zero statusで終了する。

GUIでは失敗した評価runを破棄し、直前のlast-good-renderを保持する。
失敗runの途中RenderIR、SourceEdit、resource transaction等をcommitしてはならない。

外部処理の通常の失敗と`UnhandledEffect`を区別する。

```text
handlerが存在しない:
UnhandledEffect runtime fault

handlerは存在するが外部操作が失敗:
ResourceError等の通常のtyped failure
```

`UnhandledEffect`は、次のような型安全境界の破損を検出するための安全網である。

* foreign primitiveの誤実装
* runtimeまたはcompilerの欠陥
* ABI不整合
* 不正なserialized Core
* 壊れたcache
* unsafe plugin
* handler link構成の欠陥

目標性質は次のとおりである。

```text
well-typed
+
well-linked
+
valid foreign boundary
+
valid runtime configuration
────────────────────────────
UnhandledEffectへ到達しない
```

***

### `DD-EFF-020`: cleanupとの接続要件

`確定`

resumptionをresumeせずに破棄する場合、捕捉された計算を単に破棄してはならない。

捕捉された計算の動的scope内にcleanup frameが存在する場合、runtimeは継続を
unwindし、必要なcleanupを実行しなければならない。

ただし、汎用cleanup／finalizationの完全な意味論は`OPEN-EFF-001`の中心仕様から
分離する。

`OPEN-EFF-001`では次だけを要求する。

```text
1. resumeされた継続:
   cleanupをまだ実行せず、計算へ復帰する。

2. 破棄された継続:
   復帰不能になったscopeをunwindし、cleanup frameを処理する。

3. 外側へ一時的に伝播したoperation:
   外側のhandlerがresumeする可能性がある間はcleanupしない。

4. forwardされた継続:
   cleanupの所有責任も外側へ移譲する。

5. cleanupを無視したままcontinuation segmentを破棄してはならない。
```

汎用`ensure(body, cleanup)`の詳細は、`OPEN-MEM-001`および`OPEN-ERR-001`へ移管する。

移管する事項は次のとおりである。

* cleanupの完全なCore構文と型規則
* cleanupのexactly-once保証
* cleanup自身が発生させるeffect
* bodyとcleanupが共に失敗した場合のエラー合成
* resource exhaustion
* process

## `OPEN-TYP-001` Bounded dynamic、cast evidence、dynamic failure

### 状態

`解決済み`

本決定は、RPX v1におけるbounded dynamicについて、次を固定する。

- `dynamic S`の意味
- static型および`any`との関係
- dynamic値の生成境界
- dynamic値をstatic型として使用する条件
- runtime castの挿入
- cast成功後の型
- cast failureの分類
- cast evidenceの基本構造
- record、variant、function、abstract typeのcast方針
- effectful function castの制限
- polymorphismとdynamic境界
- gradual guaranteeの設計目標
- 数値promotionとdynamic castの順序

次はRPX v1のbounded dynamicには含めず、将来拡張または別の`OPEN-*`へ移管する。

- 一般的なblame polarityとblame theorem
- polymorphic dynamic
- dynamic値から`forall`型へのcast
- gradual effect-row cast
- runtime effect monitoring
- dynamic handler cast
- dynamic resumption cast
- scoped／affine capabilityのdynamic化
- 一般的なanswer-type modification
- cast failureと例外effectの統一
- space-efficient cast calculusの完全な最適化規則
- gradual guaranteeの形式証明

---

### `DD-TYP-DYN-001`: static型とgradual型の分離

`確定`

RPXの型を、概念上次の二つに分ける。

```text
S ::= static type

G ::= S
    | dynamic S
````

`S`はstatic semantic typeであり、少なくとも次を含む。

```text
S ::= any
    | never
    | base-type
    | union(S+)
    | intersect(S+)
    | not(S)
    | diff(S, S)
    | function-type
    | record-type
    | variant-type
    | type-variable
    | recursive-type
```

`dynamic S`は、static型`S`を上限として持つgradual型である。

`dynamic S`は次のどちらとも型等価ではない。

```text
dynamic S ≄ S
dynamic S ≄ any
```

static fragmentにおけるsemantic subtypingおよび型等価は、従来どおり値集合によって
定義する。

```text
S <: T
iff
[[S]] ⊆ [[T]]
```

```text
S ≃ T
iff
S <: T and T <: S
```

`dynamic S`はstatic semantic typeの値集合演算へ直接混入させない。

特に、次のような演算をstatic型代数の通常演算として定義してはならない。

```text
not(dynamic S)
diff(T, dynamic S)
```

dynamicを含むcompatibility、precision、cast insertionは、static semantic subtypingとは
別の判断として定義する。

***

### `DD-TYP-DYN-002`: `dynamic S`の意味

`確定`

```text
dynamic S
```

は、次の意味を持つ。

> 実行時値がstatic型`S`の意味領域に属することは保証されているが、
> その値をどの程度精密なstatic型として使用できるかは、runtime情報に依存する。

したがって、次を不変条件とする。

```text
x : dynamic S
ならば
runtime value of x ∈ [[S]]
```

`dynamic S`は「おそらく`S`」または「未検証の値」を意味しない。

型保証のない値を`dynamic S`へ導入する場合は、dynamic値を生成する境界で、
値が実際に`S`へ属することを検証しなければならない。

例：

```text
dynamic(union(number, str))
```

は、実行時値が次のいずれかであることを保証する。

```text
int
f64
str
```

この上限と交わらない型として値を利用しようとした場合は、runtimeまで待たず
静的に拒否する。

***

### `DD-TYP-DYN-003`: static top型`any`

`確定`

RPXは、static型代数のtop型として`any`を持つ。

```text
[[any]]
=
すべてのunrestricted RPX runtime value
```

すべてのunrestricted static型`S`について、次が成り立つ。

```text
S <: any
```

`any`はdynamic型ではなく、runtime castの暗黙挿入を許可しない。

```text
x : any
```

を型`T`が必要な位置で使用する場合、静的に`any <: T`が成立しない限り拒否する。

```lisp
; x : any

(+ x 1)
```

は静的エラーである。

これに対して、

```text
x : dynamic any
```

を数値として使用する場合は、runtime narrowingを挿入できる。

```text
any
≠
dynamic any
```

`any`はsemantic negationおよびdifferenceの基準となるstatic universeである。

```text
union(T, not(T)) ≃ any
intersect(T, not(T)) ≃ never
diff(any, T) ≃ not(T)
```

RPX v1の`any`が含むのは、通常のunrestricted valueだけとする。

次は、一般の`any`または`dynamic any`へupcastできない。

* affine resumption
* named/scoped effect evidence
* handler instance frame
* 局所resource capability
* private runtime authority
* raw foreign pointer
* その他、scopeまたは使用回数の制約を消去すると安全性が失われる値

これらは通常のunrestricted value universeの外側にある、専用の型付きcapabilityとして
扱う。

`any`という名称は表面型名として採用する。ただし、`any`が型検査回避機構ではなく
static top型であることを仕様と診断で明示する。

***

### `DD-TYP-DYN-004`: `never`およびdynamicの正規形

`確定`

`never`はstatic型代数の空型とする。

```text
[[never]] = ∅
```

すべてのstatic型`S`について次が成り立つ。

```text
never <: S
union(S, never) ≃ S
intersect(S, never) ≃ never
```

`dynamic S`の上限`S`はstatic型だけに限定する。

次の型は表面型として認めない。

```text
dynamic(dynamic S)
```

複数のdynamic境界が連続した場合、型表現は一重の`dynamic S`へ正規化するが、
各境界のprovenanceは保持する。

```text
型:
dynamic S

境界履歴:
boundary-1
→ boundary-2
```

`dynamic never`には正常なruntime値が存在しないため、次へ正規化する。

```text
dynamic never
≃
never
```

***

### `DD-TYP-DYN-005`: dynamic値をstatic型として使用する三段階判定

`確定`

```text
x : dynamic S
```

を、static型`T`が必要な位置で使用するとき、次の三段階で判定する。

#### 1. 上限全体が要求型へ含まれる場合

```text
S <: T
```

なら、runtime checkなしで使用を許可する。

```text
dynamic int
→ expected number
```

では、

```text
int <: number
```

なので、runtime narrowingは不要である。

#### 2. 上限と要求型が互いに素である場合

```text
intersect(S, T) ≃ never
```

なら、runtime checkは必ず失敗するため静的に拒否する。

```text
dynamic(union(number, str))
→ expected picture
```

かつ、

```text
intersect(union(number, str), picture)
≃ never
```

なら静的エラーとする。

#### 3. 一部だけ重なる場合

次を共に満たす場合、

```text
S </: T
intersect(S, T) ≄ never
```

runtime checkを伴うcastをtyped Coreへ挿入する。

```text
dynamic(union(number, str))
→ expected number
```

では、実行時値が`number`なら成功し、`str`なら失敗する。

この判定をまとめると次のとおりである。

```text
S <: T
  → runtime checkなし

intersect(S, T) ≃ never
  → 静的エラー

それ以外
  → runtime castを挿入
```

***

### `DD-TYP-DYN-006`: cast成功後の型

`確定`

```text
dynamic S
```

をstatic型`T`へruntime checkし、検査に成功した場合、結果のstatic型は単なる`T`ではなく
次とする。

```text
intersect(S, T)
```

例：

```text
source bound:
union(int, str)

required type:
number
```

の場合、

```text
intersect(union(int, str), number)
≃ int
```

なので、cast成功後の値は`number`ではなく、より精密な`int`として扱える。

これは、元の上限が持っていたstatic情報をcastによって失わないためである。

利用箇所では次が成り立つ。

```text
intersect(S, T) <: T
```

したがって、要求型`T`の値として安全に使用できる。

***

### `DD-TYP-DYN-007`: occurrence typingとの関係

`確定`

認識可能な型判定により、dynamic値の型をbranch内で絞り込む。

```text
x : dynamic S
```

について、型`T`への所属判定を行った場合、then branchでは、runtime検査が成功済みであるため
static型へ移行する。

```text
then branch:
x : intersect(S, T)
```

else branchでは、`T`でないことは分かるが、残余範囲内の正確な型は未確定であるため、
原則としてdynamicを維持する。

```text
else branch:
x : dynamic(diff(S, T))
```

`diff(S, T) ≃ never`なら、else branchは到達不能である。

例：

```text
x : dynamic(union(number, str))
```

```lisp
(if (number? x)
    (+ x 1)
    (str-length x))
```

branch環境は概念的に次となる。

```text
then:
x : number

else:
x : dynamic str
```

残余範囲が単一のruntime-check済み型として利用可能であることを型検査器が証明できる場合は、
else側もstatic型へ精密化してよい。

ただし、可変変数、effectful expression、property pathおよび再評価によって値が変化し得る
場所には、この単純な絞り込みを適用しない。

***

### `DD-TYP-DYN-008`: static値からdynamic値への導入

`確定`

static型`T`の値を`dynamic S`へ導入できる条件は次である。

```text
T <: S
```

概念的なCore項を次とする。

```text
to-dynamic<S>(value)
```

型付けの概念形：

```text
value : T
T <: S
────────────────────────────
to-dynamic<S>(value) : dynamic S
```

この変換は失敗しない。

元の値が型検査済みであり、`T <: S`が静的に証明されているため、runtime membership
checkは不要である。

次の変換は静的に拒否する。

```text
T </: S
```

特に、static値からdynamic値への導入を、値の型を狭めるchecked castとして使用しては
ならない。

```text
to-dynamic:
失敗しないdynamic境界の生成

checked cast:
失敗し得る型のnarrowing
```

この二つを区別する。

***

### `DD-TYP-DYN-009`: dynamic上限のwidening

`確定`

```text
x : dynamic S
```

について、

```text
S <: U
```

なら、runtime checkなしで次へwidenできる。

```text
dynamic S
→ dynamic U
```

概念的なCore項：

```text
widen-dynamic<U>(x)
```

この変換は元のdynamic boundary identityおよびprovenanceを保持する。

これに対して、

```text
dynamic S
→ dynamic U
```

で`S <: U`が成立しない場合は、暗黙のwideningとして認めない。

上限を狭める必要がある場合は、型判定またはchecked castを使用する。

checked castに成功した値は、原則として狭い`dynamic U`ではなく、確認済みのstatic型
`intersect(S, U)`として扱う。

***

### `DD-TYP-DYN-010`: foreign値のdynamic導入

`確定`

型保証のないforeign runtime valueを`dynamic S`へ導入する場合、境界で`S`へのruntime
validationを必須とする。

概念的な操作：

```text
import-dynamic<S>(foreign-value)
```

型：

```text
ForeignValue
→ Result<dynamic S, boundary-error>
```

型`S`がruntime検査可能であることを次の判断で表す。

```text
RuntimeCheckable(S)
```

型付けの概念形：

```text
foreign : ForeignValue
RuntimeCheckable(S)
────────────────────────────────────────
import-dynamic<S>(foreign)
: Result<dynamic S, boundary-error>
```

validationに成功した値だけを`dynamic S`として公開する。

validationに失敗した場合は、後段のdynamic cast failureではなく、値の導入境界における
`boundary-error`とする。

safe RPXでは、未検証のforeign valueを`dynamic S`と仮定する操作を公開しない。

```text
unsafe-assume-dynamic<S>
```

相当の機能が必要な場合は、trusted kernelまたは監査済みforeign adapterに限定する。

***

### `DD-TYP-DYN-011`: decoderとgradual foreign boundaryの分離

`確定`

外部bytesや構造化データをRPX値へ変換するdecoderと、foreign runtime objectをdynamic値として
導入する境界を区別する。

#### Static decoder

```text
decode<S>(bytes)
: Result<S, decode-error>
```

decoderは外部表現を完全に検証し、通常のstatic RPX値`S`へ再構築する。

#### Gradual foreign boundary

```text
import-dynamic<S>(foreign-value)
: Result<dynamic S, boundary-error>
```

foreign objectのidentityまたは動的表現を維持したまま、上限`S`だけを保証する。

JSON、設定ファイル、document source等の通常の外部データについては、可能な限りdecoderを使用し、
安易に`dynamic any`へ導入しない。

***

### `DD-TYP-DYN-012`: runtime-checkableな型

`確定`

RPX v1では、型`S`がruntimeで安全に検査可能である場合だけ、foreign valueから
`dynamic S`を生成できる。

直接検査可能な型の候補は次である。

* `bool`
* `unit`
* `int`
* `f64`
* `str`
* runtime tagを持つ通常のprimitive型
* runtime identityを持つvariant
* 検査可能なfieldからなるrecord
* 有限union
* 検査可能な型からなるintersection
* moduleが正規のruntime identityを提供するnominal型

wrapperによって検査可能な型の候補は次である。

* fixed-arity function
* 検査可能な要素型を持つ一部のcollection
* foreign adapterが正規のcontractを提供するcallable

一般のforeign境界から生成できない型は次である。

* affine resumption
* named/scoped effect evidence
* handler instance
* private resource capability
* raw runtime authority
* moduleの認証なしでは生成できないopaque abstract type
* runtime検査不能な任意のsemantic negation
* scope identityの保持が必要な型

```text
¬RuntimeCheckable(S)
```

の場合、

```text
import-dynamic<S>(foreign)
```

を静的に拒否する。

***

### `DD-TYP-DYN-013`: implicit cast failure

`確定`

型検査器が挿入したimplicit castがruntimeで失敗した場合、次の構造化された言語上の
異常終端を生成する。

```text
dynamic-type-error {
  dynamic-bound,
  required-type,
  refined-target,
  actual-runtime-type,
  introduction-boundary,
  cast-site,
  module-path,
  cast-trace
}
```

`dynamic-type-error`は通常のalgebraic effectではない。

次の扱いを採用する。

* effect rowへ追加しない
* 通常のRPX handlerからcatchできない
* Rust panicにしない
* undefined behaviorにしない
* compiler/runtime defectとは分類しない
* gradual typingによって許容された明示的な異常終端とする

CLIでは、cleanup後に構造化診断を表示し、非zero statusで終了する。

GUIでは、失敗した評価runを停止し、次を行う。

* cleanupを実行する
* そのrunのsource transactionをrollbackする
* 途中のRenderIRを破棄する
* last-good-renderを保持する
* introduction boundaryとcast siteの両方を診断表示する

***

### `DD-TYP-DYN-014`: 明示的safe cast

`確定`

cast失敗を正常な分岐として処理したい場合は、implicit castではなく明示的なsafe castを
使用する。

単純に成否だけが必要な場合：

```text
try-cast<T> :
dynamic S
→ Option<intersect(S, T)>
```

失敗理由が必要な場合：

```text
check-cast<T> :
dynamic S
→ Result<intersect(S, T), cast-mismatch>
```

`intersect(S, T) ≃ never`である場合、safe castも成功不能なので静的に拒否してよい。

implicit castとexplicit safe castを区別する。

```text
implicit cast failure:
dynamic-type-errorでrunを停止

explicit safe cast failure:
OptionまたはResultとして利用者コードが処理
```

認識可能な型判定とoccurrence typingで十分な場合は、safe castより型判定を優先できる。

***

### `DD-TYP-DYN-015`: cast evidence

`確定`

すべてのimplicit castを、typed Coreの明示的な`Cast`項へelaborateする。

```text
Cast(expression, evidence, cast-id)
```

cast evidenceは、runtime検査および成功後の型保証を表す内部証拠である。

conceptual evidence algebraを次のように定める。

```text
Evidence ::=
    Identity
  | Widen
  | TagCheck
  | UnionCheck
  | IntersectionCheck
  | RecordCheck
  | VariantCheck
  | FunctionGuard
  | NominalCheck
  | Compose
```

それぞれの意味は次のとおりである。

#### `Identity`

runtime検査が不要なcast。

#### `Widen`

dynamic上限の安全な拡張。

#### `TagCheck`

primitiveまたはruntime tagによる直接検査。

#### `UnionCheck`

複数候補のいずれかに属することの純粋な検査。

#### `IntersectionCheck`

すべての構成型の条件を満たすことの検査。

#### `RecordCheck`

必要field、field型およびrow条件の検査。

#### `VariantCheck`

constructor identityおよびpayload型の検査。

#### `FunctionGuard`

関数呼出時に引数および結果を検査するwrapper。

#### `NominalCheck`

moduleまたは型所有者が発行したruntime type identityによる検査。

#### `Compose`

二つのevidenceを順番に適用する意味的合成。

evidenceは通常のRPX値として公開しない。

次を提供しない。

* evidenceの取得
* evidenceの等値比較
* evidenceのhash
* evidenceのserialization
* private nominal identityの観測
* evidenceを利用したmodule sealingの回避

***

### `DD-TYP-DYN-016`: cast evidenceの純粋性

`確定`

implicit cast evidenceの実行は純粋でなければならない。

implicit castは次を発生させてはならない。

* I/O
* Resource operation
* FileRead／FileWrite
* Network
* Random
* Clock
* SourceEdit
* 利用者定義の非制限effect
* handler stackの観測可能な変更

castは、次のいずれかだけを行う。

1. 値をそのまま返す。
2. runtime型または構造を検査する。
3. guarded wrapperを生成する。
4. `dynamic-type-error`で終了する。

型注釈または型精度の変更だけで、成功実行のeffect traceを変えてはならない。

***

### `DD-TYP-DYN-017`: evidence compositionと最適化

`確定`

連続するcastの規範意味論は、まず`Compose`による逐次適用として定義する。

```text
Compose(e1, e2)
```

は、`e1`を適用し、成功した場合に`e2`を適用する。

implementationは、次をすべて保存できる場合だけevidenceを簡約してよい。

* 成功する値の集合
* 失敗する値の集合
* 成功後の型
* 値identityに関する仕様
* effect trace
* module abstraction
* dynamic導入境界
* cast使用箇所
* 利用者向け診断

許可される代表的な簡約は次である。

```text
Compose(Identity, e)
→ e
```

```text
Compose(e, Identity)
→ e
```

```text
Widen(S, T)
;
Widen(T, U)
→
Widen(S, U)
```

ただし、中間境界のprovenanceを失ってはならない。

一次値について、最終検査と診断履歴を保存できる場合は、連続するtag checkを最終的な
より狭い検査へ縮約してよい。

function evidenceは、RPX v1の規範意味論では順次wrapperとして維持する。

実装がfunction wrapperを融合する場合は、引数検査、結果検査、失敗境界および
呼出回数を保存しなければならない。

***

### `DD-TYP-DYN-018`: cast provenance

`確定`

runtime検査用evidenceと、利用者診断用のcast provenanceを分離する。

```text
ExecutableEvidence {
  runtime検査に必要な最小情報
}
```

```text
CastProvenance {
  dynamic導入境界,
  中間widening,
  narrowing箇所,
  使用箇所,
  source type,
  target type,
  module path,
  source span
}
```

evidenceを最適化しても、cast provenanceを失ってはならない。

dynamic値の導入境界および各narrowing使用箇所には、一意なboundary IDまたはcast IDを
付与する。

private module内の型identityおよびrepresentationは、診断表示時にauthorityに従って
隠蔽する。

***

### `DD-TYP-DYN-019`: recordおよびvariant cast

`確定`

不変recordについては、runtimeで次を検査できる。

* 値がrecordであること
* 必須fieldの存在
* 各field値の型
* open／closed row条件
* nominalまたはstructural identityの必要条件

open record型では、指定されていない追加fieldを許す。

closed record型に追加fieldを許すかどうかは`ROW-001`のclosed record意味論に従う。

recordが通常の不変RPX値である場合、一度検査したfield evidenceを再利用してよい。

foreign mutable objectをrecordとして公開する場合は、導入時の一度の検査だけで将来の
field型を保証してはならない。必要ならproxyまたはfield access guardを使用する。

variant castは、constructorの文字列名ではなく、一意なconstructor identityで検査する。

payloadを持つconstructorでは、payloadについて対応するevidenceを適用する。

***

### `DD-TYP-DYN-020`: opaque abstract typeのcast

`確定`

opaque abstract typeは構造によってcastしてはならない。

異なるsealed moduleが所有するabstract typeは、内部表現が同じであっても別型である。

```text
module-a.t
≠
module-b.t
```

dynamic castでは、型所有者が発行した非公開のnominal runtime identityを使用する。

```text
NominalCheck(abstract-type-id)
```

一般のforeign boundaryまたはstructural validatorは、opaque abstract typeの値を
新しく生成できない。

abstract typeの正規の値を生成できるのは、少なくとも次に限る。

* 型所有moduleのconstructor
* 型所有moduleの認証済みdecoder
* 型所有moduleが発行したruntime evidence
* 許可されたmodule-private foreign adapter

dynamic cast、provenanceおよび診断は、abstract typeのprivate representationを
外部へ漏らしてはならない。

***

### `DD-TYP-DYN-021`: fixed-arity function cast

`確定`

RPX v1は、具体的な単相fixed-arity function型へのdynamic castを許す。

source function型を次とする。

```text
(A1, ..., An) ->{Es} R
```

target function型を次とする。

```text
(B1, ..., Bn) ->{Et} U
```

function castを許可する基本条件は次である。

1. sourceとtargetがfixed-arity functionである。
2. arityが等しい。
3. 各`Bi`から`Ai`へのcastが定義可能である。
4. `R`から`U`へのcastが定義可能である。
5. latent effect rowが静的に判明している。
6. `Es`のeffect要求を`Et`が満たす。
7. scoped、affineまたはprivate capabilityをdynamic境界から漏らさない。

関数値をcastした時点では、次を確認する。

* 値がcallableである
* arityが一致する
* 必要なfunction metadataまたはadapterが存在する
* effect contractが静的に適合する

引数および結果の型はfunction call時にguardする。

***

### `DD-TYP-DYN-022`: function引数の反変cast

`確定`

function guardでは、target側から受け取った各引数をsource functionの引数型へcastする。

```text
target argument:
Bi

source function requires:
Ai

cast direction:
Bi → Ai
```

これは関数引数の反変性に対応する。

概念的なwrapper：

```lisp
(fn (b1 ... bn)
  (let ((a1 (cast b1 a1-type))
        ...
        (an (cast bn an-type)))
    (source-function a1 ... an)))
```

`Bi <: Ai`ならruntime checkは不要である。

```text
intersect(Bi, Ai) ≃ never
```

なら、その関数castは成功不能なので静的に拒否する。

それ以外の場合は、実際の関数呼出時にruntime checkを行う。

***

### `DD-TYP-DYN-023`: function結果の共変cast

`確定`

source functionの戻り値は、source result型`R`からtarget result型`U`へcastする。

```text
cast direction:
R → U
```

これは関数結果の共変性に対応する。

概念的なwrapper：

```lisp
(fn (arguments...)
  (let ((source-result
          (source-function converted-arguments...)))
    (cast source-result target-result-type)))
```

`R <: U`ならruntime checkは不要である。

```text
intersect(R, U) ≃ never
```

なら、関数castを静的に拒否する。

それ以外の場合は、source functionが戻った時点でruntime checkを行う。

引数または結果のguardが失敗した場合は`dynamic-type-error`とする。

***

### `DD-TYP-DYN-024`: function arityの制限

`確定`

RPX v1のdynamic function castは、固定個数の位置引数を持つ関数だけを対象とする。

```text
() -> T
(A) -> T
(A, B) -> T
...
```

dynamic function castではarityの一致を必須とする。

次はRPX v1のdynamic function cast対象外とする。

* 可変長引数
* optional引数
* keyword引数
* overload
* 自動currying変換
* 自動partial application
* pattern parameter
* 動的に変化するforeign calling convention

これらを追加する場合は、call signatureとwrapper semanticsを個別に規定する。

***

### `DD-TYP-DYN-025`: effectful function cast

`確定`

functionの値型引数および結果型にはdynamic castを許すが、latent effect rowはruntime cast
しない。

source functionのeffect rowを`Es`、target functionが許容するeffect rowを`Et`とする。

function castを許可するには、両方のrowが静的に判明し、次が成立しなければならない。

```text
Es ⊑ Et
```

ここで`⊑`は、source functionのeffect要求をtarget側のambient effect環境が満たせることを
表す。

例：

```text
source:
number ->{resource} str

target:
number ->{resource, log} str
```

は許可できる。

一方、次は拒否する。

```text
source:
number ->{resource, file-write} str

target:
number ->{resource} str
```

`file-write`がtarget側の許容範囲に含まれないためである。

effect rowが不明な関数を、具体的なeffectful function型へcastしてはならない。

```text
dynamic effect row
→ static effect row
```

のruntime監視およびgradual effect castは、RPX v1には導入しない。

参照実装は段階的に実装してよい。

```text
段階1:
pure fixed-arity function

段階2:
静的に既知のeffect rowを持つfixed-arity function

将来:
gradual effect typing
```

***

### `DD-TYP-DYN-026`: dynamic境界を通れない制御値

`確定`

次の値は、通常のfunctionまたは`dynamic any`として扱ってはならない。

* handler value
* handler instance
* affine resumption
* named/scoped effect evidence
* private resource capability
* source transaction authority
* module-private runtime authority

特に、次を禁止する。

```text
Resume<R, B, E>
→ dynamic any
```

```text
dynamic function
→ Resume<R, B, E>
```

```text
Evidence<L, s>
→ dynamic any
→ scope外へescape
```

handler valueはfirst-classだが、通常のdynamic function castの対象ではない。

handler専用dynamic castはRPX v1には導入しない。

***

### `DD-TYP-DYN-027`: polymorphismとdynamic境界

`確定`

RPX v1では、多相値を多相性を維持したままdynamic境界へ通さない。

rank-1 polymorphic valueをdynamic境界へ渡す場合、境界が要求する具体的な単相型へ
instantiateする。

例：

```text
identity :
forall a. a -> a
```

を次の境界へ渡す場合、

```text
dynamic(int -> int)
```

まず次へinstantiateする。

```text
int -> int
```

その後で通常のstatic injectionまたはfunction castを適用する。

境界型だけからinstantiationを決定できない場合、明示的な型注釈を要求する。

次を禁止する。

```text
dynamic function
→ forall a. a -> a
```

有限のruntime検査では、すべての型についての普遍量化を保証できないためである。

dynamic値から取り出せるfunction型は、具体的な単相fixed-arity function型に限定する。

effect-row polymorphicな値は、具体的なstatic EffectRowへinstantiateしてからdynamic境界へ
渡す。

record-row polymorphicな値は、具体的なRecordRowへinstantiateしてからdynamic境界へ渡す。

多相handler valueは一般のdynamic境界へ通さない。

typed Coreへ到達する前にdynamic境界を通る多相値を単相化し、runtime cast calculusは
原則として単相型だけを扱う。

***

### `DD-TYP-DYN-028`: gradual guarantee

`確定`（設計目標、未証明）

RPXは、bounded dynamicを含む型precision関係を定義する。

precision関係はsemantic subtypingとは別の判断とする。

概念的に次の記号を用いる。

```text
G1 ⊑p G2
```

これは、`G2`が`G1`以上に精密であることを表す。

RPXはstatic gradual guaranteeおよびdynamic gradual guaranteeを設計目標とする。

#### Static gradual guarantee

型情報を不精密にしたことだけを理由として、以前型検査可能だったプログラムを
不必要に拒否しないことを目標とする。

ただし、次の制約は型precisionの低下によって消去できない。

* affine resumption
* scoped evidence
* module-private nominal identity
* resource capability
* handler authority
* effect-row安全境界

#### Dynamic gradual guarantee

精密な版と不精密な版が共に正常終了する場合、観測可能な結果が等しいことを目標とする。

最低限の観測対象は次である。

* return value
* handled effect trace
* generated artifact
* committed source transaction
* external output
* resource operationの意味的順序

型精度を上げた結果として、次は許容する。

* runtime castの削除
* `dynamic-type-error`の静的検出
* より早い型エラー
* より具体的なdynamic診断

型精度を変更しただけで、両方が正常終了する実行について次を変更してはならない。

* return value
* effectの順序または回数
* handler selection
* generated artifactの意味
* committed `source-edit`
* external output
* resource操作の意味的順序

次はRPX v1のgradual guarantee対象外とする。

* polymorphic dynamic
* gradual effect-row cast
* dynamic handler cast
* dynamic resumption cast
* scoped／affine capabilityのdynamic化
* 実行時間の一致
* allocation回数の一致
* 最大memory使用量の一致
* resource exhaustionの完全な一致

gradual guarantee、blame safetyおよび通常の型安全性は、別々の性質として扱う。

これらは現時点では未証明である。

***

### `DD-TYP-NUM-001`: RPX v1の基本数値型

`確定`

RPX v1は、次の二つを別のprimitive型として持つ。

```text
int
f64
```

#### `int`

```text
int
= 任意精度の正確な符号付き整数
```

整数overflowを通常の`int`へ設けない。値の大きさは利用可能memoryによってのみ制限される。

#### `f64`

```text
f64
= IEEE 754 binary64 floating-point
```

`f64`は数学的な実数全体ではなく、有限精度の近似数値型である。

#### `number`

```text
number
≃
union(int, f64)
```

次が成り立つ。

```text
int <: number
f64 <: number
```

次は成り立たない。

```text
int <: f64
f64 <: int
```

`int`と`f64`は別のruntime値領域とする。

```text
int-value(1)
≠
f64-value(1.0)
```

数値比較によって等しい場合があっても、runtime表現型は異なる。

***

### `DD-TYP-NUM-002`: 数値promotion

`確定`

`int`から`f64`への変換はsemantic subtypingではなくnumeric promotionとする。

混合算術では、原則として`int`を`f64`へpromotionし、結果を`f64`とする。

```text
int + int
→ int

f64 + f64
→ f64

int + f64
→ f64

f64 + int
→ f64
```

同様の規則を減算、乗算、比較等へ適用できる。

任意精度`int`から`f64`へのpromotionでは、値によって精度が失われる可能性がある。
これは数値仕様および診断で明示する。

通常の除算`/`は、RPX v1では次の型を基本とする。

```text
int / int
→ f64
```

整数除算および剰余は別の`int`専用operationとして提供する。

表面名は別途決定するが、概念的には次である。

```text
int-div : int × int -> int
mod     : int × int -> int
```

`real`という型名はRPX v1では導入せず、将来のnumeric tower拡張用に予約する。

`rational`、`decimal`、`f32`、固定幅整数等は将来拡張またはlibrary型として扱う。

***

### `DD-TYP-NUM-003`: dynamic castとnumeric promotionの順序

`確定`

dynamic narrowingとnumeric promotionを一つの特殊castとして混在させない。

処理順序を次のように定める。

```text
1. dynamic値を、必要なstatic数値型へruntime narrowingする。
2. narrowingに成功した値へ通常のnumeric promotionを適用する。
```

例：

```text
x : dynamic int
expected:
f64
```

処理：

```text
1. xがint上限を満たすことを利用する。
2. int値をf64へnumeric promotionする。
```

例：

```text
x : dynamic number
expected:
f64
```

実行時値が`f64`の場合：

```text
1. f64としてnarrowing成功
2. promotion不要
```

実行時値が`int`の場合：

```text
1. number内のintとしてnarrowing
2. intからf64へnumeric promotion
```

実行時値が`number`の上限外である場合は、dynamic値の不変条件違反であり、
通常のcast failureではなく境界またはruntime defectとして扱う。

数値promotionの失敗、精度損失警告および有限性要件の詳細は、
`OPEN-SYN-002`および`LIT-001`へ移管する。

***

### `DD-NAME-001`: 組込み型名と識別子の小文字規約

`確定`

RPXの通常識別子は、namespaceにかかわらず小文字で開始する。

この規則を次へ適用する。

* value
* function
* type
* type parameter
* record row
* effect row
* module
* signature
* effect
* operation
* constructor
* syntax／macro
* package内member

組込み型の主要な表記を次とする。

```text
any
never
bool
unit
str
int
f64
number
```

複数語の識別子には`kebab-case`を推奨する。

```text
source-edit
render-ir
last-good-render
resource-id
dynamic-type-error
```

略語も小文字で表記する。

```text
svg
pdf
pptx
ir
id
uri
```

大文字開始の通常識別子は拒否する。

```lisp
(val Page ...)
```

診断例：

```text
識別子は小文字で開始する必要があります。

found:
  Page

suggestion:
  page
```

型、値、module、effect、syntax等の区別は、大文字小文字ではなくnamespaceおよび
名前解決後の`BindingId`で行う。

constructorも小文字で表記する。

```text
some
none
ok
err
true
false
```

***

### `DD-NAME-002`: namespace間の同綴り衝突

`確定`

異なるnamespaceに属する識別子は、型システム上は同じ綴りを持つことができる。

概念例：

```text
type namespace:
picture

value namespace:
picture
```

ただし、同一moduleまたは同一public APIで、複数namespaceに同じ綴りのpublic memberを
公開する場合は衝突警告を出す。

```text
warning:
public name `picture` is exported in multiple namespaces

namespaces:
  type
  value
```

この警告は、名前解決上の曖昧性がない場合でも、API検索、診断、documentationおよび
利用者の読みやすさを改善するために出す。

内部memberまたは意図的なfactory／constructor patternについて、warningを局所的に
抑制できる仕組みは将来の診断設定で定める。

namespace間同綴りを言語上の全面的なエラーにはしない。

***

### dynamic typingのCore構文

RPX v1のtyped Coreへ、概念的に次を追加する。

```text
e ::= ...
    | ToDynamic(static-bound, expression, boundary-id)
    | WidenDynamic(target-bound, expression, boundary-id)
    | Cast(expression, evidence, cast-id)
    | TryCast(expression, evidence)
    | ImportDynamic(foreign-expression, validator, boundary-id)
    | FunctionGuard(function, signature-evidence, cast-id)
```

実際のCore表現では、`FunctionGuard`を`Cast` evidenceの一種として統合してもよい。

ただし、規範的意味は次を区別しなければならない。

```text
ToDynamic:
失敗しないstatic injection

WidenDynamic:
失敗しないdynamic upper-bound widening

Cast:
失敗時にdynamic-type-errorとなるimplicit narrowing

TryCast:
失敗をOption／Resultとして返す明示的safe cast

ImportDynamic:
foreign境界で上限を検証し、失敗をboundary-errorとして返す

FunctionGuard:
call時に引数および結果を検査する高階境界
```

***

### dynamic typingの終端状態

RPXの評価結果へ、少なくとも次を追加する。

```text
EvaluationOutcome<A> =
    completed(A)
  | dynamic-type-error(DynamicTypeErrorInfo)
  | runtime-fault(RuntimeFaultInfo)
  | divergence
```

`dynamic-type-error`と`runtime-fault`を区別する。

```text
dynamic-type-error:
許可されたgradual castがruntime値に対して失敗した、
言語仕様上の明示的な異常終端

runtime-fault:
well-typed／well-linkedな実行では到達不能であるべき、
runtime、foreign boundaryまたは実装整合性の欠陥
```

`boundary-error`および明示的safe castの失敗は通常のRPX値として扱う。

***

### 適合試験

#### static injection

```text
42 : int
```

を次へ導入する。

```text
dynamic number
```

期待：

```text
成功
runtime checkなし
```

#### invalid static injection

```text
picture
→ dynamic number
```

期待：

```text
静的エラー
```

#### dynamic widening

```text
dynamic int
→ dynamic number
```

期待：

```text
成功
runtime checkなし
元のboundary provenanceを保持
```

#### safe static use

```text
x : dynamic int
```

を`number`が必要な位置で使用する。

期待：

```text
int <: number
runtime checkなし
```

#### partial overlap

```text
x : dynamic(union(number, str))
```

を`number`が必要な位置で使用する。

期待：

```text
runtime Castを挿入
```

実体が`int`または`f64`：

```text
成功
```

実体が`str`：

```text
dynamic-type-error
```

#### disjoint use

```text
x : dynamic(union(number, str))
```

を`picture`が必要な位置で使用する。

期待：

```text
静的エラー
```

#### cast precision

```text
x : dynamic(union(int, str))
```

を`number`として検査する。

期待される成功後の型：

```text
int
```

#### `any`と`dynamic any`

```text
x : any
```

```lisp
(+ x 1)
```

期待：

```text
静的エラー
```

```text
y : dynamic any
```

```lisp
(+ y 1)
```

期待：

```text
runtime number checkを挿入
```

#### foreign ingress

```text
foreign value
→ import-dynamic<number>
```

実体が`int`：

```text
ok(dynamic number)
```

実体が`picture`：

```text
err(boundary-error)
```

#### implicit cast failure

```text
x : dynamic(union(number, str))
actual x = "hello"
```

```lisp
(+ x 1)
```

期待：

```text
dynamic-type-error
```

通常のeffect handlerによるcatch：

```text
不可
```

#### explicit safe cast

```text
try-cast<number>(x)
```

実体が`str`：

```text
none
```

評価run自体は継続する。

#### fixed-arity function cast

```text
source:
int ->{} str

target:
number ->{} str
```

期待：

```text
FunctionGuardを生成
number引数をcall時にintへ検査
```

`f64`を渡した場合：

```text
dynamic-type-error
```

#### function result cast

```text
source:
unit ->{} dynamic(union(number, str))

target:
unit ->{} number
```

期待：

```text
source結果をnumberとして検査
```

#### effect-compatible function cast

```text
source:
number ->{resource} str

target:
number ->{resource, log} str
```

期待：

```text
受理
```

#### effect-incompatible function cast

```text
source:
number ->{resource, file-write} str

target:
number ->{resource} str
```

期待：

```text
静的エラー
```

#### polymorphic value boundary

```text
identity:
forall a. a -> a
```

境界：

```text
dynamic(int -> int)
```

期待：

```text
a := intとしてinstantiate
単相化後にdynamic境界へ導入
```

境界：

```text
dynamic function
```

期待：

```text
具体的なinstantiationを決定できないため静的エラー
```

#### dynamicからforall

```text
dynamic function
→ forall a. a -> a
```

期待：

```text
静的エラー
```

#### numeric promotion

```text
x : dynamic number
actual x = int-value(2)
expected = f64
```

期待：

```text
number内のintとして確認
intからf64へpromotion
result = 2.0 : f64
```

#### namespace collision warning

同一moduleが次をpublic exportする。

```text
type namespace:
picture

value namespace:
picture
```

期待：

```text
名前解決は成功
public namespace collision warningを出す
```

***

### 未解決事項の移管

`OPEN-TYP-001`を閉じるため、次を別項目へ移管する。

#### `OPEN-TYP-002`

* static semantic subtypingの実装可能な決定手続き
* emptiness判定の正確な範囲
* cast挿入が依存するalgorithmic approximation
* type／row／effect constraint solverの完全性
* checker limitationによる保守的拒否
* principal solutionの有無

#### `OPEN-SYN-002`／`LIT-001`

* `int`および`f64`の完全な字句文法
* 数値suffix
* hexadecimal／binary literal
* underscore separator
* NaN／Infinityの字面
* signed zero
* 浮動小数点の丸め
* `f64` equalityおよびhash
* intからf64への精度損失診断
* zero division
* unit literalのtokenization
* `i64`等の固定幅数値型

#### `OPEN-ERR-001`

* `dynamic-type-error`と他の異常終端の統合分類
* GUI／CLI診断の最終形式
* cleanup中にdynamic failureが発生した場合
* 複数failureの合成
* exception effectとの関係

#### `OPEN-MOD-001`

* abstract typeのruntime nominal identity
* separate compilationをまたぐidentity安定性
* plugin境界でのabstract type evidence
* signatureへruntime-checkabilityを記述する方法

#### `OPEN-KER-001`

* ForeignValue ABI
* runtime validator ABI
* trusted foreign adapter
* unsafe dynamic assumptionの権限
* foreign function effect契約の監査

#### 将来拡張

* polymorphic dynamic
* gradual effect typing
* handler cast
* resumption cast
* runtime effect monitor
* formal blame calculus
* blame theorem
* gradual guaranteeの形式証明
* space-efficient coercion calculus

***

### 解決後の基本原則

```text
static top type:
any

static bottom type:
never

basic numeric types:
int
f64

numeric union:
number = int | f64

gradual type:
dynamic S
```

```text
dynamic SをTとして使用:

S <: T
  → checkなし

intersect(S, T) ≃ never
  → 静的エラー

それ以外
  → runtime Cast
```

```text
static Tからdynamic S:

T <: S
  → 安全な導入

それ以外
  → 静的エラー
```

```text
foreign valueからdynamic S:

RuntimeCheckable(S)
  → runtime validation

validation失敗
  → boundary-error
```

```text
implicit cast失敗:
dynamic-type-error

explicit safe cast失敗:
OptionまたはResult
```

```text
関数cast:

引数:
target → source

結果:
source → target

effect row:
runtime castせず、静的包含を要求
```

以上により、RPX v1のbounded dynamic、cast insertion、runtime failure、
function guard、polymorphism境界および基本的なgradual guaranteeの範囲が特定されたため、
`OPEN-TYP-001`を解決済みとする。

## `OPEN-BND-001` 再帰束縛、局所可変状態、一般化制約

### 状態

`解決済み`

本決定は、RPX v1における次の事項を固定する。

- 通常の不変束縛
- `letrec`の対象となる右辺
- 自己再帰と相互再帰
- 再帰関数の型推論
- `(type ...)`による明示的な型注釈
- 多相再帰の扱い
- `var`のscope、読出し、更新
- closureによる局所`var`の捕捉
- local state identityのescape制約
- one-shot resumptionと局所state
- `val`、`let`および`letrec`におけるvalue restriction

次はRPX v1の基本束縛規則には含めず、別項目または将来拡張へ移管する。

- 任意式による再帰束縛
- 再帰的recordおよび無限データ構造
- recursive module
- 多相再帰
- relaxed value restriction
- escape可能なheap cell／reference
- 一般的なnon-escaping callback型
- affine kind system
- multi-shot resumption下の局所state
- closure環境およびheap cellの具体的memory管理

---

### `DD-BND-001`: 通常の不変束縛

`確定`

RPXでは、値の不変束縛に`val`を使用する。

```lisp
(val radius 40mm)
````

関数も通常の値として`val`へ束縛する。

```lisp
(val add
  (fn (x y)
    (+ x y)))
```

named function sugarを提供する場合、次の形式を許してよい。

```lisp
(val (add x y)
  (+ x y))
```

これはhygienicに次へelaborateする。

```lisp
(val add
  (fn (x y)
    (+ x y)))
```

`define`は最終Surface構文として使用しない。

Coreの不変な局所束縛は、単一bindingの`let`として表す。

```text
let(x, initializer, body)
```

評価順序は次のとおりである。

1. `initializer`を現在の環境で評価する。
2. 評価結果を`x`へ束縛する。
3. `x`が見える環境で`body`を評価する。

新しく導入される`x`は、自身の`initializer`から参照できない。

```lisp
(let ((x initializer))
  body)
```

`initializer`内に同名の`x`が現れた場合、それは外側のbindingを参照する。
外側に対応するbindingがなければ未束縛参照となる。

***

### `DD-BND-002`: sequentialなSurface `let`

`確定`

Surfaceの`let`は一個以上のbindingを持つことができる。

```lisp
(let ((x 10)
      (y (+ x 1)))
  (+ x y))
```

Surfaceの複数binding `let`はsequentialとする。

先行するbindingは、後続するinitializerおよび`let`本体から参照できる。

上記は、単一binding Core `let`の入れ子へ左から右にelaborateする。

```lisp
(let ((x 10))
  (let ((y (+ x 1)))
    (+ x y)))
```

parallel bindingはRPX v1の通常の`let`には採用しない。

必要になった場合は、通常の`let`と異なる明示的構文またはlibrary macroとして
導入する。

同一のSurface `let` binding list内では、同じ名前のbinderを重複してはならない。

意図的なshadowingは明示的な入れ子で表現する。

***

### `DD-BND-003`: `(type ...)`による型注釈

`確定`

RPXでは、値bindingおよび関数bindingの明示的な型注釈を、
独立した`(type ...)`形式で記述する。

基本形は次のとおりである。

```lisp
(type name type-expression)
```

例：

```lisp
(type radius length)

(val radius 40mm)
```

関数の型注釈：

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

named function sugarと組み合わせる場合も、型注釈は独立した`type` formとする。

```lisp
(type add
  (fn (int int) int))

(val (add x y)
  (+ x y))
```

次のようなbinder内埋込み型注釈は、RPX v1の標準構文には採用しない。

```lisp
; 採用しない
(val add
  : (fn (int int) int)
  (fn (x y)
    (+ x y)))
```

```lisp
; 採用しない
(fn ((x : int) (y : int)) ...)
```

`(type ...)`は、同じlexical scopeにある対応する値bindingの型を宣言する。

型annotationと値bindingは、名前解決後の同一`BindingId`へ対応づける。

単なる名前文字列の一致だけで関連付けてはならない。

***

### `DD-BND-004`: 型注釈のscopeと対応関係

`確定`

`(type name type-expression)`は、その型宣言が属するlexical declaration group内の
対応する値bindingを注釈する。

top-level例：

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

局所例：

```lisp
(let-declarations
  ((type increment
     (fn () int))

   (val increment
     (fn ()
       1)))
  (increment))
```

`let-declarations`という名称および正確なSurface構文は`OPEN-SYN-002`または
束縛構文の後続決定で確定する。

規範上必要なのは、局所scopeにおいても次の二種類の宣言を同じdeclaration groupへ
置けることである。

```text
type annotation declaration
value binding declaration
```

`type`宣言だけが存在し、対応する値bindingが存在しない場合は静的エラーとする。

```lisp
(type missing-value int)
```

診断例：

```text
型注釈に対応する値bindingがありません。

name:
  missing-value
```

同一declaration group内で、一つの値bindingに複数の互換でない型注釈を与えてはならない。

```lisp
(type value int)
(type value str)

(val value 1)
```

は静的エラーとする。

同一の型注釈を重複して記述することも、原則としてduplicate declaration errorとする。

***

### `DD-BND-005`: 型注釈は検査される

`確定`

`(type ...)`は処理系への無検査の仮定ではない。

値bindingに型注釈が存在する場合、型検査器はbinding initializerをその型に対して検査する。

```lisp
(type value int)

(val value 42)
```

は受理する。

```lisp
(type value str)

(val value 42)
```

は静的エラーとする。

概念的な規則：

```text
Γ ⊢ annotation-type : type
Γ ⊢ initializer ⇐ annotation-type ! E
──────────────────────────────────────
bindingはannotation-typeを持つ
```

型注釈は、initializerの実際の型と矛盾してはならない。

subtypingを許可する文脈では、initializerの型がannotation typeのsubtypeであれば
受理できる。

型注釈にdynamic boundaryが含まれる場合は、通常のbounded dynamic cast規則に従う。

型注釈は、expansive expressionを構文的valueへ変換しない。

したがって、型注釈を付けてもvalue restrictionを迂回できない。

***

### `DD-BND-006`: `letrec`の対象

`確定`

RPX v1は、再帰関数を定義するための`letrec`を持つ。

`letrec`の各値bindingの右辺は、Surface elaboration後に構文的な`fn`でなければならない。

```lisp
(letrec ((factorial
           (fn (n)
             (if (= n 0)
                 1
                 (* n (factorial (- n 1)))))))
  (factorial 5))
```

次のような任意式による再帰束縛は認めない。

```lisp
(letrec ((x (+ x 1)))
  x)
```

```lisp
(letrec ((x x))
  x)
```

次も、initializerの結果型が関数であるという理由だけでは認めない。

```lisp
(letrec ((f (make-function f)))
  ...)
```

`make-function`の評価中に再帰bindingを参照する可能性があり、
初期化前参照を防げないためである。

RPX v1の`letrec`が直接扱うのは、再帰関数および相互再帰関数だけとする。

次は対象外とする。

* 一般の再帰的な値
* 再帰的record
* 再帰的handler value
* effectful initializer
* strictな無限データ構造
* 初期化前に値を読み出す再帰定義

***

### `DD-BND-007`: `letrec`内の型注釈

`確定`

再帰関数についても、型注釈は独立した`(type ...)` formで記述する。

自己再帰関数の概念例：

```lisp
(letrec
  ((type factorial
     (fn (int) int))

   (val factorial
     (fn (n)
       (if (= n 0)
           1
           (* n (factorial (- n 1)))))))
  (factorial 5))
```

相互再帰の概念例：

```lisp
(letrec
  ((type even?
     (fn (int) bool))

   (type odd?
     (fn (int) bool))

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
  (even? 10))
```

`letrec`内のdeclaration groupは、少なくとも次を含められる。

```text
type annotation declaration
recursive function value declaration
```

型注釈は省略可能である。

無注釈の場合、型検査器が再帰関数の型を推論する。

一部の関数だけに型注釈を付けることも許す。

```lisp
(letrec
  ((type parse-block
     (fn (block-token) block))

   (val parse-block
     (fn (token)
       ...))

   (val parse-inline
     (fn (token)
       ...)))
  ...)
```

この場合、`parse-block`は注釈型に対して検査し、
`parse-inline`の型はグループ全体の制約から推論する。

***

### `DD-BND-008`: `letrec`の実行意味

`確定`

`letrec`は、すべての再帰関数が同じ再帰環境を捕捉するように評価する。

概念的には次の循環環境を構成する。

```text
ρrec =
ρ[
  f1 ↦ closure(ρrec, parameters1, body1),
  ...
  fn ↦ closure(ρrec, parametersn, bodyn)
]
```

評価手順は次のとおりである。

1. 再帰関数グループ用の環境を作る。
2. 各関数について、その再帰環境を捕捉するclosureを作る。
3. すべての再帰binderを対応するclosureで初期化する。
4. すべての初期化完了後に`letrec`本体を評価する。

`fn`を評価してclosureを生成する時点では、関数本体を評価しない。

したがって、well-formedなRPX v1の`letrec`では、利用者コードから
未初期化bindingを観測できない。

***

### `DD-BND-009`: 相互再帰

`確定`

RPX v1は、一つの`letrec` declaration group内で相互再帰を許す。

```lisp
(letrec
  ((val even?
     (fn (n)
       (if (= n 0)
           true
           (odd? (- n 1)))))

   (val odd?
     (fn (n)
       (if (= n 0)
           false
           (even? (- n 1))))))
  (even? 10))
```

同一グループ内のすべての再帰値binderは、次から参照できる。

* グループ内のすべての再帰関数本体
* `letrec`本体

型注釈宣言も、同じdeclaration group内の対応する値bindingを前方参照できる。

相互再帰を、関数内関数またはdispatcher関数への手動変換で代用させない。

一関数だけを含む`letrec`は通常の自己再帰となる。

moduleをまたぐ相互再帰とrecursive moduleは`OPEN-MOD-001`へ移管する。

***

### `DD-BND-010`: 再帰関数の型推論

`確定`

無注釈の再帰グループでは、型検査器は各再帰関数へfreshな単相metavariableを割り当てる。

概念例：

```text
even? : a -> b
odd?  : c -> d
```

その後、グループ内のすべての関数本体を同じ仮型環境で検査する。

```text
Γ,
even? : a -> b,
odd?  : c -> d
⊢ even-body

Γ,
even? : a -> b,
odd?  : c -> d
⊢ odd-body
```

生成された制約をグループ単位で解く。

再帰グループの検査中、各再帰binderは単相として扱う。

グループの検査完了後に限り、通常のvalue restrictionとgeneralization規則に従って
rank-1一般化を行うことができる。

型検査器が型を決定できない場合、必要な関数への`(type ...)`注釈を要求する。

診断例：

```text
この再帰グループの型を局所推論だけでは決定できません。

次のbindingへ型注釈を追加してください:
  parse-inline

例:
  (type parse-inline
    (fn (inline-token) inline-result))
```

***

### `DD-BND-011`: 多相再帰の禁止

`確定`

RPX v1は多相再帰を許可しない。

再帰グループ内では、すべての再帰呼出しが同じ単相型を共有する。

```text
再帰グループ内部:
monomorphic recursion

再帰グループ検査完了後:
許可されたrank-1 generalization
```

`(type ...)`による明示的な型注釈を付けても、多相再帰は有効化しない。

多相再帰は将来拡張として保留する。

***

### `DD-BND-012`: `var`の基本意味

`確定`

RPX v1の`var`は、escape不能な局所可変状態を表す。

基本Surface構文は明示的なbodyを持つ。

```lisp
(var count 0
  (set count (+ count 1))
  count)
```

概念的なCoreまたはelaboration中間形：

```text
local-var(binding, initializer, body)
```

評価順序は次のとおりである。

1. `initializer`を外側の環境で評価する。
2. initializerが正常終了した後、freshなlocal state identityを生成する。
3. initializerの結果を局所stateの初期値とする。
4. bodyを局所state handlerのscope内で評価する。
5. bodyの結果を`var`式全体の結果とする。
6. scope終了時に局所stateを破棄する。

新しい`var` bindingは、自身のinitializerから参照できない。

initializerには通常のeffectを許す。

initializerが異常終了した場合はlocal state instanceを生成しない。

***

### `DD-BND-013`: `var`の型注釈

`確定`

`var`についても、格納型を明示する場合は独立した`(type ...)` formを使用する。

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

正確なSurface構文名は後続の構文仕様で確定する。

必要な規範上の性質は次のとおりである。

* `type`宣言と`var` bindingを同じlexical declaration groupで関連付けられる
* 注釈がない場合はinitializerから格納型を推論する
* 注釈がある場合はinitializerをその型に対して検査する
* すべての`set`右辺も同じ格納型に対して検査する

単純な一binding形式へ型注釈を直接含める糖衣を将来追加する場合も、
意味上は独立した`type` declarationへelaborateする。

次のcolon形式は基本規範構文にはしない。

```lisp
; 採用しない
(var value : (union int str) 0
  ...)
```

***

### `DD-BND-014`: `var`の格納型は固定

`確定`

`var`の格納型は、initializerまたは対応する`(type ...)`宣言から一度決定し、
その後は固定する。

無注釈例：

```lisp
(var value 0
  ...)
```

initializerが`int`なので、格納型は`int`となる。

後続の代入から格納型を自動的にunionへ拡張してはならない。

```lisp
(var value 0
  (set value "text")
  value)
```

は静的エラーである。

複数型を保存したい場合は、明示的な型注釈を使用する。

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

型注釈がある場合、initializerおよびすべての`set`右辺が注釈型へ適合しなければならない。

***

### `DD-BND-015`: `var`の読出し

`確定`

Surface上では、`var`の現在値を通常の変数参照と同じ綴りで読み出す。

```lisp
(var count 0
  (+ count 1))
```

名前解決後のCoreでは、不変bindingの参照と局所`var`の読出しを区別する。

```text
不変binding:
read-immutable(binding-id)

局所var:
read-local-state(state-instance-id)
```

effect handlerへのelaboration後は、概念的に次へ変換できる。

```text
get<state-instance-id>
```

利用者へ明示的な`get`構文を要求しない。

***

### `DD-BND-016`: `set`

`確定`

局所`var`の更新には特殊形式`set`を使用する。

```lisp
(set count value)
```

`set`は通常関数ではなく、第一引数を局所可変bindingとして名前解決する。

名前解決後の概念形：

```text
set-local-state(state-instance-id, value-expression)
```

評価順序は次のとおりである。

1. 更新対象bindingを解決する。
2. 右辺expressionを評価する。
3. 右辺の値が格納型へ適合することを確認する。
4. 局所stateを更新する。
5. `unit`を返す。

`set`の結果型は常に`unit`である。

不変な`val`または`let` bindingに`set`してはならない。

```lisp
(let ((x 10))
  (set x 20))
```

は静的エラーとする。

***

### `DD-BND-017`: closureによる`var`のcapture

`確定`

内側のclosureが、外側の`var`をcaptureすることを許す。

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    (seq
      (increment)
      (increment))))
```

closureを`var`のscope内で作成し、scope内で使用することは有効である。

問題となるのはcapture自体ではなく、captureしたclosureがlocal stateのscope外へ
escapeすることである。

***

### `DD-BND-018`: local state identity

`確定`

各`var`の評価時にfreshなlocal state identity`s`を生成する。

```text
var count
→ fresh local state identity s
```

`count`の読出しおよび更新は、概念的に次のoperationを使用する。

```text
get<s>
put<s>
```

`count`をcaptureするclosureは、型または内部capture setに`s`への依存を持つ。

概念的な関数型：

```text
unit ->{local-state<s>} int
```

`s`は生成元の`var` scopeに限定される。

異なる`var`評価から生成されたidentityは、同じsource bindingに由来していても異なる。

***

### `DD-BND-019`: local state escapeの禁止

`確定`

local state identity`s`は、生成元`var`のscope外へescapeしてはならない。

少なくとも次をescapeとして扱う。

* `var`をcaptureしたclosureを結果として返す
* そのclosureを返却されるrecordまたはvariantへ格納する
* 外側のheap cellまたはmutable storeへ保存する
* moduleまたはglobal registryへ登録する
* public APIへexportする
* `any`または`dynamic any`へ変換してscope identityを消去する
* foreign runtimeへ渡す
* scope外で使用され得るcallbackとして登録する

概念的な条件：

```text
s ∉ free-scopes(result-type)
s ∉ residual-effects
s ∉ exported-captures
s ∉ external-store-effects
```

状態付きclosureを返す次の形式は静的エラーとする。

```lisp
(val make-counter
  (fn ()
    (var count 0
      (fn ()
        (set count (+ count 1))
        count))))
```

escape可能な状態が必要な場合は、将来の`cell`または`ref`機構を使用する。

***

### `DD-BND-020`: non-escaping callback

`確定`

局所stateをcaptureしたclosureは、同じscope内で直接呼び出してよい。

また、呼出し先がそのclosureを保存せず、呼出し中だけ使用することを
型または組込み契約で保証できる場合は、non-escaping callbackとして渡してよい。

```lisp
(var count 0
  (for-each items
    (fn (item)
      (set count (+ count 1)))))
```

これを許可するには、`for-each`がcallbackをscope外へ保存しないことを保証しなければならない。

RPX v1では次のいずれかに限定する。

* closureの直接呼出し
* 同じscope内の局所関数への引渡し
* コンパイラまたは標準ライブラリがnon-escapingと認識する高階関数
* 将来定義されるscoped parameter型を持つ関数

non-escaping性を証明できない一般の高階関数への引渡しは、保守的に拒否してよい。

***

### `DD-BND-021`: local state effectの除去

`確定`

`var` body内の局所state操作は、fresh identity`s`に対応するlocal state effectを持つ。

```text
body :
a ! <local-state<s> | e>
```

`var`はこのeffectを内部handlerによって処理する。

local state identity`s`がscope外へescapeしない場合、`var`式全体から
`local-state<s>`を一層除去する。

```text
initializer :
t ! ei

body :
a ! <local-state<s> | e>

var expression :
a ! f
```

ここで`f`は、initializerのeffect`ei`とbodyの残余effect`e`を満たすambient rowである。

```text
ei ⊑ f
e  ⊑ f
```

局所stateに対応するeffectは外側へ残さない。

***

### `DD-BND-022`: one-shot resumptionと局所state

`確定`

局所stateを含む計算がoperationによって中断され、one-shot resumptionとして
捕捉された場合、そのresumptionは同じlocal state instanceを保持する。

```text
operation発生時:
state = v

resume:
同じinstanceで再開

再開後:
state = vから継続
```

stateを複製しない。

resumptionを使用せず破棄した場合は、捕捉されたscopeをunwindし、
local state instanceを破棄する。

forwardした場合は、resumptionと共にlocal state scopeの継続責任を外側へ移譲する。

multi-shot resumption下でstateを共有するか複製するかは、multi-shot導入時まで保留する。

***

### `DD-BND-023`: escape可能な状態との分離

`確定`

RPX v1の`var`はescape不能な局所状態だけを表す。

状態付きclosureを返す用途など、scope外へ生存する状態が必要な場合は、
`var`のescape制約を緩和して表現しない。

将来、必要に応じて次のような明示的heap state機構を導入できる。

```text
cell<a>
ref<a>
```

この機構は、少なくとも次を別途規定する。

* runtime identity
* heap lifetime
* read／write effect
* closure間の共有規則
* memory management
* equality／hashの可否
* serializationの可否
* GUI再評価時の寿命
* provenanceおよびsource versionとの関係

`cell`／`ref`の仕様は`OPEN-MEM-001`へ移管する。

***

### `DD-BND-024`: 型一般化point

`確定`

RPX v1で型一般化を行う場所は原則として次とする。

* `val` binding
* `let` binding
* `letrec`グループの検査完了後

次は一般化pointではない。

* `type` annotation declarationそのもの
* 関数parameter
* `var` binding
* pattern binder
* operation clause parameter
* resumption binder
* named effect evidence
* local state identity
* resource capability

一般化はrank-1に限定する。

`(type ...)`宣言は、値の型を制約・公開するものであり、それ自体が新しいruntime bindingを
作るものではない。

***

### `DD-BND-025`: 構文的value restriction

`確定`

RPX v1は、ML系に近い厳格な構文的value restrictionを採用する。

`val`または`let`のinitializerが、規定されたgeneralizable syntactic valueである場合に限り、
型変数、record-row変数およびeffect-row変数を一般化する。

評価結果が最終的にvalueになることだけでは不十分である。

```text
結果がvalue
≠
構文的value
```

generalizable syntactic valueには、少なくとも次を含める。

* literal
* `fn`
* handler literal
* generalizable valueだけからなる不変record
* generalizable valueをpayloadとするvariant constructor
* 評価時に利用者コードやeffectを実行しない不変value form

一般化しない式には、少なくとも次を含める。

* function application
* effectful computation
* `var`の読出し
* `set`
* handlerの適用
* dynamic import
* implicit cast
* explicit safe cast
* foreign validation
* resource取得
* その他のexpansive expression

`(type ...)`注釈を付けても、expansive expressionを構文的valueとして扱ってはならない。

***

### `DD-BND-026`: effectful function valueの一般化

`確定`

`fn`の本体がeffectfulであっても、closure生成自体がpureであれば、
`fn`はgeneralizable syntactic valueである。

```lisp
(type log-value
  (forall ((a type))
    (fn (a)
        (effects log)
        a)))

(val log-value
  (fn (x)
    (log x)
    x))
```

次を区別する。

```text
closure生成時のeffect:
<>

関数呼出し時のlatent effect:
<log>
```

latent effectが存在することだけを理由に、関数値の一般化を禁止しない。

同様に、handler clauseがeffectfulでも、handler value生成自体がpureで、
scopedまたはaffineな値をcaptureしないならhandler literalを一般化できる。

***

### `DD-BND-027`: capability captureによる一般化禁止

`確定`

構文的にはvalueであっても、次をcaptureする場合は一般化しない。

* local `var`
* local state identity
* affine resumption
* named/scoped effect evidence
* private resource capability
* source transaction authority
* その他、scopeまたは使用回数に制約を持つ値

例：

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    ...))
```

`increment`は構文的には`fn`だが、local state identityをcaptureするため単相とする。

型検査器は構文形だけでなく、valueのcapture setおよびscope依存性も確認する。

***

### `DD-BND-028`: 明示的`forall`注釈

`確定`

明示的な多相型注釈も`(type ...)`内で記述する。

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

明示的`forall`注釈が存在する場合でも、initializerがvalue restrictionを満たさないなら、
その注釈によってgeneralizationを強制してはならない。

```lisp
(type generated-identity
  (forall ((a type))
    (fn (a) a)))

(val generated-identity
  (make-identity))
```

`make-identity`がfunction applicationである場合、initializerはexpansiveである。

このbindingを上記の多相型で受理してはならない。

診断例：

```text
このbindingは多相型として一般化できません。

理由:
  initializerが構文的valueではありません

annotation:
  forall a. a -> a
```

明示的注釈は推論を補助し、結果を検査するが、unsafeなgeneralizationを許可する機能ではない。

***

### `DD-BND-029`: 一般化される変数

`確定`

一般化可能なbindingの型を`t`、外側の環境を`Γ`とする。

一般化する変数は概念的に次である。

```text
free-variables(t)
-
free-variables(Γ)
```

一般化対象をkindごとに区別する。

* 通常の型変数
* record-row変数
* effect-row変数

local scope identity、affine capability identity、private runtime authority等は
一般化対象にしない。

明示的な`forall`注釈がある場合は、そのquantifierのkindとscopeを検査し、
initializerの推論型または検査結果が注釈schemeへ適合することを確認する。

***

### `DD-BND-030`: 一般化されないmetavariable

`確定`

一般化不可のbindingに未解決metavariableが存在する場合、それらを
monomorphicなweak metavariableとして扱う。

同一bindingのすべての使用箇所で同じmetavariableを共有する。

最初の使用で型が確定した後は、別の型へ再instantiateできない。

明示的な`(type ...)`宣言を後から追加して、既に別の型へ確定したweak metavariableを
不整合に一般化してはならない。

***

### `DD-BND-031`: `letrec`とvalue restriction

`確定`

`letrec`グループの検査中、すべての再帰binderは単相である。

グループ内の全関数本体を検査し、制約を解決した後に限り、
グループ外向けの一般化を検討する。

各右辺は構文的`fn`なので、通常はgeneralizable valueになり得る。

ただし、次の場合は一般化しない。

* local stateをcaptureする
* scoped evidenceをcaptureする
* affine capabilityをcaptureする
* 外側環境に未一般化metavariableが現れる
* その他、通常のvalue restrictionを満たさない

対応する`(type ...)`宣言がある場合は、推論結果をその型schemeに対して検査する。

多相再帰は許可しない。

***

### `DD-BND-032`: relaxed value restrictionの保留

`確定`

RPX v1はrelaxed value restrictionを導入しない。

varianceまたは型変数の出現位置に基づいて、expansive expressionの一部を追加で一般化することは
行わない。

必要性が確認された場合は、`OPEN-TYP-002`でsoundness、decidabilityおよび
semantic subtypingとの相互作用を検証したうえで将来拡張として検討する。

***

### Coreおよびdeclaration elaboration

型注釈を含むSurface declaration groupを、名前解決と型検査のため次のように整理する。

Surface例：

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

declaration elaboration：

```text
1. `type add ...`からannotation declarationを作る。
2. `val add ...`からvalue binding declarationを作る。
3. 同一scopeの名前解決によって両者を同じBindingIdへ関連付ける。
4. annotation typeをkind checkする。
5. initializerをannotation typeに対して検査する。
6. value restrictionに基づいてgeneralization可否を判定する。
7. runtime Coreには値bindingだけを残す。
8. 型注釈はtyped Core metadataおよびdiagnostic provenanceへ保持する。
```

`type` annotation declarationはruntimeで評価されない。

したがって、Core evaluatorの項として独立した`type`式を持たせる必要はない。

ただし、typed Core nodeは次を保持してよい。

```text
BindingMetadata {
  binding-id,
  inferred-type,
  declared-type?,
  declaration-span,
  annotation-span?
}
```

***

### 適合試験

#### top-level型注釈

```lisp
(type radius length)

(val radius 40mm)
```

期待：

```text
受理
radius : length
```

#### 関数型注釈

```lisp
(type add
  (fn (int int) int))

(val add
  (fn (x y)
    (+ x y)))
```

期待：

```text
受理
```

#### 注釈不一致

```lisp
(type value str)

(val value 42)
```

期待：

```text
静的エラー
```

#### 対応bindingのない型注釈

```lisp
(type missing int)
```

期待：

```text
静的エラー:
型注釈に対応する値bindingがない
```

#### 自己再帰の型注釈

```lisp
(letrec
  ((type factorial
     (fn (int) int))

   (val factorial
     (fn (n)
       (if (= n 0)
           1
           (* n (factorial (- n 1)))))))
  (factorial 5))
```

期待：

```text
120
```

#### 相互再帰の型注釈

```lisp
(letrec
  ((type even?
     (fn (int) bool))

   (type odd?
     (fn (int) bool))

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
  (even? 10))
```

期待：

```text
true
```

#### 一部だけ注釈

```lisp
(letrec
  ((type even?
     (fn (int) bool))

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
  (odd? 9))
```

期待：

```text
odd?の型をグループ制約から推論
結果はtrue
```

#### 任意式の再帰拒否

```lisp
(letrec
  ((val x
     (+ x 1)))
  x)
```

期待：

```text
静的エラー:
letrecの値binding initializerはfnでなければならない
```

#### 明示的多相型

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (fn (x) x))
```

期待：

```text
受理

identity :
forall a. a -> a
```

#### 注釈によるvalue restriction回避の拒否

```lisp
(type identity
  (forall ((a type))
    (fn (a) a)))

(val identity
  (make-identity))
```

`make-identity`がfunction applicationの場合の期待：

```text
静的エラー:
expansive initializerを明示注釈で多相一般化できない
```

#### local `var`

```lisp
(var count 0
  (set count (+ count 1))
  (set count (+ count 1))
  count)
```

期待：

```text
2
```

#### `set`の戻り値

```lisp
(var count 0
  (set count 1))
```

期待される結果型：

```text
unit
```

#### 格納型の固定

```lisp
(var value 0
  (set value "text")
  value)
```

期待：

```text
静的エラー:
expected int, found str
```

#### 明示union格納型

概念例：

```lisp
(var-declarations
  ((type value
     (union int str))

   (var value 0))
  (set value "text")
  value)
```

期待：

```text
受理
```

#### closure capture

```lisp
(var count 0
  (let ((increment
          (fn ()
            (set count (+ count 1))
            count)))
    (seq
      (increment)
      (increment))))
```

期待：

```text
2
```

#### closure escape

```lisp
(val make-counter
  (fn ()
    (var count 0
      (fn ()
        (set count (+ count 1))
        count))))
```

期待：

```text
静的エラー:
local state captured by escaping closure
```

#### effectful関数値の一般化

```lisp
(type log-value
  (forall ((a type))
    (fn (a)
        (effects log)
        a)))

(val log-value
  (fn (x)
    (log x)
    x))
```

期待：

```text
受理
```

***

### 未解決事項の移管

#### `OPEN-SYN-002`

* top-levelおよび局所declaration groupの完全な構文
* `letrec`内で`type`と`val`を並べる正確な括弧構造
* `var`と対応する`type`宣言を置く局所構文
* `val (f x)` sugarの完全な構文
* `forall`、kind annotation、effect rowの表面構文
* declaration orderingとforward annotation
* duplicate annotation diagnostic
* `type` annotationとtype alias declarationの構文上の区別

#### `OPEN-TYP-002`

* 注釈付き／無注釈再帰グループのalgorithm
* annotation subsumption
* explicit `forall`のchecking
* annotationとprincipal typingの関係
* non-escaping callbackの判定
* generalizable syntactic valueの正確なalgorithmic分類
* semantic subtyping下のgeneralization
* checker limitationによる注釈要求

#### `OPEN-MEM-001`

* escape可能な`cell`／`ref`
* heap allocation
* closure環境のmemory管理
* shared mutable cell
* cyclic state
* stateful closureのidentity
* GUI再評価をまたぐstate lifetime

#### `OPEN-MOD-001`

* public valueへの`(type ...)`注釈要件
* signature内の`type`とvalue annotationの区別
* module間相互再帰
* recursive module
* exported closureのscope検査

#### 将来拡張

* 多相再帰
* lazy recursive value
* recursive record
* recursive handler value
* affine kind system
* 一般的なborrowed closure
* relaxed value restriction
* multi-shot state semantics

***

### 解決後の基本原則

```text
不変値binding:
val

型注釈:
type

再帰関数binding:
letrec内のval

局所可変状態:
var

更新:
set
```

```text
型注釈:

(type name type-expression)

(val name initializer)
```

```text
型注釈は:
- 省略可能
- 対応する値bindingを検査する
- runtime bindingを作らない
- value restrictionを迂回しない
```

```text
letrec:
- initializerはfnのみ
- 自己再帰と相互再帰を許可
- グループ内にtype annotationを置ける
- グループ内部では単相
- 検査後にrank-1一般化可能
```

```text
var:
- 明示bodyを持つ
- initializerから新bindingは見えない
- 格納型はinitializerまたはtype annotationから決定
- setはunit
- closure captureは可能
- scope外escapeは禁止
```

```text
一般化:
- val／let／letrec検査後
- generalizable syntactic valueのみ
- affine／scoped capabilityをcaptureしないこと
- explicit type annotationは一般化条件を変更しない
```

以上により、RPX v1の再帰関数、相互再帰、独立した`(type ...)`型注釈、
局所可変状態、scope escapeおよび構文的value restrictionが特定されたため、
`OPEN-BND-001`を解決済みとする。
