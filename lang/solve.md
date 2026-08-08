# openを解決する

## OPEN-CON-001 構造化Concurrency・Task・Cancellation・Task間共有
### DD-CON-001 決定概要
#### DD-CON-001.1 状態
Status:
RESOLVED

Scope:
TaskとThreadの区別
構造化Concurrency
Task scope
Task handle
TaskResult
spawn／await／cancel
Fail-fastとCollect-all
Cancellation
Effect環境
Task間の値の移送
Perceusとの接続
Schedulerの観測可能な保証
決定的なTest Scheduler
公開Task package

#### DD-CON-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- RPXは正格CBVで、同一計算内の評価順序は左から右
- Effect handlerはdeep handler
- Continuationはone-shot
- 通常Failureはfailure Eで表す
- CancellationはFailureおよびDefectと区別する
- bracketは正常終了・Failure・Cancellation・継続破棄時にcleanupする
- 通常値はPerceus方式で管理する
- varはescape不能な局所可変状態
- 一般的なescape可能cell／refはv1では導入しない
- Scoped resource handleはscope外へescapeできない
- DocumentSnapshotとEditTransactionは不変値
- Documentへのcommitは直列化される

### 0. 基本方針
#### 0.1 Concurrencyの目的

Concurrencyは、複数の処理を時間的に重ねて進めるために使用する。

想定用途：

- GUIを停止させずにRenderingする
- 複数のResource読込みを進める
- PreviewをBackgroundで生成する
- ExportやImportを独立したJobとして実行する
- 複数Fileを一括変換する
- PluginやRender Jobを隔離する

#### 0.2 v1の範囲

v1では、構造化されたTaskの最小機能を提供する。

- Task scope
- 子Taskの開始
- Task結果の回収
- Cancellation
- Fail-fastな並行処理
- 全結果を収集する並行処理
- Schedulerとの協調
- Task単位のFault isolation


次はv1へ含めない。

- OS Threadの直接生成
- Thread IDの安定した公開
- Mutex／Semaphore
- 一般Channel
- Actor
- 共有可変cell
- Detached task
- Task priority
- Schedulerの直接制御
- 分散Task
- Realtime保証

### 1. TaskとThread
#### 1.1 Task

Taskは、一つの独立した計算単位である。

Taskは開始後、次のいずれかの終了状態へ到達する。

- 正常完了
- 型付きFailure
- Cancellation
- Defect

#### 1.2 Thread

Threadは、TaskをCPU上で実行する実装手段である。

Task:
言語・Runtime上の計算単位

Thread:
Taskを物理的に実行する手段


一つのThreadが複数Taskを切り替えてもよく、複数Threadが複数Taskを並列実行してもよい。

#### 1.3 配置の非公開性

通常のspawnは、別Threadでの実行を保証しない。

同一Thread上の論理的Concurrency:
許可

複数Thread上の並列実行:
安全な場合にRuntimeが選択可能


TaskがどのThreadまたはCPU coreで実行されたかは、通常のRPXプログラムから観測できない。

### 2. 構造化Concurrency
#### 2.1 親子関係

すべての子Taskは、作成時のTask scopeへ所属する。

親Task
└─ Task scope
   ├─ 子Task A
   ├─ 子Task B
   └─ 子Task C


Task scopeは、所属する全子Taskの寿命に責任を持つ。

#### 2.2 子Taskの放置禁止

Task scopeは、未終了の子Taskを残したまま終了しない。

Scope bodyが終了しようとした時点で未完了の子Taskがある場合、Runtimeは次を行う。

1. 未完了の子TaskへCancellationを要求
2. 子Taskからその子孫へCancellationを伝播
3. 全子Taskのcleanup完了を待つ
4. 全Task handleを終了状態へ移す
5. Task scopeを終了する

#### 2.3 Detached task

親scopeを離れて独立して動き続けるDetached taskは、v1の通常APIでは提供しない。

ProcessやApplication全体に所属する長寿命serviceは、trusted RuntimeまたはHostが別の上位scopeとして管理する。

### 3. Task scope
#### 3.1 抽象型

Task scopeは、概念的に隠れたscope parametersを持つ。

TaskScope<s>


利用者はsを直接記述しない。

#### 3.2 Scope生成

概念的な公開API：

(with-task-scope
  (fn (scope)
    body))


内部的には、呼出しごとにfreshなsを生成する。

#### 3.3 結果制約

Task scopeのsは、with-task-scopeの結果型へ現れてはならない。

概念型：

with-task-scope :
  (forall s.
    TaskScope<s> -> A
    effects {task, e})
  -> A
  effects {task, e}

#### 3.4 Escape禁止

次を禁止する。

- TaskScope<s>をscope外へ返す
- TaskScope<s>を外側stateへ保存する
- TaskScope<s>をdynamicへ封入して逃がす
- TaskScope<s>をescapeするClosureへ捕捉する
- TaskScope<s>を別scopeまたは別Taskへ送る

### 4. Task handle
#### 4.1 抽象型

Task handleは概念的に次の型を持つ。

Task<s, A, E>


各parameterの意味：

s:
所属するTask scope

A:
正常完了時の値

E:
型付きFailureのpayload型

#### 4.2 内部Effect row

子Task内部で使用した通常Effect rowは、Task handleの型parameterには保存しない。

たとえば子計算が次を要求するとする。

() -> Image
effects {
  resource,
  clock,
  failure ImageError
}


resourceとclockは子Task実行環境内で処理され、Task handleは次となる。

Task<s, Image, ImageError>


ただし、子Taskのrequired effectsを無視するわけではない。spawn時に、子Taskの実行環境が必要なEffectを提供可能か静的に検査する。

#### 4.3 Escape禁止

Task handleは所属scope外へescapeできない。

禁止：

- Task handleをscopeの結果として返す
- 外側record、list、dataへ保存する
- Module-level bindingへ保存する
- dynamicへ封入して逃がす
- EscapeするClosureへ捕捉する
- 別のTask scopeへ送る

#### 4.4 一回限りの回収

Task handleの終了結果は高々一回だけ回収できる。

Pending
↓ await／await-result
Consumed


二重回収は、静的に判定可能なら静的エラーとする。実行時まで残った場合はDefectとする。

#### 4.5 等値性等

通常のTask handleには次を提供しない。

- 構造的等値
- Identity比較
- Hashing
- Serialization
- 安定したTask ID


Toolingは診断用の一時的なdebug identityを使用できるが、通常プログラムの意味には使用させない。

### 5. Taskの終了結果
#### 5.1 TaskResult

低水準Task APIは、Taskの終了を通常data値として返す。

概念型：

(data (task-result a e)
  (completed a)
  (failed e)
  (cancelled cancellation-info)
  (defected defect-report))

#### 5.2 Completed
Completed(A):
Taskが正常に値Aを返した

#### 5.3 Failed
Failed(E):
Task内部でfailure Eが未処理のままTask境界へ到達した


Task境界はfailure Eを捕捉し、Failed(E)へ変換する。

#### 5.4 Cancelled
Cancelled(CancellationInfo):
Cancellation要求により構造化unwindを完了した

#### 5.5 Defected
Defected(DefectReport):
Task-localなDefectをFault boundaryが隔離した


Runtimeまたはscope全体の健全性を保証できないDefectは、TaskResultへ変換せずTerminal failureとする。

### 6. Cancellation情報
#### 6.1 理由

Cancellationは通常Failureと区別する。

概念的な理由：

(data cancellation-reason
  user-requested
  parent-cancelled
  scope-closed
  sibling-failed
  timeout)

#### 6.2 CancellationInfo

CancellationInfoは少なくとも理由を保持できる。

CancellationInfo {
  reason
}


Task ID、実時刻、Thread ID等を規範的な公開情報にはしない。

#### 6.3 Cleanup failure

Cancellation中に発生した通常のcleanup failureは、Taskの主終了状態をFailedへ変更せず、suppressed diagnostic metadataとして記録する。

Primary termination:
Cancelled

Suppressed:
cleanup failure


Resource ownershipやRuntimeの不変条件が破られた場合はDefectまたはTerminal failureへ昇格する。

### 7. spawn
#### 7.1 意味

spawnはTask scopeへ所属する子Taskを作成する。

概念型：

spawn :
  TaskScope<s>
  -> (() -> A
      effects {child-effects, failure E})
  -> Task<s, A, E>
  effects {task}

#### 7.2 遅延された計算

spawnへ渡す計算は無引数関数とする。

(spawn scope
  (fn ()
    (load-image resource)))


RPXがCBVであるため、計算を無引数関数へ包むことで、spawn前の事前評価を防ぐ。

#### 7.3 Effect検査

spawn時に次を検査する。

RequiredEffects(child-computation)
⊆
ProvidedEffects(child-task-environment)


提供不能なEffectがあれば静的エラーとする。

Task requires an Effect unavailable
in its execution environment:
  window

#### 7.4 親TaskへのEffect

spawn自体が親Taskで発生させるEffectはtaskである。

子Task内部で処理されるresource、clock等を親Taskのeffect rowへ単純に加えない。

#### 7.5 子Failure

子Task内部のfailure Eは、spawn時には親へ伝播しない。

spawn:
Task handleを返す

Task完了:
Failed(E)として保存

await:
必要に応じて親へ再伝播

### 8. await-result
#### 8.1 型
await-result :
  Task<s, A, E>
  -> TaskResult<A, E>
  effects {task}

#### 8.2 意味
- Taskが未完了なら現在TaskをSuspendedにする
- Runtimeは別の実行可能Taskを進める
- 対象Task完了後に現在Taskを再開する
- Task handleを消費する
- TaskResultを通常値として返す

#### 8.3 Failure

await-resultは、子Taskのfailure Eを親のFailureとして再発生させない。

詳細な終了状態を通常値として扱いたい場合に使用する。

### 9. await
#### 9.1 型

概念型：

await :
  Task<s, A, E>
  -> A
  effects {
    task,
    failure E
  }

#### 9.2 終了状態の変換
Completed(A):
Aを返す

Failed(E):
親Taskでfailure Eを再発生

Cancelled:
親TaskへCancellationを伝播

Defected:
外側Fault boundaryへDefectを再通知


Cancellationは一般EffectRowの公開labelへ追加せず、task Effectの実行契約として扱う。

#### 9.3 使い分け
終了状態を個別処理する:
await-result

正常値を取得し、Failure等を伝播する:
await

### 10. cancel
#### 10.1 型
cancel :
  Task<s, A, E>
  -> unit
  effects {task}

#### 10.2 意味

cancelはTaskを直ちに破壊せず、Cancellation要求を記録する。

cancel:
要求

await-result:
終了確認

#### 10.3 Handle消費

cancelはTask handleを消費しない。

(seq
  (cancel task)
  (await-result task))


のように、Cancellation後のcleanup完了を確認できる。

#### 10.4 冪等性

Cancellation要求は冪等とする。

未要求Taskへのcancel:
要求を設定

既にCancelling:
追加変化なし

既に終了:
追加変化なし


Consumed済みhandleは再使用できない。

### 11. Cancellation
#### 11.1 協調的な中止

Cancellationは任意の機械命令地点でTaskを強制破壊する操作ではない。

Cancellation要求
↓
安全なCancellation pointで観測
↓
構造化unwind
↓
Cleanup
↓
Cancelled

#### 11.2 Cancellation point

少なくとも次をCancellation pointとする。

- await
- await-result
- yield
- check-cancelled
- 非同期I/O待機
- Timer待機
- Runtimeが挿入する安全なsafepoint

#### 11.3 長時間の純粋計算

長時間の純粋計算へ対応するため、次を併用する。

- Libraryが明示的にcheck-cancelledを呼ぶ
- Compilerがloop back-edge等へ安全なsafepointを挿入できる


任意命令地点での強制停止は行わない。

#### 11.4 通常Handlerからの分離

Cancellationは、通常利用者が一般handleで捕捉して握り潰せるEffectにはしない。

実装上は非再開型の制御移動として既存handler機構を利用できるが、処理責任はTask Runtime境界に限定する。

#### 11.5 Effect row

Cancellationは公開EffectRowの独立labelへ載せない。

TaskはCancellationされ得る:
task Effectの契約

Cancellationが通常Failureである:
否

### 12. Cancellationの伝播
#### 12.1 親から子

親TaskがCancellationを観測した場合、所属する全子TaskへCancellationを伝播する。

親
├─ 子A
│  └─ 孫A1
└─ 子B


終了順：

1. 子A・子BへCancellation
2. 孫A1へCancellation
3. 子孫Taskのcleanup
4. 子Task終了
5. 親Taskのcleanup
6. 親Task終了

#### 12.2 子から親

子Taskが単独でCancelledになっても、親Taskを自動的にcancelしない。

親または高水準combinatorが、そのCancelled結果をどのように扱うか決める。

#### 12.3 Scope終了

Task scope bodyが終了するときに残っている未完了Taskには、scope-closedを理由としてCancellationを要求する。

### 13. Cleanup
#### 13.1 Cancellation時

Cancellationを観測したTaskは、次の手順で終了する。

1. 子TaskへCancellationを伝播
2. 子Taskの終了を待つ
3. bracket cleanupをLIFOで実行
4. Task所有rootをPerceusでdrop
5. Cancelledとして終了

#### 13.2 Cleanup中のmask

Cleanup中にCancellationを再観測して後始末を中断してはならない。

Cleanup区間:
Cancellation観測を一時的に延期

Cancellation要求:
消去しない

#### 13.3 利用者によるmask

通常利用者が任意の長時間Cancellation maskを作るAPIは、v1では提供しない。

Runtimeとbracketの限定されたcleanup区間だけがmaskを使用できる。

### 14. yieldとcheck-cancelled
#### 14.1 yield

概念型：

yield :
  unit -> unit
  effects {task}


意味：

- 現在TaskをRunnableへ戻す
- Schedulerへ次Task選択の機会を返す
- Cancellation要求を確認する
- 再開後にunitを返す


yieldは特定の別Taskが必ず実行されることを保証しない。

#### 14.2 check-cancelled

概念型：

check-cancelled :
  unit -> unit
  effects {task}


Cancellation要求がなければunitを返す。

要求があれば構造化Cancellationを開始する。

### 15. Fail-fast：all
#### 15.1 用途

allは、全ての子計算が成功しなければ一つの結果を構築できない場合に使用する。

例：

- 複数pageから一つのPDFを作る
- Font、Image、Layoutを全て準備する
- 一つのBuild artifactを複数工程で作る

#### 15.2 概念型

同一結果型のcollectionについて：

all :
  List<() -> A
       effects {child-effects, failure E}>
  -> List<A>
  effects {
    task,
    failure E
  }

#### 15.3 意味
全Task成功:
入力順のList<A>を返す

一つがFailed:
未完了兄弟Taskをcancel
全Taskのcleanup完了を待つ
Primary Failureをraise

一つがCancelled:
未完了兄弟Taskをcancel
現在TaskもCancelled

Scope-level Defect:
全兄弟をcancel
全体をDefected

#### 15.4 Primary Failure

Runtimeは最初に観測したFailureの時点でCancellationを開始する。

外側へ返すPrimary Failureは、全Task終了後、入力順で最初にFailedとなったTaskから選ぶ。

Cancellation開始:
最初に観測したFailure

外部へ返すPrimary:
入力順で最初のFailed Task


その他のFailureとcleanup failureはsuppressed metadataとして保持する。

### 16. Collect-all：collect
#### 16.1 用途

collectは、各処理の結果が独立しており、一つのFailureで他の処理を止めるべきでない場合に使用する。

例：

- 複数Fileの一括変換
- 複数DocumentのValidation
- 独立したAssetのImport

#### 16.2 概念型
collect :
  List<() -> A
       effects {child-effects, failure E}>
  -> List<TaskResult<A, E>>
  effects {task}

#### 16.3 意味
一つの通常Failure:
他Taskをcancelしない

全Task終了:
入力順でTaskResultを返す

親TaskのCancellation:
全子Taskをcancel

Scope-level Defect:
全子Taskをcancel


Task-localに安全に隔離可能なDefectは、Defectedとして個別結果に含められる。

### 17. Collection API
#### 17.1 Map形式

入力collectionと関数を受け取る高水準APIを提供できる。

collect-map :
  List<X>
  -> (X -> A
      effects {child-effects, failure E})
  -> List<TaskResult<A, E>>
  effects {task}


同様にall-mapを提供できる。

#### 17.2 同時実行数

大量Taskの無制限生成を避けるため、同時進行数を制限できるAPIを提供する。

collect-map-limit :
  PositiveInt
  -> List<X>
  -> (X -> A
      effects {child-effects, failure E})
  -> List<TaskResult<A, E>>
  effects {task}


規則：

limit = 1:
順次実行

limit = N:
最大N Taskを同時進行

limit = 0:
型または入力検査で拒否


同時実行数はThread数を意味しない。

#### 17.3 空入力
all([]):
[]

collect([]):
[]


空入力は正常成功とする。

#### 17.4 結果順

Taskの実際の開始・切替・完了順とは無関係に、結果は入力順で返す。

### 18. 異なる結果型のTask

allとcollectは、同一型のcollectionを基本とする。

異なる結果型のTaskは、低水準APIで明示的に組み合わせる。

(with-task-scope
  (fn (scope)
    (val font-task
      (spawn scope load-font))

    (val image-task
      (spawn scope load-image))

    (record
      (font (await font-task))
      (image (await image-task)))))


all2、all3等をCore仕様へ大量に追加しない。

ApplicativeなTask abstractionは将来のLibrary設計にできる。

### 19. Taskへ渡せる値
#### 19.1 Closure

子Taskとして実行するClosureは、捕捉値をTask環境へ移送または共有する。

#### 19.2 所有権移送

親Taskが以後使用しない値は、子Taskへ所有権を移送できる。

親 owns x
↓
子へtransfer
↓
子 owns x

#### 19.3 共有

親と子の両方が使用する不変値は共有できる。

Perceusは必要なdupを挿入する。

#### 19.4 捕捉禁止

v1では、次を子Task Closureへ捕捉できない。

- var
- Scoped resource handle
- TaskScope
- Task handle
- One-shot continuation
- Thread-affine GUI handle
- 非Task-safeなForeign値

#### 19.5 Dynamic

dynamic Sは、上限型SがTask境界で安全と証明できる場合だけTaskへ渡せる。

dynamic int:
候補

dynamic str:
候補

dynamic any:
原則として禁止


dynamicによってscope・Resource・sendability検査を迂回してはならない。

### 20. Scoped Resource
#### 20.1 Task capture

v1では、親Taskで取得したscoped resource handleを子Taskへ捕捉させない。

不適合：

(with-file path
  (fn (file)
    (spawn scope
      (fn ()
        (read-all file)))))


推奨：

(spawn scope
  (fn ()
    (with-file path
      (fn (file)
        (read-all file)))))


Resourceの取得・利用・解放を同じTaskへ閉じ込める。

#### 20.2 将来拡張

Scope包含・Thread affinity・borrowを型検査できる将来版では、限定的なResource captureを検討できる。

v1では単純な禁止規則を採用する。

### 21. Document編集との関係
#### 21.1 Task-safeな値
DocumentSnapshot:
Task間で共有可能

EditTransaction:
Task間で移送可能

DocumentId／NodeId:
Task間で利用可能

#### 21.2 現在文書

DocumentHandleは現在状態への接続を表すため、既定ではTask Closureへ直接捕捉させない。

#### 21.3 編集手順

子Taskは次の手順を使用する。

1. 不変Snapshotを受け取る
2. 計算・解析を行う
3. EditTransactionをpureに構築する
4. EditTransactionを結果として親またはserviceへ返す
5. Document serviceがcommitを直列化する


これにより、Task間の共有可変文書状態を導入せずに並行編集計算を行える。

### 22. 子TaskのEffect環境
#### 22.1 原則

子Taskは、親Taskのhandler stack全体を暗黙に継承しない。

Task開始時に、Runtimeが子Task用のEffect環境を構築する。

#### 22.2 分類

EffectまたはCapabilityを次の4種類として扱う。

Reinstalled:
子Task用にRuntimeが新しく設置

Inherited:
安全に子Taskへ引き継げる

Explicit:
明示Capabilityとして渡す必要がある

Prohibited:
子Taskで使用できない

#### 22.3 Reinstalled

少なくとも次は子Task用に再設置する。

- Scheduler context
- Cancellation context
- Task-local memory budget
- Fault boundary
- Task-local diagnostic context

#### 22.4 Inherited

次は、handler実装がTask-safeである場合に引き継げる。

- 読み取り専用Configuration
- Clock service
- Package resource loader
- 安全なLogging service
- 読み取り専用Module／Package environment

#### 22.5 Explicit

権限が強いものは、明示Capabilityとして子Taskへ渡す。

例：

- Network access
- Storage write
- Process起動
- Document commit service
- Backend service

#### 22.6 Prohibited

次を子Taskへ引き継がない。

- varの局所State handler
- 親のscoped Resource handler
- 親のone-shot continuation
- Task scope
- Transaction適用途中の一時handler
- Thread専用GUI handler

#### 22.7 Effect検査

子Taskのrequired effectsは、子Task環境が提供するEffectの部分集合でなければならない。

RequiredEffects(child)
⊆
ProvidedEffects(child-task-environment)

### 23. Task境界でのEffect処理
#### 23.1 生のEffectを親へ転送しない

子Task内の未処理Effectを、時間的に独立した親Taskの現在handlerへ直接forwardしない。

子Taskの通常Effectは子Task環境内で処理されなければならない。

#### 23.2 終了変換

Task境界は次を行う。

正常return:
Completed(A)

failure E:
Failed(E)

Cancellation:
Cancelled

Task-local Defect:
Defected(DefectReport)


親へ渡るのは生のEffectではなくTaskResultである。

### 24. Perceusとの接続
#### 24.1 同一Thread

同一Thread上のTask間では、通常の非atomicなPerceus参照カウントを使用できる。

#### 24.2 所有権移送

値を親から子へ完全に移送する場合、移送後に親はその値を使用しない。

Thread A owns x
↓ transfer
Thread B owns x

#### 24.3 Thread間共有

親子が同じ値を別Threadから参照する場合、その値をShared表現へ昇格させる。

Local:
一つのThreadだけから参照可能

Shared:
複数Threadから参照可能


Shared値には、atomic参照カウントまたは同等のthread-safeな管理を使用する。

#### 24.4 昇格
Local -> Shared:
許可

Shared -> Local:
v1では規範的に要求しない


内部一意性解析による最適化は許可するが、利用者から観測可能にはしない。

#### 24.5 Reuse

Thread間で共有されている値は、一意性が確認できない限りreuseしてはならない。

#### 24.6 通常spawn

通常spawnは別Thread配置を要求しないため、非sendableだがTask-safeな計算を同一Threadで動かす実装余地を持つ。

ただし、var、scoped Resource、Continuation等の捕捉禁止規則は同一Threadの場合にも維持する。

### 25. Scheduler
#### 25.1 同一Task内

同一Task内では、既存のCBV・左から右の評価順序を維持する。

first
↓
second
↓
third

#### 25.2 Task間

異なるTask間の、次の順序は原則として保証しない。

- 実行開始順
- Task切替順
- Effect発生の相対順
- 完了順

#### 25.3 明示的依存

Task間に順序が必要な場合、await等で明示的な依存関係を作る。

Task Aをawait
↓
Task Bを開始

#### 25.4 await時

await中のTaskはSuspendedとなり、無関係な実行可能Taskの進行を不必要に妨げない。

#### 25.5 弱い進行保証

Runtimeが継続して稼働し、Taskが実行可能であり続け、より高位の終了条件がない場合、そのTaskを意図的に永久放置してはならない。

次は保証しない。

- 一定時間以内の実行
- 一定Turn以内の実行
- Task間の実行割合
- Realtime deadline
- 完了時間の上限

#### 25.6 Priority

利用者指定Task priorityはv1の公開APIへ含めない。

### 26. Safepoint
#### 26.1 役割

Safepointでは、Runtimeが安全に次を行える。

- Cancellation確認
- Task切替
- Memory budget確認
- Fault確認

#### 26.2 候補
- Loop back-edge
- 再帰呼出し
- Function call境界
- Allocation
- Effect operation
- await
- yield

#### 26.3 挿入

Compilerは安全な位置へsafepointを挿入できる。

具体的頻度は実装依存とする。

Safepointの挿入によって、Cancellationがない正常実行の意味、同一Task内の評価順序、Effect順序を変えてはならない。

### 27. Blocking操作
#### 27.1 標準待機操作

標準Task対応APIは、待機中に無関係なTaskの進行を不必要に停止させてはならない。

可能な場合：

- TaskをSuspendedにする
- OS非同期I/Oを使用する
- Blocking callを専用workerへ移す

#### 27.2 Foreign call

Foreign callのblocking性、Cancellation可能性、Thread affinityはOPEN-KER-001で宣言する。

標準Schedulerは、任意の未注釈Foreign callがnon-blockingであるとは仮定しない。

### 28. Test Scheduler
#### 28.1 目的

並行処理の再現可能なTestのため、決定的Schedulerを提供可能にする。

#### 28.2 規定可能な動作

一例として、Test Schedulerは次を使用できる。

Runnable queue:
FIFO

spawn:
queue末尾へ追加

yield:
現在Taskをqueue末尾へ戻す

await:
現在TaskをSuspendedへ移す

Task完了:
待機Taskを規定順でRunnableへ戻す


正確なTest APIはOPEN-TST-001で定める。

#### 28.3 仮想Clock

Test環境では実Clockを仮想Clockへ置き換えられる。

- 明示的に時間を進める
- Timer起床順を再現する
- Timeoutを実時間へ依存させない

#### 28.4 Effect環境

Hermetic Test Runtimeは、必要に応じて次を提供する。

- Deterministic Scheduler
- Virtual Clock
- Seeded Random
- In-memory Resource handler
- Mock Network
- Captured Diagnostic sink

#### 28.5 Trace

Schedulerはdebug・test用途に切替traceを記録・再生できる。

Traceの具体形式は通常プログラムの意味論ではなく、tooling仕様とする。

### 29. 公開package
#### 29.1 位置付け

Task APIはCore特殊形式として多数追加せず、compiler-nativeな標準packageとして提供する。

公開意味:
通常の型付きpackage API

実装:
Rust Runtime intrinsic

Test:
Test Scheduler handlerへ差替え可能

#### 29.2 Package名

正式な標準namespaceが確定するまで、本仕様ではtask packageと呼ぶ。

候補例：

std/task

#### 29.3 Effect名

Task操作が要求する公開Effect名はtaskとする。

例：

(type render-pages
  (fn document render-output
    (effects
      task
      render
      (failure render-error))))

#### 29.4 抽象型

少なくとも次を抽象型または内部scope付き型とする。

TaskScope<s>
Task<s, A, E>
DefectReport


TaskResult<A, E>、CancellationInfoおよびCancellationReasonは、安全な公開dataとして提供できる。

### 30. 推奨する最小API
with-task-scope
spawn
await-result
await
cancel
yield
check-cancelled
all
collect
all-map
collect-map
collect-map-limit


正確な関数名は標準package編成時に調整できるが、意味論は本項目に従う。

### 31. Task状態機械

概念的な状態：

Created
Runnable
Running
Suspended
Cancelling
Completed
Failed
Cancelled
Defected
Consumed


代表的遷移：

Created
-> Runnable
-> Running


待機：

Running
-> Suspended
-> Runnable


正常終了：

Running
-> Completed
-> Consumed


Failure：

Running
-> Failed
-> Consumed


Cancellation：

Created／Runnable／Running／Suspended
-> Cancelling
-> Cancelled
-> Consumed


Defect：

Running
-> Defected
-> Consumed


終了状態からRunningへ戻る遷移は不正である。

### 32. DefectとTerminal failure
#### 32.1 Task-local Defect

Runtimeの健全性を維持して隔離できるDefectは、Task-local Fault boundaryで処理する。

- 子Taskを終了
- Cleanupを実行
- TaskResult.Defectedを生成

#### 32.2 Scope-level Defect

Task scopeまたは共有Runtime状態へ影響するDefectでは、全子Taskをcancelし、scope全体をDefectedとして終了する。

#### 32.3 Terminal failure

次の場合はTerminal failureとする。

- Scheduler状態の整合性を保証できない
- 同じContinuationを二重resumeし、所有状態が不明
- Completed Taskを再実行した
- Atomic reference count等のmemory safetyが破損
- Cleanup stackの整合性が不明


Terminal failureでは通常のTaskResultを返すことを保証しない。

### 33. 適合試験
CON-01：正常Task
spawn
-> Completed(A)
-> await-result


期待結果：

Completed(A)を返し、handleをConsumedにする

CON-02：Task handle escape

Task scopeからTask handleを返す。

期待結果：

static error:
Task handle escapes its Task scope

CON-03：二重await

同じTask handleを二回await-resultする。

期待結果：

静的に検出可能ならstatic error
そうでなければDefect

CON-04：Scope終了時の未回収Task

Scope bodyが未完了Taskを残して終了する。

期待結果：

- TaskへCancellation
- Cleanup完了を待つ
- 子Taskを残さずscope終了

CON-05：allのFail-fast

入力3件のうち2件目がFailed。

期待結果：

- 未完了兄弟をcancel
- 全cleanup完了を待つ
- 入力順でPrimary Failureを選ぶ

CON-06：collect

入力3件のうち2件目がFailed。

期待結果：

- 1件目と3件目を続行
- 入力順のTaskResultを返す

CON-07：結果順

完了順がC、A、Bであっても、入力順A、B、Cで結果を返す。

CON-08：親Cancellation

親Taskをcancelする。

期待結果：

- 全子孫Taskへ伝播
- 子孫cleanup
- 親cleanup
- 親がCancelled

CON-09：子Cancellation

子Taskだけをcancelする。

期待結果：

- 子TaskはCancelled
- 親Taskを自動cancelしない

CON-10：Cancellation中cleanup

Cancellation後のrelease中に通常Failureが発生。

期待結果：

- Taskの主結果はCancelled
- Cleanup failureをsuppressedとして記録

CON-11：var capture

varを捕捉するClosureをspawnへ渡す。

期待結果：

static error

CON-12：Scoped Resource capture

親で取得したFile handleを子Task Closureへ捕捉する。

期待結果：

static error

CON-13：ResourceをTask内部で取得

子Task内部でwith-fileを実行する。

期待結果：

正常に実行可能
Task終了前にFileをrelease

CON-14：Task Effect不足

子Taskがwindow Effectを要求し、子Task環境が提供しない。

期待結果：

static error:
unavailable Effect in Task environment

CON-15：DocumentSnapshot共有

親と子Taskが同じ不変Snapshotを読む。

期待結果：

安全に共有
文書内容を変更しない

CON-16：EditTransaction生成

子TaskがSnapshotからEditTransactionを構築して返す。

期待結果：

正常完了
CommitはDocument serviceが直列化

CON-17：await中の進行

Task AがTask Bをawaitし、Task CがRunnable。

期待結果：

Task AをSuspended
Task Cが進行可能

CON-18：yield

実行可能な別Taskがある状態でyieldする。

期待結果：

Schedulerへ選択機会を返す
ただし特定Taskの実行は保証しない

CON-19：決定的Test Scheduler

同じspawn、yield、await、Clock操作列を再実行する。

期待結果：

同じ規定Task切替traceを再現

CON-20：Scheduler Defect

Consumed済みContinuationを再開しようとする。

期待結果：

隔離可能ならDefect
所有状態が不明ならTerminal failure

### 34. 関連する未決定項目

#### `OPEN-KER-001`

- Foreign callのblocking性
- Cancellation可能性
- Thread affinity
- Owned／Borrowed／Shared
- Worker Threadへの移送
- Native async I/O

#### `OPEN-TST-001`

- Test SchedulerのSurface
- Virtual Clock
- Scheduler trace
- Failure／Cancellation／Defectの期待構文

#### `OPEN-PKG-ENTRY-001`

- Application root Task scope
- CLI／GUI／ServerのRuntime profile
- Process終了時のTask cleanup

#### `OPEN-MEM-PROF-001`

- Taskごとのmemory利用
- Local／Shared参照カウント統計
- SchedulerとContinuation保持量

#### `OPEN-CON-CHANNEL-001`

- Channel
- Stream
- Backpressure
- Task間message passing
- Select

#### `OPEN-CON-PAR-001`

- 明示的なCPU並列API
- Sendable／Shareableの公開表現
- Worker pool
- Parallel collection

### 35. 最終状態

```text
OPEN-CON-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 親子関係を持つ構造化Task
- Task scope外へescapeしないTask handle
- Completed／Failed／Cancelled／Defectedの明確な区別
- 協調的Cancellation
- Cancellation時のbracket cleanup
- Fail-fastなall
- 全結果を収集するcollect
- 入力順で安定した結果
- Taskへ持ち込める値とEffectの制限
- Perceusの所有権移送とthread共有への接続
- 配置を固定しない論理的Concurrency
- 弱い進行保証
- Safepointとyield
- 決定的なTest Schedulerと仮想Clock
- Compiler-nativeな標準Task package

## OPEN-KER-001 Kernel・Native package・Foreign boundary・Trusted Adapter ABI
### DD-KER-001 決定概要
#### DD-KER-001.1 状態
Status:
RESOLVED

Scope:
Kernelの責務
Native package
Portable fallback
Foreign dataとopaque handle
Validation boundary
Native representation
Foreign ownership
Resource lifetime
Task Runtimeとの接続
Blocking／Cancellation
Executor affinity
Callback／Reentrancy
Rust panic／unsafe
Native Adapter ABI
Package／Binding／Type identity
Typed Value Builder
Async completion
Native packageの初期化・終了
Security policy
Conformance test

#### DD-KER-001.2 既存仕様との関係

本項目は、既に決定されている次の仕様を前提とする。

- Domain機能は可能な限りRPX packageとして実装する
- 通常値はPerceus方式で自動管理する
- 外部Resourceの意味的寿命はbracketで管理する
- Effect handlerはdeep handler
- Continuationはone-shot
- Failureはfailure Eとして型へ記録する
- DefectとTerminal failureを通常Failureから区別する
- Taskは構造化Concurrencyに従う
- Cancellationは協調的な構造化中断である
- Native packageは通常package APIを維持する
- PackageInstanceId、ModuleId、BindingId、TypeIdを利用できる
- Module sealingとabstract type identityを維持する
- 一般的なcell／ref、borrow system、weak referenceはv1へ含めない

### 0. 基本原則
#### 0.1 Kernelの目的

Kernelは、RPXの型付き世界を、Rust Runtime、OS、外部ライブラリ、BackendおよびNative packageへ安全に接続する最小限の信頼層である。

KernelはDomain概念を無制限に追加する場所ではない。

Kernelが担当するもの:
- Runtime valueとの安全な接続
- Effect／Continuation Runtime
- Perceus
- Task Scheduler
- Resource gateway
- Foreign call gateway
- Validation boundary
- Native Adapter ABI
- Typed Value Builder
- Fault isolation

通常RPX packageが担当するもの:
- Document
- Slide
- Circle
- Chart
- Animation
- Layout
- Image処理の高水準API
- Font処理の高水準API
- Backendの公開抽象化

#### 0.2 Native packageの正式な許可

頻繁に利用される機能、性能上の必要性が高い機能、OSや外部ライブラリとの接続が必要な機能には、RustによるNative実装を認める。

特に、標準packageはNative実装を持てる。

例:
- task
- resource
- image
- font
- path
- text shaping
- compression
- PDF／SVG／PPTX backend
- GUI
- network


Native実装の存在は、機能をKernel primitiveへ移すことを意味しない。

公開上は通常のpackage APIを維持する。

### 1. Package APIとNative実装
#### 1.1 三層構造

Native対応packageを次の三層に分ける。

Package API:
利用者から見える型、Effect、Failure、module、documented behavior

Portable implementation:
通常のRPXで記述された参照実装またはfallback

Native implementation:
Rust、OS API、外部library等を使用する実装

#### 1.2 意味上の正本

意味上の正本は、Native実装ではなく次である。

- Packageの公開signature
- 規範的な意味
- Resource／Cancellation契約
- Conformance test


Native版は、その契約を実装する一つの手段である。

#### 1.3 呼出し側からの不可視性

利用者は通常のimportと関数呼出しを使用する。

(import std/image)

(val image
  (image/decode source-bytes))


Portable版かNative版かによって、次を変えてはならない。

- 公開型
- Required Effect
- Failure型
- Resource lifetime
- Cancellation semantics
- Scope規則
- Binding identity
- 規範的な結果


次の差は許容する。

- 実行時間
- Allocation回数
- Memory使用量
- SIMD利用
- Worker数
- Cache
- Perceus reuseの成立

### 2. Native化の単位
#### 2.1 登録単位

Native実装の登録単位はpackageとする。

NativePackageImplementation
├─ Binding A
├─ Binding B
├─ Validator
├─ Codec
└─ Representation family

#### 2.2 置換単位

実際のNative置換はbinding単位で行える。

std/image:
decode   -> Native
resize   -> Native
encode   -> Native
metadata -> Portable

#### 2.3 独立package原則

意味的に独立し、Native実装、ABIまたはCapabilityを個別管理する価値がある機能は、原則として独立packageへ分離する。

推奨:
std/task
std/image
std/font
std/resource

避ける:
一つの巨大なstd/nativeへ全機能を集約


Native binding一個ごとにpackageを分ける必要はない。

### 3. Native実装の分類

Native bindingを次の二種類に分類する。

OptionalAcceleration:
Portable版があり、Native版は高速化

NativeRequired:
Host capabilityまたはNative実装なしでは機能を提供できない

#### 3.1 Optional acceleration

Native版が利用できなければPortable版へfallbackする。

例：

- 文字列検索
- 画像resize
- 数値kernel
- Compression

#### 3.2 Native required

Native実装が必要な機能は、要件を明示する。

例：

- OS Window
- Native File gateway
- GPU driver
- Platform clipboard


利用不能な場合は、構造化されたpackage load errorまたはCapability unavailableを返す。

### 4. 信頼レベル

Native codeを次の信頼層に分ける。

1. Runtime intrinsic
2. Trusted standard native package
3. Explicitly authorized workspace native package
4. Third-party native package
5. Portable RPX package

#### 4.1 Runtime intrinsic

次のようなRuntime中核だけが使用する。

- Perceus内部
- Continuation
- Scheduler
- Cleanup stack
- Typed Core lowering


Runtime-private ABIへアクセスできる。

#### 4.2 標準Native package

Toolchainとともにbuild、配布およびtestされる。

安定したAdapter ABIを使用し、Runtime-private ABIへ原則アクセスしない。

#### 4.3 Workspace Native package

ManifestまたはHost policyによる明示許可を要求する。

#### 4.4 第三者Native package

v1では既定で自動実行しない。

将来対応では、少なくとも次を必要とする。

- Origin
- Artifact hash
- Signature
- Host capability
- ABI compatibility
- Platform target
- Isolation policy

### 5. Foreign値の分類
#### 5.1 外部データ

内容を検査して通常のRPX値へ変換できるもの。

- File bytes
- JSON
- Image data
- Font data
- Network message
- Document file

#### 5.2 外部オブジェクト

外部側に実体があり、通常dataへ完全変換しないもの。

- File descriptor
- Socket
- GPU texture
- Native window
- Font face
- Database connection


外部データにはValidation boundaryを使用する。

外部オブジェクトには型付きopaque handleとResource lifetimeを使用する。

### 6. 未検証値
#### 6.1 通常RPXコードへの非公開

未検証の外部表現を、次の形で通常RPXコードへ公開しない。

- 万能ForeignValue
- Raw pointer
- Untyped native object
- dynamic anyによるResource包装


未検証状態はtrusted adapter内部に閉じ込める。

#### 6.2 Adapter内部表現

Adapter内部では、Rust型等として表現できる。

RawForeignValue
UnvalidatedImage
NativePointer
OsHandle


これらはRPXの通常型ではない。

#### 6.3 dynamicとの区別
dynamic S:
通常のRPX値に関するgradual typing

Foreign handle:
外部Resourceへのopaque capability

Unvalidated external data:
Adapter内部状態


dynamicをForeign Resource、ownershipまたはvalidationの代替にしない。

### 7. Validation boundary
#### 7.1 定義

Validation boundaryは、未検証の外部表現から、宣言された型の不変条件を満たすRPX値へ移る境界である。

Untrusted representation
↓
Representation validation
↓
Type validation
↓
Semantic validation
↓
Policy validation
↓
Typed RPX value

#### 7.2 Representation validation

次を検査する。

- Length
- Alignment
- Tag
- Pointer validity
- Encoding
- Integer overflow

#### 7.3 Type validation
- Constructor
- Field数
- Field型
- Record shape
- Required property

#### 7.4 Semantic validation
- Dimensionが有効
- Buffer長が一致
- Path command列が有効
- Document所有treeがacyclic
- Resource状態が有効

#### 7.5 Policy validation
- Memory budget
- File size limit
- Capability policy
- Sandbox policy
- Decompression ratio

### 8. Validator API
#### 8.1 原則

Validatorの正準的な公開結果は、原則としてresultとする。

validate:
Input -> result<Validated, ValidationError>


理由：

- 外部入力不正は通常予想される
- Errorを値として表示・蓄積できる
- 複数errorを収集できる


LoaderやDecoderは、必要なら明示的にFailureへ変換する。

(or-raise
  (validate-image bytes))

#### 8.2 純粋性

可能な限り次を分離する。

Acquire／Read:
Effectful

Parse／Validate:
Pure

Register／Commit:
Effectful

#### 8.3 Handle validator

Live handleの検査はRuntime tableやexecutorへ依存し得るため、pureである必要はない。

通常RPXコードへ直接公開せず、adapter call境界で内部的に実施できる。

### 9. Validator authority
#### 9.1 Type identityとの関連付け

Validatorは文字列名ではなく次へ関連付ける。

PackageInstanceId
ModuleId
TypeId
ValidatorVersion

#### 9.2 登録権限

Validatorを登録できる主体を次へ限定する。

- TypeIdを定義したpackage
- Manifestで正式指定されたNative implementation
- Toolchain同梱Runtime intrinsic


依存packageが、別packageのsealed abstract type用Validatorを勝手に登録できない。

#### 9.3 成功後の保証

Validator成功後、少なくとも次を保証する。

- Runtime tagと宣言型の一致
- Constructor／fieldの整合
- 型固有不変条件
- Scopeの整合
- Ownership状態
- Adapter identity
- ABI version
- Perceus管理情報

### 10. Typed Value Builder
#### 10.1 基本方針

Native packageはRuntime内部layoutを直接操作せず、型付きBuilder APIを使用してRPX値を構築する。

候補操作：

build-unit
build-bool
build-int
build-f64
build-str
build-bytes
build-list
build-tuple
build-record
build-variant
build-abstract-native-value
build-resource-handle

#### 10.2 Builder検査
- Runtime identity
- TypeId
- ConstructorId
- Field数
- Field型
- Scope
- Ownership
- RepresentationId
- Memory budget

#### 10.3 Transactional construction

複雑な構築はBuilder session内で行う。

Builder session
├─ 一時所有
├─ Field追加
├─ Validation
└─ Commit／Abort


Commit前に失敗した場合、全ての一時値をcleanupする。

CommitまたはAbort後のsessionは再利用できない。

#### 10.4 Error分類
外部入力不正:
Validation error

Memory budget不足:
resource-exhausted

Native実装とmetadataの矛盾:
Defect

Runtime内部破損:
Terminal failure

### 11. Abstract typeとNative representation
#### 11.1 Native表現の許可

正式に登録されたNative implementationは、自packageのabstract typeをNative固有表現で実装できる。

公開型:
image

Portable表現:
RPX data

Native表現:
Rust buffer／GPU object等

#### 11.2 条件
- PackageInstanceId一致
- TypeId一致
- Public API hash一致
- ABI version一致
- Validator authorityあり
- Typed Builder使用

#### 11.3 Representation family

表現を次で識別する。

RepresentationIdentity {
  PackageInstanceId,
  TypeId,
  RepresentationId,
  RepresentationAbiVersion
}

#### 11.4 Binding分類
RepresentationNeutral:
内部表現へ依存しない

RepresentationAware:
特定RepresentationIdへ依存する


Representation-aware binding群は同じ表現familyへ整合しなければならない。

### 12. Representation変換
#### 12.1 Unsafe reinterpretの禁止

Portable表現のpointerをNative表現として読み替える等の暗黙reinterpretを禁止する。

#### 12.2 Converter

表現間変換は正式なconverterとして登録する。

MoveConverter:
元値を消費

CopyConverter:
元値を維持


Shared backing converterはv1の一般機構へ含めない。

#### 12.3 Failure
入力内容／Resource上の変換失敗:
resultまたはfailure conversion-error

Representation metadata不一致:
Defect

ABI不一致:
Native implementationのLoad拒否

### 13. Foreign handle
#### 13.1 用途別抽象型

一般的なRaw foreign pointerを公開しない。

用途別abstract typeを使用する。

file-handle<s>
socket<s>
gpu-buffer<s>
font-face<s>

#### 13.2 Handle検査

少なくとも次を検査する。

- nullではない
- 正しいadapter
- 正しいResource kind
- 正しいRuntime instance
- 正しいABI version
- 有効なscope
- Open状態
- Executor affinity

#### 13.3 偽造禁止

通常RPXコードは次を行えない。

- intからhandleを作る
- pointerを取得する
- Handle headerを書き換える
- Type／Adapter identityを偽装する
- 内部表現をserializeする

### 14. Foreign ownership contract

各引数と結果は、ownership metadataを持つ。

Borrowed
Owned
Shared
Copied
Scoped


通常RPX利用者はこれらを手書きしない。

Trusted adapter metadataとCompiler／Runtimeが処理する。

### 15. Borrowed
#### 15.1 意味

Foreign call中だけ値を使用し、Call終了後は保持しない。

RPX owns x
↓
ForeignがCall中だけ参照
↓
Call終了
↓
RPXは引き続きxを使用可能

#### 15.2 禁止
- Global stateへの保存
- 非同期operationへの直接保存
- Callback Closureへの捕捉
- Call後のpointer使用


Call後も必要ならOwned、SharedまたはCopiedを使用する。

#### 15.3 違反
安全に検出可能:
Defect

Use-after-free等の可能性:
Terminal failure

### 16. Owned
#### 16.1 意味

Call開始時点で所有権をForeign側へ移送する。

Call前:
RPX owns x

Call後:
Foreign owns x


RPX側は以後その値を使用できない。

#### 16.2 FailureとCancellation

Owned引数はCallの成功、FailureまたはCancellationにかかわらず、RPXへ暗黙には戻らない。

再試行可能なAPIでは、専用結果型によって値を明示的に返す。

SubmitResult<A, E> =
  Submitted
  | Rejected(A, E)

#### 16.3 Foreign側の責任

Foreign側は最終的に次のいずれかを行う。

- drop
- 別ownerへ移送
- 正式な結果としてRPXへ返す
- Resource cleanupへ移す

### 17. Shared
#### 17.1 意味

RPXとForeign側がCall後も同じ値を保持する。

#### 17.2 Retain token

Shared参照にはopaqueなretain tokenを使用する。

retain
↓
Foreign保持
↓
release token
↓
Perceus drop


Tokenは高々一回だけreleaseできる。

#### 17.3 Thread共有

別Threadで保持する場合はPerceus Shared表現とatomic参照カウント等を使用する。

#### 17.4 利用制限

Sharedは必要な場合だけ使用する。

可能ならBorrowed、OwnedまたはCopiedを優先する。

### 18. Copied
#### 18.1 意味

Foreign側へ独立表現を複製する。

RPX value
↓ copy
Foreign value


両者の寿命は独立する。

#### 18.2 解放責任

Foreign copyはForeign側が解放する。

#### 18.3 利点と位置付け

Copy costはあるが、ABI安定性と安全性が高い。v1ではzero-copyよりCopiedを選ぶことを許容する。

### 19. Scoped
#### 19.1 意味

特定scope内だけ有効なviewまたはhandle。

ForeignView<s, T>

#### 19.2 Escape

scope sの外へ返す、保存する、Callbackへ保持させる、Taskへ送ることを禁止する。

#### 19.3 v1での制限

一般borrow systemは導入しないため、Scoped viewは限定されたtrusted APIへ限る。

標準一般APIはOwned結果またはResource handleを優先する。

### 20. Foreign result

結果を次のように分類する。

OwnedResult
ForeignResource
ScopedResult
SharedResult

#### 20.1 OwnedResult

RPX側が新しい値として所有し、通常値ならPerceus管理へ入れる。

#### 20.2 ForeignResource

Acquire成功時にopaque handleを構築し、bracket cleanupを登録する。

#### 20.3 ScopedResult

既存Resource scopeへ依存する。結果型にscopeが残り、escapeできない。

#### 20.4 SharedResult

Package固有のmanaged handleとして設計する。万能Shared foreign objectは提供しない。

### 21. Resource lifetime
#### 21.1 責任分離
RPX wrapper memory:
Perceus

外部Resource:
bracket／adapter cleanup

#### 21.2 Acquireと登録

Acquire成功とcleanup登録を不可分に扱う。

Acquire
↓
Handle構築
↓
Cleanup登録
↓
RPXへ公開


公開後にcleanup責任を未登録の状態を作らない。

#### 21.3 状態
Open
-> Closing
-> Closed


Releaseは高々一回だけ実行する。

#### 21.4 Release failure

既存のPrimary／suppressed規則に従う。

Body正常 + Release failure:
Release failureがPrimary

Body Failure + Release failure:
Body FailureがPrimary

Cancellation + Release failure:
Cancelledが主状態

### 22. Partial construction
#### 22.1 Commit point

Adapter内部で、成功結果公開前にcommit pointを設ける。

Prepare
↓
Validate
↓
Commit
↓
Publish

#### 22.2 Failure
Commit前:
Adapterが部分構築物をcleanup

Commit後:
結果またはbracketがcleanup責任を持つ


中間状態を通常RPXコードへ公開しない。

### 23. Execution contract

各Native bindingは次を宣言する。

ExecutionClass
CancellationClass
ExecutorAffinity
CallbackPolicy
PanicPolicy
IsolationClass

### 24. ExecutionClass
Inline
MayBlock
CpuBound
Async

#### 24.1 Inline

Scheduler executor上で直接実行してよい短時間処理。

#### 24.2 MayBlock

I/Oや外部library待機等により長時間戻らない可能性がある。原則としてblocking worker等へ隔離する。

#### 24.3 CpuBound

長時間CPUを占有し得る。CPU用worker pool等へ送る。

#### 24.4 Async

外部operationを開始し、完了通知でTaskを再開する。

具体的な時間閾値は仕様化しない。

### 25. CancellationClass
Cancellable
Cooperative
NonCancellable

#### 25.1 Cancellable

外部cancel APIを用いて停止要求を送れる。

#### 25.2 Cooperative

Native処理がCancellation tokenを定期確認する。

#### 25.3 NonCancellable

処理完了まで停止できない。Cancellation要求は記録し、Call終了後に結果を破棄してTaskをCancelledにする。

#### 25.4 停止確認

Cancel要求と停止完了を分離する。

外部operation停止確認前にBorrowed値、Shared token、Callback Closure等を解放しない。

### 26. Executor affinity
Any
SchedulerLocal
MainUI
Dedicated(ExecutorId)

#### 26.1 Any

任意の適切なworker上で実行可能。

#### 26.2 SchedulerLocal

現在Scheduler executor上でのみ実行する。原則として短いInline処理へ限定する。

#### 26.3 MainUI

UI executorでのみ実行可能。

#### 26.4 Dedicated

GPU、Foreign VM、Database等の専用executorで実行する。

OS Thread IDを通常RPX APIへ公開しない。

### 27. IsolationClass
InProcess
WorkerThread
WorkerProcess

#### 27.1 InProcess

信頼された小さなRust実装等に使用する。

#### 27.2 WorkerThread

MayBlockまたはCpuBound処理をScheduler threadから隔離する。

#### 27.3 WorkerProcess

信頼度の低いcodec、C／C++ library、crashしやすいPlugin等をprocess単位で隔離できる。

Thread内crashがHost process全体を壊し得る場合に使用する。

### 28. Execution契約の整合性

明らかに不適切な組合せをLoad時に拒否する。

例：

MayBlock + SchedulerLocal:
原則拒否

CpuBound + MainUI:
原則拒否

Async + Dedicated:
許可候補

Inline + MainUI:
短時間UI操作として許可候補


例外的に必要な組合せは、明示的なtrusted overrideと適合試験を要求する。

### 29. Cancellationと完了の競合

Foreign operationの主結果は一度だけ確定する。

Pending
├─ Complete
└─ AcceptCancellation

#### 29.1 Cancellationが先

後から届いた結果をTaskへ返さない。

所有権契約に従い、次を行う。

- RPX値をdrop
- ForeignResourceをrelease
- Shared tokenをrelease
- Callbackを解除

#### 29.2 完了が先

正常結果またはFailureをTaskへ渡す。

後から来たCancellation要求は終了状態を変更しない。

### 30. Callback
#### 30.1 Policy

通常Native adapterでは次だけを認める。

None
Queued

#### 30.2 Queued callback

Foreign threadからRPX Closureを直接実行しない。

Foreign event
↓
Runtime queue
↓
Payload validation
↓
適切なTask／executor
↓
RPX Closure実行

#### 30.3 Registration token

Callback登録にはopaque tokenを使用する。

CallbackRegistrationToken


登録時にClosureをShared retainし、解除後にreleaseする。

#### 30.4 Unregister
1. 新規callback受付停止
2. 実行中callbackの終了またはpolicy確定
3. Foreign登録を解除
4. Closure tokenをrelease

#### 30.5 Capture制約

Callback Closureは次を捕捉できない。

- var
- Scoped Resource
- TaskScope
- Task handle
- One-shot continuation
- Thread-affine値
- 非Shared対応値

### 31. Reentrancy

通常Adapter ABIでは、Foreign call中の同期的RPX Callbackを禁止する。

理由：

- Borrowed lifetime
- Handler stack
- Builder transaction
- Cleanup
- Perceus ownership
- Transaction state


Runtime intrinsicだけが非公開の専用契約で同期Reentrancyを利用できる。

### 32. Rust panicとunsafe
#### 32.1 Panic

Rust panicを通常のfailure Eへ変換しない。

契約内の外部失敗:
failure E

Rust panic:
Defect候補

#### 32.2 Panic境界

Native call境界でpanicを隔離可能な場合は、所有権とRuntimeの健全性を確認する。

健全性維持:
Defect

健全性不明:
Terminal failure

#### 32.3 Panic policy
NoPanicContract
PanicSafe
AbortOnly


標準Native packageはNoPanicContractを目標とする。

#### 32.4 Unsafe
- unsafeをAdapter境界内へ限定
- safety conditionを文書化
- Raw pointerをRPXへ公開しない
- PanicをFFI境界外へunwindさせない
- Undefined behaviorを正規意味論に含めない

### 33. Adapter ABIの二層構造
#### 33.1 Adapter ABI

Native packageが使用するversion付きの比較的安定した境界。

- Package descriptor
- Version negotiation
- Capability negotiation
- Opaque value handle
- Typed Builder
- Ownership contract
- Execution contract
- Completion token
- Callback registration

#### 33.2 Runtime-private ABI

Toolchain内部専用であり、互換性を保証しない。

- Perceus header
- Reference count field
- Reuse token
- Continuation frame
- Cleanup stack
- Scheduler queue
- Typed Core
- Runtime Type DAG pointer


一般Native packageへ公開しない。

### 34. ABI安定性
#### 34.1 v1の方針

version付きopaque Adapter ABIを採用するが、長期binary互換性を過度には約束しない。

保証:
明示されたABI majorとtoolchain互換範囲

許容:
Toolchain更新時のNative package再build

非保証:
Runtime-private layoutの互換性

#### 34.2 互換性レベル
Source API compatibility
Adapter ABI compatibility
Toolchain-private compatibility


Source API compatibilityを最優先する。

### 35. ABI version
AbiVersion {
  major,
  minor
}

#### 35.1 Major

互換性を壊す変更で増加する。

Major不一致時はNative実装をLoadしない。

#### 35.2 Minor

後方互換な機能追加に使う。

RuntimeがNative packageの要求するminorおよびCapabilityを提供できる場合に使用可能。

#### 35.3 Implementation version

Bug fixや性能改善はABI versionとは別のimplementation versionで管理する。

### 36. Runtime capability

Native packageは必要Capabilityを列挙する。

例：

typed-value-builder-v1
borrowed-bytes-view-v1
shared-value-token-v1
async-completion-v1
cancellation-token-v1
queued-callback-v1
main-ui-executor-v1
native-representation-v1


条件：

RequiredCapabilities(native)
⊆
ProvidedCapabilities(runtime)


不足時：

OptionalAcceleration:
Portable fallback

NativeRequired:
Load failure

### 37. Native package descriptor
#### 37.1 Entry point

Native packageは単一の登録entrypointを公開する。

概念的名称：

rpx-native-package-entry

#### 37.2 Descriptor
NativePackageDescriptor {
  descriptor-size,
  descriptor-version,
  ABI-version,
  implementation-version,
  package-identity,
  public-API-hash,
  required-runtime-capabilities,
  supplied-capabilities,
  binding-descriptors,
  representation-descriptors,
  validator-descriptors,
  codec-descriptors
}

#### 37.3 拡張性

各descriptorは次を持てる。

- Struct size
- Struct version
- Flags


未知fieldを勝手に解釈しない。

所有権やExecution classの未知enum値を既知値へ暗黙変換しない。

### 38. Identity
#### 38.1 Package
PackageInstanceId

#### 38.2 Binding
BindingIdentity {
  PackageInstanceId,
  ModuleId,
  BindingId
}

#### 38.3 Type
TypeIdentity {
  PackageInstanceId,
  ModuleId,
  TypeId
}

#### 38.4 Representation
RepresentationIdentity {
  PackageInstanceId,
  TypeId,
  RepresentationId,
  RepresentationAbiVersion
}


文字列名やRust symbol名だけで対応付けない。

### 39. Public API hash
#### 39.1 計算対象
- 公開module identity
- BindingId
- TypeId
- 関数型
- Required Effect
- Failure型
- Abstract type identity
- 公開constructor
- Resource／scope制約
- Task境界制約
- Representation要件

#### 39.2 除外対象
- コメント
- 空白
- Source path
- Private binding
- Portable実装本体
- Diagnostic wording

#### 39.3 構造照合

Hash一致だけでなく、Load時に構造化metadataも照合する。

### 40. Binding descriptor
NativeBindingDescriptor {
  binding-identity,
  function-signature,
  entrypoint,
  ownership-contract,
  execution-contract,
  representation-requirements,
  required-runtime-capabilities,
  optional-or-required
}


Rust symbol名はdescriptor内部のentrypointにすぎず、公開RPX identityではない。

### 41. Opaque Value handle
#### 41.1 基本

Native packageはRuntime valueへの生pointerを受け取らず、opaque handleを使用する。

RpxValueHandle

#### 41.2 Runtime所属

Handleは一つのRuntime instanceに所属する。

別Runtimeでの使用を禁止する。

#### 41.3 操作

Runtimeのchecked APIを通じて行う。

- Primitiveの読出し
- Value kindの検査
- FieldのBorrowed access
- Typed Builder
- Owned transfer
- Shared retain／release

### 42. Typed Borrowed view

性能が必要な型に、Call中限定のread-only viewを提供できる。

BorrowedUtf8View
BorrowedBytesView
BorrowedIntArrayView
BorrowedFloatArrayView


規則：

- Call中だけ有効
- Runtimeがpointerとlengthを検証
- Call後の保持禁止
- Async operationへの直接保存禁止
- Mutable viewは一般ABIへ含めない


Call後も必要ならOwnedまたはCopiedへ変換する。

### 43. Async completion ABI
#### 43.1 Token
AsyncCompletionToken


One-shotである。

#### 43.2 状態
Pending
Completed
Cancelled
Consumed

#### 43.3 通知
complete-success(token, owned-result)
complete-failure(token, owned-error)
complete-cancelled(token)


一度だけ通知できる。

#### 43.4 Resume

Native threadからContinuationを直接resumeしない。

Runtime queueへ通知し、SchedulerがTaskを再開する。

#### 43.5 Late result

Cancellation後に到着した結果はTaskへ返さず、ownership契約に従ってdropまたはreleaseする。

### 44. Cancellation token ABI

Operation限定のopaque tokenを使用する。

Native側は次を行える。

- Cancellation要求の確認
- Cancel hookの登録


次は行えない。

- Cancellation解除
- 別TaskのCancellation
- Tokenのoperation外保持
- Runtime内部状態への直接アクセス

### 45. Error ABI

Rust固有errorをそのままRPXへ返さない。

公開Failure型へ明示変換する。

Rust error
↓
Adapter mapping
↓
Typed Builder
↓
公開Failure値


想定外の内部errorを、文字列化して通常Failureへ偽装しない。

契約内:
Failure

Adapter不具合:
Defect

Memory safety不明:
Terminal failure

### 46. Process isolation
#### 46.1 Worker process

信頼度の低い外部libraryはWorker processへ隔離できる。

#### 46.2 値の受渡し

Process境界では次を使用する。

- Version付きcodec
- 明示的なShared memory protocol
- Validation


Runtime内部pointerやNative representationのraw dumpを送らない。

#### 46.3 Crash分類
外部serviceの停止:
Service failure候補

Toolchain同梱Native実装のcrash:
Defect

Host process自体の整合性喪失:
Terminal failure

### 47. SerializationとCodec

Opaque Native valueのmemory表現をそのまま保存しない。

Package定義のversion付きcodecを使う。

Value
↓ encode
Portable representation
↓ save／send


再読込時：

Portable representation
↓ decode／validate
Typed value


CodecはPackageInstanceId、TypeId、CodecVersionへ関連付ける。

### 48. Cross compilation
#### 48.1 HostとTarget
Build-host adapter
Target-runtime adapter


を区別する。

通常のTarget Native packageをBuild host上で実行しない。

#### 48.2 Sidecar metadata

Native artifactには静的に読めるdescriptor metadataを添付できる。

Native binary
+
Hashed／signed sidecar descriptor


Runtime Load時にbinary descriptorと照合できる。

### 49. Native package初期化
#### 49.1 Describe

Metadataを返す段階。

原則として次を行わない。

- Network access
- Window作成
- Background Thread開始
- User Resource読込み
- Callback登録

#### 49.2 Initialize

Runtime capabilityを受け取り、必要な状態を構築する。

Partial failure時には確保済み状態をcleanupする。

#### 49.3 Arbitrary top-level effect

Native library load時の任意副作用を許可しない。

初期化EffectはRuntimeが明示的に管理する。

### 50. ShutdownとUnload
#### 50.1 Shutdown順序
1. 新規Call受付停止
2. Async operationの終了・Cancellation
3. Callback解除
4. Native Resource release
5. Shared token release
6. Package local state shutdown

#### 50.2 Unload

v1ではNative libraryの物理的な動的unloadを保証しない。

Runtime instanceの生存中はlibraryをloadしたままにできる。

論理shutdownと物理unloadを区別する。

### 51. Security policy
#### 51.1 ABIと許可の分離

ABI互換であっても、そのNative codeを実行してよいとは限らない。

#### 51.2 確認項目
- Origin
- Artifact hash
- Signature
- Requested Capability
- Platform target
- Isolation policy
- Workspace／Host permission

#### 51.3 v1既定
Toolchain同梱標準Native:
許可

Workspace Native:
明示許可

Registry由来Native:
既定では実行しない、または将来対応

### 52. Diagnostic

Native実装を使用できない理由を構造化して報告する。

- ABI major mismatch
- Required runtime capability unavailable
- Public API hash mismatch
- Binding signature mismatch
- Ownership contract mismatch
- Execution contract unsupported
- Representation conflict
- Missing required binding
- Native artifact unavailable
- Security policy rejection


OptionalAccelerationがPortable版へfallbackした場合、通常の実行errorにはしない。

Verboseまたはdebug modeでは理由を表示できる。

### 53. 適合試験
KER-01：未検証値の非公開

Native adapterがRaw foreign valueを通常RPX値として返そうとする。

期待結果：

拒否

KER-02：Validator成功

有効な外部入力をValidatorへ渡す。

期待結果：

宣言型の不変条件を満たすRPX値を返す

KER-03：Validator失敗

不正な画像dimension等を渡す。

期待結果：

ValidationError
Defectにはしない

KER-04：Validator contract違反

成功を返した値が型不変条件を満たさない。

期待結果：

Defect
Memory safety不明ならTerminal failure

KER-05：Sealed type authority

別packageがsealed abstract type用Validatorを登録する。

期待結果：

Load時に拒否

KER-06：Borrowed保持違反

Foreign側がBorrowed値をCall後も保持する。

期待結果：

検出可能ならDefect
Use-after-freeの可能性があればTerminal failure

KER-07：Owned移送

Owned値をForeignへ渡した後、RPX側で再利用する。

期待結果：

静的拒否またはDefect

KER-08：Owned Call failure

Owned引数を受け取ったCallがFailureで終了する。

期待結果：

値をRPXへ暗黙に戻さない
Foreign側がcleanup責任を持つ

KER-09：Shared release

Foreign側が保持終了時にShared tokenを一回releaseする。

期待結果：

正常drop


二回release：

Defect

KER-10：Partial construction

Builder commit前にFailure。

期待結果：

全一時値をcleanup
部分値を公開しない

KER-11：Resource acquire

Acquire成功。

期待結果：

Handle公開前にcleanup登録済み

KER-12：Late async result

Cancellation確定後にAsync結果が到着。

期待結果：

Taskへ返さない
結果をdrop／release
二重resumeしない

KER-13：Callback unregister race

Callback実行中にunregisterする。

期待結果：

実行中Callbackが安全に終了後、
Closure Shared tokenをrelease

KER-14：Reentrant callback

通常adapterがForeign call中に同期Callbackする。

期待結果：

契約違反として拒否

KER-15：MayBlock

MayBlock bindingを呼ぶ。

期待結果：

無関係Taskを不必要に停止させない

KER-16：Affinity違反

MainUI限定bindingを不適切なexecutorから直接呼ぶ。

期待結果：

静的拒否、dispatch、または安全なRuntime拒否

KER-17：Rust panic

Native bindingがpanicする。

期待結果：

通常Failureへ変換しない
隔離可能ならDefect
不可能ならTerminal failure

KER-18：ABI mismatch

ABI majorが異なるNative package。

期待結果：

Native実装をLoadしない
OptionalならPortable fallback

KER-19：Capability不足

必要CapabilityがRuntimeにない。

期待結果：

Optionalならfallback
RequiredならLoad failure

KER-20：Public API mismatch

Public API hashまたは構造化signatureが一致しない。

期待結果：

Native bindingを接続しない

KER-21：Representation mismatch

値とbindingのRepresentationIdが一致しない。

期待結果：

正式converter、fallback、またはLoad拒否
Unsafe reinterpretをしない

KER-22：Builder型不一致

Native adapterが不正なconstructor fieldを渡す。

期待結果：

Builderが拒否
Adapter Defectとして分類

KER-23：Runtime instance mismatch

Runtime AのValue handleをRuntime Bで使用する。

期待結果：

Defect

KER-24：Native／Portable同値性

同じ規範入力をPortable版とNative版へ与える。

期待結果：

Package仕様が定める同値条件を満たす

### 54. 関連する未決定項目

#### `OPEN-NATIVE-ABI-SCHEMA-001`

- Descriptorの具体的binary schema
- 整数bit幅
- C ABI型
- Rust SDK trait
- Symbol decoration
- Sidecar形式

#### `OPEN-NATIVE-SEC-001`

- 第三者Native package署名
- Registry policy
- Sandbox
- Capability permission UI

#### `OPEN-KER-CODEC-001`

- Codec version
- Migration
- Cross-process serialization
- Shared memory protocol

#### `OPEN-CON-PAR-001`

- 明示的CPU並列API
- Public Sendable／Shareable制約
- Worker pool policy

#### `OPEN-TST-001`

- Fake executor
- ABI race test
- Native／Portable differential test
- Panic／process crash test

### 55. 最終状態

```text
OPEN-KER-001:
RESOLVED
```


本解決により、Reciplexaは次を提供する。

- 最小Kernel
- 通常packageとして見えるRust Native implementation
- Binding単位のNative置換
- Portable fallback
- 未検証外部値を閉じ込めるValidation boundary
- Package権限に基づくValidator
- Typed Value Builder
- Opaque Foreign handle
- Borrowed／Owned／Shared／Copied／Scoped ownership
- bracketによる外部Resource管理
- Blocking／CPU／Async実行契約
- CancellationとLate resultの安全な処理
- Executor affinity
- Queued Callback
- Reentrancy制限
- Panic／unsafeのFault分類
- Native representation family
- Version付きopaque Adapter ABI
- Package／Binding／Typeの安定identity
- Capability negotiation
- Native codeのSecurity policy
- Comprehensive conformance tests

## OPEN-IR-001 Layered IR・増分コンパイル・描画意味論・Motion・Backend境界
### DD-IR-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Layered IR
- 差分コンパイルと増分評価
- Geometryと座標系
- 2D StackingとCompositing
- Paint、Color、Alpha、Blend
- 多言語Text基盤
- Time、Signal、Keyframe
- Nested CompositionとTime Transform
- Visual Processing DAG
- Backend Capability
- Fallback、Planning、Emission
- IR拡張・公開・Validation

### 0. 基本方針

Reciplexaの描画系は、RPXコードから直接Backend命令を生成する単一変換にはしない。

次のLayered IRを基本構造とする。

RPX Source
↓
Typed Core IR
↓
Package固有Domain IR
↓
Layout IR
↓
Visual IR
↓
Evaluated Visual IR
↓
Compositing IR
↓
Resolved Geometry／Render IR
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


各IRを設ける目的は、IRの数を減らすことではなく、各変換を次の性質へ限定することである。

- 単純
- 局所的
- 検証可能
- Cache可能
- 差分更新可能
- 並列実行可能
- Backendから分離可能


全IRを毎回完全にmaterializeすることは要求しない。必要に応じて、遅延view、fragment、Page、Frame、Tile、chunkとして生成できるものとする。

### 1. 性能を設計要件とする

性能は将来の実装最適化だけの問題ではなく、IR設計上の必須要件とする。

対象とする規模には、少なくとも次を含む。

- 数千Pageの長大文書
- 数百〜数千Slideの資料
- 数十万Text node
- 多数のFont、画像、動画Resource
- 多数Layerを持つComposition
- Frameごとに変化するAnimation
- Filter、Mask、Blendを含む映像処理
- 高解像度Previewと動画Export


各処理段階は、次を可能にしなければならない。

- 変更されていないsubtreeの再利用
- 部分的な再Layout
- Page／Slide／Frame／Tile単位の処理
- Resource decodeの遅延
- Node単位のCache
- Static subtreeのFrame間共有
- Task Runtimeによる並列化
- Stale結果のCommit拒否
- Memory budgetに基づくCache eviction

### 2. 差分コンパイルと増分処理

SourceからArtifactまでを、差分更新可能な依存pipelineとして設計する。

Source edit
↓
Incremental parse
↓
差分Macro展開
↓
差分名前解決・型検査
↓
Typed Core更新
↓
Domain IR更新
↓
Layout更新
↓
Visual／Compositing fragment更新
↓
Backend再計画
↓
Artifact更新


次の三処理を区別する。

差分コンパイル:
SourceからTyped Coreまで

増分評価:
Typed CoreからDomain／Visual IRまで

差分Rendering:
Render fragment、Tile、Backend出力の更新


各段階は概念的に次を追跡できなければならない。

- Stable identity
- Revision
- Content fingerprint
- Dependency set
- Change set
- Provenance
- Diagnostics


差分処理は常に使用する必要はない。小規模な入力では全体再構築を選択できる。

Incremental result
=
Full rebuild result


を必須条件とする。

### 3. IR追加の基準

IRの層数を最小化すること自体を目標にしない。

次のいずれかが成立する場合、独立IRを設ける価値がある。

- 前後で成立する不変条件が異なる
- 独立したValidatorを配置できる
- 独立したCache境界になる
- 情報を意図的に失う段階である
- 増分更新の粒度が異なる
- 異なるexecutorやResourceを必要とする
- 複数Backendから共有される
- 専用最適化passが必要である


単なるfield名変更だけの変換には、独立IRを設けない。

### 4. Domain IR

Domain IRは、文書・Slide・Chart・Diagram等の高水準な意味を保持する。

- Document
- Section
- Paragraph
- Slide
- Table
- Chart
- Figure
- Caption
- Semantic group
- Accessibility structure


Domain IRは一つの万能schemaにはしない。各packageが固有Domain IRを定義できる。

Document IR
Slide IR
Chart IR
Diagram IR
Math IR


各Domain IRは、最終的に共通Visual IRまたは対応するLayout IRへloweringできなければならない。

巨大なDomain構造を即座に完全展開せず、次を許可する。

- Lazy collection
- Virtualized sequence
- Repeated template
- Shared style
- Master／instance
- Query-backed data

### 5. Layout IR

Layout前後を独立した段階として扱う。

Layout前には次を保持できる。

- Flow
- Constraint
- Intrinsic size
- Relative length
- Style
- Inline object
- Page／Column条件


Layout後には次を確定する。

- 位置
- Size
- Line break
- Page break
- Baseline
- Column placement
- Continuation state


長文では、変更位置から再Layoutし、新旧のLayout stateが一致するstabilization point以降を再利用できるようにする。

### 6. Visual IR

Visual IRはBackend非依存の意味的な視覚構造である。

基本要素には次を含む。

- Group
- Shape
- Path
- Text
- Image
- Transform
- Clip
- Mask
- Paint
- Opacity
- Blend
- Filter／Visual Processor


Visual IRは不変かつ階層的とする。

親のTransform、Opacity、Style等を全子へ早期に焼き込まず、局所propertyとして保持する。

VisualNode {
  geometry,
  local-transform,
  paint,
  children,
  stacking,
  provenance
}


これにより、親propertyだけの変更で全subtreeを再構築することを避ける。

### 7. 座標空間

少なくとも次の座標空間を区別する。

Local space
Parent space
Composition space
Viewport space
Device space


共通2D座標系は次とする。

原点:
左上

+x:
右

+y:
下


Layout IR、Visual IR、Compositing IR、Render IRはこの規則へ統一する。

Backend固有座標系への変換はBackend loweringで行う。

内部IRでは空間identityを保持し、概念的に次を区別できるものとする。

Point<Local>
Point<Composition>
Transform<Local, Parent>

### 8. 長さ・角度・Transform

規範的な長さ単位はlogical pointとする。

1 inch = 72 logical points


相対単位はLayout段階で解決する。

- %
- em
- logical px


Visual IR以降では、原則として解決済みlogical pointを使用する。

角度は次の規則とする。

0度:
+x方向

正の角度:
時計回り


TransformはSurfaceに記述された順に適用する。

T1
→ T2
→ T3


Visual IRでは編集可能なTransform componentを保持し、Render側ではAffine matrixへ正規化できる。

position
anchor
scale
rotation
skew


2D Transformに前後関係を含めない。

2D Transform:
Geometry配置

Stacking:
2D前後関係

3D position.z:
3D幾何学的Depth

### 9. 2D Stacking

2Dの前後関係の正本は、Stacking context内の子sequenceとする。

先頭:
奥

末尾:
手前


コード記述順は、既定のStacking sequenceを生成する。

ただし、唯一の順序指定方法にはしない。

補助的に整数のstack-levelを使用できる。

小さいstack-level:
奥

大きいstack-level:
手前

同じstack-level:
sequence順


既定値は0とする。

stack-levelは幾何学的なz座標ではない。

Groupは必要に応じて独立Stacking contextを形成し、子のstack-levelは親contextを越えない。

### 10. ShapeとPath

Visual IRでは高水準Shapeを保持できる。

- Line
- Rectangle
- RoundedRectangle
- Circle
- Ellipse
- Polygon
- Polyline
- Arc
- Path


Resolved Geometry IRでは、共通Pathへloweringする。

Render Pathの基本命令は次とする。

MoveTo
LineTo
CubicTo
Close


Quadratic curveやArcは必要に応じてCubicへ変換する。

Pathは複数subpathを持てる。

Fill ruleはGeometryとは分離する。

NonZero
EvenOdd


Path storageはchunk化・構造共有・部分更新可能でなければならない。

### 11. Bounds

Boundsを一種類に統一しない。

GeometryBounds
PaintBounds
EffectBounds
VisibleBounds
ConservativeBounds


状態は次とする。

Bounds =
  Empty
  | Finite(Rect)
  | Unbounded
  | Unknown


ConservativeBoundsは、実際の影響範囲を必ず包含しなければならない。

Actual affected region
⊆
ConservativeBounds


過大評価は許可するが、過小評価は禁止する。

差分再描画の基本Dirty regionは次とする。

DirtyRegion =
  union(old-bounds, new-bounds)


Layer順変更、Blend、Filter、Isolationについては、影響するCompositing groupの安全な範囲へ拡張する。

### 12. Geometry ToleranceとHit testing

Geometry近似はQualityPolicyとdevice-space errorに基づく。

次を区別する。

Geometry approximation tolerance
Bounds safety margin
Hit-test tolerance
Tessellation tolerance


低品質Previewの近似結果をFinal出力へ使用してはならない。

Hit testingは次の段階で行う。

1. Spatial index
2. ConservativeBounds
3. Clip／Mask判定
4. Geometry判定
5. 手前から候補を選択
6. Provenanceから編集対象へ変換


描画順が奥から手前であるのに対し、Hit testingは手前から奥へ行う。

### 13. PaintとColor

GeometryとPaintを分離する。

v1の基本Paintは次とする。

- None
- SolidColor
- LinearGradient
- RadialGradient


基準色空間はsRGBとする。

合成計算の既定working spaceはLinear sRGBとする。

公開Color:
Straight alphaのsRGB

内部Compositing:
Premultiplied alphaのLinear sRGB


Alphaは0..1を基本範囲とする。

内部Filter計算では有限な範囲外値を許可できるが、出力変換時に明示的なmappingを行う。

### 14. Compositing

既定合成演算はsource-overとする。

次を区別する。

Paint alpha
Node opacity
Group opacity


Group opacityは、子へ個別適用するのではなく、Group内を合成した後の結果へ適用する。

必要な場合はIsolated groupを形成する。

標準Layer処理順は次とする。

1. GeometryをPaint
2. 子を合成
3. Filterを適用
4. Maskを適用
5. Layer opacityを適用
6. Blend modeで親へ合成


ClipはGeometry制約、MaskはCompositing操作として分離する。

### 15. Compositing IR

Compositing IRを独立層とする。

概念要素は次のとおりとする。

- PaintGeometry
- DrawText
- DrawImage
- Sequence
- Clip
- Mask
- Filter
- IsolatedGroup
- Composite


この段階で次を解決する。

- Back-to-front順
- Stacking context
- Isolation boundary
- Group opacity
- Blend mode
- Mask
- Filter順序
- 中間surfaceの必要性


意味的Groupを無条件に平坦化してはならない。

### 16. 多言語Text基盤

共通Text基盤は日本語専用にはしない。

次の仮定を禁止する。

- 一文字は一Glyph
- 一Unicode scalarは一表示文字
- Textは常に左から右
- 空白だけが単語境界
- Glyph順は論理Text順と同じ


共通処理は次の段階に分ける。

Unicode Text
↓
Unicode Analysis IR
↓
Shaping Plan IR
↓
Shaped Text IR
↓
Line Break Plan IR
↓
Line Layout IR
↓
Positioned Text IR


LanguageとScriptを別に保持する。

language
script
direction
writing-mode


Writing modeは少なくとも次を含む。

HorizontalTB
VerticalRL
VerticalLR

### 17. Text RunとShaping

次を区別する。

Source order
Logical order
Visual order
Glyph storage order


Grapheme clusterとShaping clusterを同一視しない。

Grapheme:
編集・Cursor操作上の単位

Shaping cluster:
Glyphと元Text rangeの対応単位


Shaping Run内では、少なくとも次を一定とする。

- Font
- Font size
- Script
- Language
- Direction
- Writing mode
- Font feature
- Variation axes


Shaping結果は元Unicode text、Text range、Cluster mappingを失ってはならない。

### 18. 言語package拡張

言語固有規則を共通Render IRへ埋め込まない。

共通Text Layoutは、次の拡張protocolを提供できる。

TextClassifier
LineBreakProvider
HyphenationProvider
JustificationProvider
AnnotationLayoutProvider
GlyphOrientationProvider
FontFallbackPolicy


日本語組版、ルビ、禁則、縦中横等は日本語向けpackageで実装する。

JLReqは、日本語組版packageの主要な規範資料として参照する。共通Text IRは、その実装に必要なUnicode text、縦横方向、注釈関係、Cluster、Line layout情報を失わない。JLReqは主としてJIS X 4051に基づく日本語組版要件を扱うため、低水準Glyph描画仕様ではなく、日本語向けText／Document Layout profileの参照資料として位置付ける。

### 19. Resource

Visual IRではNative handleを直接保持せず、安定Resource identityを参照する。

ResourceId
ContentHash
Revision
Metadata


対象には次を含む。

- Font
- Image
- Video
- Audio
- Color profile
- Pattern


ImageやVideoは必要時にdecodeし、Preview／Final品質を分離する。

Backend固有ResourceはBackend cacheで管理する。

共通IR:
Resource identity

Native Backend:
Decoded image、GPU texture、Font face等

### 20. Time

時間型は次のように分離する。

Time:
符号付きの正確な時刻

Duration:
非負の時間長

TimeOffset:
符号付きの時間差

FrameRate:
正の有理数

FrameIndex:
0-basedの非負整数


規範的な時間は有理時間とする。

実装はtime base付き整数tickを使用できる。

時間区間は半開区間とする。

[start, end)


無期限は数値InfinityではなくUnboundedで表す。

Frameの基準評価時刻はFrame開始時刻とする。

TimeからFrameへの変換では、丸めpolicyを明示する。

Floor
Ceil
Nearest
Exact

### 21. Signal

Signal<A>は解析可能なpureな時間依存値である。

一般のTime -> A関数と完全には同一視しない。

基本分類は次とする。

- Constant
- Keyframed
- Mapped
- Combined
- Sampled
- Selected


通常値AはConstant Signalとして自動的に使用できる。

Signal評価は次を満たさなければならない。

- Pure
- 決定的
- 同じSnapshotとTimeなら同じ結果
- 暗黙I/Oを行わない
- 暗黙の現在時刻を読まない


Randomや外部入力はseedまたはSnapshotとして明示する。

Signal依存関係はv1ではDAGとし、循環を拒否する。

### 22. Keyframe

Keyframe列は時刻順の非空sequenceとする。

Keyframe<A> {
  identity,
  time,
  value,
  outgoing-interpolation
}


同一Signal・同一時刻のKeyframeは高々一つとする。

Keyframeが一つだけの場合、意味上はConstantと等価である。

最初のKeyframeより前と最後のKeyframeより後は、端の値をHoldする。

隣接Keyframeの補間区間は次とする。

[t0, t1)


補間policyは前Keyframeが所有する。

標準補間は次とする。

Hold
Linear
CubicBezierEasing
Steps


時間Easingと空間上のMotion pathを分離する。

### 23. 型別補間

補間能力は型ごとのprotocolとして扱う。

補間可能な候補：

- Number
- Length
- Point
- Vector
- Opacity
- Color
- Angle
- Transform component


離散型はHoldまたはStepsを使用する。

- Boolean
- Text
- Font identity
- Resource identity
- Blend mode
- Stack level


Angleには次の補間policyを持たせる。

Shortest
Clockwise
CounterClockwise
Unwrapped


Colorの既定補間はLinear sRGBかつPremultiplied Alphaとする。

TransformはAffine matrix全体ではなく、position、anchor、scale、rotation等を個別に補間する。

Path morphは専用機能とする。

### 24. Time Transform

親Timelineと子Timelineを区別する。

Temporal Placementは次を持つ。

- Parent active range
- Child source range
- Time transform
- Range policy


Time Transformは任意関数ではなく解析可能な専用IRとする。

標準要素は次とする。

Identity
Offset
Scale
Reverse
Freeze
Compose


範囲policyは次を扱える。

Transparent
Hold
Loop
PingPong
Failure


LoopはEuclidean moduloを使用する。

空のsource rangeへのLoopはValidation errorとする。

可変速度は単純な速度Signalの乗算ではなく、Time Remapとして表す。

動画Frame選択と音声のpitch処理はTime Transformとは分離する。

### 25. Visual Processing DAG

Blur、色補正、Mask、Composite等の処理依存関係を、型付きVisual Processing DAGとして表す。

これは言語のEffect handlerとは別概念である。

公開上のVisual Processorは通常のpackage関数として見える。

(blur source radius)


Compilerは必要に応じてTyped Semantic DAGへloweringする。

DAGはv1ではacyclicとする。

Feedback、Simulation、Previous-frame参照は専用機構として将来扱う。

### 26. Processing Node契約

Nodeは概念的に次を持つ。

- Stable identity
- Typed input
- Typed output
- Parameters
- Bounds function
- Required input region
- Time behavior
- Tileability
- Cache contract
- Execution contract
- Fallback contract


PortとParameterを区別する。

Port:
他Nodeの出力

Parameter:
Static値またはSignal


Graph topologyと時間依存Parameterを分離する。

Parameterだけの変化で、DAG全体をFrameごとに再構築しない。

### 27. Processing DAG評価

出力からのdemand-driven evaluationを基本とする。

現在必要な出力へ到達しないNodeは評価しない。

Node処理は原則としてpureであるため、Dead node除去を安全に行える。

Boundsは入力から出力へ前方伝播する。

必要入力領域は、要求出力領域から入力側へ後方伝播する。

Tileabilityは次へ分類する。

Local
Overlapping
Global
NonTileable


Node cacheは次の単位を持てる。

- Node
- Time
- Time interval
- Spatial region
- Tile
- Quality
- Backend representation


差分無効化は概念的に次の軸で行う。

Node × Time × Spatial region

### 28. Semantic DAGとExecution Plan

利用者の意味構造と最適化後の実行構造を分ける。

Semantic DAG:
RPXコード、GUI、Provenance上の処理

Execution Plan:
Fusion、Task分割、Backend選択、Cache計画


隣接Nodeは、意味を保つ限り融合できる。

ColorMatrix A
→ ColorMatrix B


を一つの実行operationへ融合できる。

ただし、GUI上の個別Node、Failure origin、Provenanceを失ってはならない。

### 29. GUI表現

言語を正本とする方針を維持する。

同じ意味構造に対し、次のviewを提供できる。

Code:
正本、抽象化、Module、Test

Property inspector:
個別parameterの編集

Processing Stack:
直列処理の編集

Processing Graph View:
分岐、合流、共有、依存解析

Timeline／Curve:
時間依存propertyの編集


Graph上のNode位置は意味論に含めない。

Graph GUIは、Visual Processing DAGの一つの表示・編集方法であり、必須のSource形式ではない。

### 30. Backend境界

Render IRまでをBackend非依存の規範的描画意味とする。

対象Backendには次を含む。

- PDF
- SVG
- PPTX
- Raster
- GPU Preview
- Video


各BackendはCapabilityを構造化して宣言する。

- Geometry
- Paint
- Text
- Image
- Compositing
- Filter
- Animation
- Accessibility
- Interactivity
- Document structure


対応水準は次とする。

Native
EquivalentLowering
Approximate
RasterFallback
Unsupported


単純なBooleanだけでCapabilityを表さない。

### 31. Backend PlanningとEmission

Capability判定は描画hot loopではなくPlanning段階で完了させる。

Render IR
↓
Capability Snapshot
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


Backend Planning IRは、各subtreeの実現方法を確定する。

- Native vector
- Native text
- Geometry lowering
- Font embedding
- Raster fallback
- Resource embedding


Backend Emission IRは、形式固有のObject、命令、part、command等を表す。

### 32. Fallback Policy

次の基本policyを持つ。

Strict
Compatible
Editable
BestEffort

Strict

意味を十分保持できなければFailureとする。

Compatible

規定済みの等価loweringまたは局所Raster化を許可する。

Editable

見た目の完全一致より、出力先で編集可能な構造を優先する。

BestEffort

Preview等で警告付き近似を許可する。

Output policyは次の価値を別軸として扱える。

- Visual fidelity
- Editability
- Accessibility
- Searchability
- Interactivity
- Performance
- Artifact size

### 33. Raster Fallback

非対応機能は、可能な限り最小subtreeだけをRaster化する。

Page全体:
可能なら維持

非対応Filter group:
Raster island


Raster islandは次を持つ。

- Bounds
- Resolution
- Color space
- Alpha mode
- Quality policy
- Time identity


Backdrop依存のBlendやFilterでは、正しいCompositing結果を得るためRaster範囲を拡張する。

低解像度Preview用Raster cacheをFinal出力へ使用しない。

### 34. Text Fallback

Text Backend出力は次の段階を持つ。

1. Native Unicode text
2. Positioned glyph＋Font embedding
3. Glyph outline
4. Raster image


Output policyに基づいて選択する。

Accessibility重視:
Native text／Positioned glyph

Visual fidelity重視:
Positioned glyph／Outline

Editability重視:
Backend native text

Preview:
Rasterを許可可能


Font substitutionをFinal出力で黙って行わない。

### 35. Streamingと増分Backend

長大なArtifactでは、全Render IRを同時に保持しない。

Planning pass
↓
Page／Slide／Frame単位のEmission
↓
Finalize


必要に応じて次を全体Planningで収集する。

- Font subset
- Shared image
- Cross reference
- Page count
- Link destination
- Outline


差分更新では、変更Page、Slide、Frame、Resourceだけを再計画できるものとする。

最終ファイル形式が全bytesの再生成を要求する場合でも、Backend計算自体の差分再利用を可能にする。

### 36. IR公開と拡張

Domain IRはpackage固有で自由に定義できる。

共通Visual IRとRender IRは、安全性・Backend互換性・最適化のため、原則として閉じた規範constructor集合を持つ。

拡張が必要な場合は、version付きExtension nodeを登録できる。

Extensionには少なくとも次を要求する。

- Extension identity
- Version
- Type signature
- Validator
- Bounds function
- Required input region
- Tileability
- Time behavior
- Cacheability
- Backend requirement
- Portable fallbackまたはNativeRequired宣言


通常packageが未検証のRender IR constructorを直接作ることは許可しない。

Render IRはTyped BuilderまたはValidatorを介して構築する。

### 37. IR VersionとSerialization

各永続化可能IRにはversionを持たせる。

IRIdentity
IRVersion
SchemaVersion


Runtime内部cacheにはtoolchain-private表現を使用できる。

長期保存・process間転送にはversion付きportable codecを使用する。

内部memory dump:
禁止

Portable serialization:
Validatorを通して復元


古いIR versionは、明示Migration passによって新versionへ変換する。

Migration不能な場合は、元Sourceまたは上位IRから再構築する。

### 38. ValidationとFailure

各IR境界にValidatorを配置する。

Domain Validator
Layout Validator
Visual Validator
Compositing Validator
Render Validator
Backend Validator


通常Failureには次を含む。

- Resource resolution failure
- Font resolution failure
- Unsupported backend feature
- Memory budget exceeded
- Approximation tolerance exceeded
- Output profile violation
- Encoder failure


Defectには次を含む。

- Validated IR内のcycle
- Known Boundsの過小評価
- 解決済みResourceの欠落
- Validated Render IR内のNaN
- Capability PlanとBackend実装の矛盾
- Cache key不一致による誤再利用


RuntimeのMemory safetyや全体整合性が不明な場合はTerminal failureとする。

### 39. 適合試験

少なくとも次を検証する。

- Incremental compileとFull compileの同値性
- Visual subtreeの構造共有
- Transform適用順
- Stack-levelとsequence順
- Group opacityとIsolation
- Boundsの保守性
- Dirty region
- Tile overlap
- Path近似Tolerance
- 多言語ShapingとCluster対応
- BidiのLogical／Visual mapping
- Static SignalのFrame間共有
- Keyframe境界
- Time TransformのLoop／Reverse／Freeze
- Processing DAGのcycle拒否
- Node差分無効化
- Node cacheとFull evaluationの同値性
- Semantic DAGとExecution Planの同値性
- Backend Capability planning
- Fallback policy
- Raster islandの安全性
- Text fallbackの情報保持
- Page／Slide／Frame streaming
- Portable／Native Backendの適合性

### 40. 性能原則
PERF-IR-01:
変更されていないsubtreeの再構築を要求しない

PERF-IR-02:
各loweringをsubtree単位でcache可能にする

PERF-IR-03:
Page／Slide／Frame／Tile単位で要求可能にする

PERF-IR-04:
全IRの完全materializationを要求しない

PERF-IR-05:
Resourceをstable identityとrevisionで共有可能にする

PERF-IR-06:
時間非依存subtreeをFrame間で再利用可能にする

PERF-IR-07:
BoundsとDependency summaryを取得可能にする

PERF-IR-08:
Backend capabilityをhot loop前に解決する

PERF-IR-09:
Cache evictionによって規範結果を変えない

PERF-IR-10:
並列化によって規範結果を変えない

PERF-IR-11:
Stale Jobを現在revisionへCommitしない

PERF-IR-12:
Provenanceを共有・圧縮可能なidentityで保持する

### 41. 公開APIと実装

共通IRは通常の型付きpackageとして公開できる。

性能上重要な処理にはRust Native implementationを許可する。

- Text shaping
- Font処理
- Image decode
- Video decode
- Geometry処理
- Tessellation
- Rasterization
- PDF／PPTX Backend
- GPU Backend


Native実装はOPEN-KER-001のTrusted Adapter ABIに従う。

公開意味:
通常package

実装:
Portable RPX／Rust Native／GPU／Worker process


Native版とPortable版は、各packageが定める適合条件を満たさなければならない。

### 42. 関連する後続項目

#### `OPEN-TEXT-LAYOUT-001`

- 共通Text Layout protocolの正確な型
- Font fallback
- Shaping
- Bidi
- Line breaking
- Justification

#### `OPEN-TEXT-JA-001`

- JLReq参照範囲
- 日本語文字クラス
- 禁則
- 約物
- ルビ
- 縦中横
- 行調整

#### `OPEN-IR-3D-001`

- 3D Scene
- Camera
- Lighting
- Depth
- 2.5D Layer
- 3Dと2D Compositingの接続

#### `OPEN-IR-SIM-001`

- Feedback
- Previous frame
- Simulation state
- Stateful processor
- Fixed-step evaluation

#### `OPEN-BACKEND-PROFILE-001`

- PDF profile
- SVG profile
- PPTX profile
- GPU profile
- Accessibility requirements

### 43. 最終状態

```text
OPEN-IR-001:
RESOLVED
```


本解決により、Reciplexaの描画系は次の性質を持つ。

- 多層かつ検証可能なIR
- SourceからArtifactまでの差分処理
- 不変値と構造共有
- 大規模文書・Slideへの部分Layout
- Frame・Tile単位の動的Rendering
- 階層的な2D Stacking
- 3D depthとの明確な分離
- Backend非依存のGeometryとCompositing
- 多言語対応のText基盤
- 正確な時間・Signal・Keyframe
- Nested CompositionのTime Transform
- 型付きVisual Processing DAG
- Code／Property／Stack／Graph／Timelineの複数view
- Backend Capabilityに基づくPlanning
- 明示的なFallback policy
- Portable／Native Backendの共存
- 高速な増分評価とCache


## OPEN-IR-001 Layered IR・増分コンパイル・描画意味論・Motion・Backend境界
### DD-IR-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Layered IR
- 差分コンパイルと増分評価
- Geometryと座標系
- 2D StackingとCompositing
- Paint、Color、Alpha、Blend
- 多言語Text基盤
- Time、Signal、Keyframe
- Nested CompositionとTime Transform
- Visual Processing DAG
- Backend Capability
- Fallback、Planning、Emission
- IR拡張・公開・Validation

### 0. 基本方針

Reciplexaの描画系は、RPXコードから直接Backend命令を生成する単一変換にはしない。

次のLayered IRを基本構造とする。

RPX Source
↓
Typed Core IR
↓
Package固有Domain IR
↓
Layout IR
↓
Visual IR
↓
Evaluated Visual IR
↓
Compositing IR
↓
Resolved Geometry／Render IR
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


各IRを設ける目的は、IRの数を減らすことではなく、各変換を次の性質へ限定することである。

- 単純
- 局所的
- 検証可能
- Cache可能
- 差分更新可能
- 並列実行可能
- Backendから分離可能


全IRを毎回完全にmaterializeすることは要求しない。必要に応じて、遅延view、fragment、Page、Frame、Tile、chunkとして生成できるものとする。

### 1. 性能を設計要件とする

性能は将来の実装最適化だけの問題ではなく、IR設計上の必須要件とする。

対象とする規模には、少なくとも次を含む。

- 数千Pageの長大文書
- 数百〜数千Slideの資料
- 数十万Text node
- 多数のFont、画像、動画Resource
- 多数Layerを持つComposition
- Frameごとに変化するAnimation
- Filter、Mask、Blendを含む映像処理
- 高解像度Previewと動画Export


各処理段階は、次を可能にしなければならない。

- 変更されていないsubtreeの再利用
- 部分的な再Layout
- Page／Slide／Frame／Tile単位の処理
- Resource decodeの遅延
- Node単位のCache
- Static subtreeのFrame間共有
- Task Runtimeによる並列化
- Stale結果のCommit拒否
- Memory budgetに基づくCache eviction

### 2. 差分コンパイルと増分処理

SourceからArtifactまでを、差分更新可能な依存pipelineとして設計する。

Source edit
↓
Incremental parse
↓
差分Macro展開
↓
差分名前解決・型検査
↓
Typed Core更新
↓
Domain IR更新
↓
Layout更新
↓
Visual／Compositing fragment更新
↓
Backend再計画
↓
Artifact更新


次の三処理を区別する。

差分コンパイル:
SourceからTyped Coreまで

増分評価:
Typed CoreからDomain／Visual IRまで

差分Rendering:
Render fragment、Tile、Backend出力の更新


各段階は概念的に次を追跡できなければならない。

- Stable identity
- Revision
- Content fingerprint
- Dependency set
- Change set
- Provenance
- Diagnostics


差分処理は常に使用する必要はない。小規模な入力では全体再構築を選択できる。

Incremental result
=
Full rebuild result


を必須条件とする。

### 3. IR追加の基準

IRの層数を最小化すること自体を目標にしない。

次のいずれかが成立する場合、独立IRを設ける価値がある。

- 前後で成立する不変条件が異なる
- 独立したValidatorを配置できる
- 独立したCache境界になる
- 情報を意図的に失う段階である
- 増分更新の粒度が異なる
- 異なるexecutorやResourceを必要とする
- 複数Backendから共有される
- 専用最適化passが必要である


単なるfield名変更だけの変換には、独立IRを設けない。

### 4. Domain IR

Domain IRは、文書・Slide・Chart・Diagram等の高水準な意味を保持する。

- Document
- Section
- Paragraph
- Slide
- Table
- Chart
- Figure
- Caption
- Semantic group
- Accessibility structure


Domain IRは一つの万能schemaにはしない。各packageが固有Domain IRを定義できる。

Document IR
Slide IR
Chart IR
Diagram IR
Math IR


各Domain IRは、最終的に共通Visual IRまたは対応するLayout IRへloweringできなければならない。

巨大なDomain構造を即座に完全展開せず、次を許可する。

- Lazy collection
- Virtualized sequence
- Repeated template
- Shared style
- Master／instance
- Query-backed data

### 5. Layout IR

Layout前後を独立した段階として扱う。

Layout前には次を保持できる。

- Flow
- Constraint
- Intrinsic size
- Relative length
- Style
- Inline object
- Page／Column条件


Layout後には次を確定する。

- 位置
- Size
- Line break
- Page break
- Baseline
- Column placement
- Continuation state


長文では、変更位置から再Layoutし、新旧のLayout stateが一致するstabilization point以降を再利用できるようにする。

### 6. Visual IR

Visual IRはBackend非依存の意味的な視覚構造である。

基本要素には次を含む。

- Group
- Shape
- Path
- Text
- Image
- Transform
- Clip
- Mask
- Paint
- Opacity
- Blend
- Filter／Visual Processor


Visual IRは不変かつ階層的とする。

親のTransform、Opacity、Style等を全子へ早期に焼き込まず、局所propertyとして保持する。

VisualNode {
  geometry,
  local-transform,
  paint,
  children,
  stacking,
  provenance
}


これにより、親propertyだけの変更で全subtreeを再構築することを避ける。

### 7. 座標空間

少なくとも次の座標空間を区別する。

Local space
Parent space
Composition space
Viewport space
Device space


共通2D座標系は次とする。

原点:
左上

+x:
右

+y:
下


Layout IR、Visual IR、Compositing IR、Render IRはこの規則へ統一する。

Backend固有座標系への変換はBackend loweringで行う。

内部IRでは空間identityを保持し、概念的に次を区別できるものとする。

Point<Local>
Point<Composition>
Transform<Local, Parent>

### 8. 長さ・角度・Transform

規範的な長さ単位はlogical pointとする。

1 inch = 72 logical points


相対単位はLayout段階で解決する。

- %
- em
- logical px


Visual IR以降では、原則として解決済みlogical pointを使用する。

角度は次の規則とする。

0度:
+x方向

正の角度:
時計回り


TransformはSurfaceに記述された順に適用する。

T1
→ T2
→ T3


Visual IRでは編集可能なTransform componentを保持し、Render側ではAffine matrixへ正規化できる。

position
anchor
scale
rotation
skew


2D Transformに前後関係を含めない。

2D Transform:
Geometry配置

Stacking:
2D前後関係

3D position.z:
3D幾何学的Depth

### 9. 2D Stacking

2Dの前後関係の正本は、Stacking context内の子sequenceとする。

先頭:
奥

末尾:
手前


コード記述順は、既定のStacking sequenceを生成する。

ただし、唯一の順序指定方法にはしない。

補助的に整数のstack-levelを使用できる。

小さいstack-level:
奥

大きいstack-level:
手前

同じstack-level:
sequence順


既定値は0とする。

stack-levelは幾何学的なz座標ではない。

Groupは必要に応じて独立Stacking contextを形成し、子のstack-levelは親contextを越えない。

### 10. ShapeとPath

Visual IRでは高水準Shapeを保持できる。

- Line
- Rectangle
- RoundedRectangle
- Circle
- Ellipse
- Polygon
- Polyline
- Arc
- Path


Resolved Geometry IRでは、共通Pathへloweringする。

Render Pathの基本命令は次とする。

MoveTo
LineTo
CubicTo
Close


Quadratic curveやArcは必要に応じてCubicへ変換する。

Pathは複数subpathを持てる。

Fill ruleはGeometryとは分離する。

NonZero
EvenOdd


Path storageはchunk化・構造共有・部分更新可能でなければならない。

### 11. Bounds

Boundsを一種類に統一しない。

GeometryBounds
PaintBounds
EffectBounds
VisibleBounds
ConservativeBounds


状態は次とする。

Bounds =
  Empty
  | Finite(Rect)
  | Unbounded
  | Unknown


ConservativeBoundsは、実際の影響範囲を必ず包含しなければならない。

Actual affected region
⊆
ConservativeBounds


過大評価は許可するが、過小評価は禁止する。

差分再描画の基本Dirty regionは次とする。

DirtyRegion =
  union(old-bounds, new-bounds)


Layer順変更、Blend、Filter、Isolationについては、影響するCompositing groupの安全な範囲へ拡張する。

### 12. Geometry ToleranceとHit testing

Geometry近似はQualityPolicyとdevice-space errorに基づく。

次を区別する。

Geometry approximation tolerance
Bounds safety margin
Hit-test tolerance
Tessellation tolerance


低品質Previewの近似結果をFinal出力へ使用してはならない。

Hit testingは次の段階で行う。

1. Spatial index
2. ConservativeBounds
3. Clip／Mask判定
4. Geometry判定
5. 手前から候補を選択
6. Provenanceから編集対象へ変換


描画順が奥から手前であるのに対し、Hit testingは手前から奥へ行う。

### 13. PaintとColor

GeometryとPaintを分離する。

v1の基本Paintは次とする。

- None
- SolidColor
- LinearGradient
- RadialGradient


基準色空間はsRGBとする。

合成計算の既定working spaceはLinear sRGBとする。

公開Color:
Straight alphaのsRGB

内部Compositing:
Premultiplied alphaのLinear sRGB


Alphaは0..1を基本範囲とする。

内部Filter計算では有限な範囲外値を許可できるが、出力変換時に明示的なmappingを行う。

### 14. Compositing

既定合成演算はsource-overとする。

次を区別する。

Paint alpha
Node opacity
Group opacity


Group opacityは、子へ個別適用するのではなく、Group内を合成した後の結果へ適用する。

必要な場合はIsolated groupを形成する。

標準Layer処理順は次とする。

1. GeometryをPaint
2. 子を合成
3. Filterを適用
4. Maskを適用
5. Layer opacityを適用
6. Blend modeで親へ合成


ClipはGeometry制約、MaskはCompositing操作として分離する。

### 15. Compositing IR

Compositing IRを独立層とする。

概念要素は次のとおりとする。

- PaintGeometry
- DrawText
- DrawImage
- Sequence
- Clip
- Mask
- Filter
- IsolatedGroup
- Composite


この段階で次を解決する。

- Back-to-front順
- Stacking context
- Isolation boundary
- Group opacity
- Blend mode
- Mask
- Filter順序
- 中間surfaceの必要性


意味的Groupを無条件に平坦化してはならない。

### 16. 多言語Text基盤

共通Text基盤は日本語専用にはしない。

次の仮定を禁止する。

- 一文字は一Glyph
- 一Unicode scalarは一表示文字
- Textは常に左から右
- 空白だけが単語境界
- Glyph順は論理Text順と同じ


共通処理は次の段階に分ける。

Unicode Text
↓
Unicode Analysis IR
↓
Shaping Plan IR
↓
Shaped Text IR
↓
Line Break Plan IR
↓
Line Layout IR
↓
Positioned Text IR


LanguageとScriptを別に保持する。

language
script
direction
writing-mode


Writing modeは少なくとも次を含む。

HorizontalTB
VerticalRL
VerticalLR

### 17. Text RunとShaping

次を区別する。

Source order
Logical order
Visual order
Glyph storage order


Grapheme clusterとShaping clusterを同一視しない。

Grapheme:
編集・Cursor操作上の単位

Shaping cluster:
Glyphと元Text rangeの対応単位


Shaping Run内では、少なくとも次を一定とする。

- Font
- Font size
- Script
- Language
- Direction
- Writing mode
- Font feature
- Variation axes


Shaping結果は元Unicode text、Text range、Cluster mappingを失ってはならない。

### 18. 言語package拡張

言語固有規則を共通Render IRへ埋め込まない。

共通Text Layoutは、次の拡張protocolを提供できる。

TextClassifier
LineBreakProvider
HyphenationProvider
JustificationProvider
AnnotationLayoutProvider
GlyphOrientationProvider
FontFallbackPolicy


日本語組版、ルビ、禁則、縦中横等は日本語向けpackageで実装する。

JLReqは、日本語組版packageの主要な規範資料として参照する。共通Text IRは、その実装に必要なUnicode text、縦横方向、注釈関係、Cluster、Line layout情報を失わない。JLReqは主としてJIS X 4051に基づく日本語組版要件を扱うため、低水準Glyph描画仕様ではなく、日本語向けText／Document Layout profileの参照資料として位置付ける。

### 19. Resource

Visual IRではNative handleを直接保持せず、安定Resource identityを参照する。

ResourceId
ContentHash
Revision
Metadata


対象には次を含む。

- Font
- Image
- Video
- Audio
- Color profile
- Pattern


ImageやVideoは必要時にdecodeし、Preview／Final品質を分離する。

Backend固有ResourceはBackend cacheで管理する。

共通IR:
Resource identity

Native Backend:
Decoded image、GPU texture、Font face等

### 20. Time

時間型は次のように分離する。

Time:
符号付きの正確な時刻

Duration:
非負の時間長

TimeOffset:
符号付きの時間差

FrameRate:
正の有理数

FrameIndex:
0-basedの非負整数


規範的な時間は有理時間とする。

実装はtime base付き整数tickを使用できる。

時間区間は半開区間とする。

[start, end)


無期限は数値InfinityではなくUnboundedで表す。

Frameの基準評価時刻はFrame開始時刻とする。

TimeからFrameへの変換では、丸めpolicyを明示する。

Floor
Ceil
Nearest
Exact

### 21. Signal

Signal<A>は解析可能なpureな時間依存値である。

一般のTime -> A関数と完全には同一視しない。

基本分類は次とする。

- Constant
- Keyframed
- Mapped
- Combined
- Sampled
- Selected


通常値AはConstant Signalとして自動的に使用できる。

Signal評価は次を満たさなければならない。

- Pure
- 決定的
- 同じSnapshotとTimeなら同じ結果
- 暗黙I/Oを行わない
- 暗黙の現在時刻を読まない


Randomや外部入力はseedまたはSnapshotとして明示する。

Signal依存関係はv1ではDAGとし、循環を拒否する。

### 22. Keyframe

Keyframe列は時刻順の非空sequenceとする。

Keyframe<A> {
  identity,
  time,
  value,
  outgoing-interpolation
}


同一Signal・同一時刻のKeyframeは高々一つとする。

Keyframeが一つだけの場合、意味上はConstantと等価である。

最初のKeyframeより前と最後のKeyframeより後は、端の値をHoldする。

隣接Keyframeの補間区間は次とする。

[t0, t1)


補間policyは前Keyframeが所有する。

標準補間は次とする。

Hold
Linear
CubicBezierEasing
Steps


時間Easingと空間上のMotion pathを分離する。

### 23. 型別補間

補間能力は型ごとのprotocolとして扱う。

補間可能な候補：

- Number
- Length
- Point
- Vector
- Opacity
- Color
- Angle
- Transform component


離散型はHoldまたはStepsを使用する。

- Boolean
- Text
- Font identity
- Resource identity
- Blend mode
- Stack level


Angleには次の補間policyを持たせる。

Shortest
Clockwise
CounterClockwise
Unwrapped


Colorの既定補間はLinear sRGBかつPremultiplied Alphaとする。

TransformはAffine matrix全体ではなく、position、anchor、scale、rotation等を個別に補間する。

Path morphは専用機能とする。

### 24. Time Transform

親Timelineと子Timelineを区別する。

Temporal Placementは次を持つ。

- Parent active range
- Child source range
- Time transform
- Range policy


Time Transformは任意関数ではなく解析可能な専用IRとする。

標準要素は次とする。

Identity
Offset
Scale
Reverse
Freeze
Compose


範囲policyは次を扱える。

Transparent
Hold
Loop
PingPong
Failure


LoopはEuclidean moduloを使用する。

空のsource rangeへのLoopはValidation errorとする。

可変速度は単純な速度Signalの乗算ではなく、Time Remapとして表す。

動画Frame選択と音声のpitch処理はTime Transformとは分離する。

### 25. Visual Processing DAG

Blur、色補正、Mask、Composite等の処理依存関係を、型付きVisual Processing DAGとして表す。

これは言語のEffect handlerとは別概念である。

公開上のVisual Processorは通常のpackage関数として見える。

(blur source radius)


Compilerは必要に応じてTyped Semantic DAGへloweringする。

DAGはv1ではacyclicとする。

Feedback、Simulation、Previous-frame参照は専用機構として将来扱う。

### 26. Processing Node契約

Nodeは概念的に次を持つ。

- Stable identity
- Typed input
- Typed output
- Parameters
- Bounds function
- Required input region
- Time behavior
- Tileability
- Cache contract
- Execution contract
- Fallback contract


PortとParameterを区別する。

Port:
他Nodeの出力

Parameter:
Static値またはSignal


Graph topologyと時間依存Parameterを分離する。

Parameterだけの変化で、DAG全体をFrameごとに再構築しない。

### 27. Processing DAG評価

出力からのdemand-driven evaluationを基本とする。

現在必要な出力へ到達しないNodeは評価しない。

Node処理は原則としてpureであるため、Dead node除去を安全に行える。

Boundsは入力から出力へ前方伝播する。

必要入力領域は、要求出力領域から入力側へ後方伝播する。

Tileabilityは次へ分類する。

Local
Overlapping
Global
NonTileable


Node cacheは次の単位を持てる。

- Node
- Time
- Time interval
- Spatial region
- Tile
- Quality
- Backend representation


差分無効化は概念的に次の軸で行う。

Node × Time × Spatial region

### 28. Semantic DAGとExecution Plan

利用者の意味構造と最適化後の実行構造を分ける。

Semantic DAG:
RPXコード、GUI、Provenance上の処理

Execution Plan:
Fusion、Task分割、Backend選択、Cache計画


隣接Nodeは、意味を保つ限り融合できる。

ColorMatrix A
→ ColorMatrix B


を一つの実行operationへ融合できる。

ただし、GUI上の個別Node、Failure origin、Provenanceを失ってはならない。

### 29. GUI表現

言語を正本とする方針を維持する。

同じ意味構造に対し、次のviewを提供できる。

Code:
正本、抽象化、Module、Test

Property inspector:
個別parameterの編集

Processing Stack:
直列処理の編集

Processing Graph View:
分岐、合流、共有、依存解析

Timeline／Curve:
時間依存propertyの編集


Graph上のNode位置は意味論に含めない。

Graph GUIは、Visual Processing DAGの一つの表示・編集方法であり、必須のSource形式ではない。

### 30. Backend境界

Render IRまでをBackend非依存の規範的描画意味とする。

対象Backendには次を含む。

- PDF
- SVG
- PPTX
- Raster
- GPU Preview
- Video


各BackendはCapabilityを構造化して宣言する。

- Geometry
- Paint
- Text
- Image
- Compositing
- Filter
- Animation
- Accessibility
- Interactivity
- Document structure


対応水準は次とする。

Native
EquivalentLowering
Approximate
RasterFallback
Unsupported


単純なBooleanだけでCapabilityを表さない。

### 31. Backend PlanningとEmission

Capability判定は描画hot loopではなくPlanning段階で完了させる。

Render IR
↓
Capability Snapshot
↓
Backend Planning IR
↓
Backend Emission IR
↓
Artifact


Backend Planning IRは、各subtreeの実現方法を確定する。

- Native vector
- Native text
- Geometry lowering
- Font embedding
- Raster fallback
- Resource embedding


Backend Emission IRは、形式固有のObject、命令、part、command等を表す。

### 32. Fallback Policy

次の基本policyを持つ。

Strict
Compatible
Editable
BestEffort

Strict

意味を十分保持できなければFailureとする。

Compatible

規定済みの等価loweringまたは局所Raster化を許可する。

Editable

見た目の完全一致より、出力先で編集可能な構造を優先する。

BestEffort

Preview等で警告付き近似を許可する。

Output policyは次の価値を別軸として扱える。

- Visual fidelity
- Editability
- Accessibility
- Searchability
- Interactivity
- Performance
- Artifact size

### 33. Raster Fallback

非対応機能は、可能な限り最小subtreeだけをRaster化する。

Page全体:
可能なら維持

非対応Filter group:
Raster island


Raster islandは次を持つ。

- Bounds
- Resolution
- Color space
- Alpha mode
- Quality policy
- Time identity


Backdrop依存のBlendやFilterでは、正しいCompositing結果を得るためRaster範囲を拡張する。

低解像度Preview用Raster cacheをFinal出力へ使用しない。

### 34. Text Fallback

Text Backend出力は次の段階を持つ。

1. Native Unicode text
2. Positioned glyph＋Font embedding
3. Glyph outline
4. Raster image


Output policyに基づいて選択する。

Accessibility重視:
Native text／Positioned glyph

Visual fidelity重視:
Positioned glyph／Outline

Editability重視:
Backend native text

Preview:
Rasterを許可可能


Font substitutionをFinal出力で黙って行わない。

### 35. Streamingと増分Backend

長大なArtifactでは、全Render IRを同時に保持しない。

Planning pass
↓
Page／Slide／Frame単位のEmission
↓
Finalize


必要に応じて次を全体Planningで収集する。

- Font subset
- Shared image
- Cross reference
- Page count
- Link destination
- Outline


差分更新では、変更Page、Slide、Frame、Resourceだけを再計画できるものとする。

最終ファイル形式が全bytesの再生成を要求する場合でも、Backend計算自体の差分再利用を可能にする。

### 36. IR公開と拡張

Domain IRはpackage固有で自由に定義できる。

共通Visual IRとRender IRは、安全性・Backend互換性・最適化のため、原則として閉じた規範constructor集合を持つ。

拡張が必要な場合は、version付きExtension nodeを登録できる。

Extensionには少なくとも次を要求する。

- Extension identity
- Version
- Type signature
- Validator
- Bounds function
- Required input region
- Tileability
- Time behavior
- Cacheability
- Backend requirement
- Portable fallbackまたはNativeRequired宣言


通常packageが未検証のRender IR constructorを直接作ることは許可しない。

Render IRはTyped BuilderまたはValidatorを介して構築する。

### 37. IR VersionとSerialization

各永続化可能IRにはversionを持たせる。

IRIdentity
IRVersion
SchemaVersion


Runtime内部cacheにはtoolchain-private表現を使用できる。

長期保存・process間転送にはversion付きportable codecを使用する。

内部memory dump:
禁止

Portable serialization:
Validatorを通して復元


古いIR versionは、明示Migration passによって新versionへ変換する。

Migration不能な場合は、元Sourceまたは上位IRから再構築する。

### 38. ValidationとFailure

各IR境界にValidatorを配置する。

Domain Validator
Layout Validator
Visual Validator
Compositing Validator
Render Validator
Backend Validator


通常Failureには次を含む。

- Resource resolution failure
- Font resolution failure
- Unsupported backend feature
- Memory budget exceeded
- Approximation tolerance exceeded
- Output profile violation
- Encoder failure


Defectには次を含む。

- Validated IR内のcycle
- Known Boundsの過小評価
- 解決済みResourceの欠落
- Validated Render IR内のNaN
- Capability PlanとBackend実装の矛盾
- Cache key不一致による誤再利用


RuntimeのMemory safetyや全体整合性が不明な場合はTerminal failureとする。

### 39. 適合試験

少なくとも次を検証する。

- Incremental compileとFull compileの同値性
- Visual subtreeの構造共有
- Transform適用順
- Stack-levelとsequence順
- Group opacityとIsolation
- Boundsの保守性
- Dirty region
- Tile overlap
- Path近似Tolerance
- 多言語ShapingとCluster対応
- BidiのLogical／Visual mapping
- Static SignalのFrame間共有
- Keyframe境界
- Time TransformのLoop／Reverse／Freeze
- Processing DAGのcycle拒否
- Node差分無効化
- Node cacheとFull evaluationの同値性
- Semantic DAGとExecution Planの同値性
- Backend Capability planning
- Fallback policy
- Raster islandの安全性
- Text fallbackの情報保持
- Page／Slide／Frame streaming
- Portable／Native Backendの適合性

### 40. 性能原則
PERF-IR-01:
変更されていないsubtreeの再構築を要求しない

PERF-IR-02:
各loweringをsubtree単位でcache可能にする

PERF-IR-03:
Page／Slide／Frame／Tile単位で要求可能にする

PERF-IR-04:
全IRの完全materializationを要求しない

PERF-IR-05:
Resourceをstable identityとrevisionで共有可能にする

PERF-IR-06:
時間非依存subtreeをFrame間で再利用可能にする

PERF-IR-07:
BoundsとDependency summaryを取得可能にする

PERF-IR-08:
Backend capabilityをhot loop前に解決する

PERF-IR-09:
Cache evictionによって規範結果を変えない

PERF-IR-10:
並列化によって規範結果を変えない

PERF-IR-11:
Stale Jobを現在revisionへCommitしない

PERF-IR-12:
Provenanceを共有・圧縮可能なidentityで保持する

### 41. 公開APIと実装

共通IRは通常の型付きpackageとして公開できる。

性能上重要な処理にはRust Native implementationを許可する。

- Text shaping
- Font処理
- Image decode
- Video decode
- Geometry処理
- Tessellation
- Rasterization
- PDF／PPTX Backend
- GPU Backend


Native実装はOPEN-KER-001のTrusted Adapter ABIに従う。

公開意味:
通常package

実装:
Portable RPX／Rust Native／GPU／Worker process


Native版とPortable版は、各packageが定める適合条件を満たさなければならない。

### 42. 関連する後続項目

#### `OPEN-TEXT-LAYOUT-001`

- 共通Text Layout protocolの正確な型
- Font fallback
- Shaping
- Bidi
- Line breaking
- Justification

#### `OPEN-TEXT-JA-001`

- JLReq参照範囲
- 日本語文字クラス
- 禁則
- 約物
- ルビ
- 縦中横
- 行調整

#### `OPEN-IR-3D-001`

- 3D Scene
- Camera
- Lighting
- Depth
- 2.5D Layer
- 3Dと2D Compositingの接続

#### `OPEN-IR-SIM-001`

- Feedback
- Previous frame
- Simulation state
- Stateful processor
- Fixed-step evaluation

#### `OPEN-BACKEND-PROFILE-001`

- PDF profile
- SVG profile
- PPTX profile
- GPU profile
- Accessibility requirements

### 43. 最終状態

```text
OPEN-IR-001:
RESOLVED
```


本解決により、Reciplexaの描画系は次の性質を持つ。

- 多層かつ検証可能なIR
- SourceからArtifactまでの差分処理
- 不変値と構造共有
- 大規模文書・Slideへの部分Layout
- Frame・Tile単位の動的Rendering
- 階層的な2D Stacking
- 3D depthとの明確な分離
- Backend非依存のGeometryとCompositing
- 多言語対応のText基盤
- 正確な時間・Signal・Keyframe
- Nested CompositionのTime Transform
- 型付きVisual Processing DAG
- Code／Property／Stack／Graph／Timelineの複数view
- Backend Capabilityに基づくPlanning
- 明示的なFallback policy
- Portable／Native Backendの共存
- 高速な増分評価とCache


## OPEN-TST-001 Test意味論・決定的Test Runtime・Property Test・適合試験
### DD-TST-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Test対象と成果物制作の分離
- Module-local Test
- External Contract Test
- Test-only declaration
- Test宣言と探索
- Test identity
- 依存置換
- Test double
- Test Runtime
- Test isolation
- 決定的Scheduler
- Virtual Clock
- Seeded Random
- TestOutcome
- Assertion
- Property-based testing
- Shrinking
- Differential testing
- Schedule exploration
- Public Test API
- Trusted Conformance API
- 増分BuildとTest選択

### 0. 基本原則

ReciplexaのTest機構は、すべてのRPXコードへTest作成を要求するものではない。

特に、文書、スライド、図、Animationなどの具体的成果物を記述する本文については、通常のUnit Testを要求しない。

成果物本文
↓
Compile
↓
Validate
↓
Preview
↓
Build／Export


具体的成果物を生成して確認すること自体を、その制作過程における主要な検証と位置付ける。

一方、次についてはTestを記述できるものとする。

- 再利用可能な関数
- Module
- Library package
- Parser
- Validator
- Layout algorithm
- Visual Processor
- Backend
- Native implementation
- Compiler
- Runtime
- Application entry
- Package間の統合


Testは明示的にTestとして宣言されたbindingだけを対象とする。

通常の関数、成果物binding、unitを返す関数、特定の名前を持つ関数が、自動的にTestとして認識されることはない。

### 1. Testと他の検証機構の分離

次を明確に区別する。

User-authored Test:
利用者が期待条件を明示して検査する

Validation:
Compiler／Runtimeが不正状態を自動的に拒否する

Artifact Check:
生成成果物に利用者固有の要件を課す

Lint:
不正ではないが望ましくない状態を報告する

Preview:
人間が成果物の見た目や内容を確認する

Export Verification:
生成Artifactの形式的整合性を検査する


成果物本文へUser-authored Testを書かない場合でも、型検査、IR Validation、Resource検査、Backend Validation等は行う。

Testを書かない
≠
Validationを行わない

### 2. Testの対象と所属

Testの対象とTestの所属を分離する。

Test対象
- Binding
- Module
- Package内の複数Module
- 複数Package
- Application
- Compiler
- Runtime
- Native implementation
- Backend

Test所属
- Module-local
- Package external
- Workspace integration
- Documentation
- Toolchain conformance


PackageはTestのBuild、依存管理、探索、配布における上位単位であるが、Test対象をPackage全体に限定しない。

### 3. Test分類

Testの主要分類を次とする。

TestKind =
  ModuleLocal
  | ExternalContract
  | PackageIntegration
  | WorkspaceIntegration
  | Application
  | Documentation
  | Conformance


この分類はTestの規模だけでなく、次を決定する。

- 可視性
- Compile environment
- Test-only declarationへのアクセス
- Runtime profile
- Isolation class
- Test doubleの注入権限
- Build artifact

### 4. Module-local Test

Module-local Testは、対象Moduleの内部実装を検査するWhite-box Testである。

対象:
- private helper
- private type
- 内部constructor
- Module内部の不変条件
- 公開前の中間表現


Module-local Testは、論理的に対象Moduleの抽象化境界内部へ所属する。

Module M
├─ Public binding A
├─ Private binding B
├─ Private type T
└─ Module-local Tests


Module-local Testからは、対象Moduleのpublicおよびprivate memberへアクセスできる。

公開interfaceだけを対象とするBlack-box Testと、内部実装を対象とするWhite-box Testの双方には異なる役割があり、Moduleの抽象化境界とTestの境界が常に一致するとは限らない。

### 5. Private値のescape禁止

Module-local Testがprivate memberへアクセスしても、privateな型、値、constructor authorityをTest scope外へ公開してはならない。

許可:
- private値の構築
- private関数の呼出し
- 内部状態の比較
- 内部不変条件のAssertion

禁止:
- private値をTest runnerへ返す
- private型を別Moduleへ公開する
- private constructorを他Testへ移送する
- Module sealingを迂回する


Test runnerへ返るのは、構造化されたTestOutcome、Diagnostic、Attachment identity等に限定する。

### 6. External Contract Test

External Contract Testは、対象ModuleまたはPackageを外部利用者と同じ可視性で検査するBlack-box Testである。

見える:
- Public module
- Public binding
- Public type
- Public constructor
- Public Effect
- Public Failure

見えない:
- Private binding
- Private type representation
- Test-only helper
- Native representation
- Runtime-private identity


同じRepositoryまたはPackage内に物理的に配置されていても、CompilerはExternal Contract Testを論理的に外部Moduleとして型検査する。

Rustにおいても、Source近傍のUnit Testはprivate itemへアクセスできる一方、Integration Testは外部crateと同様に公開APIを検査する。この区別は、Module-local TestとExternal Contract Testの設計上の参考とする。

### 7. Integration Test
Package Integration Test

一つのPackage内の複数Moduleを組み合わせて検査する。

Parser
+
Validator
+
Layout
+
Renderer


Package内部interfaceへのアクセスが必要なTestと、外部公開interfaceだけを使うTestは、可視性metadataによって区別する。

Workspace Integration Test

複数Packageの接続を検査する。

text-layout
+
font
+
language-specific-layout
+
PDF backend


Workspace Integration Testは、通常、Test専用PackageまたはTest targetへ所属し、対象Packageのprivate memberへはアクセスしない。

### 8. Application Test

Application Testは、Application entry pointと最上位Runtime環境を検査する。

- CLI argument
- Configuration
- Document初期化
- GUI起動
- Root Task scope
- Startup Failure
- Graceful shutdown
- Exit status


Application Testの詳細なentry契約はOPEN-PKG-ENTRY-001で定める。

### 9. Documentation Test

Documentation Testは、文書内で明示されたCode例を検査する。

DocumentationTestKind =
  Compile
  | Run
  | ExpectFailure
  | Render
  | Ignore


通常の成果物本文をDocumentation Testとして自動実行しない。

Documentation内でTest対象として明示された例だけをTestDescriptorへ変換する。

RustのDocumentation Testは、文書中のCode例を抽出し、compileまたは実行する仕組みを持つ。Reciplexaでも、公開APIの使用例を検証する仕組みとして参考にする。

### 10. Conformance Test

Conformance Testは、言語・Runtime・標準package・Native implementation・Backend等の規範契約を検査する。

- Incremental compileとFull compileの同値性
- Portable実装とNative実装の同値性
- Task Cancellation
- Resource ownership
- Native Adapter ABI
- Backend Capability
- Raster fallback
- IR Validator


Conformance Testは、通常Testより強い権限を必要とする場合があるため、Trusted Conformance層へ所属させる。

### 11. Artifact Check

Artifact Checkは通常Testとは別分類とする。

ArtifactCheckKind =
  Structural
  | Layout
  | Resource
  | Accessibility
  | BackendCompatibility
  | VisualRegression
  | Custom


例：

- Page数が規定以内である
- 内容がPageからはみ出していない
- Link切れがない
- 画像解像度が基準以上である
- 指定Fontだけを使用している
- Accessibility情報が存在する
- Raster fallbackが発生していない


Artifact Checkは任意であり、通常の文書・Slide・Animation本文へ記述を要求しない。

実行基盤やReportはTest機構と共有できるが、意味上はUnit／Contract Testと区別する。

### 12. Test-only declaration

Module-local Testでは、次のTest専用宣言を定義できる。

- Fixture
- Test data
- Fake
- Comparison helper
- Generator
- Internal constructor helper


Test-only declarationは通常Buildから除外する。

通常Build:
Production declarationのみ

Test Build:
Production declaration
+
必要なTest-only declaration


Test-only declarationは公開.rpiへ含めない。

Test-only codeの追加または変更によって、本体bindingのidentityや公開API hashを変化させない。

### 13. Test Buildによる本体置換の禁止

Test BuildはProduction declarationへ宣言を追加できるが、本番bindingを暗黙に置換してはならない。

許可:
Test helperを追加する

禁止:
本番関数を同名Test関数へ置換する
Test時だけ本番関数の意味を変更する
Test時だけ別のprivate実装へ差し替える


依存実装を差し替える場合は、Effect handler、Module substitutionまたはImplementation selectionを使用する。

### 14. Test宣言

Testは意味上、通常の型付きbindingとTest metadataから構成する。

Typed Test Binding
+
TestDescriptor


Surfaceには簡潔なtest宣言を構文糖衣として提供できる。

(test addition-works
  (assert-equal
    (+ 1 2)
    3))


Typed Coreでは、通常bindingとTestDescriptorへloweringする。

Rustも、通常関数へ#[test]属性を付けてTest harnessへ登録する方式を採る。この「Test bodyは通常言語で記述し、Test性をmetadataとして付加する」点を参考にする。

Testの最終的なSurface構文は別項目で確定する。

### 15. Test entryの型

Test entryの意味上の型は、概念的に次とする。

TestContext -> unit
effects {
  test,
  required-test-effects...
}


Surfaceでは通常、TestContext引数を省略できる。

CompilerはTest bindingの型とRequired Effectを検査し、TestDescriptorへ記録する。

例：

Pure Test:
effects {test}

Task Test:
effects {test, task}

Clock Test:
effects {test, task, clock}

Resource Test:
effects {test, resource}


RunnerはRequired Effectから必要なTest Runtime profileを構築する。

### 16. test Effect

Assertion failure、Test skip、Observation、Test section等は、通常Programのfailure Eとは分離する。

概念的なtest Effect operationは次を含み得る。

- assertion-failed
- record-observation
- attach-diagnostic
- attach-artifact
- skip-test
- enter-section
- leave-section


Assertion APIは通常packageで定義し、不一致をtest Effectを通じてTest Runtimeへ報告する。

Program Failure:
対象Programの公開意味

Test Failure:
Test期待条件の不一致

Defect:
契約違反

Terminal failure:
回復不能


これらを混同しない。

### 17. Test identity

Test identityは、表示名だけではなく、次から構成する。

TestIdentity {
  PackageInstanceId,
  ModuleId,
  BindingId,
  optional CaseId
}


表示名は別に保持する。

Stable identity:
機械的識別

Display name:
人間向け名称

Qualified display name:
Package／Moduleを含む表示


Parameterized TestやProperty Testの個別caseは、TestIdentityとCaseIdで識別する。

### 18. TestDescriptor

CompilerはTestごとに構造化されたTestDescriptorを生成する。

TestDescriptor {
  test-identity,
  display-name,
  test-kind,
  visibility-mode,
  binding-type,
  required-effects,
  required-capabilities,
  isolation-class,
  resource-requirements,
  tags,
  source-origin,
  dependency-summary,
  timeout-policy,
  execution-budget,
  ignored-state,
  parameterization,
  implementation-policy
}


Descriptorは静的に読み取れるmetadataとし、一覧表示やTest選択のために任意のTest codeを実行する必要がないようにする。

### 19. Test探索

Test runnerは、Compilerが生成したTestDescriptor indexからTestを探索する。

Source
↓
Incremental compile
↓
Module TestDescriptor fragment
↓
Package Test index
↓
Workspace Test index
↓
Runner selection


Runnerが毎回Workspace全体のSourceをparseしてTestを探す方式にはしない。

Test探索を次へ依存させない。

- Binding名の接頭辞
- 戻り型
- Directory名だけ
- File名だけ


明示Test metadataを正本とする。

### 20. Test選択

TestDescriptorに基づき、次の条件でTestを選択できる。

- Workspace
- Package
- Module
- Test identity
- Display name
- Test kind
- Tag
- Isolation class
- Required capability
- Implementation policy
- Source変更との関係


具体的なCLI Filter構文はTooling仕様へ分離する。

### 21. Disabled・Ignored・Skipped

次を区別する。

Disabled:
Build／Discovery段階で除外

Ignored:
Testとして存在するが通常実行では選択しない

Skipped:
選択後に実行条件を満たさず実行しない


SkippedはPassedではない。

CI profileは、Skipを許可するか、Test suite failureとして扱うかを指定できる。

### 22. Parameterized Test

同一Test logicを複数の明示caseへ適用できる。

ParameterizedTest<Input> {
  cases,
  body
}


各caseは独立CaseIdを持つ。

TestIdentity
+
CaseId


Caseの表示値そのものをIdentityへ使用しない。

大きなDocument、画像、非hashable値等をcaseとして扱えるよう、CaseIdと表示内容を分離する。

### 23. 依存置換の基本方針

任意のbindingをTest実行中に名前でpatchする方式は原則として採用しない。

理由は次のとおりである。

- InliningやFusionと矛盾する
- Binding identityとCacheが不安定になる
- Native実装との対応が壊れる
- Test時だけProgram意味が変わる
- private実装への過度な結合を促す


差し替え可能なのは、依存境界として明示されたものに限定する。

- Effect service
- Module parameter
- Capability
- Backend implementation
- Resource provider

### 24. Effect Test handler

外部作用・Runtime serviceはEffect Test handlerで置換する。

本番:
clock Effect
→ System Clock handler

Test:
clock Effect
→ Virtual Clock handler

本番:
resource Effect
→ File Resource handler

Test:
resource Effect
→ In-memory Resource handler


対象ProgramのSourceとRequired Effectは変更しない。

HandlerだけをTest Runtime profileで差し替える。

### 25. Module substitution

純粋な抽象処理や業務的依存は、型付きModule substitutionで置換する。

本番:
DatabaseRepository

Test:
InMemoryRepository


Test implementationは本番依存と同じModule signatureを満たさなければならない。

RustにおけるtraitとDependency Injectionの利用は、抽象依存へ別実装を供給する方式として参考になる。

小さな依存は関数引数や明示recordで渡し、環境全体の依存はModule compositionまたはRuntime profileで選択できる。

### 26. Implementation selection

同じ公開意味に対するPortable／Native／CPU／GPU等の実装差は、MockではなくImplementation selectionとして扱う。

ImplementationPolicy =
  PortableOnly
  | NativeOnly(ImplementationId)
  | PreferNative
  | Differential


External Contract TestまたはConformance Testを、複数実装へ適用できる。

### 27. Test double

Test doubleを次へ分類する。

Stub:
決められた値を返す

Fake:
簡易だが動作する代替実装

Spy:
呼出しや結果を記録する

Mock:
期待操作を事前宣言して検査する

FaultInjector:
Failure、遅延、Cancellation等を注入する

Simulator:
状態遷移や外部環境を模擬する


既定の推奨順は次とする。

1. Fakeと最終状態の検査
2. Spyによる必要な観測
3. Protocol上必要なInteraction Mock


Strict Mockを通常の業務Testの既定にはしない。

RustのMockallは、trait等からMock型を生成し、引数matcher、呼出し回数、順序、戻り値を設定する方式を提供する。Reciplexaでは同種の機能を型付きTest doubleとして提供できるが、任意binding patchには依存しない。

### 28. Interaction expectation

Interaction Testでは、次を検査できる。

- Call count
- Argument constraint
- Ordering
- Partial ordering
- Lifetime
- No-more-calls
- Eventually


並行処理では、全操作の完全順序より部分順序を優先する。

Acquire before Use
Release after all Use
Commit after A and B


意味のないTask間順序を固定しない。

### 29. Interaction履歴

呼出し履歴は構造化eventとして記録する。

InteractionEvent {
  sequence-id,
  logical-time,
  scheduler-step,
  task-id,
  service-id,
  operation-id,
  arguments-summary,
  outcome,
  resource-identity,
  source-origin
}


巨大値や機密値を無制限に保存しない。

Operationごとに記録policyを指定できる。

- Full value
- Structural summary
- Stable identity
- Hash
- Redacted
- Not recorded

### 30. Test isolation

Test isolationを次の階層に分ける。

IsolationClass =
  Logical
  | Runtime
  | Process
  | Sandbox

Logical

軽量なPure Test向け。

Runtime

独立Scheduler、Clock、Resource table等を必要とするTest向け。

Process

Process-global state、Main-thread API、GPU context、Native panic等を隔離する。

Sandbox

信頼度の低いNative code、Codec、Security Test等へ使用する。

cargo-nextestがTestごとにprocessを分ける理由には、process-global state、Main-thread限定API、GPU context、環境変数、強制終了の隔離が含まれる。Reciplexaではこれを参考にしつつ、すべてのPure Testを常に別processにせず、必要隔離を階層化する。

RunnerはTestが宣言した最低Isolationより強いIsolationを選べるが、弱いIsolationへ下げてはならない。

### 31. Test root Task scope

各Testへ独立したroot Task scopeを作成する。

Test
└─ Root Task scope
   ├─ Child Task A
   └─ Child Task B


Test body終了時に未完了Taskが残っている場合、既定ではTest failureとする。

Runtimeは次を行う。

1. Task leakを記録
2. Root Task scopeへCancellation
3. 子孫Taskへ伝播
4. Cleanup完了を待つ
5. 終了しない場合はIsolation boundaryを停止


構造化ConcurrencyのCancellation自体を検査するTestでは、明示的な内側scopeを観測対象にできる。

### 32. 決定的Scheduler

Test Runtimeの既定Schedulerは、同じ入力、seed、Runtime Snapshotから同じTask選択順を生成する。

基本方針は次とする。

- Runnable queueは安定順序
- spawnは決定的にqueueへ追加
- yieldは規定位置へ戻す
- completionは安定したEvent順で処理
- HashMap列挙順やOS Thread順へ依存しない


Test Schedulerは次のModeを持つ。

SchedulerMode =
  DeterministicDefault
  | Replay
  | Randomized
  | Explore

### 33. Scheduler trace

重要なTask eventを構造化traceとして記録する。

SchedulerEvent {
  event-id,
  logical-step,
  task-id,
  event-kind,
  related-resource-id,
  virtual-time,
  source-origin
}


Traceは次を含み得る。

- Task spawn
- Task run
- yield
- await
- completion
- Cancellation request
- Cleanup
- Timer event
- Native completion


Failure時にTraceを保存し、Replay modeで再現できるようにする。

### 34. Virtual Clock

Test RuntimeはVirtual Clockを提供する。

ClockMode =
  Manual
  | AutoAdvance

Manual

Test bodyが明示的にClockを進める。

AutoAdvance

Runnable Taskが存在せず、Timerだけが残る場合に、次のTimer時刻へ自動的に進める。

Virtual Clockは、言語の正確なTime、Duration、TimeRangeを使用する。

### 35. Timeout

次の三種類を区別する。

VirtualTimeout:
Program意味上のTimeout

ExecutionBudgetExceeded:
Task stepや探索数の上限超過

WallClockTimeout:
Host watchdogによる停止


Wall-clock watchdogはRPX上のClockとは独立し、Native hangやRuntime deadlockへ対する最終防御とする。

### 36. Seeded Random

Random入力は次から決定的に導出する。

RunSeed
+
TestIdentity
+
CaseIdentity
+
StreamIdentity
→ Random stream


Testの物理実行順やworker数でRandom入力が変化しないようにする。

並行TaskではTask identityごとにStreamを分割する。

Input seed
Scheduler seed
Fault seed


を独立して管理する。

### 37. ResourceとNative completion

Test用Resource serviceはIn-memory providerまたはScripted providerとして実装できる。

- Read
- Write
- Create
- Delete
- Metadata
- Failure injection
- Delayed completion
- Cancellation


Native callbackやAsync completionは、Taskを直接再開せずTest Scheduler queueへEventとして登録する。

これにより、次を再現できる。

- CompletionがCancellationより先
- CancellationがCompletionより先
- Cancellation後のLate result
- Callback解除中の発火

### 38. Test終了とLeak検査

TestOutcomeはTest bodyの終了だけで確定しない。

終了処理は次の順序とする。

1. Test bodyの暫定Outcomeを記録
2. Root Task scopeを閉じる
3. 未完了Taskを検査
4. 必要ならCancellation
5. Callback受付を停止
6. Async operationを解決・破棄
7. Resource cleanup
8. Test double expectationを検査
9. Leak検査
10. Primary／suppressed failureを整理
11. 最終TestOutcomeを確定


Leak分類には次を含む。

- TaskLeak
- ResourceLeak
- CallbackLeak
- SharedTokenLeak
- CompletionTokenLeak
- BuilderSessionLeak
- NativeOperationLeak
- PendingExpectation

### 39. TestOutcome

Test全体の結果を次へ分類する。

TestOutcome =
  Passed
  | Failed(TestFailureReport)
  | Skipped(SkipReason)
  | TimedOut(TimeoutReport)
  | Defected(DefectReport)
  | Aborted(InfrastructureAbort)

Passed

Assertion、Cleanup、Leak検査をすべて通過した。

Failed

Testは実行できたが期待条件を満たさなかった。

Skipped

必要条件がなく実行されなかった。

TimedOut

Virtual timeout、Execution budgetまたはWall-clock watchdogに達した。

Defected

契約違反やRuntime Defectが発生した。

Aborted

Test実行基盤が結果を信頼可能な形で確定できなかった。

### 40. 対象処理のOutcome

対象Programの結果はTestOutcomeと分離する。

SubjectOutcome<A, E> =
  Success(A)
  | Failure(E)
  | Cancelled
  | Defect(DefectReport)


例として、期待された型付きFailureが発生した場合は次となる。

SubjectOutcome:
Failure(expected-error)

TestOutcome:
Passed


Assertion failureを対象Programのfailure Eへ混ぜない。

### 41. TestFailure

TestFailureの基本分類を次とする。

TestFailureKind =
  ValueMismatch
  | PredicateNotSatisfied
  | UnexpectedSuccess
  | UnexpectedFailure
  | FailureMismatch
  | UnexpectedCancellation
  | MissingCancellation
  | InteractionMismatch
  | DiagnosticMismatch
  | SnapshotMismatch
  | VisualMismatch
  | TaskLeak
  | ResourceLeak
  | TimeoutExpectationMismatch
  | ConformanceMismatch

### 42. Assertion

Assertionは通常packageで定義し、不一致を構造化Mismatchとしてtest Effectへ渡す。

基本Assertionには次を含む。

- 真偽条件
- Equality
- Inequality
- Ordering
- Pattern
- Collection
- Approximate equality
- Expected Failure
- Expected Cancellation
- Expected Diagnostic
- Interaction expectation


Assertionの失敗を通常FailureやDefectとして扱わない。

### 43. EqualityとDiff

型ごとのEqualityとDiagnostic用Diffを区別する。

Equatable<A> {
  equal:
    A -> A -> bool
}

Diffable<A> {
  diff:
    A -> A -> Diff
}


すべての型に万能なpointer identity比較を提供しない。

次の型には専用比較を要求する。

- Function
- Task handle
- Resource handle
- Signal
- Native object
- GPU Resource
- Document
- 浮動小数点


Diff計算にはBudgetを持たせる。

DiffBudget {
  max-depth,
  max-items,
  max-bytes,
  max-computation
}

### 44. 近似比較

浮動小数点、Geometry、Color、Pixel等には専用の近似比較を使用する。

Tolerance =
  Absolute
  | Relative
  | Combined
  | ULP
  | DomainSpecific


単一の既定Toleranceをすべての用途へ適用しない。

Geometry:
logical pointまたはdevice-space error

Color:
component差または知覚色差

Pixel:
位置差・色差・許容pixel数

### 45. Expected Failure・Cancellation・Defect
Expected Failure

型付きFailureの型、constructor、payload patternを検査する。

Expected Cancellation

Cancellation要求だけでなく、TaskがCancelledとなりCleanupを完了したことを検査する。

Expected Defect

主としてTrusted Conformance Testへ限定する。

Expected Terminal failure

ProcessまたはSandbox isolationで実行し、親Runnerが終了種別、Diagnostic、Crash category等を検査する。

### 46. Diagnostic比較

Diagnosticの表示文字列全体ではなく、構造化情報を比較する。

Diagnostic {
  code,
  severity,
  primary-origin,
  related-origins,
  category,
  arguments,
  notes,
  rendered-message
}


通常のDiagnostic Testでは次を比較する。

- Diagnostic code
- Severity
- Source origin
- 重要argument
- Related diagnostic


表示文全体の完全一致はDiagnostic renderer自身のTestへ限定する。

### 47. Fail-fastと収集

Assertion制御を次へ分ける。

require:
失敗時に現在のTest sectionを中断

check:
失敗を記録して継続


前提条件にはrequireを使い、互いに独立した複数fieldの検査等にはcheckを使用できる。

複数Assertion failureは、集合として一つのTest failure reportへまとめる。

### 48. Primaryとsuppressed failure

Test bodyのFailureとCleanup Failureが同時に発生した場合、次の規則を適用する。

Test bodyが既にFailure:
Test body FailureをPrimary

Cleanup／Leak:
suppressed


Test bodyが成功していた場合、CleanupまたはLeak FailureをPrimaryとする。

Runtimeの既存Primary／suppressed規則と整合させる。

### 49. Snapshot Test

Snapshot Testは補助機能として提供する。

適切な対象：

- Diagnostic tree
- Normalized IR
- Stable JSON／XML
- Generated SVG
- Layout summary


不適切になりやすい対象：

- 生pointerを含むDebug出力
- 非決定的Map順序
- Timestamp入りArtifact
- 巨大Document全体
- 環境依存Font出力


Snapshot mismatch時に期待値を自動上書きしない。

Mismatch
↓
Actualを別Artifactとして保存
↓
差分確認
↓
明示的accept


CIでは自動acceptを禁止する。

### 50. Visual Diff

Visual Diffは固定したBackend、Color profile、Quality policy、Font environmentで実行する。

比較方式は次を含む。

- ExactPixel
- PerChannelTolerance
- PerceptualDifference
- RegionMask
- StructuralVisualCheck


Visual snapshotを使用する前に、Bounds、Text内容、Layout tree等の構造的Assertionを優先する。

### 51. Property-based testing

Property Testは通常Testを多数caseについて実行する上位形式とする。

Property<A> {
  generator,
  body,
  shrink-policy,
  case-budget,
  discard-budget,
  seed-policy
}


Property-based testingは、入力範囲について性質を記述し、Frameworkがedge caseを含む入力を生成する。通常の手書きTestを置き換えるものではなく、補完するものとして扱う。

### 52. Generator

Generator<A>は、Choice streamを消費して有効なAを決定的に構築する。

同じGenerator identity
同じGenerator version
同じChoice trace
同じConfiguration
→ 同じ値


Generatorを単純な非決定的Random関数と同一視しない。

Generatorは型単位ではなく値領域単位を基本とする。

int generator
positive-length generator
valid-port generator
color-component generator


型ごとのCanonical Generatorは便利な既定として自動導出できるが、明示Generatorを常に許可する。

### 53. Discard

Propertyの前提を満たさないcaseはDiscardとする。

Discard:
検査対象外

Failure:
有効caseがPropertyを破った


Discard数にはBudgetを設ける。

有効caseを十分に生成できない場合は、Property反例ではなくGeneration configuration failureとして報告する。

### 54. Shrinking

Failureを維持したまま入力を簡略化する処理をShrinkingとする。

ShrinkingはGeneratorと整合した構造を基本とする。

Proptestは、生成とShrinkingを型単位ではなく値生成Strategy単位で定義し、制約を維持しながら反例を簡略化する。

Shrinkingは、定義された簡略化順序とBudget内で、より単純な反例を探索する。

数学的な大域最小反例の発見は保証しない。

### 55. Choice traceとFailure persistence

Property Test失敗時には、Seedだけでなく次を保存する。

- Test identity
- Property version
- Generator identity
- Generator version
- Random algorithm version
- Run seed
- Case seed
- Choice trace
- Serialized counterexample
- Failure fingerprint
- Schedule trace
- Fault script


次回実行時には、保存済みRegression caseを新規Random caseより先に実行する。

Proptestも、発見済み失敗caseのSeedを永続化し、後続実行で再生するFailure persistenceを備える。

### 56. Failure fingerprint

Shrinking中に別のFailureへ変化することを防ぐため、Failure identityを構造化する。

FailureFingerprint {
  outcome-kind,
  failure-code,
  assertion-id,
  source-origin-class,
  defect-category
}


既定では、同じFailure fingerprintを維持するShrink候補だけを採用する。

### 57. Stateful Property Test

状態を持つ対象については、操作列とReference Modelを使う。

Reference Model
↓ operation sequence
Expected state

System Under Test
↓ same operations
Actual state


対象例：

- Resource lifetime
- Document editor
- Undo／Redo
- Task scope
- Callback registration
- Incremental compiler session


有効操作列と不正操作Testを分ける。

### 58. Differential testing

同一入力を複数実装または処理経路へ与え、結果を比較する。

Input
├─ Implementation A
└─ Implementation B


対象例：

- Portable vs Native
- Interpreter vs Compiled
- Unoptimized vs Optimized
- Full vs Incremental compile
- Full-frame vs Tile render
- CPU vs GPU
- Backend version間


Differential testingは、異なる実装へ同一入力を与え、挙動の差からSemantic bug候補を検出する手法である。

不一致だけから、どちらの実装が誤っているかを自動断定しない。

### 59. Differential equivalence

比較対象ごとに同値関係を明示する。

Equivalence =
  Exact
  | Structural
  | Normalized
  | NumericTolerance
  | PixelTolerance
  | Semantic
  | Observational


未規定・実装依存の値を比較対象へ含めない。

Raw result
↓
Observation projection
↓
Normalization
↓
Equivalence comparison

### 60. Metamorphic testing

独立実装がない場合は、入力変換前後の意味保存関係を検査できる。

deserialize(serialize(x))
=
x

optimize(optimize(ir))
=
optimize(ir)

translate(path, 0, 0)
≡
path


Metamorphic TestはProperty Testの一形式とする。

### 61. Schedule exploration

並行Taskの合法な実行順を複数探索する。

対象となるBranch pointには次を含む。

- Runnable Task選択
- yield
- await completion
- Timer順序
- Cancellation受理
- Native completion
- Callback delivery


Loomは並行実行の有効な順序を決定的に探索し、状態削減によって組合せ爆発を抑えるConcurrency Test toolである。Reciplexaでは、まずTask Runtime上の論理schedule探索の参考とする。

### 62. 探索の完全性

Schedule探索結果を次へ分類する。

ExplorationStatus =
  ExhaustiveWithinModel
  | BoundedComplete
  | BudgetExhausted
  | CounterexampleFound


Budget内でFailureが見つからなかったことを、無条件な正しさの証明とはみなさない。

探索には次の上限を持つ。

- 最大schedule数
- 最大Task切替数
- 最大preemption数
- 最大Event数
- 最大探索深度

### 63. 多次元反例

並行Property Testの反例は次の複合構造を持ち得る。

Counterexample {
  input,
  schedule,
  fault-script,
  time-choices,
  implementation-selection
}


入力、Schedule、Faultを最初から完全直積で探索しない。

推奨段階は次とする。

1. 多数入力を既定Scheduleで検査
2. 一部入力をRandomized Scheduleで検査
3. 重要入力をBounded exploration
4. 発見反例を入力Shrinking
5. Schedule Shrinking
6. Fault Script Shrinking

### 64. Flaky Test

同じ反例、Schedule、Snapshotで結果が変わる場合は、Flakyまたは環境依存Testとして分類する。

- StableFailure
- IntermittentFailure
- TraceIncompatible
- EnvironmentDependent


Failure発見後に同一条件でReplayし、Failure fingerprintが安定していることを確認する。

### 65. 性能Property

性能Testでは、Wall-clock時間だけでなく構造的costを優先する。

- Recompiled binding数
- Evaluated Node数
- Allocation量
- Shapingされたrun数
- Render tile数
- Cache hit率
- Scheduler step数


例：

一文字変更で、
全文書の全Paragraphを再Shapingしない


性能Propertyは機器差の影響を受けにくい規範的な上限または漸近的性質を優先する。

### 66. Public Test API

通常packageが利用できるPublic Test APIには、次を含む。

- Assertion
- Equality／Diff
- Approximate comparison
- Expected Failure
- Expected Cancellation
- Fake／Stub／Spy／Mock
- Generator
- Shrinker
- Property Test
- Artifact Check
- Virtual Clock操作
- Test handler構成


Public Test APIからRuntime memory safetyやModule sealingを破ることはできない。

### 67. Trusted Conformance API

次はTrusted Conformance APIへ限定する。

- Raw IRの不正構築
- Invalid handle
- 二重completion
- Rust panic／abort注入
- ABI mismatch
- Perceus refcount観測
- Scheduler内部状態
- Cleanup stack
- Terminal failure trigger


Trusted APIは、Toolchain、標準Runtime、正式に認可されたNative conformance suiteだけが利用できる。

通常の依存宣言だけで権限を取得できるものにはしない。

### 68. 実装責任境界
通常Test package
- Assertion combinator
- Equality
- Diff
- Generator
- Shrinker
- Fake
- Spy
- Mock
- Domain固有Check

Compiler
- Test metadata
- Module-local private access
- External Contract Test環境
- Test-only declaration
- TestDescriptor
- 増分Test index
- Compile依存解析

Test Runtime
- Root Task scope
- 決定的Scheduler
- Virtual Clock
- Seeded Random
- Handler installation
- Leak検査
- Defect isolation

Runner／Host
- Test選択
- Process／Sandbox isolation
- Wall-clock watchdog
- Regression corpus
- Attachment
- Report
- CI分割

Trusted Conformance層
- Raw representation
- 内部観測
- 低水準Fault injection

### 69. 増分BuildとTest再実行

Testの再Compileと再実行を分ける。

Compile dependency:
Test bodyの型・名前解決が変わるか

Execution dependency:
対象実装の挙動が変わるか

Test bodyだけ変更
再Compile:
該当Test

通常Artifact:
維持

Private実装変更
Module-local Test:
再Compile／再実行候補

External Contract Test:
本体の再Compileは通常不要
再実行は必要

Public interface変更
External Contract Test:
再型検査・再実行

依存Package:
再型検査候補


Changed-only Test selectionは開発時の高速化として提供する。

CI等では全Testを実行する経路を維持する。

### 70. Test report

Test RuntimeとRunnerは構造化Reportを生成する。

Structured Test Report
↓
CLI
IDE
CI
HTML
Machine-readable output


安定情報は次とする。

- Test identity
- Outcome kind
- Failure code
- Source origin
- Structured diff
- Scheduler trace identity
- Counterexample identity
- Attachment identity


人間向けの表示文言を機械的interfaceにしない。

### 71. 適合試験

少なくとも次を検証する。

- Test discoveryが明示metadataだけを対象とする
- 成果物bindingがTestとして発見されない
- Module-local Testがprivate memberへアクセスできる
- Private値がTest scope外へescapeしない
- External Contract Testからprivate memberが見えない
- Test-only code変更で公開API hashが変わらない
- Effect Test handlerがTest scopeへ隔離される
- 任意binding patchが行われない
- Fakeが本番signatureを満たす
- TestごとのScheduler／Clock／Randomが独立する
- 同じseedとTraceで同じ結果を再現する
- 未完了TaskとResource leakを検出する
- Assertion failureとProgram Failureを区別する
- Expected CancellationがCleanup完了まで確認する
- Property Testが反例を再現する
- ShrinkingがFailure fingerprintを維持する
- Regression corpusの反例を再実行する
- Differential mismatchを構造化報告する
- Schedule explorationのBudget超過を証明扱いしない
- Terminal failureがProcessへ隔離される

### 72. 性能要件
PERF-TST-01:
Test discoveryのためにWorkspace全Sourceを毎回再parseしない

PERF-TST-02:
Test-only変更でProduction artifactを無効化しない

PERF-TST-03:
External Contract Testの再Compileと再実行を分離する

PERF-TST-04:
PureなTestをProcess起動なしで実行可能にする

PERF-TST-05:
必要なTestだけProcess／Sandboxへ隔離する

PERF-TST-06:
Property Testの各Caseで不要なCompiler artifactを再構築しない

PERF-TST-07:
Immutableな安全なArtifactをTest間で共有可能にする

PERF-TST-08:
Test-local mutable stateを共有しない

PERF-TST-09:
Trace記録量をProfileによって制御できる

PERF-TST-10:
Changed-only selectionと全Test実行の両経路を維持する

### 73. 残る後続項目

#### `OPEN-TST-SURFACE-001`

- test宣言の具体構文
- require／checkの名称
- Parameterized Test構文
- Property Test構文
- Test metadata記法

#### `OPEN-TST-RUNNER-001`

- CLI
- Filter式
- Process pool
- CI shard
- Report format
- Regression corpus配置

#### `OPEN-TST-SNAPSHOT-001`

- Snapshot形式
- Version
- Normalization
- Accept workflow
- Visual Diff

#### `OPEN-TST-CONFORMANCE-001`

- Trusted API
- Fault injection
- Runtime内部観測
- Native Adapter適合試験

#### `OPEN-PKG-ENTRY-001`

- Application entry
- Runtime profile
- Root Task scope
- 起動と終了

### 74. 最終状態

```text
OPEN-TST-001:
RESOLVED
```


本決定により、ReciplexaのTest機構は次の性質を持つ。

- 成果物制作へTest記述を強制しない
- 通常TestとArtifact Checkを分離する
- White-box TestとBlack-box Testを明確に区別する
- Private memberを公開せずにTestできる
- Test-only codeをProduction artifactから分離する
- Effect handlerとModule substitutionで依存を置換する
- 任意binding patchに依存しない
- Testごとに必要な強さの隔離を選べる
- Scheduler、Clock、Randomを決定的にする
- Task、Resource、CallbackのLeakを検出する
- Program Failure、Assertion failure、Cancellation、Defectを区別する
- Property TestとShrinkingを通常Testへ統合する
- Portable／Native、Full／Incremental等をDifferential Testできる
- 並行TaskのScheduleを再現・探索できる
- Test機能をpackage、Compiler、Runtime、Runnerへ適切に分担する
- Test-only変更を増分Buildで局所化する

## OPEN-PKG-ENTRY-001 Package Target・Entry Binding・Runtime Profile・Application Lifetime
### DD-PKG-ENTRY-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- PackageとTargetの分離
- Targetの種類と依存関係
- Entry binding
- Artifact／Application／Backend／Test／Check Target
- Runtime Profile
- Effect Handler
- Capability
- Resource namespace
- Secret Provider
- Native package instance
- Backend Registry
- ExecutorとBudget
- Root Scope
- Application lifetime
- Shutdown
- Supervision
- ApplicationOutcome
- Process Exit
- Artifact BuildとAtomic commit
- Package Manifest
- Resolved Target Descriptor
- Build Request
- Versioning
- 増分Build

### 0. 基本原則

Reciplexaでは、Packageそのものを直接起動・Buildする単位とはしない。

Packageは、配布、Version、依存解決、互換性を管理する単位とする。その内部に、Build、実行、出力、検査の単位であるTargetを複数定義できるものとする。

Package:
配布、Version、依存解決、互換性の単位

Target:
Build、実行、出力、検査の単位

Entry binding:
Targetの評価を開始するRPX binding


一つのPackageには、次のような複数の用途を共存させられる。

Package
├─ Library Target
├─ Artifact Target
├─ Application Target
├─ Backend Target
├─ Test Target
└─ Check Target


Targetごとに必要な依存、Effect、Capability、Runtime Profile、出力規則を分離する。

これにより、Testだけの変更で成果物を再Buildしたり、文書生成だけにGUIやGPU依存が混入したりすることを避ける。

### 1. PackageとTarget

Packageは複数Targetを持つことができる。

Targetの基本分類は次とする。

TargetKind =
  Library
  | Artifact
  | Application
  | Backend
  | Test
  | Check


Targetは、Package内で一意のTarget IDを持つ。

TargetIdentity {
  PackageInstanceId,
  TargetId
}


Target IDは、Entry bindingのidentityおよび人間向けの表示名から分離する。

TargetId:
Target自体の安定したidentity

EntryBindingId:
Targetが現在使用するEntry

DisplayName:
人間向けの名称


Entry bindingや表示名を変更しても、Target自体を同一Targetとして追跡できる構造を許容する。

### 2. Library Target

Library Targetは、他のPackageまたは同一Package内の別Targetから使用されるModule interfaceを提供する。

Library Targetは、Hostから直接起動するEntry bindingを必要としない。

LibraryTarget {
  exported-modules,
  public-interface,
  portable-implementation,
  native-implementations
}


Libraryの実行例やDemonstrationが必要な場合は、次を別Targetとして定義する。

- Application Target
- Artifact Target
- Documentation Test
- Test Target


Library本体へ便宜的なmainを要求しない。

Package内の複数Targetが共有するprivate Moduleは、Package外へ公開せずに共有できる。

### 3. Artifact Target

Artifact Targetは、文書、スライド、図、Animationなどの具体的成果物の意味値を生成する。

ArtifactInput
↓
Artifact Entry
↓
Document／Presentation／Composition等
↓
Validation
↓
Backend Planning
↓
Emission
↓
Artifact


Artifact Targetの標準workflowは次とする。

Build
↓
Validate
↓
Preview
↓
Export


Artifact Targetへ通常のUnit Test作成を要求しない。

成果物本文を書いて実際に生成・Preview・Exportすることを、制作上の主要な検証手段とする。

ただし、型検査、IR Validation、Resource検査、Backend Validationは常時行う。

### 4. Application Target

Application Targetは、継続的なInteractionまたはService lifetimeを持つProgramを表す。

ApplicationClass =
  Cli
  | Gui
  | Server
  | Worker
  | PluginHost
  | Custom


TargetKindはApplicationとし、CLI、GUI、Server等の区別はApplication Classとして別fieldに保持する。

これにより、Root Scope、Shutdown、Fault supervisionなどの共通規則を共有しつつ、起動入力とRequired EffectをApplication Classごとに定められる。

### 5. Backend Target

Backend Targetは、Render IR等を特定の形式または実行環境へ変換するBackend implementationを提供する。

対象例は次のとおりとする。

- PDF
- SVG
- PPTX
- Raster
- GPU Preview
- Video


Backend Targetは通常のApplication Entryを持つのではなく、専用のBackend Provider契約を提供する。

BackendProvider {
  descriptor,
  capability-query,
  planning-entry,
  emission-entry,
  validation-entry
}


Backendの詳細なCapability Profileは、別途Backend仕様で定める。

### 6. Test Target

Test Targetは、OPEN-TST-001で定めたTestDescriptor群とTest-only declarationを含む。

Test TargetはProduction Targetへ依存できる。

Test Target
→ Library／Artifact／Application Target


逆方向の依存は禁止する。

禁止:
Production Target
→ Test Target


これにより、Test-only codeやTest-only dependencyがProduction artifactへ混入することを防ぐ。

### 7. Check Target

Check Targetは、Artifactまたは中間IRへ、利用者固有の要件を適用する。

CheckStage =
  Domain
  | Layout
  | Visual
  | Render
  | EmittedArtifact


例として次を検査できる。

- Layout overflowがない
- Link切れがない
- 指定Font以外を使用していない
- Accessibility要件を満たしている
- Raster fallbackが発生していない
- 出力Artifactが指定Profileへ適合する


Check TargetはArtifact Targetへ依存する。

Check Target
→ Artifact Target


Artifact TargetがCheck Targetへ依存することはない。

ReleaseやCIにおいてCheck成功を要求する場合は、Target dependencyではなくBuild policyまたはTarget groupで表現する。

### 8. Target依存関係

Target間の依存関係はDAGとする。

Library
↓
Artifact
↓
Check


循環依存はBuild順序とLifecycleを不明確にするため、v1では禁止する。

Target依存とPackage依存を区別する。

Package dependency:
外部PackageのModule interfaceへの依存

Target dependency:
TargetのBuild結果や実行結果への依存


外部PackageのLibraryへは通常のPackage依存を使用する。

外部Artifactへ依存する場合は、Build artifact dependencyとして明示し、物理File名ではなく論理Artifact identityで参照する。

外部Application TargetをLibraryのようにimportすることは認めない。別Applicationの実行が必要な場合は、Process serviceまたはHost operationとして明示する。

### 9. Targetごとの依存とVersion解決

Test、GUI、Backend等のTargetだけが必要とする追加依存を宣言できる。

PackageDependencies:
Package全体で解決される依存

TargetDependencies:
特定Targetが利用する依存


ただし、Targetごとに同一Package依存のVersionを独立解決することは原則として行わない。

Workspace／Package:
Versionを共通解決

Target:
解決済み依存の利用範囲だけを指定


Test Targetだけが別Versionを暗黙に使用する設計は避ける。

### 10. Entry binding

Entry bindingは、通常の型付きRPX bindingである。

Entry専用の別言語や、固定された特殊関数を導入しない。

Entry binding:
通常のRPX binding

TargetDescriptor:
そのbindingをHostから呼び出すことを宣言


通常bindingが存在するだけではEntryにならない。

TargetDescriptorがEntry bindingを明示的に参照して初めて、そのTargetのEntryとなる。

固定名mainは便利な既定候補としてToolingが利用できるが、規範的なEntry identityにはしない。

### 11. Entry bindingの参照

ManifestではEntry bindingをSymbolic referenceとして指定する。

BindingReference {
  module-path,
  binding-name,
  expected-binding-kind
}


Compilerによる名前解決後、Stable Binding IDへ変換する。

Manifest Binding Reference
↓
Name resolution
↓
BindingId


HostやBuild systemは、解決済みBinding IDを用いる。

Entry bindingは、最終的に一つのmonomorphicな実行可能instanceへ解決されなければならない。

- 型parameter解決済み
- Effect row解決済み
- Module parameter解決済み
- Implementation selection可能

### 12. Artifact Entry

Artifact Entryの概念的な型は次とする。

ArtifactEntry<Input, Output> =
  Input -> Output
  effects {
    required-effects...
  }


出力型の例は次のとおりとする。

- Document
- Presentation
- Diagram
- Composition
- Image
- AudioComposition
- ArtifactBundle


Artifact Entryは、通常、PDFやPPTXのbytesを直接生成しない。

Artifact Entry:
成果物の意味値を生成

Build Host:
Backendを選択し、ArtifactをEmissionする


これにより、同じ本文をPreview、PDF、SVG、PPTX等に使用できる。

### 13. Artifact Entryの入力

Artifact Entryには、型付き入力値を渡す。

ReportInput {
  reporting-period,
  language,
  source-data,
  theme
}


外部Configは、CodecによってEntry input型へdecodeする。

External Configuration
↓
Decode
↓
Typed Entry Input
↓
Validation
↓
Artifact Entry


入力decodeまたはValidationに失敗した場合、Entry bodyを実行しない。

入力を必要としないArtifactは、unit入力として正規化できる。

### 14. Artifactの複数出力

一つのArtifact Targetが複数の論理成果物を生成する場合、型付きArtifact Bundleを返す。

ArtifactBundle {
  main,
  supplement,
  thumbnails,
  metadata
}


各成果物には安定した論理Member IDを与える。

ArtifactMemberId


論理Member IDと物理File名を分離する。

Artifact Member:
main

Output Mapping:
annual-report.pdf


Entry bodyへ物理File名を埋め込まない。

### 15. Streaming Artifact

巨大な動画、音声、長大文書等については、Streaming Artifactを許容する。

Input
->
ArtifactProducer<Chunk>


ProducerはHost管理のEncoderまたはSinkへ構造化されたchunkを供給する。

生のFile handleへ任意に書き込む方式にはしない。

Streaming Producerは次を満たさなければならない。

- Cancellation可能
- Backpressure対応
- 順序契約が明確
- Resource lifetimeが明示的
- Failure時にrollback可能
- 部分出力を完成Artifactとして扱わない


v1ではMaterializedまたはLazy Artifactを基本とし、Streamingは必要なTargetへ限定する。

### 16. Application Entry

Application Entryの概念的な型は次とする。

ApplicationEntry<Input, Exit> =
  Input -> Exit
  effects {
    task,
    required-application-effects...
  }


Entryへ巨大なRuntimeContextを直接渡さない。

通常の起動情報:
型付きEntry input

外部作用:
Effect

権限:
Runtime ProfileのCapability

終了管理:
Root Scope


これにより、Entryが不要なCapabilityへアクセスすることを防ぎ、Effect rowから依存関係を把握できる。

### 17. Application Classごとの入力
CLI
CliInput {
  arguments,
  allowed-environment-view,
  working-directory-view,
  invocation-metadata
}


標準streamやFilesystemは、Resource EffectとCapabilityを通じて利用する。

GUI
GuiInput {
  launch-kind,
  initial-documents,
  restoration-state,
  invocation-metadata
}


Window、Input、GPUはEffectおよびCapabilityとして提供する。

Server
ServerInput {
  configuration,
  listener-descriptions,
  startup-state
}


実SocketやNetwork authorityはNetwork EffectとCapabilityとして提供する。

CLI、GUI、Serverのすべてを一つの万能入力recordへ統合しない。

### 18. Configuration

Configurationを次の二種類に分ける。

Serializableで不変な設定:
Entry inputとして渡す

Dynamic service、Secret、Resource authority:
EffectとCapabilityとして提供する


Entry inputへ適するものの例：

- Theme
- Locale
- Feature selection
- Initial document list
- Output size


通常Configurationへ含めないものの例：

- Password
- API token
- GPU device
- Window handle
- Database connection
- Raw Native pointer

### 19. Generic Entry

一般のRPX関数はgenericであってよい。

ただし、Hostが起動する正準Entry instanceでは、すべての型parameter、Effect parameter、Module parameterが解決済みでなければならない。

Generic component
↓
Target composition
↓
Specialization
↓
Monomorphic Entry Instance


Open Effect rowを残したままEntryとして起動することは認めない。

内部関数:
effects {resource | e}
を許容

正準Entry:
effects {task, resource, diagnostic}
のように閉じている必要がある

### 20. Check Entry

Check Targetは、対象Stageの値を入力として受け取り、構造化されたCheck Reportを生成する。

CheckEntry<A> =
  A -> CheckReport
  effects {
    check,
    required-effects...
  }


Checkが対象Artifactを自分で再Buildするのではなく、Build graphから対象値またはArtifactを受け取る。

複数Findingを収集するため、Check専用Effectを利用できる。

### 21. Runtime Profile

Runtime Profileは、Entryが要求するEffect、Capability、Resource、Native implementation等を具体化する、型付き実行契約である。

Runtime Profile =
  Effect Handler
  + Capability grants
  + Resource namespaces
  + Secret providers
  + Native instances
  + Backend registry
  + Executor set
  + Budgets
  + Fault／Shutdown policies


概念的な正規形は次とする。

RuntimeProfile {
  identity,
  profile-class,
  effect-bindings,
  capability-grants,
  resource-namespaces,
  secret-providers,
  native-instances,
  backend-registry,
  executor-set,
  budgets,
  diagnostic-policy,
  fault-policy,
  shutdown-policy,
  version
}

### 22. Runtime Profile Class

Profile Classの基本分類は次とする。

RuntimeProfileClass =
  Cli
  | Gui
  | Server
  | Worker
  | Batch
  | Preview
  | Test
  | Sandbox
  | Custom


Profile Classは既定構成を選択するための分類であり、最終的なCapability集合そのものではない。

同じGUI Profile Classでも、EditorとViewerでは異なるCapabilityを持てる。

### 23. Runtime Profileの解決

Runtime Profileは次の要求・供給・制約から解決する。

Target requirements
∩
Host capabilities
∩
User grants
∩
Workspace policy
∩
Execution restrictions
↓
Resolved Runtime Profile


TargetやPackageはCapabilityを要求できるが、自らgrantを作成することはできない。

最終的なCapability grantはHostまたはHostに認可された主体が行う。

### 24. Profile解決Failure

Profile解決Failureを構造化する。

ProfileResolutionFailure =
  MissingEffectHandler
  | MissingCapability
  | VersionMismatch
  | ConflictingHandler
  | ExecutorUnavailable
  | NativeImplementationUnavailable
  | BackendUnavailable
  | BudgetUnsatisfiable
  | PolicyDenied
  | InvalidProfileComposition


Required EffectまたはRequired Capabilityを満たせない場合、Entryを開始しない。

### 25. Effect Handler

EntryのEffect rowは、必要な作用を表す。

Runtime Profileは、それを処理する具体的Handlerを提供する。

Entry:
effects {
  task,
  resource,
  clock,
  render
}

Runtime Profile:
task     → Task Scheduler
resource → File／Package Resource Handler
clock    → System Clock
render   → GPU Preview Backend


HandlerはEffect serviceまたは一貫性domain単位で選択する。

次のような不整合な寄せ集めを既定では認めない。

clock.now:
System Clock

clock.sleep:
Virtual Clock


部分overrideを行う場合は、明示的なComposite Handlerとして構築し、整合性を検証する。

### 26. Effect Handlerのidentity

次を区別する。

EffectIdentity
EffectContractVersion
HandlerImplementationIdentity
HandlerImplementationVersion


Handlerは、Required Effect contractへ適合する必要がある。

文字列名が一致するだけでは適合とみなさない。

### 27. EffectとCapabilityの分離

EffectとCapabilityは関連するが別概念である。

Effect:
どの外部作用を要求するか

Capability:
どの対象に、どの範囲で作用できるか


例：

resource Effect:
Fileを開く操作を要求できる

Filesystem Capability:
特定Directory以下を読める


Entryがresource Effectを持つことは、任意Fileへのアクセス権を意味しない。

### 28. Capability

Capabilityの概念的構造は次とする。

Capability {
  capability-identity,
  capability-class,
  authority-scope,
  allowed-operations,
  constraints,
  lifetime,
  owner-runtime,
  transfer-policy,
  audit-policy
}


Capability Classの例：

- FilesystemRead
- FilesystemWrite
- NetworkConnect
- WindowCreate
- GpuUse
- SecretRead
- ProcessSpawn
- BackendUse


Capabilityは通常のPath文字列やBoolean flagではない。

対象を知っていることと、その対象へ操作する権限を持つことを分離する。

### 29. Capabilityの縮小

Capabilityは、より狭いCapabilityへ制限できる。

Directory ReadWrite
↓
Directory ReadOnly
↓
Specific File ReadOnly


親Capabilityより強いCapabilityへ拡大することはできない。

許可:
ReadWrite → ReadOnly

禁止:
ReadOnly → ReadWrite


Capability grantはHost authorityだけが行える。

Applicationや子Taskは、自分が保有するCapabilityを縮小して渡すことだけができる。

### 30. CapabilityのLifetimeと移送

CapabilityのLifetimeは、Owner ScopeのLifetime以下でなければならない。

Capability lifetime
⊆
Owner Scope lifetime


Task間のCapability移送は、CapabilityのTransfer Policyへ従う。

- 継承可能
- Shared
- Exclusive
- Move only
- Non-transferable


子Taskは親Taskが持たないCapabilityを取得できない。

Exclusive Capabilityの例：

- Transactional output
- Single-owner Builder
- Exclusive GPU context


これらを暗黙に複製しない。

### 31. Ambient authorityの禁止

Applicationが現在のRuntime Profile全体を列挙し、任意Capabilityを名前で取得できる万能APIは提供しない。

禁止:
current-runtime-profile()
get-any-capability(name)


Required CapabilityはHostが起動前に解決する。

Optional Capabilityだけを、型付きCapability queryで確認できる。

query-capability:
CapabilityRequirement
-> CapabilityAvailability

### 32. Required・Optional・Alternative Capability
CapabilityNecessity =
  Required
  | Optional
  | Alternative

Required

なければTargetを起動できない。

Optional

存在すれば追加機能を利用できる。

Alternative

候補のいずれかを必要とする。

GPU Renderer
or
CPU Renderer


FallbackはRuntime Profile Planningで解決し、意味や品質の変化をDiagnosticへ記録する。

### 33. Resource Namespace

Resourceの名前空間を分離する。

ResourceNamespace =
  Package
  | Document
  | UserSelected
  | Temporary
  | Cache
  | HostProvided


同じPath文字列でも、Namespaceが異なれば別Resourceである。

Package:"images/logo.png"

Document:"images/logo.png"


を混同しない。

### 34. Package Resource

Packageに同梱された不変Resourceは、Package Resource Namespaceへ所属する。

対象例：

- Font
- Image
- Template
- Locale data


原則としてRead-onlyであり、Content-addressed identityを持つ。

### 35. Document Resource

Document Resourceは、特定DocumentまたはWorkspaceへ所属する。

DocumentごとにCapability viewを分離できる構造を推奨する。

Application Profile
├─ Document A Profile View
└─ Document B Profile View


Document AのTaskがDocument BのResource capabilityを暗黙に持たないようにする。

### 36. User-selected Resource

利用者がFile picker等で選択したResourceについて、そのResourceへ限定されたCapabilityを発行できる。

Applicationへ任意Filesystem capabilityを与える代わりに、利用者が選択した対象への局所Capabilityだけを与える。

### 37. Temporary ResourceとCache

Temporary ResourceはScope終了時にcleanupする。

Cache Resourceは再生成可能でなければならず、削除によってProgramの規範結果が変化してはならない。

Temporary:
Lifecycle管理対象

Cache:
Eviction可能

### 38. Secret Provider

Secret値を通常Config、Manifest、Build cacheへ保存しない。

SecretRequirement {
  secret-id,
  purpose,
  necessity,
  access-mode,
  lifetime
}


実際のSecret値はRuntime ProfileのSecret Providerから取得する。

Secret値は次へ出力しない。

- Diagnostic
- Test snapshot
- Cache key
- Manifest
- Scheduler trace
- Build metadata


TestではProduction Secret Providerではなく、Test用Providerを使用する。

### 39. Native Package Instance

Native packageの状態は、可能な限りRuntime instance、Device、Worker process、Document session等の明示単位へ所属させる。

Process-global mutable singletonを既定にしない。

NativeInstance {
  package-instance-id,
  implementation-id,
  abi-version,
  instance-state,
  executor-affinity,
  capabilities,
  shutdown-contract
}


Profile構築時にDescriptor、ABI、Version、Capability、Executorを検査した後でinstanceを作成する。

### 40. Backend Registry

Runtime Profileは、利用可能なBackend implementationを登録する。

BackendRegistry {
  backends,
  profiles,
  capability-snapshots,
  selection-policy
}


ApplicationがRegistry内部を自由に変更することはできない。

TargetまたはJobがBackend requirementを提出し、Plannerが適合するBackendを選択する。

一つのJob中はBackend selectionを固定する。

### 41. Executor Set

Runtime Profileは利用可能なExecutorを持つ。

ExecutorSet {
  default,
  main?,
  cpu-pool?,
  io?,
  gpu?,
  dedicated-executors
}


各Executorは次を宣言する。

- Executor identity
- Executor class
- Thread／Process affinity
- Blocking policy
- Capacity
- Shutdown behavior


Entryの初期ExecutorはTargetDescriptorで指定できる。

EntryExecutor =
  Default
  | Main
  | Dedicated(ExecutorClass)


Entry全体を一Executorへ固定するものではない。Taskごとに必要なExecutorへ移動できる。

### 42. Runtime Budget

Runtime ProfileはResource Budgetを持つ。

RuntimeBudgets {
  memory,
  cpu,
  task-count,
  open-resources,
  native-operations,
  gpu-memory,
  worker-processes,
  cache,
  output-size,
  diagnostic-size
}


Hard LimitとSoft Limitを区別する。

HardLimit:
超過を許可しない

SoftLimit:
Cache eviction、backpressure、品質調整等を試みる


Budget超過は原則として型付きFailureとする。

Runtime bookkeepingの破損による超過はDefectである。

### 43. Runtime Profileの合成

Profileは複数のProfile fragmentから構築できる。

Base GUI Profile
+
Filesystem fragment
+
GPU Backend fragment
+
Restricted Network fragment


合成規則を次に分類する。

- Disjoint
- Compatible
- Restrictive
- Conflict


後から宣言されたHandlerが黙って優先される形式にはしない。

Effect HandlerやCapability requirementが競合する場合、明示的な解決を要求する。

### 44. Profile Snapshot

Rendering、Export、長時間Job等の開始時に、Runtime Profile Snapshotを作成する。

RuntimeProfileSnapshot {
  profile-identity,
  profile-revision,
  handler-bindings,
  capability-grants,
  resource-revisions,
  native-instance-revisions,
  backend-capabilities,
  executor-identities,
  budget-slice
}


Job実行途中で、別Backend、別Font環境、別Native実装へ暗黙に切り替えない。

Capability revocationはSnapshotより優先する。

Revoked Capabilityを使用しようとしたJobは、Authorization failureまたはCancellationで停止する。

### 45. JobごとのProfile縮小

JobはApplication Root Profile全体を受け取らない。

例としてPDF Export Jobには次だけを与える。

- Document Resource read
- Font Resource read
- PDF Backend
- Output write
- CPU executor


Window、Network、他Document書込み等の不要Capabilityを与えない。

Application Profile
↓ restrict
Export Job Profile Snapshot


子Taskも縮小済みProfile viewだけを継承する。

### 46. Root Scope

一回のApplicationまたはArtifact BuildのLifetimeを、Root Scopeで管理する。

Root Scope
├─ Entry Task
├─ Child Task scopes
├─ Service Tasks
├─ Resource scopes
├─ Callback registrations
├─ Native operations
├─ Runtime Profile Snapshot
├─ Diagnostic context
├─ Fault boundary
└─ Shutdown controller


Entry TaskはRoot Scopeの最初のTaskである。

ApplicationのLifetimeは、Entry Task単体ではなくRoot Scope全体で決まる。

### 47. Root Scopeの状態
RootScopeState =
  Preparing
  | Running
  | Quiescing
  | Cancelling
  | Finalizing
  | Completed
  | Aborted

Preparing

Profile、Input、Native instance等を準備する。

Running

通常のTask、Resource、Operationを実行する。

Quiescing

正常終了へ向け、新規の通常処理を停止し、進行中処理を完了させる。

Cancelling

Grace periodを超えたTask等へCancellationを要求する。

Finalizing

Resource、Backend、Native instance、Diagnostic等を最終化する。

Completed

正常または構造化された異常結果を確定した。

Aborted

Terminal failure等により通常Finalizationを完了できなかった。

### 48. Entryの返却とShutdown

Entry Taskが戻っても、即座にProcessを終了しない。

Entry returns
↓
Root Scope enters Quiescing
↓
Child Taskを待つ
↓
必要ならCancellation
↓
Cleanup
↓
ApplicationOutcome確定


Entryが返った瞬間に子Taskを破棄することも、子Taskを無期限に放置することも認めない。

### 49. 正常終了要求とCancellation

正常な終了要求とTask Cancellationを区別する。

正常な終了要求:
Application-levelの終了意思

Cancellation:
未完了Taskを途中終了させる制御


通常の終了経路は次とする。

Shutdown request
↓
Quiescing
↓
自然終了を待つ
↓ 未完了なら
Cancelling


利用者の通常の終了操作を、最初から異常Cancellationとして扱わない。

### 50. Shutdown Request
ShutdownReason =
  EntryCompleted
  | UserRequested
  | HostRequested
  | ParentCancelled
  | OperatingSystemRequested
  | UnhandledFailure
  | Defect
  | ResourceExhaustion
  | ProfileRevoked


複数のShutdown Requestが発生した場合、最初の要求を記録しつつ、後から発生したより重大な原因へPrimary reasonを昇格できる。

### 51. Shutdown中のTask作成

Taskの目的を分類する。

TaskPurpose =
  NormalWork
  | ShutdownWork
  | CleanupWork


状態ごとの許可は次とする。

Running:
すべて許可

Quiescing:
NormalWorkを禁止
ShutdownWork／CleanupWorkを許可

Cancelling:
原則CleanupWorkだけを許可

Finalizing:
Runtime認可済みCleanupWorkだけを許可


Shutdown中に通常処理を増やし続けることを防ぐ。

### 52. Service Task

GUI event loop、Autosave、File watcher、Server listener等をService TaskとしてRoot Scope内で管理する。

ServiceShutdownContract {
  begin-quiesce,
  cancel,
  finalize,
  deadline-policy
}


Application外の暗黙global Taskとして動作させない。

### 53. Detached Task

v1では、Application Root Scopeから完全に独立する一般的なDetached Taskを禁止する。

長寿命処理は次へ所属させる。

- Root Scope内のService Task
- Host-managed service scope
- 別Application Target
- Worker process


Host-managed scopeへTaskを移す場合は、明示的なOwnership transferとCapabilityを要求する。

### 54. Grace periodとForced Shutdown

Shutdownを次の段階に分ける。

1. Graceful／Quiescing phase
2. Cancellation phase
3. Finalization
4. 必要ならHostによるForced termination


Grace periodには、Virtual Durationだけでなく、Wall Duration、Scheduler step、Pending operation数等のBudgetを使用できる。

GracePolicy {
  virtual-duration?,
  wall-duration?,
  scheduler-step-budget?,
  pending-operation-budget?
}


Cancellation後もCleanupのための猶予を与える。

Terminal failureやNative hangに対しては、ProcessまたはSandbox境界をHostが停止する。

### 55. Fault Policy
FaultPolicy =
  FailFast
  | Supervised
  | Collect

FailFast

未処理Child FailureでRoot Scopeを終了する。

Supervised

Supervisorが再起動、隔離、無効化、RootへのEscalationを判断する。

Collect

複数JobのOutcomeを収集し、最後にまとめる。

DefectやTerminal failureを通常Failureとして収集するものではない。

### 56. Supervisor
SupervisorPolicy {
  restart-policy,
  restart-budget,
  backoff,
  failure-classification,
  child-shutdown-policy,
  escalation-policy
}


Restart Policy：

Never
OnFailure
OnSelectedFailure
Always


無限再起動を防ぐため、回数、時間窓、累積cost等のBudgetを持つ。

Budget超過時は上位SupervisorまたはRoot ScopeへFailureを昇格する。

### 57. 未観測Task Failure

Task handleが明示的にawaitされなかった場合でも、Root ScopeはTask Outcomeを把握する。

未観測Failureを黙って破棄しない。

UnhandledChildFailure
↓
Fault Policy


Task handleのdropによってFailureまで消える規則にはしない。

### 58. Resource Cleanup

Task、Service、Resource、Native instanceの終了依存関係をDAGとして管理できる。

Service A depends on Service B


Shutdownは逆Topological orderで行う。

Aを停止
↓
Bを停止


依存cycleはProfile Validation failureとする。

### 59. Cleanup Failure

Cleanup中にもFailureが起こり得る。

- File flush failure
- Backend finalize failure
- Native shutdown failure
- Temporary Resource cleanup failure


既にPrimary Failureがある場合、Cleanup Failureをsuppressedとして保持する。

Entryが成功していた場合は、Cleanup FailureがPrimaryとなり得る。

複数の独立Cleanup Failureは集合として保持できる。

### 60. DefectとTerminal Failure

Defectの影響範囲を分類する。

DefectScope =
  Task
  | Service
  | RuntimeInstance
  | Process
  | Unknown


Task-local Defectは、Runtime全体の健全性が確認できる場合に限って隔離できる。

Runtime instanceの整合性が疑わしい場合、そのRoot Scopeを終了する。

Memory safety、Native ABI、Runtime bookkeepingの健全性が不明な場合はTerminal failureとする。

Terminal failureを通常Failureへ降格しない。

### 61. ApplicationOutcome

Root Scope終了後の正準Outcomeを次とする。

ApplicationOutcome<A, E> =
  Completed(A)
  | Requested(ExitIntent)
  | Failed(E)
  | Cancelled(CancellationReport)
  | Defected(DefectReport)
  | Aborted(InfrastructureAbort)

Completed

EntryとRoot Scopeが正常終了した。

Requested

正常な終了意図により終了した。

Failed

型付きProgram Failureにより終了した。

Cancelled

親Host等からのCancellationで終了した。

Defected

隔離可能なDefectを構造化して報告した。

Aborted

Terminal failure、Watchdog、Worker通信断等により通常結果を確定できなかった。

### 62. ExitIntent
ExitIntent {
  category,
  user-message?,
  restart-hint?,
  data?
}


Categoryの例：

- Success
- NoChanges
- UserCancelledOperation
- RestartRequested
- UpdateRequested
- Custom


ExitIntentはOSの整数Exit codeではない。

GUI Host内の一Application instanceだけを閉じる場合にも利用できる。

### 63. Process Exit

Standalone CLIやApplicationでは、ApplicationOutcomeをProcess Exit Statusへ変換する。

ApplicationOutcome
↓
ExitMappingPolicy
↓
ProcessExitStatus


具体的な数値はPlatformおよびApplication policyへ委ねる。

次の変換は禁止する。

Defect → Success
Terminal Failure → Success


通常Failureを成功として扱いたい場合は、Application logic内で意味のある正常結果へ変換する。

### 64. Restart

Application自身がRoot Scopeを直接再利用してRestartしない。

Old Root Scope
↓ Shutdown
Requested(Restart)
↓
Hostが新Profileを解決
↓
New Root Scope


旧Task、旧Resource、旧Capabilityを新Root Scopeへ暗黙に持ち越さない。

必要状態は、型付きかつPortableなRestoration Stateとして明示的に受け渡す。

### 65. Artifact Build Root

Artifact BuildもRoot Scopeで管理する。

Artifact Build Root
├─ Entry evaluation
├─ Resource loading
├─ Layout
├─ Backend planning
├─ Emission
└─ Output transaction


正常経路：

Artifact意味値生成
↓
子Task完了
↓
Validation
↓
Backend emission
↓
Artifact validation
↓
Atomic commit
↓
Completed


FailureまたはCancellation時にはOutput transactionをrollbackする。

### 66. Output Transaction
OutputTransactionState =
  Preparing
  | Writing
  | Finalizing
  | Committed
  | RolledBack


完成前の出力を正式Artifactとして公開しない。

Failure時には次を行う。

- Temporary file削除
- Partial upload破棄
- Manifest rollback
- Lock解除


Streaming Artifactでは、Producer、Encoder finalize、Artifact validation、Sink commitがすべて成功して初めてCompletedとなる。

### 67. Manifestの基本方針

Package Manifestは、静的な宣言dataとする。

Manifest自体を任意のRPX Programとして評価する方式は採用しない。

Manifestは次の性質を持つ。

- 実行せずに解釈できる
- 決定的である
- IDEが読める
- Cross compilation前に検査できる
- Security reviewが可能
- Versioningできる


ManifestへProgram logic、File読込み、Network access、任意関数を入れない。

### 68. Manifest・Descriptor・Planの三層
Manifest Declaration
↓ Compiler／Resolver
Resolved Target Descriptor
↓ Host／Build Request
Execution／Build Plan

Manifest Declaration

利用者が記述する静的宣言。

Resolved Target Descriptor

ManifestとSourceを型検査・解決した正規形。

Execution／Build Plan

具体的なHandler、Capability、Backend、Executor、Budgetを割り当てた実行計画。

### 69. ManifestとCompiler推論の境界

CompilerがSourceから導出する情報：

- Entry input型
- Entry output型
- Required Effect
- Binding identity
- Module依存
- Generic解決状態
- Portable codecの有無


Manifestが宣言する情報：

- Target ID
- Target kind
- Entry binding
- Application Class
- Profile preset
- Capability requirement
- Target dependency
- Output preset
- Default Target


HostまたはBuild Requestが与える情報：

- 実Capability
- Secret値
- 実際のOutput先
- GPU device
- Memory budget
- Resource Snapshot


Compilerが推論できる型やEffectを、Manifestから意味的に上書きすることは認めない。

Manifestは追加制約を課すことはできる。

### 70. Resolved Target Descriptor

概念的な正規形は次とする。

ResolvedTargetDescriptor {
  target-identity,
  display-metadata,
  target-kind,

  entry-binding-identity?,
  exported-modules?,

  input-type?,
  output-type?,
  required-effects,

  target-dependencies,
  package-dependencies,
  artifact-dependencies,

  profile-requirement,
  capability-requirements,
  executor-requirement,
  implementation-policy,
  isolation-requirement,
  reentrancy-policy,

  shutdown-policy?,
  output-contract?,
  check-contract?,

  provenance,
  descriptor-version
}


TargetKindに応じて不要なfieldは存在しないものとして扱う。

利用者へ万能recordを直接書かせず、TargetKindごとのManifest schemaから正規形へloweringする。

### 71. Capability Requirement
CapabilityRequirement {
  requirement-id,
  capability-class,
  necessity,
  authority-shape,
  operation-set,
  constraints,
  purpose,
  fallback-group?
}


Manifestでは実際のCapability tokenや絶対Pathを保存しない。

必要なAuthorityの形とPurposeだけを宣言する。

- UserSelectedFileRead
- UserSelectedOutputWrite
- DocumentDirectoryRead
- SpecificServiceConnect
- GpuRendering


Permission UIやSecurity reviewは静的Purpose metadataを利用できる。

### 72. Secret Requirement
SecretRequirement {
  secret-id,
  purpose,
  necessity,
  access-mode,
  lifetime
}


ManifestにはSecret requirementだけを記録する。

Secret値はManifest、Lockfile、Build Requestのportable部分、Cache keyへ保存しない。

### 73. Build Request
BuildRequest {
  target-identity,
  input,
  build-configuration,
  runtime-profile-request,
  output-request,
  validation-policy,
  check-policy,
  cache-policy,
  execution-policy
}


ManifestはTargetの静的定義であり、Build Requestは一回の具体的なBuildまたは起動要求である。

### 74. Build Requestの正規化
External Request
↓
Decode
↓
Typed Request
↓
Validation
↓
Normalized Build Request
↓
Fingerprint


同じ意味のRequestは、可能な限り同じFingerprintを生成する。

Capability token、Secret値、Native pointer等をFingerprintへ含めない。

結果へ影響するResource identity、Profile selection、Implementation version等を含める。

### 75. Output Request
OutputRequest {
  artifact-member,
  output-format,
  output-profile,
  destination-capability,
  naming-policy,
  overwrite-policy,
  commit-policy
}


Output先をPath文字列だけで指定しない。

Destination Capabilityと組み合わせる。

Manifestへ利用者環境固有の絶対Pathを保存しない。

### 76. Overwrite Policy
OverwritePolicy =
  FailIfExists
  | ReplaceAtomically
  | CreateNewVersion
  | HostPrompt


Interactive HostではHostPromptを利用できる。

Batch／CIでは決定的なPolicyを使用する。

Entry bodyが直接確認Dialogを出してOutput overwriteを解決する方式は避ける。

### 77. Default Target

Package内にTargetが一つだけなら、自動選択できる。

複数Targetがある場合は、ManifestでDefault Targetを指定できる。

DefaultTargetId


複数Targetが存在し、Defaultがない場合は、利用者またはToolingへ選択を要求する。

宣言順や名前順で恣意的に選択しない。

### 78. Target Configuration

同じEntry bindingと依存構造を使い、入力値やOutput Profileだけが異なる場合は、Targetを複製するのではなくBuild Configurationとして扱う。

Target:
annual-report

Configuration A:
Japanese PDF

Configuration B:
English PDF

Configuration C:
Editable PPTX


Entry bindingまたは依存構造が意味的に異なる場合は、別Targetとする。

### 79. Manifest Versioning

Manifest schemaはPackage Versionとは独立したVersionを持つ。

ManifestVersion


次を区別する。

- Backward-compatible extension
- Migration-required change
- Unsupported future version


未知の必須fieldを無視して続行しない。

Namespace付きOptional Extensionは、規則に従って警告付きで無視できる場合がある。

### 80. Resolved Descriptor Version

Compilerが生成するResolved Target Descriptorも独立Versionを持つ。

TargetDescriptorVersion


Build system、IDE、Runner、Runtime launcherは適合Versionを検査する。

未知VersionをMemory layoutの推測で読み込まない。

### 81. Manifest Extension

第三者Tooling向けmetadataはNamespace付きExtensionとして保存できる。

extensions {
  namespace -> versioned-data
}


ExtensionがTargetの実行意味を密かに変更してはならない。

Execution semanticsを変更する情報は、規範Manifest schemaへ含める必要がある。

### 82. .rpi・Test・Native metadataとの分離

論理schemaを次のように分離する。

.rpi:
Public type／Module／Effect interface

Target Metadata:
Resolved Target Descriptor

Test Metadata:
TestDescriptor index

Native Metadata:
Native implementation descriptor

Backend Metadata:
Backend Capability descriptor


これらを一つのPackage artifact containerへ格納することはできるが、意味上は分離する。

### 83. Incremental Build

Target単位で次のHashを分ける。

TargetInterfaceHash
TargetImplementationHash
TargetConfigurationHash
TargetDependencyHash
TargetDisplayHash


表示名変更だけでCompiled codeやArtifactを無効化しない。

Capability requirement変更では、Entry codeを再利用しつつProfile ResolutionとLaunch Planだけを無効化できる。

Output mapping変更では、Artifact意味値を再利用し、EmissionとCommitだけを再実行できる。

### 84. Target変更時の無効化
Test-only変更
Test Target:
再Build

Production Target:
維持

Artifact本文変更
Artifact Target:
再評価

依存Check:
再実行

無関係Target:
維持

Entry型変更
Target Descriptor:
再Validation

Host integration:
再Build候補

Required Effect変更
Runtime Profile:
再適合検査

Launch Plan:
無効化

Backend Profile変更
Artifact意味値:
再利用可能

Backend Planning／Emission:
再実行

### 85. Manifest Validation

少なくとも次を検査する。

- Target IDの重複
- Default Targetの存在
- Entry bindingの存在
- Entry型の適合
- 未解決generic parameter
- Open Effect row
- Target dependency cycle
- ProductionからTestへの依存
- Capability requirementの矛盾
- Profile presetの適合
- Executor availability
- Reentrancy制約
- Artifact output型
- Secret requirementの妥当性
- Manifest Version


Validated ManifestとResolved DescriptorだけをBuild Planningへ渡す。

### 86. Diagnostic

ManifestおよびEntry関連のDiagnosticを構造化する。

- ManifestParseFailure
- ManifestSchemaFailure
- TargetResolutionFailure
- EntryTypeMismatch
- DependencyCycle
- CapabilityRequirementConflict
- ProfileResolutionFailure
- OutputMappingFailure
- VersionMismatch


Diagnosticには次を含める。

- Stable diagnostic code
- Manifest Source span
- 関連Entry bindingのSource origin
- 推論された型・Effect
- 修正候補

### 87. Security review

ManifestおよびResolved Descriptorから、実行前に次を一覧化できるようにする。

- Filesystem authority要求
- Network接続要求
- Secret要求
- Native implementation
- Backend requirement
- Process spawn
- Output write
- GPU利用


PackageはCapabilityを要求できるが、Manifestを書くだけでgrantを取得することはできない。

Hostまたは利用者のAuthorizationを必要とする。

### 88. 適合試験

少なくとも次を検査する。

- Packageが複数Targetを持てる
- Library TargetがEntryを要求しない
- Target identityとEntry identityが分離される
- Target依存cycleを拒否する
- ProductionからTestへの依存を拒否する
- Artifact Entryの戻り型を正しく認識する
- Application Classと入力型の不一致を拒否する
- Open Effect rowを持つEntryを拒否する
- Profile不足時にEntryを開始しない
- Capability grantなしでResourceへアクセスできない
- Capability縮小によってAuthorityが増えない
- 子Taskが親にないCapabilityを取得できない
- Scope終了後のCapability使用を拒否する
- SecretがDiagnosticやSnapshotへ出ない
- Entry返却後にRoot ScopeのCleanupを行う
- Quiescing後のNormalWorkを拒否する
- 未観測Task FailureをOutcomeへ反映する
- Detached Task escapeを拒否する
- Artifact失敗時にOutputをcommitしない
- Manifestの推論値上書きを拒否する
- Output先にCapabilityを要求する
- Test-only変更でProduction Targetを無効化しない
- Display metadata変更でCompiled artifactを無効化しない

### 89. 性能要件
PERF-PKG-ENTRY-01:
Target単位でBuild・Validation・Cacheを分離する

PERF-PKG-ENTRY-02:
Test-only変更でProduction Targetを再Buildしない

PERF-PKG-ENTRY-03:
Output Profile変更でArtifact意味値を再生成しない

PERF-PKG-ENTRY-04:
Profile変更時に無関係なCompiler artifactを維持する

PERF-PKG-ENTRY-05:
Manifest表示情報変更でSemantic Target Hashを変えない

PERF-PKG-ENTRY-06:
長大ArtifactでStreamingおよびPage／Frame単位処理を許可する

PERF-PKG-ENTRY-07:
Jobごとに必要CapabilityとBudgetだけを切り出す

PERF-PKG-ENTRY-08:
Runtime Profile全体の変更で無関係Cacheを捨てない

PERF-PKG-ENTRY-09:
Root Scope終了時に未管理Taskを残さない

PERF-PKG-ENTRY-10:
Same Target＋Same SnapshotのBuildを再利用可能にする

### 90. 実装責任境界
Package Manifest／Package Manager
- Package identity
- Target declaration
- Package dependency
- Target dependency
- Default Target
- Profile／Output preset

Compiler
- Entry binding解決
- Entry型・Effect検査
- Resolved Target Descriptor
- Public interface
- Target dependency summary

Build System
- Build Request
- Target graph
- Incremental invalidation
- Artifact dependency
- Output planning
- Atomic commit

Runtime Launcher
- Runtime Profile解決
- Capability grant
- Root Scope作成
- Entry起動
- Shutdown
- ApplicationOutcome

Host
- User authorization
- Secret Provider
- Process／Sandbox
- OS signal
- Output destination
- Process Exit mapping

### 91. 後続項目

#### `OPEN-PKG-MANIFEST-SURFACE-001`

- Manifestの具体構文
- Target宣言構文
- Capability requirement記法
- Entry binding参照記法
- Output preset記法

#### `OPEN-PKG-ENTRY-SURFACE-001`

- Entry annotation
- Artifact Entryの簡略構文
- Application Entryの簡略構文
- Typed Configurationの記述

#### `OPEN-BACKEND-PROFILE-001`

- Backend Capability Profile
- Output Profile
- Backend selection
- Backend conformance

#### `OPEN-PKG-HOT-RELOAD-001`

- Warm rebuild
- Application state migration
- Runtime Profile更新
- Hot reload可能なbinding

### 92. 最終状態

```text
OPEN-PKG-ENTRY-001:
RESOLVED
```


本決定により、ReciplexaのPackage実行境界は次の性質を持つ。

- PackageとBuild／実行Targetを分離する
- 一Packageに複数Targetを持てる
- Libraryへ不要なEntryを要求しない
- Artifact本文とBackendを分離する
- Entryを通常の型付きRPX bindingとして扱う
- 万能RuntimeContextをEntryへ渡さない
- 外部作用をEffect、権限をCapabilityとして分離する
- Runtime Profileを型付き実行契約として扱う
- JobごとにCapabilityとBudgetを縮小する
- Task、Resource、Native operationをRoot Scopeで管理する
- 正常終了とCancellationを区別する
- Detached Taskを原則禁止する
- Cleanup完了後にApplicationOutcomeを確定する
- ArtifactをAtomic commitする
- Manifest Declaration、Resolved Descriptor、Execution Planを分離する
- Build Requestを型付きかつ再現可能にする
- Target単位で増分BuildとCacheを行う

## OPEN-BACKEND-PROFILE-001 Backend Capability・Output Profile・Planning・情報損失・Artifact検証
### DD-BACKEND-PROFILE-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Backend Capability Schema
- Capability Snapshot
- Output Profile
- Hard ConstraintとPreference
- ApproximationとTolerance
- Text、Accessibility、Raster、Font、Color Policy
- Backend Planning Algorithm
- Backend Planning IR
- Raster Island
- Semantic Mapping
- Resource Planning
- Output Decision
- 情報損失
- 明示承認
- Planning Failure
- Alternative Plan
- Artifact Verification
- 増分Planning
- Backend Conformance

### 0. 基本原則

Reciplexaでは、Render IRをBackend Emitterへ直接渡し、Emission中にBackendが場当たり的なFallbackを選択する構造を採用しない。

出力処理は、次の段階に分離する。

Render IR
+
Document Semantics
+
Backend Capability Snapshot
+
Output Profile Snapshot
+
Resource Snapshot
↓
Backend Planning
↓
Validated Backend Planning IR
↓
Backend Emission
↓
Emitted Artifact
↓
Artifact Verification


各段階の責任は次のとおりとする。

Render IR:
Backend非依存の規範的な視覚意味

Backend Capability:
Backendが技術的に実現できること

Output Profile:
今回の出力で保存すべき性質と、
許される変換・損失

Backend Planning:
各NodeまたはSubtreeの具体的な表現方法

Backend Emission:
確定済みPlanに従ったArtifact生成

Artifact Verification:
実際の出力がPlanどおりであることの検査


Backendは、Output Profileに含まれない近似、Raster化、省略、Font代替等を独自判断で行ってはならない。

### 1. Backend Capability

Backend Capabilityは、Backendが特定の機能をどの方法で表現できるかを示す構造化情報である。

単純な対応／非対応のBooleanにはしない。

CapabilitySupport =
  Native
  | EquivalentLowering
  | Approximate
  | RasterFallback
  | Unsupported


Capabilityには、適用条件、制限値、保存される性質、失われる性質、Cost、適合試験情報を含める。

### 2. Native

Nativeは、対象BackendまたはFormatが機能を直接表現できる状態を示す。

例：

SVG Path
PDF Vector Path
PPTX Text Box
GPU Texture


Nativeであっても、すべての意味を完全に保持するとは限らない。

たとえばNative Textであっても、Viewer側のFont環境によって再配置される可能性がある。

したがって、Native Capabilityにも次を記録する。

NativeSupport {
  conditions,
  preserved-properties,
  constrained-properties,
  limits,
  cost,
  conformance-reference
}

### 3. Equivalent Lowering

EquivalentLoweringは、別表現への変換によって、指定された観測意味を保存できる状態を示す。

例：

Rounded Rectangle
→ Path

Arc
→ Cubic Bézier

Stroke
→ Filled Outline


何に関して等価であるかを明示する。

EquivalenceDomain =
  Visual
  | Geometric
  | Textual
  | Temporal
  | Semantic
  | Accessibility


視覚上は等価でも、高水準Objectとしての編集可能性を失う場合には、その損失を別途記録する。

### 4. Approximate

Approximateは、完全な意味保存はできないものの、規定Tolerance内で近似できる状態を示す。

例：

Complex Gradient
→ 有限個の領域へ近似

Curve
→ Polylineへ近似

Unsupported Color Space
→ 利用可能なColor Spaceへ変換


Approximationには、必ず次を要求する。

ApproximationSupport {
  conditions,
  error-model,
  error-bound,
  quality-range,
  losses,
  cost,
  deterministic
}


誤差上限を評価できない近似は、Strict Profileでは使用できない。

BestEffort Profileでは未知誤差として候補化できるが、明示的なWarningを必要とする。

### 5. Raster Fallback

RasterFallbackは、NodeまたはSubtreeをRaster imageへ変換することで、主として視覚的な結果を保持する方式である。

Complex Filter Group
→ Raster Island

Unsupported Blend Group
→ Raster Image

Text Effect
→ Rasterized Text


Raster化では、次が失われる可能性がある。

- Text editability
- Searchability
- Semantic Text
- Accessibility
- Vector scalability
- Object separation
- Animation structure
- Interactivity


Raster fallbackを単なる「対応可能」として扱わず、情報損失を伴うPlanning Decisionとして扱う。

### 6. Unsupported

Unsupportedは、現在のBackend CapabilityとOutput Profileの組合せでは、有効な表現経路が存在しない状態である。

Unsupported {
  reason,
  failed-requirements,
  attempted-representations,
  possible-alternatives
}


Unsupportedであることは、必ずしも直ちにBuild全体の失敗を意味しない。

Output Profileに応じて、次の処理候補を検討できる。

- 別Backend
- 近似
- Raster fallback
- Property省略
- 複数Artifactへの分割
- Build failure

### 7. Capability Domain

Backend Capabilityの主要Domainを次とする。

BackendCapabilities {
  document,
  geometry,
  paint,
  text,
  image,
  compositing,
  filter,
  animation,
  interaction,
  accessibility,
  color,
  resource,
  emission
}

### 8. Document Capability

Document Capabilityには次を含む。

- Page
- Slide
- Layer
- Group
- Master structure
- Section
- Metadata
- Reading order
- Hyperlink
- Annotation
- Notes


たとえばPPTXではSlideやMasterをNativeに保持できる一方、SVGでは複数PageをArtifact BundleへLoweringする必要がある。

### 9. Geometry Capability

Geometry Capabilityには次を含む。

- Path
- Primitive Shape
- Cubic Curve
- Arc
- Transform
- Clip
- Mask
- Stroke
- Dash
- Fill Rule
- Boolean Path
- Coordinate Range


Capabilityは、機能名だけでなくParameter範囲や制限値を持つ。

Path segment数上限
座標精度
Transform範囲
Clip階層上限

### 10. Paint Capability

Paint Capabilityには次を含む。

- Solid Color
- Linear Gradient
- Radial Gradient
- Conic Gradient
- Mesh Gradient
- Pattern
- Image Paint
- Opacity
- Blend Mode


Gradientでは、次の条件もCapabilityに含める。

- Stop数
- Spread Mode
- Color Interpolation Space
- Transform
- Alpha
- Precision

### 11. Text Capability

Text Capabilityには次を含む。

- Semantic Text
- Positioned Glyph Run
- Font Embedding
- Font Subsetting
- Font Substitution
- Variable Font
- Color Font
- Vertical Writing
- Bidirectional Text
- Text on Path
- Per-glyph Position
- Per-glyph Transform
- Selection
- Searchability
- Editability


Textは、視覚と意味を分離して評価する。

Visual fidelity
Semantic Text
Searchability
Editability
Accessibility
Reading order


Glyph outline化は、視覚上は等価でも、意味Textや編集可能性を失う可能性がある。

### 12. Image Capability

Image Capabilityには次を含む。

- Raster Image
- Vector Image
- Alpha
- Crop
- Transform
- Resampling
- Color Profile
- Bit Depth
- Codec
- Tiling
- Large Image Streaming


対応Codec、最大Dimension、最大Pixel数、HDR、Alpha形式等を制限値として保持する。

### 13. Compositing Capability

Compositing Capabilityには次を含む。

- Group Opacity
- Isolation
- Knockout
- Blend Mode
- Mask
- Backdrop
- Offscreen Group


Compositing Capabilityは、Raster Islandの境界判断へ直接影響する。

### 14. Filter Capability

Filter Capabilityには次を含む。

- Blur
- Shadow
- Color Matrix
- Displacement
- Morphology
- Convolution
- Lighting
- Custom Filter


各Filter単体の対応だけでなく、Filter chain全体の処理順、Color Space、中間Surfaceの扱いも検査する。

### 15. Animation Capability

Animation Capabilityには次を含む。

- Timeline
- Keyframe
- Interpolation
- Easing
- Transform Animation
- Paint Animation
- Path Animation
- Text Animation
- Filter Animation
- Nested Time
- Loop
- Event Trigger


静的Backendについては、Output Profileで指定された時刻に評価するFlattening候補を生成できる。

Backendが独自判断で最初のFrameだけを出力してはならない。

### 16. Interaction Capability

Interaction Capabilityには次を含む。

- Hyperlink
- Navigation
- Button
- Form Field
- Hover
- Input Event
- Script
- Media Control


Backendが技術的にScriptを埋め込めても、Output ProfileまたはSecurity Policyが禁止する場合がある。

### 17. Accessibility Capability

Accessibility Capabilityには次を含む。

- Semantic Role
- Reading Order
- Alt Text
- Language
- Heading Level
- Table Structure
- Caption
- Form Label
- Decorative Marking


Accessibilityは視覚的な品質とは独立した軸とする。

Raster化で見た目を保持できても、Accessibilityを失う場合は独立したLossとして記録する。

### 18. Color Capability

Color Capabilityには次を含む。

- RGB
- CMYK
- Gray
- Spot Color
- ICC Profile
- Wide Gamut
- HDR
- Alpha Model
- Rendering Intent
- Color Management


Color変換のError model、Gamut mapping、Profile埋込みをCapabilityへ含める。

### 19. Resource Capability

Resource Capabilityには次を含む。

- Font Embedding
- Image Embedding
- External Reference
- Resource Deduplication
- Streaming
- Incremental Emission
- Compression


外部Resource参照が技術的に可能でも、Output ProfileがStandalone Artifactを要求する場合には使用しない。

### 20. Emission Capability

Emission Capabilityには次を含む。

- Streaming Output
- Seek Requirement
- Incremental Update
- Atomic Finalization
- Deterministic Output
- Compression
- Encryption
- Signing


Backendがseek可能なSinkを必要とする場合、HostはTemporary seekable Artifactを経由するPlanningを選択できる。

### 21. 条件付きCapability

CapabilityはFeature parameterによって変化する。

例：

Blur radius <= 100:
Native

Blur radius > 100:
RasterFallback

Gradient stops <= 16:
Native

Gradient stops > 16:
ApproximateまたはRasterFallback


条件付きCapabilityを次の構造で表す。

CapabilityRule {
  feature-identity,
  parameter-pattern,
  conditions,
  support-kind,
  preservation,
  losses,
  limits,
  cost,
  deterministic,
  conformance-reference
}


条件は、Capability SnapshotとFeature parameterだけから決定されるpureな条件とする。

Network、現在時刻、非固定Global stateへ依存させない。

### 22. Capability Snapshot

Backend Capabilityは、Backend Version、Format Version、Device、Driver、Resource制限等によって変化する。

Planning開始時に、Capability Snapshotを固定する。

BackendCapabilitySnapshot {
  backend-identity,
  backend-version,
  format-version,
  device-identity?,
  driver-identity?,
  feature-rules,
  limits,
  snapshot-revision
}


Job中にCapabilityが変化した場合、古いPlanの意味を変更せず、Jobを停止して必要なら新しいSnapshotで再Planningする。

### 23. Preservation Axis

変換による保存性を、単一の品質Scoreへまとめない。

PreservationAxes {
  visual-fidelity,
  geometric-precision,
  semantic-text,
  searchability,
  editability,
  accessibility,
  interactivity,
  temporal-behavior,
  color-fidelity,
  resource-fidelity
}


各軸を次で評価する。

PreservationLevel =
  Exact
  | Equivalent
  | Approximate
  | Lost
  | NotApplicable


Hard Constraintに違反するLossを、別軸の高得点で相殺してはならない。

### 24. Output Profile

Output Profileは、今回の出力で何を必ず保存し、どの変換を許可し、複数候補の中で何を優先するかを表す。

OutputProfile =
  Hard Constraints
  + Permitted Transformations
  + Preferences
  + Tolerances
  + Failure Policy
  + Reporting Policy


概念的な正規形は次とする。

OutputProfile {
  identity,
  profile-version,

  hard-constraints,
  permitted-transformations,
  preferences,
  tolerances,

  text-policy,
  accessibility-policy,
  vector-policy,
  raster-policy,
  animation-policy,
  interaction-policy,
  color-policy,
  font-policy,
  resource-policy,

  performance-policy,
  artifact-size-policy,

  failure-policy,
  reporting-policy
}

### 25. Hard Constraint

Hard Constraintは、満たさない候補を無条件に除外する。

HardConstraint {
  property,
  required-level,
  scope,
  violation-policy
}


例：

- Semantic Textを保持する
- Reading Orderを保持する
- Raster fallbackを禁止する
- Spot Colorを保持する
- Animation flatteningを禁止する
- 外部Resource参照を禁止する


Hard ConstraintをPreferenceやArtifact Sizeで相殺してはならない。

### 26. Constraint Scope

Hard ConstraintはArtifact全体だけでなく、特定のSemantic RoleやResource Classへ適用できる。

ConstraintScope =
  WholeArtifact
  | Page
  | Slide
  | Layer
  | Subtree
  | SemanticRole
  | ResourceClass
  | NodeSelection


通常は、Source Node IDの大量列挙より、Semantic RoleやResource Classによる指定を優先する。

例：

本文Text:
Semantic Text必須

装飾Text:
Outline化を許可

Logo:
Vector必須

背景Effect:
Raster化を許可

### 27. Preference

Preferenceは、Hard Constraintを満たす複数候補の選択順を定める。

Preference {
  objective,
  direction,
  priority-class,
  scope
}

PreferencePriority =
  Critical
  | High
  | Normal
  | Low


例：

High:
Semantic Textを最大化

Normal:
Object Editabilityを最大化

Low:
Artifact Sizeを最小化


低PriorityのPreferenceが高PriorityのPreferenceを上回らないよう、辞書式に比較する。

### 28. Approximation Tolerance

Approximationを許可する場合、Error modelと上限を明示する。

ApproximationTolerance {
  domain,
  metric,
  maximum-error,
  aggregation-policy,
  scope
}


例：

Geometry:
最大位置誤差

Color:
知覚色差

Raster:
許容外Pixel割合

Animation:
時刻誤差と位置誤差


Toleranceを超える候補はHard Constraintと同様に除外する。

### 29. Editability Policy

編集可能性を複数軸へ分ける。

EditabilityAxes {
  object-separation,
  text-editability,
  shape-editability,
  style-editability,
  animation-editability,
  group-structure,
  master-structure,
  semantic-binding
}


Levelを次とする。

EditabilityLevel =
  NativeHighLevel
  | NativePrimitive
  | Decomposed
  | FlattenedVector
  | Rasterized
  | Lost


PPTX等で見た目を一枚画像として完全に保持しても、Object編集性はRasterizedとなる。

### 30. Text Policy
SemanticTextPolicy {
  preserve-character-sequence,
  preserve-language,
  preserve-reading-order,
  preserve-searchability,
  preserve-selection,
  preserve-editability,
  allow-glyph-outline,
  allow-rasterization
}


Text表現を次へ分類する。

1. Semantic Native Text
2. Semantic Text＋固定Glyph配置
3. Positioned Glyph Run
4. Glyph Outline＋Semantic Overlay
5. Raster Text＋Semantic Overlay
6. Glyph Outlineのみ
7. Rasterのみ


AccessibleまたはSearchable Profileでは、Semantic Textまたは検証可能なSemantic Overlayを要求する。

### 31. Accessibility Policy
AccessibilityPolicy {
  required,
  reading-order,
  semantic-roles,
  headings,
  alt-text,
  language,
  table-structure,
  form-labels,
  decorative-marking,
  fallback-policy
}


Accessible Profileでは、次をHard Constraintの基本候補とする。

- Reading Order保持
- Semantic Role保持
- Language保持
- Alt Text保持
- Table Structure保持
- Heading Level保持


Accessibility要件を満たせない場合、Accessible Strict Profileでは出力Failureとする。

### 32. Searchability Policy
SearchabilityPolicy {
  unicode-mapping-required,
  logical-order-required,
  ligature-mapping-required,
  normalization-policy
}


文字内容が内部に残っているだけでSearchableとはみなさない。

実ArtifactからUnicode Textを正しいLogical orderで抽出できることを検証可能にする。

### 33. Vector Policy
VectorPolicy {
  required-scope,
  allow-outline-conversion,
  allow-vector-decomposition,
  allow-raster-islands,
  maximum-rasterized-area,
  maximum-raster-resolution
}


Vector保持と編集性、Artifact Size、印刷品質を同一視しない。

非常に複雑なVectorはRasterより大きくなる可能性があるため、各軸を個別に評価する。

### 34. Raster Policy
RasterPolicy {
  mode,
  allowed-scopes,
  forbidden-semantic-roles,
  resolution-policy,
  color-policy,
  alpha-policy,
  maximum-area,
  maximum-pixel-count,
  text-handling,
  accessibility-handling
}


Modeを次とする。

RasterMode =
  Forbid
  | AllowDecorativeOnly
  | AllowSelectedSubtrees
  | AllowWithSemanticOverlay
  | AllowAnywhere
  | PreferRaster


Raster化する範囲を可能な限り局所化する。

### 35. Raster Island

必要なSubtreeだけをRaster化する局所領域をRaster Islandとする。

Page
├─ Native Text
├─ Native Vector
└─ Raster Island
   └─ Unsupported Filter Group


Raster Island境界は、次を考慮して決定する。

- Bounds
- Clip
- Mask
- Blend
- Backdrop
- Isolation
- Color Space
- Alpha
- Text含有
- Accessibility Role
- Resolution


Backdrop依存等によって局所Raster化できない場合、親GroupまたはPageへ範囲を拡大する。

拡大理由をOutput Decision Reportへ記録する。

### 36. Raster Resolution
RasterResolutionPolicy =
  FixedDpi
  | DevicePixelRatio
  | MaximumGeometricError
  | Adaptive
  | BackendNative


PreviewではDevice Pixel Ratio、PrintではDPI、Geometry重視では最大誤差等を使用できる。

低解像度Preview RasterをFinal Outputへ流用しない。

### 37. Animation Policy
AnimationPolicy {
  mode,
  frame-time?,
  sampling-policy?,
  preserve-timeline,
  preserve-interactivity,
  loop-policy,
  easing-tolerance
}


Modeを次とする。

AnimationOutputMode =
  Preserve
  | FlattenAtTime
  | SampleFrames
  | ConvertToVideo
  | Remove
  | ForbidLoss


静的BackendがAnimation非対応であるという理由だけで、暗黙に最初のFrameを選択してはならない。

Flatteningには明示的なFrame timeまたはSampling policyを要求する。

### 38. Interaction Policy
InteractivityPolicy {
  hyperlinks,
  navigation,
  forms,
  media-controls,
  scripts,
  hover,
  input-events,
  unsupported-action-policy
}


Security上禁止されているScript等は、Backendが対応していても使用しない。

Interactive要素を静的表示へ変換する場合は、FlatteningまたはOmissionとして報告する。

### 39. Font Policy
FontPolicy {
  embedding,
  subsetting,
  substitution,
  outlining,
  fallback,
  variable-font,
  color-font,
  licensing,
  missing-font-policy
}


Font embedding Policy：

Require
Prefer
Allow
Forbid


Font substitution Policy：

Forbid
AllowMetricCompatible
AllowDeclaredFallback
AllowAnyWithWarning


Glyph Outline Policy：

Forbid
DecorativeOnly
AllowWithSemanticOverlay
Allow
Prefer


Font substitutionによってShapingやLayoutが変わる場合は、Backend Planning中に黙って置換せず、上流Layoutの再実行を要求する。

### 40. Color Policy
ColorPolicy {
  target-color-space,
  preserve-spot-colors,
  preserve-icc-profile,
  rendering-intent,
  gamut-mapping,
  alpha-flattening,
  hdr-policy,
  color-tolerance
}


Screen、Print、Archive等で異なるProfileを使用できる。

Spot Color保持がHard Constraintの場合、RGB変換候補を除外する。

### 41. Resource Policy
ResourcePolicy {
  embedding,
  external-reference,
  deduplication,
  compression,
  provenance,
  missing-resource,
  remote-resource,
  snapshot-requirement
}


Final Artifactでは、外部ResourceのSnapshot固定や埋込みを要求できる。

PreviewではProxy Resourceや外部参照を許可できる。

### 42. Artifact Size Policy
ArtifactSizePolicy {
  hard-maximum?,
  preferred-maximum?,
  compression-preference,
  image-downsampling,
  font-subsetting,
  deduplication,
  oversized-policy
}


Hard Maximumを超える候補は除外する。

Preferred MaximumはPreferenceとして扱う。

Artifact Sizeを小さくするために、Accessibility等のHard Constraintを破ってはならない。

### 43. Performance Policy
PerformancePolicy {
  planning-budget,
  emission-budget,
  memory-budget,
  latency-preference,
  streaming-preference,
  parallelism-policy
}


PreviewではLatency、FinalではFidelityを優先できる。

Wall-clock値だけでなく、次のような構造的Costを利用する。

- Raster pixel数
- Path segment数
- Temporary memory
- Backend object数
- Embedded Resource量

### 44. Failure Policy

Output Failure Modeを次とする。

OutputFailureMode =
  Strict
  | Compatible
  | BestEffort

Strict

Hard Constraintまたは許可されていないLossがあればFailureとする。

Compatible

Hard Constraintを維持し、明示的に許可されたLowering、Approximation、Raster fallbackを使用できる。

BestEffort

可能な範囲で出力を継続する。

ただし次を緩和しない。

- Memory safety
- Artifact structural validity
- Security policy
- 明示的な禁止事項


Strict／Compatible／BestEffortは品質ProfileではなくFailure Policyである。

### 45. Output Profileの既定Preset
Preview Profile
Failure:
BestEffort

Hard:
Security
Artifact構造
Memory Budget

Preferences:
低Latency
Progressive Quality
GPU
局所Raster fallback

Final Fidelity Profile
Failure:
CompatibleまたはStrict

Hard:
Visual Tolerance
Resource固定
Color Profile
Artifact Validation

Preferences:
Visual Fidelity
Vector保持
高解像度

Editable Profile
Hard:
Text Editability
Object Separation
必要なStructure

Raster:
原則禁止

Preferences:
Native high-level object
Native Text
Group structure

Accessible Profile
Failure:
Strict

Hard:
Semantic Text
Reading Order
Language
Alt Text
Heading／Table structure


TextのOutline化またはRaster化は、検証可能なSemantic Overlayがある場合を除いて原則禁止する。

### 46. Output Profileの合成

Profileは無制限な継承階層ではなく、名前付きFragmentの合成によって構築する。

Strict Failure
+
Accessible Text
+
Editable Objects
+
PPTX Constraints
↓
Resolved Output Profile


合成規則：

Hard Constraint:
Intersection

Allowed Transformation:
Intersection

Tolerance:
より厳しい上限

Preference:
Priority付き統合

Security Prohibition:
常に優先

Failure Policy:
より厳しいPolicyまたは明示Conflict


矛盾したProfile fragmentを黙って上書きしない。

### 47. Output Profile Snapshot

Resolved Output ProfileはJob開始時にSnapshot化する。

OutputProfileSnapshot {
  profile-identity,
  profile-version,
  resolved-constraints,
  resolved-transformations,
  resolved-preferences,
  resolved-tolerances,
  reporting-policy,
  snapshot-hash
}


Job中にProfile設定が変わった場合、新しいPlanning Jobを開始する。

### 48. Backend Planning

Backend Planningは、Render IRから特定Backend向けの表現方法を確定する段階である。

Render IR
+
Document Semantics
+
Capability Snapshot
+
Output Profile Snapshot
+
Resource Snapshot
↓
Backend Planner
↓
Backend Planning IR


Planningの基本段階を次とする。

1. 入力Validation
2. Requirement抽出
3. Node Candidate生成
4. Subtree Candidate生成
5. Hard Constraint適用
6. Tolerance検査
7. Resource Planning
8. Preference比較
9. 全体整合性調整
10. Planning IR生成
11. Output Decision Report生成
12. Planning IR Validation

### 49. Planning Candidate
PlanningCandidate {
  candidate-id,
  source-nodes,
  representation-kind,
  capability-answer,
  preserved-properties,
  losses,
  error-bound,
  required-resources,
  cost,
  constraints,
  child-requirements
}


Representation Kind：

RepresentationKind =
  Native
  | Lowered
  | Approximated
  | Rasterized
  | Omitted
  | SplitRepresentation

### 50. Candidate生成

候補は次の順に段階的に生成する。

1. Native
2. Equivalent Lowering
3. Profileが許せばApproximation
4. Profileが許せば局所Raster
5. 必要ならRaster範囲を親へ拡大
6. Profileが許せばOmission


Hard Constraintを満たす最初の候補を即採用せず、少数の有力候補を比較する。

Loweringによって新たなFeature Requirementが発生する場合、再帰的に解決する。

Lowering cycleは禁止する。

### 51. Node PlanningとSubtree Planning

単純なShape、Text、Image等はNode単位でPlanningできる。

Blend、Backdrop、Mask、Isolation、Filter chain等はSubtree単位で計画する。

SubtreeCandidate {
  root,
  covered-nodes,
  compositing-context,
  external-dependencies,
  representation,
  bounds,
  semantic-projection
}


局所Planningで解決できない場合、親SubtreeへPlanning範囲を広げる。

### 52. Text Planning

Textについては、次の候補経路を検討する。

1. Semantic Native Text
2. Semantic Text＋固定Glyph位置
3. Positioned Glyph Run
4. Glyph Outline＋Semantic Overlay
5. Raster Text＋Semantic Overlay
6. Glyph Outlineのみ
7. Rasterのみ


Output ProfileのText、Searchability、Editability、Accessibility Policyによって候補を除外する。

### 53. Semantic Mapping

Visual表現とSemantic表現を分ける場合、その対応をPlanning IRへ保存する。

SemanticMapping {
  semantic-node-id,
  visual-object-ids,
  text-range?,
  reading-order,
  role,
  language,
  alt-text
}


Artifact Verificationで、実際のArtifactに対応関係が保持されていることを検査する。

### 54. Animation Planning

Animationには次の候補を生成できる。

- Native Timeline
- Backend EasingへのLowering
- Sampled Keyframes
- FlattenAtTime
- Frame Sequence
- Video Conversion


Output ProfileがPreserveを要求する場合、Flatten候補を除外する。

FlattenAtTimeでは明示時刻を必要とする。

### 55. Color Planning

Planning時に次を確定する。

- Source Color Space
- Target Color Space
- Color Transform
- ICC Profile
- Rendering Intent
- Alpha Model
- HDR／SDR
- Spot Color


Gamut clippingやApproximationがある場合、Error informationとLossを記録する。

### 56. Resource Planning

Resourceごとに次を決める。

- Embed
- Subset
- Deduplicate
- External Reference
- Transcode
- Rasterize
- Stream


概念的な構造：

BackendResourcePlan {
  resource-plan-id,
  source-resource-id,
  representation,
  encoding,
  embedding-policy,
  output-members,
  dependencies
}


Font subsetは実際に使用するGlyph集合から構築する。

Image再encodeによるLossを記録する。

### 57. Candidate除外と選択

Plannerは次の順に候補を処理する。

1. Hard Constraint違反を除外
2. Tolerance違反を除外
3. 許可されていないTransformationを除外
4. Resource／Budget違反を除外
5. Preference Priority順に比較
6. Costで比較
7. Stable Candidate IDでtie-break


Thread scheduling、Hash table順、並列完了順によってPlanが変わってはならない。

### 58. Planning Budget
PlanningBudget {
  max-candidates-per-node,
  max-subtree-expansions,
  max-global-alternatives,
  max-raster-boundary-search,
  max-resource-plans,
  max-planning-steps,
  max-memory
}


結果を次に分類する。

BackendPlanningResult =
  Planned
  | Unsupported
  | BudgetExceeded
  | Defected


BudgetExceededをBackend非対応と同一視しない。

### 59. Backend Planning IR
BackendPlan {
  plan-identity,
  source-ir-identity,
  backend-capability-snapshot-id,
  output-profile-snapshot-id,
  resource-snapshot-id,

  document-plan,
  object-plans,
  resource-plans,
  semantic-mappings,
  output-members,
  output-dependencies,
  decision-report,
  verification-requirements,
  validation-summary
}


各Object Plan：

BackendObjectPlan {
  plan-node-id,
  source-node-ids,
  representation-kind,
  backend-feature,
  children,
  transform,
  clip,
  compositing,
  resource-references,
  semantic-mapping?,
  preservation,
  losses,
  error-bound?,
  provenance
}

### 60. Backend固有Extension
BackendPlanExtension {
  backend-namespace,
  schema-version,
  data
}


Backend固有のOperator、Object、Encoding等をExtensionへ保持できる。

Hard Constraint、Loss、Approximation、Semantic MappingをExtension内部へ隠してはならない。

### 61. Planning IRの不変条件

Validated Backend Planning IRは次を満たす。

- 全Source Nodeが処理済みまたは明示Omission
- Hard Constraint違反がない
- ApproximationにError情報がある
- Raster IslandにBoundsとResolutionがある
- Resource referenceが解決済み
- Semantic Mappingが一貫する
- Output Memberが一意
- Emission順序を構築可能
- Dependency graphにCycleがない
- Backend Extension schemaが適合する


EmitterはValidated Planning IRだけを受け取る。

### 62. Emitterの責務

Emitterは、Planning IRをBackend形式へ忠実に変換する。

- Backend Object生成
- Resource埋込み
- Output Transaction
- Format Validation
- Finalization


Emitterは次を行ってはならない。

- 新しいFallbackを独自選択
- Propertyを黙って省略
- Fontを黙って代替
- Textを黙ってRaster化
- Output Profileを緩和


Planning前提が崩れた場合はPlanInvalidatedとして停止し、必要なら新Snapshotで再Planningする。

### 63. Output Decision Report

Planning結果の上位ReportをOutput Decision Reportとする。

すべてのDecisionがLossを伴うわけではないため、Loss Reportだけを上位概念にはしない。

OutputDecisionKind =
  NativeEmission
  | EquivalentLowering
  | Approximation
  | RasterFallback
  | Substitution
  | Flattening
  | Omission
  | SplitRepresentation
  | Unsupported

### 64. Output Decision
OutputDecision {
  decision-id,
  source-origins,
  source-node-ids,
  semantic-node-ids,

  decision-kind,
  selected-representation,
  backend-feature,

  preserved-properties,
  changed-properties,
  losses,
  approximation?,
  substitutions,

  alternatives,
  selection-reason,
  profile-rules,
  capability-rules,

  severity,
  approval-state,
  verification-requirements
}


利用者が、何を、なぜ、どの方法で変換したかを追跡できるようにする。

### 65. Output Loss

一つのDecisionは複数Lossを持てる。

OutputLossKind =
  VisualFidelity
  | GeometricPrecision
  | ColorFidelity
  | SemanticText
  | Searchability
  | Editability
  | Accessibility
  | Interactivity
  | TemporalBehavior
  | VectorRepresentation
  | StructuralHierarchy
  | ResourceIdentity
  | Metadata
  | Portability


程度を次で表す。

LossExtent =
  None
  | Partial
  | Complete
  | Unknown

### 66. SeverityとDisposition
LossSeverity =
  Informational
  | Notice
  | Warning
  | Error
  | Critical

LossDisposition =
  AllowSilently
  | Report
  | Warn
  | RequireExplicitApproval
  | Fail


Severityは一般的な影響を表し、Dispositionは今回のOutput Profileでの扱いを表す。

### 67. Source Origin

DecisionとLossは、元のRPX Sourceまで追跡可能にする。

OutputSourceOrigin {
  package-instance,
  module-id,
  binding-id?,
  source-range?,
  generated-origin?,
  macro-expansion-chain?,
  semantic-node-id?,
  render-node-id?
}


Provenance chain：

Artifact Object
↓
Backend Planning Node
↓
Render IR Node
↓
Layout／Visual Node
↓
RPX Binding／Source


IDEから元Sourceへ移動できるようにする。

### 68. Loss Fingerprint

承認とCI差分に、安定したLoss Fingerprintを使用する。

LossFingerprint {
  target-identity,
  semantic-origin,
  decision-kind,
  loss-kinds,
  backend-contract,
  output-profile-rule,
  relevant-parameters
}


物理行番号、一時Object ID、Process ID等には依存させない。

### 69. 明示承認

RequireExplicitApprovalとなったDecisionは、EmissionまたはCommit前に承認を必要とする。

Planning
↓
ApprovalRequired
↓
明示承認
↓
Emission


承認Scope：

ApprovalScope =
  SingleDecision
  | SameFingerprint
  | SourceSubtree
  | ArtifactMember
  | WholeBuild


既定はSingleDecisionまたはSameFingerprintとする。

Whole Buildの包括承認は、新規Lossまで承認する危険があるため慎重に扱う。

### 70. CI承認

CIでは既知LossのBaselineを利用できる。

既知Loss:
継続可能

新規Loss:
Failure

消滅したLoss:
改善としてReport


Baselineを自動更新しない。

明示的なReviewによって更新する。

### 71. Loss集約

多数の同種Lossがある場合、個別Decisionを保持したまま表示を集約する。

AggregatedLoss {
  aggregation-key,
  loss-kind,
  cause,
  count,
  affected-scopes,
  representative-origins,
  decision-references
}


Machine-readable reportでは個別Decisionを保持する。

### 72. Planning Failure
PlanningFailureKind =
  MissingCapability
  | HardConstraintUnsatisfied
  | ApproximationOutsideTolerance
  | RasterizationForbidden
  | SemanticPreservationImpossible
  | AccessibilityPreservationImpossible
  | ResourceUnavailable
  | FontResolutionFailure
  | ColorConversionImpossible
  | OutputBudgetExceeded
  | CandidateConflict
  | PlanningBudgetExceeded
  | PlanInvalidated


正規構造：

PlanningFailure {
  failure-kind,
  affected-source,
  unsatisfied-constraints,
  relevant-capabilities,
  attempted-candidates,
  rejected-candidates,
  conflict-core,
  possible-alternatives,
  source-origins
}

### 73. Conflict Core

複数Hard Constraintが両立しない場合、その原因となる条件集合を示す。

PlanningConflictCore {
  constraints,
  capabilities,
  source-subtree,
  rejected-solutions
}


数学的な大域最小性は要求しない。

Planning Budget内で縮小された、理解可能なConflict Coreを提示する。

### 74. Alternative Plan
AlternativePlan {
  required-changes,
  backend?,
  output-profile-changes?,
  output-artifacts,
  preserved-properties,
  losses,
  cost,
  approval-requirements
}


例：

- Editable PPTX＋Reference PDFへ分割
- Accessible PDFへ変更
- Raster fallbackを許可
- 別Backendを選択


Alternative Planを自動採用せず、利用者またはBuild Policyの選択を要求する。

### 75. Verification Requirement

Planning Decisionごとに、Emission後の検査条件を生成できる。

VerificationRequirement {
  decision-id,
  property,
  verification-method,
  expected-result,
  tolerance?,
  severity-on-failure
}


例：

Plan:
Semantic Textを保持

Verification:
Text抽出結果と元Textを比較

Plan:
Fontを埋め込む

Verification:
Artifact内のFont Resourceを検査

### 76. Artifact Verification Report
ArtifactVerificationReport {
  artifact-identity,
  plan-identity,
  verified-decisions,
  failed-verifications,
  unverified-properties,
  validator-identities,
  overall-status
}

VerificationStatus =
  Verified
  | VerifiedWithWarnings
  | Failed
  | Incomplete
  | ValidatorUnavailable


Strict Profileでは、必須VerificationがIncompleteまたはValidatorUnavailableなら成功扱いにしない。

### 77. Plan違反

EmitterがPlanning IRと異なるFallbackを実行した場合、それは通常のLossではなく契約違反である。

Plan:
Native Text

Actual:
Raster Text


原因候補：

- Emitter Defect
- Capability declarationの誤り
- Plan Invalidated
- Artifact Validatorによる不一致検出


BestEffortであってもEmitterがPlanを勝手に変更してはならない。

### 78. Plan Invalidated
PlanInvalidated {
  changed-assumption,
  affected-decisions,
  replanning-possible,
  partial-output-state
}


原因例：

- Font Resource消失
- Capability取消し
- Device loss
- Output Sink制約判明


Output Transactionをrollbackし、可能なら新Snapshotで再Planningする。

### 79. Machine-readable Report

構造化Reportを正本とする。

BackendOutputReport {
  report-version,
  target-identity,
  build-identity,
  plan-identity,
  backend-identity,
  output-profile-identity,

  decisions,
  aggregated-losses,
  approvals,
  planning-failures,
  verification-report,
  attachments,
  redaction-level
}


CLI、IDE、HTML、CI表示は、このReportから生成する。

### 80. Diagnostic基盤の再利用

Backend専用の独立Diagnostic Frameworkを新設せず、既存の構造化Diagnostic基盤を再利用する。

共通部分：

- Stable code
- Severity
- Source origin
- Related origins
- Structured arguments
- Notes
- Attachments
- Redaction


Backend固有部分：

- Output Decision
- Preservation Axis
- Loss
- Candidate
- Approval
- Verification

### 81. Report Budget

巨大ArtifactではReport量を制限する。

ReportBudget {
  max-decisions,
  max-losses,
  max-origins-per-aggregate,
  max-alternatives,
  max-attachments,
  max-total-bytes
}


Budget超過時にも次は保持する。

- Error／Critical
- Approval Required
- Hard Constraint Failure
- Verification Failure
- 新規Loss


Reportのtruncation自体を明示する。

### 82. Redaction
RedactionPolicy {
  paths,
  text-content,
  resource-identities,
  user-data,
  external-urls,
  secret-derived-values
}


Secret値は常にredactする。

Machine-readable reportへも平文Secretを含めない。

### 83. 増分Planning

Planning結果はSubtree単位でCacheできる。

PlanningKey {
  source-subtree-identity,
  capability-dependency-summary,
  profile-dependency-summary,
  resource-dependency-summary,
  parent-compositing-context
}


変更されたSource、Capability Rule、Profile Rule、Resourceだけに依存するPlanning Nodeを無効化する。

### 84. 局所再利用できない変更

次の変更は親、Page、Artifact全体へ影響を拡大する可能性がある。

- Backdrop Filter
- Group Blend
- Reading Order
- Font substitution
- Master Structure
- Global Color Profile
- Artifact Size Hard Limit


依存Summaryに基づき、安全側へ無効化範囲を拡張する。

### 85. PreviewとFinal

PreviewとFinalは別のOutput Profile、Capability Snapshot、Planning IRを使用する。

Preview:
BestEffort
低解像度Raster
Proxy Resource
低Latency

Final:
Strict／Compatible
Resource固定
高解像度
全体Verification


Preview PlanをFinal Planへそのまま昇格しない。

ただし、Geometry、Text shaping、Resource decode等の有効な中間Cacheは再利用できる。

### 86. Backend Conformance

Backendが宣言したCapabilityを、Conformance Testによって検証する。

ConformanceStatus =
  Declared
  | Tested
  | Certified
  | KnownLimited
  | Disabled


検査対象：

- Native Capability
- Equivalent Lowering
- Approximation Error Bound
- Semantic Text
- Accessibility
- Font Embedding
- Color
- Raster Island
- Artifact Validation


Portable BackendとNative Backendへ同じContract Testを適用できる。

Bytes完全一致ではなく、Capabilityに応じた同値条件を用いる。

### 87. Capability SchemaのVersioning
BackendCapabilitySchemaVersion
BackendPlanningIrVersion
BackendOutputReportVersion


を分離して保持する。

未知の必須FeatureはUnsupportedとして扱う。

第三者拡張はNamespace付きExtensionとして表現できる。

CapabilityExtension {
  namespace,
  schema-version,
  data
}


共通Plannerが理解すべき機能は、規範Schemaへ昇格させる。

### 88. 実装責任境界
共通Backend Package
- Capability Schema
- Output Profile
- Preservation Axis
- Loss型
- Planning IR共通型

Backend Planner
- Requirement抽出
- Candidate生成
- Constraint適用
- Preference選択
- Raster Island
- Resource Planning
- Decision Report

Backend Emitter
- Planning IRの形式固有出力
- Resource埋込み
- Output Transaction
- Format Finalization

Artifact Validator
- Plan契約と実Artifactの照合
- Semantic Text検査
- Accessibility検査
- Visual Diff
- Format Validation

Host／Build System
- Output Profile選択
- 明示承認
- Backend選択
- Attachment保存
- CI Policy
- Atomic Commit

### 89. 適合試験

少なくとも次を検査する。

- CapabilityをBooleanだけで表さない
- 条件付きCapabilityがSnapshotから決定的に評価される
- Native／Lowering／Approximate／Raster／Unsupportedを区別する
- Hard ConstraintをPreferenceで相殺しない
- Tolerance超過候補を除外する
- Semantic TextとVisual Textを区別する
- Accessibility Lossを独立して記録する
- Font substitution時に必要ならLayoutを再実行する
- Raster IslandがConservative Boundsを包含する
- Backdrop依存時にRaster範囲を拡大する
- EmitterがPlan外Fallbackを行わない
- Planning Budget超過をUnsupportedと混同しない
- Output DecisionからSourceへ逆追跡できる
- Loss Fingerprintが一時IDへ依存しない
- 新規LossがCI Baselineで検出される
- 承認されていないLossをcommitしない
- Artifact VerificationがPlanと実出力の差を検出する
- Preview PlanをFinal Planとして使用しない
- Capability／Profile変更時に必要範囲だけ再Planningする
- SecretがReportへ出力されない

### 90. 性能要件
PERF-BACKEND-01:
Page／Slide／Subtree単位でPlanning可能にする

PERF-BACKEND-02:
Capability Snapshot全体の変更で無関係Planを無効化しない

PERF-BACKEND-03:
Reporting Policyだけの変更で表現Planを再生成しない

PERF-BACKEND-04:
PreviewとFinalで有効な中間Cacheを共有可能にする

PERF-BACKEND-05:
Candidate数とSubtree探索へBudgetを設ける

PERF-BACKEND-06:
巨大Reportを集約しつつError情報を保持する

PERF-BACKEND-07:
同一Resourceを複数回変換・埋込みしない

PERF-BACKEND-08:
低解像度Preview RasterをFinalへ誤利用しない

PERF-BACKEND-09:
並列Planningで規範結果を変えない

PERF-BACKEND-10:
PlanningとEmissionを独立してCache可能にする

### 91. 最終状態

```text
OPEN-BACKEND-PROFILE-001:
RESOLVED
```


本決定により、ReciplexaのBackend出力は次の性質を持つ。

- Backend Capabilityを条件付き・型付きで表現する
- Backendの能力と利用者のOutput方針を分離する
- Hard ConstraintとPreferenceを分離する
- 視覚、Text、編集性、Accessibility等を別軸で評価する
- FallbackをEmission前にPlanningする
- Raster化をSubtree単位へ局所化する
- TextのVisual layerとSemantic layerを分離できる
- Approximationへ明示的なError modelを要求する
- Planning結果を検証可能なIRとして保持する
- Emitterによる暗黙Fallbackを禁止する
- 情報損失をSourceまで追跡する
- 明示承認とCI Baselineを利用できる
- 出力後にPlan契約を検証する
- PreviewとFinalを別Profileとして扱う
- Backend実装をConformance Testで検証する


## OPEN-ERR-DIAG-001 Failure・Defect・構造化Diagnostic・Privacy・Lifecycle
### DD-ERR-DIAG-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Program Failure
- Cancellation
- Defect
- Terminal Failure
- Diagnosticの正規Schema
- Diagnostic CodeとSeverity
- Source OriginとProvenance
- Failure Record
- Diagnostic Projection
- Primary／Cause／Suppressed
- Privacy Label
- Redaction
- Attachment
- Process間転送
- Diagnostic Lifecycle
- Incremental更新と撤回
- Grouping、Suppression、Baseline
- RendererとLocalization
- Diagnostic Store

### 0. 基本原則

Reciplexaでは、Program内部で発生した事象と、それを利用者やToolへ説明するDiagnosticを分離する。

Failure
≠
Diagnostic

Defect
≠
Diagnostic

Diagnostic
≠
表示文字列


処理の基本経路を次とする。

Program／Runtime Event
↓
Outcome Classification
↓
Failure／Defect Record
↓
Diagnostic Projection
↓
Structured Diagnostic
↓
Privacy Filter
↓
Renderer
├─ CLI
├─ IDE
├─ CI
├─ Test
└─ Machine-readable Report


Diagnosticの正本は、完成済みの表示文字列ではなく、Diagnostic Code、型付きArgument、Source Origin、Cause、Privacy Label等を持つ構造化Dataとする。

### 1. OutcomeとDiagnosticの分離

次の事象を意味論上区別する。

Program Failure:
Programの公開契約に含まれる回復可能な失敗

Cancellation:
結果が不要になったか、親Scopeから停止された状態

Defect:
成立すべき契約または不変条件の違反

Terminal Failure:
RuntimeまたはProcessの健全性を保証できない状態

Diagnostic:
事象やWarningを人間またはToolへ伝える構造化情報


Diagnosticの存在だけからOutcomeを逆算しない。

Outcome:
処理の意味上の結果

Diagnostic:
その結果や関連問題の説明

Policy:
DiagnosticをBuild Failure等として扱う規則

### 2. Program Failure

Program Failureは、型付きのDomain Dataとして表す。

例：

FileFailure {
  kind: NotFound,
  resource-id,
  operation: Read
}


Failureへ完成済みの利用者向け文面を正本として格納しない。

非推奨:
Failure {
  message: "ファイルを開けませんでした"
}


理由は次のとおりである。

- Localizationが困難になる
- Diagnostic Testが文言変更で壊れる
- Cause関係を構造化できない
- Privacy Redactionが困難になる
- CLI、IDE、CIで異なる説明を生成できない

### 3. Cancellation

Cancellationは、Program Failureとは別の制御結果とする。

Failure:
処理を実行した結果、契約内の失敗が発生した

Cancellation:
処理結果が不要になったか、
親Scopeから停止を要求された


Cancellation自体を原則としてError Diagnosticにしない。

次のような予定されたCancellationは通常表示しない。

- 古いPreview Job
- Superseded Layout Job
- Shutdown中の予定されたService停止
- Test RuntimeによるCleanup


Cancellation中に次が発生した場合はDiagnostic対象となる。

- Cleanupが完了しない
- Cancellation受理が遅延した
- Native operationが停止しない
- Output rollbackが失敗した
- Resourceが残存した

### 4. Cancellation Report

Cancellationの状態を次の構造で保持する。

CancellationReport {
  cancellation-id,
  reason,
  requested-by,
  affected-scope,
  expected,
  cleanup-status,
  late-results,
  remaining-resources,
  duration-or-budget
}


Diagnostic化は、Cancellationそのものではなく、Context、期待状態、Cleanup結果を含めて判断する。

### 5. Defect

Defectは、通常のProgram契約では成立しているべき不変条件の違反である。

例：

- Validated IRにcycleが存在した
- one-shot Continuationが二重resumeされた
- Resourceが二重releaseされた
- EmitterがBackend Planning IRに違反した
- PackageInstanceIdの再計算結果が一致しなかった


Defectを通常のfailure Eへ変換し、Applicationが通常処理として握り潰すことを認めない。

### 6. Defect Report

Defectの正規Reportを次とする。

DefectReport {
  defect-id,
  defect-code,
  violated-invariant,
  defect-scope,
  subsystem,

  origin,
  operation?,
  related-state-summary,

  recovery-status,
  runtime-trust-status,

  causes,
  suppressed,
  attachments,

  privacy-classification,
  producer
}


Defect Reportには、単なる「内部エラー」ではなく、何が成立すべきで、何が観測されたかを保持する。

### 7. Defect Scope
DefectScope =
  Task
  | Service
  | RootScope
  | RuntimeInstance
  | WorkerProcess
  | Process
  | Unknown


Defect Scopeによって、停止・隔離する範囲を決める。

Task:
当該Taskを停止可能

Service:
Service全体を停止または再起動

RuntimeInstance:
Root ScopeまたはRuntimeを破棄

Process／Unknown:
Terminal Failure候補

### 8. RecoveryとRuntime Trust
RecoveryStatus =
  Isolated
  | ScopeTerminated
  | RuntimeRestartRequired
  | ProcessRestartRequired
  | Unrecoverable
  | Unknown

RuntimeTrustStatus =
  Trusted
  | PartiallyTrusted
  | Untrusted
  | Unknown


Runtime TrustがUntrustedまたはUnknownの場合、通常Diagnostic Pipelineの継続を前提としない。

### 9. Terminal Failure

Terminal Failureは、RuntimeまたはProcessの健全性を保証できない異常である。

例：

- Memory corruptionの疑い
- Native ABI破損
- Runtime bookkeeping破損
- Rust abort
- 信頼できないWorkerのCrash


Terminal Failureを通常のProgram Failureへ降格しない。

ApplicationOutcome:
Aborted(InfrastructureAbort)


として扱う。

### 10. Emergency Diagnostic Pipeline

Terminal Failure時には、通常のPackage、Localization、Plugin、Allocator、Schedulerへ依存しない最小経路を使用する。

EmergencyDiagnostic {
  emergency-code,
  subsystem,
  isolation-boundary,
  last-known-operation,
  minimal-origin?,
  crash-artifact-reference?,
  output-transaction-state?,
  integrity-state
}


Emergency Pipelineの目的は、完全な説明ではなく、次を最低限安全に記録することである。

- どのSubsystemが停止したか
- どのProcessまたはWorkerか
- 最後に確認されたOperation
- Crash Artifactの有無
- Partial Outputを使用してよいか

### 11. Native Panic

Native panicを捕捉できた場合でも、直ちに継続可能とはみなさない。

次を検査する。

- Ownership状態が確定しているか
- Pending Callbackが残っていないか
- Completion Tokenが一意か
- Native Resourceを安全に解放できるか
- Memory破損の可能性がないか


安全に隔離できる場合：

Native Panic
→ Defect Report
→ Task／Workerの終了


安全性を確認できない場合：

Native Panic
→ Terminal Failure
→ Process／Sandbox停止

### 12. Failure Record

型付きFailureへ、Origin、Cause、Retry等の実行Contextを付加するため、次の概念構造を導入する。

FailureRecord<E> {
  failure-id,
  payload: E,

  origin?,
  operation?,
  context,

  causes,
  suppressed,

  retry-classification,
  actionability,
  privacy-classification,

  producer
}


Failure payload自体を表示情報で汚染せず、必要な実行ContextをFailure Recordで保持する。

### 13. Failure Operation

Failure発生時に試みていた操作を構造化する。

FailureOperation =
  Read
  | Write
  | Decode
  | Encode
  | Resolve
  | Validate
  | Plan
  | Emit
  | Verify
  | Finalize
  | Shutdown
  | Custom


同じPermissionDeniedでも、ReadとWriteでは説明・修正方法が異なる。

### 14. Retry Classification
RetryClass =
  Never
  | Immediate
  | AfterDelay
  | AfterResourceChange
  | AfterUserAction
  | AfterProfileChange
  | Unknown


例：

一時的Network Failure:
AfterDelay

Font不足:
AfterResourceChange

権限不足:
AfterUserAction

Backend Profile不適合:
AfterProfileChange

型不一致:
Never


Retry可能性と、自動Retryを許可するPolicyは分離する。

### 15. Actionability
Actionability =
  UserActionable
  | PackageAuthorActionable
  | AdministratorActionable
  | ToolchainAuthorActionable
  | AutomaticallyRecoverable
  | NotActionable


Defectや組織Policy上の問題について、誤って利用者へ入力変更を要求しないようにする。

### 16. CauseとSuppressed

Failure Recordは複数Causeを持てる。

PackageResolutionFailure
├─ Requirement A
└─ Requirement B


Primary Failure後に発生したCleanup Failure等はcausesではなくsuppressedへ保持する。

Primary:
Artifact Emission Failure

Suppressed:
Temporary File Cleanup Failure
Diagnostic Flush Failure


Suppressed Failureを破棄しない。

### 17. Diagnostic Projection

Failure Recordから構造化Diagnosticを生成する処理をDiagnostic Projectionとする。

DiagnosticProjection<E> {
  project:
    FailureRecord<E>
    × DiagnosticContext
    -> DiagnosticBundle
}


一つのFailureから複数Diagnosticを生成できるため、戻り値をBundleとする。

DiagnosticBundle {
  primary,
  related,
  suggestions,
  attachments
}

### 18. Projectionの責任

Diagnostic Projectionは次を決定する。

- Diagnostic Code
- Category
- Severity
- Message Template ID
- Structured Arguments
- Primary Origin
- Related Origin
- Cause relation
- Suggestion
- Attachment requirement
- Privacy Label


最終的な表示文はRendererが生成する。

### 19. Projectionの所有者

Failureの意味を最も理解しているSubsystemまたはPackageが、基本Projectionを定義する。

Font Package:
FontFailureのProjection

Package Resolver:
ResolutionFailureのProjection

Backend Planner:
PlanningFailureのProjection

Task Runtime:
TaskFailureのProjection


共通Diagnostic基盤は次を担当する。

- PrivacyとRedaction
- Grouping
- Fingerprint
- Lifecycle
- Rendererへの引渡し


Package独自のProjectionに、Privacy Policyや出力媒体の判断まで委ねない。

### 20. Reframing

上位層は、低水準Failureを利用者にとって意味のある上位Failureへ包み直せる。

FontReadFailure
↓
BackendPlanningFailure
↓
ArtifactBuildFailure


Diagnosticでは上位FailureをPrimaryとし、下位FailureをCauseとして表示できる。

Primary:
Accessible PDFを生成できません

Cause:
必要なFont Resourceを読み込めません

Underlying Cause:
Font埋込みがLicense Policyで許可されていません


Reframingしても、元Payload、Origin、Privacy Label、Cause、Attachmentを失わない。

### 21. Diagnosticの正規Schema
Diagnostic {
  diagnostic-id,
  diagnostic-code,
  schema-version,

  kind,
  severity,
  category,
  lifecycle-stage,

  message,
  arguments,

  primary-origin?,
  related-origins,
  provenance?,

  causes,
  relations,
  suggestions,

  attachments,
  privacy-labels,
  redaction-state,

  producer,
  context,
  fingerprint
}

### 22. Diagnostic IDとCode
Diagnostic ID

一回の実行中に発生した個々のDiagnostic instanceを識別する。

DiagnosticId

Diagnostic Code

Diagnosticの意味上の種類を安定して識別する。

DiagnosticCode {
  namespace,
  category,
  code
}


例：

compiler/type/TYPE-0012
runtime/task/TASK-0007
backend/text/BACKEND-0041
package/resolve/PKG-0020


表示文を改善しても、意味分類が同じならCodeを維持できる。

### 23. Diagnostic Severity
DiagnosticSeverity =
  Fatal
  | Error
  | Warning
  | Notice
  | Info
  | Hint

Fatal

Runtime、ProcessまたはArtifactの信頼性を維持できない。

Error

現在のJob、Target、Test等を成功として完了できない。

Warning

継続可能だが、品質、互換性、安全性等に問題がある可能性がある。

Notice

重要だが通常は対応必須でない情報。

Info

実行状態や結果の補助情報。

Hint

修正や改善の候補。

SeverityをOutcomeと同一視しない。

### 24. Diagnostic Category
DiagnosticCategory =
  Syntax
  | Macro
  | NameResolution
  | Type
  | Effect
  | Resource
  | Task
  | Runtime
  | Native
  | Package
  | Manifest
  | Layout
  | Render
  | Backend
  | Test
  | Codec
  | GUI
  | Security
  | Performance
  | Internal

### 25. Diagnostic Lifecycle Stage

Diagnosticが発生した処理段階を記録する。

DiagnosticLifecycleStage =
  Parse
  | Expand
  | Resolve
  | TypeCheck
  | Lower
  | Evaluate
  | Plan
  | Emit
  | Verify
  | Shutdown
  | Test


同じResource Failureでも、Plan前、Emission中、Verification時を区別できる。

### 26. MessageとArgument

Diagnosticの正本を完成済み文字列だけにしない。

DiagnosticMessage {
  template-id,
  arguments
}


例：

template-id:
backend.font.substitution

arguments:
{
  requested-font,
  replacement-font,
  affected-node-count
}


Localization Catalogが、Localeに対応する表示文を生成する。

第三者Package向けのFallbackとして、安全なPlain Text summaryを許可できるが、標準SubsystemではDiagnostic Codeと構造化Argumentを要求する。

### 27. Origin

Diagnostic Originを次の種類に分ける。

DiagnosticOrigin =
  SourceOrigin
  | GeneratedOrigin
  | IrOrigin
  | ResourceOrigin
  | RuntimeOrigin
  | NativeOrigin
  | ArtifactOrigin
  | UnknownOrigin

### 28. Source Origin
SourceOrigin {
  package-instance-id,
  module-id,
  source-resource-id,
  text-range,
  syntax-node-id?,
  binding-id?
}


File path文字列だけをOriginの正本にしない。

### 29. Generated Origin
GeneratedOrigin {
  generated-range,
  generator-origin,
  expansion-chain,
  call-site,
  definition-site
}


MacroやTemplateによるDiagnosticでは、利用者が修正しやすいCall siteをPrimaryとし、Definition siteやGenerated codeをRelated Originにできる。

### 30. IR・Runtime・Native・Artifact Origin
IrOrigin {
  ir-kind,
  ir-node-id,
  source-provenance,
  pass-identity,
  revision
}

RuntimeOrigin {
  runtime-instance-id,
  root-scope-id?,
  task-id?,
  operation-id?,
  scheduler-step?
}

NativeOrigin {
  package-implementation-id,
  adapter-id,
  abi-version,
  operation-id,
  worker-process-id?,
  native-backtrace-attachment?
}

ArtifactOrigin {
  artifact-id,
  member-id?,
  page-or-slide?,
  object-id?,
  region?,
  planning-decision-id?
}


生Memory addressや一時的なObject pointerを安定Originにしない。

### 31. Primary OriginとRelated Origin

Diagnosticは、原則として高々一つのPrimary Originを持つ。

Primary Origin:
利用者が最初に確認・修正すべき箇所


その他の場所はRelated Originとする。

- 型の定義場所
- Package Requirementの発生元
- Macro definition
- 競合する別依存
- 関連Resource

### 32. Provenance

複数IRを通過したOriginを連鎖として保持する。

Artifact Object
↓
Backend Plan Node
↓
Render IR Node
↓
Visual／Layout Node
↓
Domain IR Node
↓
RPX Binding
↓
Source Range


通常表示では必要な範囲だけ展開し、IDEや詳細Reportで完全Chainを確認できるようにする。

### 33. Privacyの基本原則

Privacyは、Renderer直前の文字列置換ではなく、Diagnostic Schemaの一部として扱う。

Diagnostic内のArgument、Origin、Attachment等に個別のPrivacy Labelを付ける。

Diagnostic Code:
Public

Package名:
ProjectInternal

Source本文:
UserContent

絶対Path:
PersonalData

Native address:
SecuritySensitive

Credential:
Secret

### 34. Privacy Class
PrivacyClass =
  Public
  | ProjectInternal
  | UserContent
  | PersonalData
  | SecuritySensitive
  | Secret

Public

公開可能な一般情報。

ProjectInternal

ProjectまたはWorkspace内部の情報。

UserContent

利用者が作成・入力した内容。

PersonalData

個人や個人環境を識別し得る情報。

SecuritySensitive

攻撃や防御回避に利用され得る内部情報。

Secret

Password、Token、Private Key等の秘密値。

Secret値はDiagnostic Dataへ原則として保存しない。

### 35. Privacy Labelの粒度

Privacy Labelを次へ付与する。

- Diagnostic Argument
- Origin
- Related Origin
- Suggestion
- Attachment
- Stack Frame
- Resource Reference
- Structured Payload Field


Diagnostic全体のPrivacy Classは、含まれる情報から計算されるSummaryである。

### 36. Privacy Labelの伝播

情報の変換後も、入力より弱いPrivacyへ暗黙に降格させない。

UserContent
+
Public
→
UserContent


SecretをHash化しただけでPublicにしない。

Privacyを弱化できるのは、認可されたSanitization operationだけとする。

- 件数へ要約
- Categoryへ一般化
- Workspace-relative Pathへ変換
- Report-local Pseudonymへ変換

### 37. Disclosure Context
DisclosureContext =
  LocalInteractive
  | LocalStored
  | WorkspaceShared
  | OrganizationInternal
  | PublicBuildLog
  | ExternalCrashReport
  | Telemetry
  | TestSnapshot
  | InterProcessTransfer


同じDiagnosticでも、Contextに応じて公開可能な情報を変える。

### 38. Redaction Action
RedactionAction =
  Preserve
  | Normalize
  | Relativize
  | Summarize
  | Pseudonymize
  | ReplaceWithCategory
  | Omit
  | DenyReport


Redaction後も、Diagnostic Code、Category、一般化された理由等を保ち、問題の意味が分かるようにする。

### 39. Path・Source・URL

Pathの表示Policy：

PathDisplayPolicy =
  Absolute
  | WorkspaceRelative
  | PackageRelative
  | DocumentRelative
  | BasenameOnly
  | Pseudonymized
  | Hidden


Source Textの表示Policy：

SourceTextPolicy =
  FullRange
  | RelevantLines
  | TokenOnly
  | ShapeOnly
  | RangeWithoutContent
  | Hidden


URLはscheme、host、path、query等へ分解し、QueryとCredentialを既定でredactする。

### 40. Stack Trace
StackTracePolicy =
  FullLocal
  | Symbolic
  | PackageOnly
  | SubsystemOnly
  | FingerprintOnly
  | Hidden


FrameごとにPrivacy Labelを持たせる。

生Memory addressをPublic Reportへ出力しない。

### 41. Attachment
DiagnosticAttachment {
  attachment-id,
  kind,
  media-type,
  size,
  content-identity,
  privacy-class,
  storage-reference,
  retention-policy
}


例：

- Scheduler Trace
- Native Crash Report
- IR fragment
- Visual Diff
- Artifact
- Test Counterexample
- Package Conflict Graph


Diagnostic本文をredactしてもAttachmentに情報が残らないよう、同等以上に厳格なPolicyを適用する。

### 42. Attachment Retention
RetentionPolicy =
  Ephemeral
  | UntilJobEnd
  | UntilSessionEnd
  | LocalPersistent
  | OrganizationRetention
  | ExplicitUserRetention


期限後はReferenceとContentをPrivacy-awareなGarbage Collectionの対象とする。

### 43. Process間転送

DiagnosticをWorkerからHostへ転送する前に、送信側でPrivacy Filterを適用する。

受信側でも再検証する。

Sender:
Schema Validation
Privacy Filter
Size Budget

Receiver:
Schema Validation
Privacy再評価
Attachment隔離
Policy再適用


Sandboxed Workerや第三者PluginのPrivacy Labelを無条件に信頼しない。

### 44. TelemetryとCrash Report

Telemetryでは、原則として次だけを扱う。

- Diagnostic Code
- Subsystem
- Toolchain Version
- Outcome Class
- Sanitized Platform Class
- Occurrence Count


Source本文、Path、Document内容、Secret、Stack Trace、Attachmentを既定で送信しない。

Crash ReportはLocal詳細版と外部送信用のredacted版を分離する。

外部送信には利用者同意または組織Policyを要求する。

### 45. Diagnostic Fingerprint

Fingerprintは、異なるRun間で同じ問題を対応付けるために使う。

DiagnosticFingerprint {
  diagnostic-code,
  semantic-origin,
  relevant-arguments,
  producer-contract-version
}


次へ依存させない。

- 表示文
- 行番号だけ
- Process ID
- Taskの一時番号
- Wall-clock時刻
- Memory address
- Secret値


Sensitive値の区別が必要な場合は、Report-localなKeyed DigestまたはPseudonymを利用する。

### 46. Diagnostic Kind

DiagnosticをLifecycleの異なる二種類へ分ける。

DiagnosticKind =
  StateDiagnostic
  | EventDiagnostic

State Diagnostic

現在のSource、IR、Artifact等の状態に対して成立する問題。

- 型不一致
- Layout overflow
- Deprecated API
- Accessibility不足


状態が変われば撤回される。

Event Diagnostic

特定の実行時点で発生した出来事。

- Export失敗
- Native Worker Crash
- Shutdown Timeout
- Test失敗


過去のExecution Historyとして保存できる。

### 47. Diagnostic Lifecycle
DiagnosticLifecycleState =
  Pending
  | Active
  | Superseded
  | Resolved
  | Retracted
  | Stale
  | Archived

Pending

解析途中の暫定Diagnostic。

Active

現在Revisionに対して確認済み。

Superseded

より新しいDiagnosticへ置き換えられた。

Resolved

新しい解析で問題が解消したことを確認した。

Retracted

Producerが誤診または暫定判断を取り消した。

Stale

依存状態が変わり、現在も正しいか確認できていない。

Archived

過去実行の履歴として保存された。

### 48. Revision Context

Diagnosticは対象RevisionとRunを持つ。

DiagnosticRevisionContext {
  source-revision?,
  module-revision?,
  ir-revision?,
  resource-snapshot?,
  runtime-profile-snapshot?,
  output-profile-snapshot?,
  build-identity?,
  execution-run-id
}


Producerごとに必要なRevisionだけを使用する。

### 49. Diagnostic Owner
DiagnosticOwner {
  producer,
  analysis-unit,
  revision,
  run-id
}


Analysis Unitの例：

- Source File
- Module
- Binding
- Package
- Test
- Page
- Slide
- Render Subtree
- Artifact Member
- Root Scope


Producerは自分が所有する範囲のDiagnosticだけを更新・撤回できる。

### 50. Diagnostic Setの更新

解析Unitごとに、Diagnosticを一件ずつ追記するのではなく、新しいDiagnostic SetとしてReconciliationする。

Previous Diagnostic Set
+
New Analysis Result
↓
Diagnostic Reconciliation


基本規則：

1. Fingerprintで旧新Diagnosticを対応付ける
2. 同じ問題なら既存IDを維持して更新
3. 意味が変わればSupersede
4. 新規問題を追加
5. 消滅した問題をResolvedまたはRetracted


解析Run自体が失敗した範囲では、旧DiagnosticをResolvedにせずStaleとする。

### 51. Diagnostic Update
DiagnosticUpdate =
  Add
  | Update
  | Supersede
  | Resolve
  | Retract
  | MarkStale
  | Archive


IDEやToolingは差分Eventとして受け取れる。

### 52. Originの移動

Source行の追加等でOriginが移動しても、Stable Syntax Node ID、Binding ID、Semantic Node ID、Incremental Parserの対応情報を使って、同じDiagnosticとして追跡する。

行番号だけをDiagnostic identityへ使用しない。

### 53. Cause・Duplicate・Consequence

Diagnostic間の関係を区別する。

DiagnosticRelation =
  CausedBy
  | ConsequenceOf
  | DuplicateOf
  | SupersededBy
  | RelatedTo
  | SuppressedBy


同じ根本原因を複数Subsystemが重複報告した場合はDuplicateOfとする。

下位Failureによって上位処理も失敗した場合はConsequenceOfとする。

上位Failureを削除せず、Group内で折りたたんで表示できるようにする。

### 54. Diagnostic Group
DiagnosticGroup {
  group-id,
  group-kind,
  primary,
  children,
  summary,
  aggregate-severity,
  outcome-relation
}

DiagnosticGroupKind =
  CauseChain
  | MultipleOrigins
  | BatchFailures
  | RepeatedIssue
  | PrimaryAndSuppressed
  | PlanningConflict
  | TestFailure


Group Severityは基本的に最大Severityを使用するが、PrimaryとSuppressedの区別を維持する。

### 55. Diagnostic Suppression
DiagnosticSuppression =
  HideFromView
  | SuppressWarning
  | AcceptKnownIssue
  | DisableCheck
  | BaselineExisting

Hide From View

表示だけを隠し、Outcomeを変更しない。

Suppress Warning

指定WarningをPolicy上無視する。

Accept Known Issue

Fingerprint単位で既知問題を承認する。

Disable Check

Optional LintやCheck自体を無効化する。

型検査、Memory safety、IR Validation等の必須検査は無効化できない。

### 56. Warning as Error

WarningをErrorへ書き換えず、PolicyでBuild Outcomeへの影響を定める。

Diagnostic:
Warning

Release Policy:
このWarning Codeが存在すれば失敗


Local IDEとCIで同じDiagnosticを異なるPolicyで扱える。

### 57. Baseline

既知DiagnosticをFingerprintの集合として保存し、現在のDiagnosticと比較する。

BaselineComparison =
  Existing
  | New
  | Resolved
  | Changed


Baselineを通常Build中に自動更新しない。

Source control上でReview可能な変更として扱う。

### 58. Renderer契約

RendererはDiagnosticの意味を変更せず、特定の媒体へ表示する。

DiagnosticRenderer {
  render:
    DiagnosticReport
    × RenderPolicy
    -> Output
}


Rendererの責務：

- Localization
- Severity表示
- Source位置表示
- Groupの展開・折りたたみ
- Suggestion表示
- Attachment Link


Rendererは禁止される処理：

- Diagnostic Code変更
- Severityの意味変更
- Cause関係の再構築
- Redacted情報の復元
- Outcomeの独自決定

### 59. Localization
MessageCatalog {
  locale,
  catalog-version,
  templates
}


Diagnostic CodeとMessage Catalog Versionを分離する。

表示文を変更しても、意味が同じならDiagnostic Codeを維持する。

Testでは通常CodeとArgumentを比較し、Renderer自身のTestでのみ表示文を比較する。

### 60. Suggestion
DiagnosticSuggestion {
  suggestion-id,
  title,
  applicability,
  source-revision?,
  edits,
  preconditions,
  risks,
  requires-approval
}

SuggestionApplicability =
  MachineApplicable
  | MaybeApplicable
  | NeedsReview
  | Informational


適用前にSource RevisionとPreconditionを再確認する。

意味変更を伴う修正をMachineApplicableにしない。

### 61. Diagnostic Store
DiagnosticStore {
  active-by-owner,
  stale,
  history,
  groups,
  attachments,
  baselines
}

Active

現在Revisionに有効。

Stale

再解析待ち。

History

過去のBuild、Test、Export等のEvent Diagnostic。

Final Build時には、Staleな必須解析結果を残さない。

### 62. Report Budget
DiagnosticBudget {
  max-active,
  max-per-code,
  max-per-origin,
  max-related-origins,
  max-cause-depth,
  max-attachments,
  max-total-bytes
}


Budget超過時にも次を保持する。

- Fatal
- Error
- Security上重要な問題
- Primary Diagnostic
- Suppressed Failureの要約
- 新規Diagnostic


省略内容をTruncated metadataとしてReportへ記録する。

### 63. Diagnostic Report
DiagnosticReport {
  report-version,
  context,
  producers,

  diagnostics,
  groups,
  aggregates,

  attachments,
  baselines?,

  redaction-level,
  disclosure-context,
  truncation-state
}


Machine-readableなDiagnostic Reportを正本とする。

CLI、IDE、CI、HTML、Test等は同じReportから表示を生成する。

### 64. Diagnostic生成Failure

Diagnostic Projection、Localization、Attachment保存等が失敗しても、元Failureを失わない。

Fallback順を次とする。

1. 完全なDiagnostic Bundle
2. Diagnostic Code＋安全な最小Argument
3. Diagnostic Code＋Origin
4. Producer＋一般Summary
5. Emergency Diagnostic


Privacy Filterが失敗した場合は、未redact情報を出力せず、最小限の安全なDiagnosticへ縮退する。

### 65. Batch Failure

複数Jobの独立Failureを一つのCause chainへ連結しない。

BatchFailureReport {
  batch-id,
  item-outcomes,
  failure-groups,
  successes,
  aggregate-summary
}


Batch Policyによって全体Outcomeを決める。

Strict:
一件でも失敗なら全体失敗

Collect:
部分成功と部分失敗を保持

BestEffort:
成功分をcommitし、失敗分をReport

### 66. DiagnosticとTest

TestではDiagnosticの表示文全体ではなく、通常次を比較する。

- Diagnostic Code
- Severity
- Category
- Primary Origin
- 重要Argument
- Cause
- Related Diagnostic


Actual／Expected値にはPrivacy Labelを伝播させる。

Secret値は表示せず、不一致であることだけを報告できる。

### 67. DiagnosticとBackend

OPEN-BACKEND-PROFILE-001の次を同じDiagnostic基盤へ接続する。

- Output Decision
- Output Loss
- Planning Failure
- Alternative Plan
- Approval Required
- Artifact Verification Failure


Loss Fingerprint、CI Baseline、明示承認は共通Fingerprint、Privacy、Lifecycle機構を利用する。

### 68. DiagnosticとPackage Resolver

Package Resolution Conflictは次を構造化Diagnosticへ変換する。

Primary:
解決できないPackage

Related:
各Version Requirementの発生元

Cause:
Singleton、ABI、Security等の制約

Suggestion:
更新、分離、Portable fallback等


依存Graph全体を無制限に表示せず、Conflict Coreと必要なAttachmentを使用する。

### 69. DiagnosticとRuntime

Runtime Diagnosticは次を扱う。

- Task Failure
- Cancellation異常
- Task leak
- Resource leak
- Shutdown timeout
- Defect
- Terminal Failure


通常の予定されたTask CancellationをWarningとして大量表示しない。

Root ScopeのPrimary／suppressed関係をDiagnostic Groupへ維持する。

### 70. 適合試験

少なくとも次を検査する。

- Failureに完成済み表示文を正本として要求しない
- 一Failureから複数Diagnosticを生成できる
- 上位Failureが下位Causeを保持できる
- Cancellationを常にErrorにしない
- Cancellation Cleanup異常を検出する
- Defect ReportがInvariantとScopeを持つ
- Runtime Trustが低い場合にEmergency Pipelineへ移る
- Diagnostic生成Failureで元Failureを失わない
- Primary／suppressed関係を維持する
- Diagnostic Codeと表示文を分離する
- OriginをSourceまで追跡できる
- Secret値をDiagnosticへ保存しない
- Public CIで絶対Pathを表示しない
- Source本文をDisclosure Contextに応じてredactする
- WorkerのPrivacy Labelを受信側で再検査する
- FingerprintへSecret値を含めない
- DiagnosticをRevision単位で撤回できる
- 解析未完了時に旧DiagnosticをResolvedにしない
- Warning as ErrorがSeverityを書き換えない
- Baselineの新規／既知／解決を区別する
- RendererがCauseやSeverityの意味を変更しない
- Report Budget超過時にもFatalとErrorを保持する

### 71. 性能要件
PERF-ERR-DIAG-01:
増分解析で対象UnitのDiagnosticだけを更新する

PERF-ERR-DIAG-02:
Fingerprint一致時にIDE Objectを再利用できる

PERF-ERR-DIAG-03:
Source行移動だけでDiagnosticを再生成しない

PERF-ERR-DIAG-04:
大量の重複Diagnosticを表示時に集約できる

PERF-ERR-DIAG-05:
Privacy FilterをArgument単位で適用できる

PERF-ERR-DIAG-06:
AttachmentをDiagnostic本体と分離して遅延取得できる

PERF-ERR-DIAG-07:
Renderer変更で解析を再実行しない

PERF-ERR-DIAG-08:
Localization変更でDiagnostic identityを変えない

PERF-ERR-DIAG-09:
Stale Diagnosticを保持しつつ画面のちらつきを抑える

PERF-ERR-DIAG-10:
Final Reportを安定した順序で生成する

### 72. 実装責任境界
Domain Package
- 型付きFailure payload
- 基本Diagnostic Projection
- Argumentの意味とPrivacy Label

Compiler／Runtime／Backend
- Failure Record作成
- OriginとProvenance
- CauseとSuppressed
- Defect Scope

共通Diagnostic基盤
- Diagnostic Schema
- Code
- Fingerprint
- Privacy伝播
- Redaction
- Lifecycle
- Grouping
- Baseline
- Diagnostic Store

Host
- Disclosure Context
- Privacy Policy
- CLI／IDE／CI Renderer
- Attachment Storage
- Crash Report
- Telemetry
- Retention

### 73. 最終状態

```text
OPEN-ERR-DIAG-001:
RESOLVED
```
## OPEN-GUI-STATE-001 Stable Key・GUI State・Reconciliation・Interaction
### DD-GUI-STATE-001 決定概要
#### 状態
Status:
RESOLVED

Scope:
- Document StateとGUI Stateの分離
- GUI Stateの分類と所有権
- Stable Key
- State Schemaと互換性
- State継承・移送・破棄
- GUI DescriptionとMounted Instance
- Reconciliation PlanningとCommit
- Component Lifecycle
- Task、Subscription、ResourceのLifetime
- Focus
- Selection
- Text Editing
- CaretとText Position
- IME Composition
- Pointer Capture
- Gesture
- Drag-and-drop
- Modal Interaction
- Privacy
- GUI Session State

### 0. 基本原則

Reciplexaでは、GUIを再評価・再構築した際に、以前のGUI要素が持っていた状態を、構造上の位置だけを根拠として再利用しない。

GUI Stateの継承は、次を検査した上で行う。

- Stable Key
- State Owner
- State Schema
- Component Lifecycle
- Document／Text Revision
- Capability境界
- Privacy境界


基本原則は次のとおりとする。

- Document StateとGUI Stateを分離する
- GUI要素の位置とidentityを分離する
- 動的CollectionではStable Keyを必須とする
- Key一致だけでStateを再利用しない
- Node移動とNode置換を区別する
- GUI更新をPlanningとCommitに分離する
- Commit前にState、Focus、Task、Resourceの整合性を検証する
- Reconciliation中の再帰的更新を禁止する
- Focus、IME、Pointer Capture等を排他的Interactionとして管理する
- ComponentのTaskとResourceをComponent Scopeへ所属させる
- Owner消失時にはCancellationとCleanupを実行する
- GUI Session StateをDocument Snapshotから分離する
- Secret、Capability、Native handleをGUI Stateとして永続化しない

### 1. Document StateとGUI State

文書の意味そのものに属するStateと、文書の表示・操作に属するStateを分離する。

Document State:
文書の規範的な内容

GUI State:
文書の表示・操作・一時Interactionに関する状態


Document Stateの例：

- ParagraphのText
- Shapeの位置
- Slideの順序
- Style
- Annotation
- Document metadata


GUI Stateの例：

- Scroll位置
- Zoom倍率
- Focus
- Caret
- Selection
- Panelの展開状態
- 入力中Draft
- Pointer gesture
- IME composition


Document Stateは、OPEN-EDT-CODEC-001のDocument Transaction、Snapshot、Transaction Logによって管理する。

GUI State Storeだけに文書の規範的な値を保持してはならない。

### 2. GUI Stateの分類

GUI Stateを次へ分類する。

GuiStateClass =
  DocumentAssociatedState
  | ViewState
  | WidgetState
  | InteractionState
  | DerivedState
  | ExternalState

#### 2.1 Document-associated State

文書に関連するが、文書の規範的内容には含まれない状態である。

例：

- 文書単位の一時解析表示
- 文書に対するEditor-local annotation
- 文書単位の一時的なNavigation state


複数View間で共有する必要があるかを明示する。

文書の意味へ影響する場合は、GUI StateではなくDocument Stateへ移す。

#### 2.2 View State

特定のView instanceに属する状態である。

- Scroll位置
- Zoom倍率
- 表示中Page
- Panel layout
- Grid表示
- Ruler表示
- View上のSelection表示


同じDocumentを複数Viewで開く場合、Viewごとに別Stateを持つ。

Document A
├─ View 1: Page 1、Zoom 100%
└─ View 2: Page 20、Zoom 200%

#### 2.3 Widget State

特定Widgetの内部制御に属する状態である。

- Text FieldのCaret
- Text Fieldの局所Selection
- Tabの選択位置
- Sliderのdrag中値
- Dropdownの開閉
- ListのKeyboard navigation位置


Widget Stateは、Widget identityとState Schemaが互換である間だけ継承する。

#### 2.4 Interaction State

進行中の操作に属する短命な状態である。

- Pointer drag
- Hover
- Press gesture
- IME composition
- Drag-and-drop
- Marquee selection
- Modal interaction


Interaction Stateは、Owner、入力所有権、対象Revision、Cancellation、Cleanupを持つ。

通常の永続GUI Stateとは分離する。

#### 2.5 Derived State

入力から再構築できる派生状態である。

- Widget bounds
- Text measurement
- Hit-test index
- Visible item range
- Render cache


Derived StateはCacheとして扱う。

入力Revision、Resource、Font、Scale等が変化した場合には破棄する。

#### 2.6 External State

GUI Runtime外部のServiceやResourceに属する状態である。

- OS Window
- File picker
- Clipboard
- GPU Resource
- Network request
- Background Task
- Native control


これらを一般的なWidget Stateとして保存しない。

Task、Capability、Resource ownershipとして管理する。

### 3. Committed StateとDraft State

Documentの正式な値と、GUIで編集中の未確定値を区別する。

Committed State:
Document Transactionへ反映済みの値

Draft State:
まだ文書へ反映されていない編集中の値


即時反映型の編集では、入力ごとにDocument Transactionを生成できる。

Input
↓
Document Transaction
↓
Document更新
↓
GUI再評価


DialogやProperty Editorでは、Draft BufferをViewまたはWidget Stateとして保持できる。

Apply:
DraftをDocument Transactionへ変換

Cancel:
Draftを破棄


DraftをDocument Stateへ暗黙反映しない。

### 4. GUI State Identity

GUI State identityを次の階層で構成する。

Document Identity
└─ View Instance Identity
   └─ Semantic Owner Identity
      └─ Widget Key Path
         └─ State Slot Identity

#### 4.1 Document Identity

Documentに関連するView Stateには、OPEN-EDT-CODEC-001で定めたDocumentIdentityを使用する。

別DocumentへStateを誤って適用してはならない。

#### 4.2 View Instance Identity

同じDocumentに対する複数Viewを区別する。

ViewInstanceId


例：

- Main Editor View
- Secondary Window
- Outline View
- Presentation Preview
- Print Preview


View Identityの再利用範囲は、View Lifecycle Policyで定める。

#### 4.3 Semantic Owner Identity

GUI要素が表示・編集する意味対象を示す。

SemanticOwnerIdentity =
  DocumentNodeId
  | SemanticNodeId
  | ApplicationEntityId
  | None


例：

Slide thumbnail:
DocumentNodeId(slide)

Paragraph editor:
DocumentNodeId(paragraph)

Accessibility heading:
SemanticNodeId(heading)


Collection内の位置が変わっても、Semantic Ownerが同じならStateを同じ対象へ追従させられる。

#### 4.4 Widget Key Path

同一Semantic Ownerの内部にある複数Widgetを区別する。

WidgetKeyPath =
  sequence<WidgetKeySegment>


例：

paragraph-node-A
├─ text-editor
├─ style-button
└─ comment-indicator


完全なKeyは、親Component identityと局所Keyを含むPathとして構築する。

#### 4.5 State Slot Identity

一つのWidgetが所有する複数Stateを区別する。

StateSlotId =
  focus
  | selection
  | caret
  | draft
  | scroll-anchor
  | custom-state


State Slotは型identityとSchema Versionを持つ。

### 5. GUI State Key

概念的な正規形を次とする。

GuiStateKey {
  state-domain,
  view-instance-id,
  semantic-owner-id?,
  widget-key-path,
  state-slot-id
}


State Storeでは次を保持する。

StoredGuiState<S> {
  key,
  state-schema,
  value: S,
  owner-lifecycle,
  revision-dependencies,
  compatibility-contract,
  retention-policy,
  privacy-labels
}


Key一致は、単なるLookup成功ではなく、同じState ownerとして継承してよい可能性があることを意味する。

最終的な継承には、追加の互換性検査を必要とする。

### 6. Explicit KeyとStructural Key

GUI要素のKeyを次へ分類する。

GuiElementKey =
  ExplicitKey
  | StructuralKey

#### 6.1 Explicit Key

Componentまたは利用者が明示的に指定する。

key = StableNodeId(item)


次ではExplicit Keyを要求する。

- 動的Collection
- 並べ替え可能Collection
- Virtualized Collection
- 条件付きで生成される反復要素
- Positionが変化する要素

#### 6.2 Structural Key

静的なGUI構造からCompilerまたはRuntimeが導出する。

Toolbar
└─ Save Button


固定構造内の一意なWidgetについて許可する。

Binding ID、Component定義上のStable Syntax identity、局所slot等から導出できる。

#### 6.3 Position Indexの禁止

動的Collectionで配列IndexだけをKeyとして使用してはならない。

変更前:
0 = A
1 = B
2 = C

Bの前へXを挿入:

0 = A
1 = X
2 = B
3 = C


IndexをKeyにすると、BのStateがXへ移る。

動的CollectionではData itemのStable identityを使用する。

### 7. Keyの型安全性

すべてのKeyを単なるTextへ変換しない。

SlideId("42")
ParagraphId("42")
WidgetKey("42")


を区別できる型付きKeyを使用する。

概念的には次とする。

WidgetKeySegment {
  namespace,
  key-kind,
  value
}


または、

GuiKey<A>


として型parameterでDomainを区別する。

### 8. Duplicate KeyとMissing Key

同じOwner範囲で、同じ完全Keyが複数回現れてはならない。

DuplicateGuiKey:
Validation Failure


宣言順で区別して処理を継続しない。

動的CollectionでStable Keyがない場合：

MissingStableKey:
Diagnostic


BestEffort Previewでは一時Keyを生成できるが、State継承を保証してはならない。

Final、Test、Strict GUI validationではFailureとする。

### 9. State Schema

GUI Stateごとに型identityとSchema Versionを持たせる。

GuiStateSchema {
  state-type-id,
  schema-version,
  compatibility-class,
  migration-contract?
}


State Compatibilityを次とする。

StateCompatibility =
  Exact
  | Migratable
  | ResetRequired
  | Forbidden

#### 9.1 Exact

同じState型・同じ意味であり、そのまま再利用できる。

#### 9.2 Migratable

State Schemaは変化したが、明示されたState Migrationで変換できる。

TextFieldState v1
↓
TextFieldState v2

#### 9.3 Reset Required

Stateの意味が変化したため、新Stateを初期化する。

#### 9.4 Forbidden

Securityや契約上、Stateを移送してはならない。

通常Text Field
→ Secret Input Field

### 10. State継承条件

State継承の正規条件を次とする。

inherit-state(old, new)
iff
  old.full-key = new.full-key
  and owner-compatible(old, new)
  and schema-compatible(old, new)
  and lifecycle-allows-inheritance(old, new)
  and revision-dependencies-valid(old, new)
  and privacy-boundary-compatible(old, new)
  and capability-boundary-compatible(old, new)
  and no-explicit-reset(new)


Key一致だけを条件にしない。

### 11. Node MoveとNode Replace

Document Nodeが別位置へ移動しても、Stable Node IDが同じなら同じ論理Nodeとして扱う。

Move:
Identity維持
State継承可能


Document Nodeが削除され、別Nodeが同じ位置へ挿入された場合は、別identityである。

Replace:
新Identity
旧Stateを暗黙継承しない


Widget kindやSecurity classが変わった場合もReplaceとして扱う。

### 12. 明示的State Transfer

別identity間で一部Stateだけを移したい場合、KeyやNode IDを使い回さず、明示的なState Transferを使用する。

GuiStateTransfer {
  source-owner,
  target-owner,
  allowed-slots,
  state-mapping,
  compatibility-proof,
  capability-proof,
  privacy-proof,
  transfer-policy
}


例：

Caret:
Text mappingがある場合に移送可能

Scroll:
意味あるAnchorがある場合に移送可能

Secret Draft:
移送禁止


同じ論理Nodeならidentityを維持する。

別Nodeなら新identityとState Transferを使用する。

### 13. GUI State所有権
GuiStateOwner =
  Document
  | View
  | ComponentInstance
  | WidgetInstance
  | InteractionSession
  | Host


OwnerごとにLifetimeを分離する。

ComponentやWidgetが消えた場合、そのOwnerに属するState、Task、Resourceをcleanupする。

### 14. GUI State Store

概念的には次の構造を持つ。

GuiStateStore {
  active-state,
  retained-state,
  pending-disposal,
  state-migrations,
  owner-index,
  revision
}

#### 14.1 Active State

現在のMounted GUIから参照されているStateである。

#### 14.2 Retained State

現在は表示されていないが、短期間の再表示やVirtualizationに備えて保持されるStateである。

#### 14.3 Pending Disposal

Ownerが消え、Task、Resource、Subscription等の終了処理中にあるStateである。

Cleanup完了前に完全削除しない。

### 15. State Retention
GuiStateRetention =
  WhileMounted
  | WhileViewAlive
  | WhileDocumentOpen
  | UntilSessionEnd
  | PersistentSession
  | Custom


例：

Hover:
WhileMounted

Virtualized Item State:
WhileViewAlive

Scroll:
WhileDocumentOpenまたはPersistentSession

Window layout:
PersistentSession


文書内容そのものをGUI StateのRetentionによって保存しない。

### 16. State BudgetとEviction
GuiStateBudget {
  max-state-count,
  max-total-bytes,
  max-retained-state,
  max-per-view,
  max-per-component,
  retention-duration?
}


Evictionの基本優先順位を次とする。

1. Derived Cache
2. Unmount済みの再構築可能State
3. 古い軽微なWidget State
4. 古いView State
5. User Draft等の非再構築State


User Draftを黙ってEvictしてはならない。

次のいずれかを必要とする。

- DocumentへCommit
- Recovery Storeへの退避
- 利用者確認
- 明示破棄

### 17. GUI Stateの永続化

一部のView StateはSession間で保存できる。

- Window配置
- Panel layout
- Zoom
- 最後に表示したPage
- Scroll Anchor


Document Snapshotへは含めず、別のGUI Session State Codecへ保存する。

GuiSessionState {
  workspace-or-document-reference,
  view-kind,
  state-entries,
  gui-schema-version,
  privacy-metadata
}


State Slotごとに永続化Policyを持たせる。

GuiStatePersistence =
  Never
  | SessionOnly
  | LocalPersistent
  | WorkspaceShared


次は原則としてNeverとする。

- IME Composition
- Pointer Capture
- Gesture
- Secret Draft
- Native Session handle
- Capability token
- Task handle

### 18. GUI DescriptionとMounted Instance

宣言的なGUI構造と、実行中のInstanceを分離する。

GuiDescription:
次に望まれるGUI構造

MountedGuiInstance:
現在実際に存在し、
State、Task、Resourceを所有する構造


概念構造：

GuiDescription {
  element-kind,
  key,
  properties,
  children,
  event-bindings,
  state-requirements
}

MountedGuiInstance {
  instance-id,
  full-key,
  element-kind,
  mounted-state,
  child-instances,
  component-scope,
  resources,
  subscriptions,
  lifecycle-state
}


GuiDescriptionは原則として副作用を持たない不変値とする。

### 19. GUI Instance Lifecycle
GuiInstanceLifecycle =
  Unmounted
  | Preparing
  | Mounted
  | Updating
  | Suspended
  | Unmounting
  | Disposed
  | Defected

#### 19.1 Unmounted

Runtime上にInstanceが存在しない。

#### 19.2 Preparing

Mountに必要なState、Resource、Child等を準備している。

通常のEvent受付はまだ行わない。

#### 19.3 Mounted

GUI tree内で有効である。

- Event受付可能
- State利用可能
- Task／Subscription稼働可能
- Focus取得可能

#### 19.4 Updating

既存Instanceを維持しつつ、新状態へ更新する準備中である。

Commitまでは旧状態を有効とする。

#### 19.5 Suspended

論理的なInstanceとStateを保持しつつ、表示や活動を一時停止している。

#### 19.6 Unmounting

Event受付を停止し、Task、Subscription、Focus、Resource等を終了している。

#### 19.7 Disposed

Cleanupが完了し、Instanceを再利用できない。

#### 19.8 Defected

Lifecycle契約またはGUI Runtime不変条件が破られた。

### 20. Reconciliation

旧Mounted Treeと新GuiDescriptionを比較し、新状態への更新計画を作る。

Old Mounted GUI
+
New GUI Description
+
Existing GUI State
↓
Reconciliation Planning
↓
Validated Reconciliation Plan
↓
Commit
↓
Lifecycle Effects


GUIを差分検出順に逐次変更してはならない。

### 21. Reconciliation Operation
GuiReconciliationOperation =
  Mount
  | Update
  | Move
  | Suspend
  | Resume
  | Unmount
  | Replace
  | Retain

#### 21.1 Mount

新Descriptionにだけ存在する要素を新規作成する。

#### 21.2 Update

完全Key、Owner、Element kind、State Schemaが互換な既存要素を更新する。

#### 21.3 Move

同じInstanceを別親または別位置へ移動する。

Stateを維持できるが、Capability、Focus、Modal、Accessibility、Environment境界を再検証する。

#### 21.4 Suspend

Instance identityとStateを維持し、活動を一時停止する。

#### 21.5 Resume

Suspended Instanceを再有効化する。

Resource、Capability、Document Nodeの有効性を再検査する。

#### 21.6 Unmount

InstanceのLifetimeを終了する。

#### 21.7 Replace

旧要素をUnmountし、別identityの新要素をMountする。

Stateを暗黙移送しない。

#### 21.8 Retain

変更なし、またはState継承だけで維持できる要素である。

### 22. Reconciliation Plan
ReconciliationPlan {
  plan-id,
  source-gui-revision,
  target-gui-revision,

  instance-matches,
  state-transfers,
  mount-operations,
  update-operations,
  move-operations,
  suspend-operations,
  resume-operations,
  unmount-operations,

  focus-plan,
  pointer-capture-plan,
  ime-plan,
  accessibility-plan,

  resource-plan,
  task-scope-plan,
  subscription-plan,

  validation-summary
}


検証済みPlanを作成するまで、現在のMounted GUIへ不可逆な変更を加えない。

### 23. Reconciliationの処理段階
1. Description Evaluation
2. Matching
3. Planning
4. Validation
5. Prepare
6. Commit
7. Post-commit Lifecycle
8. Cleanup

#### 23.1 Description Evaluation

Document State、Application State、View Stateから新しいGuiDescriptionを評価する。

原則としてpureとし、Document Transactionや外部作用を実行しない。

#### 23.2 Matching

Stable Key、Semantic Owner、Structural Keyを使って旧Instanceと新Descriptionを対応付ける。

#### 23.3 Planning

State、Task、Resource、Focus、Interaction等の移行計画を生成する。

#### 23.4 Validation

少なくとも次を検査する。

- Keyの一意性
- Owner compatibility
- State Schema compatibility
- Focus一意性
- Pointer Capture一意性
- Task Scope ownership
- Capability境界
- Privacy境界
- Accessibility tree
- Native Resource移送


失敗した場合、旧GUIを維持する。

#### 23.5 Prepare

新State、Resource、Native control等をCommit可能な状態まで準備する。

外部から観測可能な状態へまだ公開しない。

#### 23.6 Commit

Mounted tree、State ownership、Focus、Interaction routing等を論理的に一つの境界で切り替える。

#### 23.7 Post-commit Lifecycle

Commit後に次を開始する。

- Component Task
- Subscription
- Animation
- Focus request
- Accessibility notification


Commit前に長寿命副作用を開始してはならない。

#### 23.8 Cleanup

旧InstanceのTask、Resource、Subscription、Native control等を終了する。

Cleanup Failureを黙って無視しない。

### 24. Commit Barrier

Commit中に届いたEventを旧treeへ配送しないよう、ViewごとのCommit barrierを設ける。

Begin Commit
↓
Event配送を保留
↓
TreeとStateを切替
↓
Focus／Capture／IME routingを更新
↓
End Commit
↓
保留Eventを新treeで再評価


高頻度Pointer Event等は、安全な範囲でcoalesceできる。

### 25. Reconciliation中の更新

Description EvaluationまたはCommit中に、同じViewのReconciliationを同期的に再開始してはならない。

更新要求はQueueへ送り、現在Cycle終了後に処理する。

Current Reconciliation
↓
Update Request
↓
Queue
↓
Next Reconciliation Cycle


短時間の複数更新は、Transaction orderingを壊さない範囲でcoalesceできる。

### 26. Lifecycle Effect

Lifecycle Hookを制限された段階へ分類する。

GuiLifecycleHook =
  BeforePrepare
  | Prepare
  | AfterCommit
  | BeginSuspend
  | Resume
  | BeginUnmount
  | Dispose


Prepareでは、外部から観測可能な永続副作用を行ってはならない。

AfterCommitではTaskやSubscriptionを開始できる。

BeginUnmount以降では新しい通常Taskを開始できない。

Dispose後にStateやResourceへアクセスしてはならない。

### 27. Component Task Scope

Stateful Componentは専用のTask Scopeを持てる。

Component Scope
├─ Data Load Task
├─ Animation Task
├─ Subscription Task
└─ Debounce Timer


Component KeyとOwnerが維持される場合、Scopeを維持できる。

Replace、Unmount、非互換State Migrationの場合は旧Scopeを終了する。

Taskを別Componentへ暗黙移送しない。

### 28. Subscription

Subscription更新を次の順で処理する。

1. 新Subscriptionを準備
2. Commit barrier
3. Routingを新Subscriptionへ切替
4. 旧Subscriptionを解除


GenerationまたはEpochを使い、同一Eventが新旧両方へ届かないようにする。

### 29. SuspendとOffscreen Retention

Offscreen要素の扱いを次へ分類する。

OffscreenMode =
  KeepMounted
  | Suspend
  | UnmountRetainState
  | UnmountDiscardState


例：

短時間のTab切替:
Suspend

Virtualized List:
UnmountRetainState

再構築可能なDecoration:
UnmountDiscardState


Unmount後に保持するのは選択されたState Slotだけであり、Component TaskやResourceは残さない。

### 30. Error Boundary

Component Subtreeの通常Failureを隔離するため、Error Boundaryを利用できる。

GuiErrorBoundary {
  boundary-key,
  handled-failure-classes,
  fallback-description,
  reset-policy,
  escalation-policy
}


通常Failureでは、Child Scopeを終了してFallback UIをMountできる。

Defectについては、GUI Runtime全体の健全性が確認できる場合に限りSubtree隔離を許可する。

State StoreやMounted treeが不整合な場合、ViewまたはGUI Runtime全体を再構築する。

### 31. Interaction State

Focus、Selection、IME、Pointer Capture等を一般Widget Stateから分離し、Interaction Sessionとして管理する。

InteractionSession {
  interaction-id,
  interaction-kind,

  owner,
  participants,
  scope,

  input-sources,
  target-document?,
  target-revision?,
  target-elements,

  lifecycle-state,
  exclusivity-policy,
  transfer-policy,
  rebase-policy,
  cancellation-policy,

  interaction-state,
  task-scope?,
  native-session-reference?,

  privacy-labels,
  diagnostic-context
}

### 32. Interaction Lifecycle
InteractionLifecycle =
  Proposed
  | Active
  | Suspended
  | Committing
  | Cancelling
  | Completed
  | Cancelled
  | Failed
  | Defected


InteractionはOwner、入力所有権、対象Revision、Cleanupを持つ。

Owner消失時の既定はCancellationとする。

### 33. Focus

FocusをChannelへ分ける。

FocusChannel =
  Keyboard
  | Accessibility
  | Pointer
  | Programmatic
  | Custom


Focus ScopeとChannelの組合せごとに、Ownerは高々一つとする。

FocusState {
  focus-channel,
  focus-scope,
  owner-widget-key?,
  focus-path,
  revision,
  reason
}


Accessibility FocusとKeyboard Focusを同一視しない。

#### 33.1 Focus Traversal

Focus移動の順序を次の優先順位で決める。

1. Explicit traversal order
2. Semantic reading order
3. Stable structural order


画面座標だけでTraversal orderを決めない。

Cycle、Duplicate order、無効Ownerを検出する。

#### 33.2 Focus Owner消失
FocusFallbackPolicy =
  Clear
  | FocusNearestFocusableAncestor
  | FocusExplicitFallback
  | FocusDefaultInScope
  | RestorePreviousOwner


同じ画面位置の別Widgetへ暗黙移動しない。

#### 33.3 FocusとDraft Commit

Focus移動とDraftのDocument commitを常に不可分にしない。

入力値が不正でもFocusを移動できる場合がある。

Validation Errorは別途Diagnosticとして表示する。

### 34. Selection

Selectionを型で区別する。

SelectionKind =
  DocumentNodeSelection
  | TextRangeSelection
  | TableCellSelection
  | TimelineRangeSelection
  | ViewRegionSelection
  | Custom


SelectionはOwner View、対象Document、対象Revisionを持つ。

#### 34.1 Document Node Selection

Stable Node IDで対象を追跡する。

Node移動ではSelectionを維持できる。

Node削除時には次のPolicyを適用する。

SelectionDeletionPolicy =
  ClearDeletedEntries
  | ClearAll
  | SelectParent
  | SelectNearestSibling
  | PreserveTombstone

#### 34.2 Multi-selection
MultiSelection {
  selected-items,
  primary-item?,
  anchor-item?,
  ordering
}


集合だけでなく、Primary item、Anchor、選択順を保持できる。

### 35. Text Position

v1のText Positionを次とする。

TextPosition {
  text-node-id,
  text-revision,
  scalar-offset,
  affinity,
  semantic-anchor?
}

TextAffinity =
  Before
  | After


規範位置にはUnicode scalar offsetを用い、UI操作ではGrapheme boundaryへ変換する。

Byte offsetやPlatform依存のcode-unit offsetを正本にしない。

### 36. Text Selection
TextSelection {
  anchor-position,
  focus-position,
  preferred-column?,
  direction,
  source-revision
}


AnchorとFocusを分け、Selection方向を保持する。

Selectionを単なる昇順Rangeへ正規化して方向情報を失わない。

### 37. Text Position Rebase

Document Transaction後にText Positionを変換する。

TextPositionTransform {
  source-text-revision,
  target-text-revision,
  operation-mappings
}


基本規則：

挿入点より前:
位置維持

挿入点より後:
挿入長だけ移動

挿入点と同じ:
Affinityに従う

削除範囲内:
規定境界へcollapse

DeletedPositionPolicy =
  CollapseToStart
  | CollapseToEnd
  | UseSemanticAnchor


Rebase不能な場合、古い位置へcommitしてはならない。

RebaseFailurePolicy =
  Clear
  | CollapseToNodeBoundary
  | UseSemanticAnchor
  | RequireUserDecision

### 38. Text Editing Session
TextEditingSession {
  interaction-id,
  owner-widget-key,
  text-node-id,

  base-document-revision,
  base-text-revision,

  selection,
  draft-mode,
  pending-operations,
  validation-state,

  ime-session?,
  commit-policy,
  cancellation-policy
}


Editing modeを次へ分ける。

Immediate:
入力ごとにDocument Transaction

Draft:
一時Bufferへ入力し、Apply時にTransaction


Draft中にDocumentが外部更新された場合、Base RevisionとのConflictを検出する。

Draftで現在値を無条件上書きしない。

### 39. IME Composition

IMEの未確定TextをDocumentへ直ちに保存せず、Interaction Stateとして保持する。

ImeCompositionSession {
  interaction-id,
  owner-widget-key,
  text-node-id,

  base-text-revision,
  replacement-range,
  marked-text,
  selected-range-in-marked-text,

  native-session-reference,
  privacy-class,
  lifecycle-state
}


処理：

IME update:
Overlayを更新

IME commit:
Document Text Transactionを生成

IME cancel:
Overlayを破棄


外部Transaction後にReplacement Rangeを安全にRebaseできない場合、Compositionをcancelする。

#### 39.1 Secret Input

Secret InputのStateには次を要求する。

- Marked TextをDiagnosticへ出さない
- GUI Session Stateへ保存しない
- 通常Text FieldへState Transferしない
- Unmount時にcancelする
- Secret DraftをDebug dumpへ含めない

### 40. Pointer Capture
PointerCapture {
  pointer-id,
  owner-widget-key,
  interaction-id,
  view-id,
  capture-generation,
  capture-mode
}

PointerCaptureMode =
  Explicit
  | GestureOwned
  | DragAndDropOwned
  | HostOwned


同じViewとPointerについて、Capture Ownerは高々一つとする。

Owner Unmount、Pointer up、Device disconnect、Modal変更等で解放する。

別Widgetへ暗黙移送しない。

### 41. Gesture

競合するGesture候補をGesture Arenaで管理する。

GestureArena {
  arena-id,
  input-streams,
  candidates,
  resolution-state,
  winner?,
  cancellation-results
}

GestureCandidateState =
  Possible
  | Accepted
  | Rejected
  | Won
  | Cancelled
  | Completed


排他的Gestureが確定した場合、競合候補をcancelする。

同時成立可能なGestureには明示的なCompatibility contractを要求する。

Gesture解決結果をCandidate登録順やHashMap順に依存させない。

### 42. Drag-and-drop
DragSession {
  interaction-id,
  source-owner,
  payload-descriptors,
  allowed-operations,
  current-target?,
  preview?,
  capability-grants,
  lifecycle-state
}


Payloadを次のDescriptorで表す。

DragPayloadDescriptor {
  media-type,
  size?,
  privacy-label,
  transfer-mode,
  provider
}

TransferMode =
  InMemory
  | Stream
  | ResourceReference
  | HostMediated


Capability tokenやSecretを一般Payloadとして転送しない。

#### 42.1 Drop Target
DropTargetContract {
  accepted-media-types,
  allowed-operations,
  required-capabilities,
  validation,
  preview-policy
}


Drop結果がDocumentへ影響する場合、Document Transactionとして原子的にcommitする。

Drop validation
↓
Resource preparation
↓
Document Transaction
↓
Commit
↓
Drag completed


Transaction失敗時にDrop成功として扱わない。

### 43. Modal Interaction
ModalScope {
  modal-id,
  owner,
  blocked-scopes,
  allowed-interactions,
  focus-boundary,
  dismissal-policy
}


Modal開始時：

- FocusをModal内へ移す
- 外部Pointer interactionをcancelまたは保留
- Accessibility traversalをModal内へ制限


Modal終了時：

- 以前のFocus ownerが有効なら復元
- 無効ならFocus Fallback Policy


Modal外へFocusやPointer Captureが漏れないようにする。

### 44. Interaction Cancellation
InteractionCancellationReason =
  OwnerUnmounted
  | TargetDeleted
  | RevisionConflict
  | ReconciliationReplacedOwner
  | ModalBoundaryChanged
  | DeviceDisconnected
  | HostRequested
  | UserCancelled
  | CapabilityRevoked
  | Timeout
  | DefectIsolation


Cancellation順序：

1. 新規Input受付停止
2. Native Sessionへcancel通知
3. 入力Ownership解放
4. Draft／Overlay処理
5. Child Task停止
6. Cleanup
7. Cancelled Outcome確定


予定されたCancellationをError Diagnosticとして大量表示しない。

### 45. Reconciliation時のInteraction継承

Interactionを継承できる条件を次とする。

- 完全Keyが一致
- Owner contractが互換
- State Schemaが互換
- Target Nodeが維持される
- Revision mappingが存在する
- Capability境界が互換
- Privacy classが弱まらない
- Interaction kindが継続可能


Updateや同一View内Moveでは継承できる場合がある。

Replaceでは原則としてcancelする。

別Ownerへの移送には明示的なInteractionTransferを要求する。

InteractionTransfer {
  source-owner,
  target-owner,
  interaction-kind,
  state-mapping,
  revision-mapping,
  capability-proof,
  privacy-proof
}

### 46. Optimistic Interaction

Document TransactionのCommit前に一時的な結果を表示できる。

OptimisticInteractionState {
  base-revision,
  preview-delta,
  pending-transaction,
  rollback-plan
}


Commit成功後に正式状態へ移行する。

失敗時にはPreviewをrollbackする。

Rollback不能な一時変更を、GUI外部へ不可逆に公開してはならない。

### 47. InteractionとUndo

Interaction完了時に生成されるTransactionへIntent metadataを付ける。

- Move Shape
- Type Text
- Resize Object
- Drop Image


多数の中間更新を一つのUndo Groupへまとめられる。

Cancelled Interactionを通常Undo historyへ追加しない。

すでに中間Transactionをcommitしていた場合は、Inverse TransactionまたはHead移動によって元へ戻す。

### 48. Privacy

GUI StateとInteraction StateにはPrivacy Labelを付ける。

対象例：

- Draft Text
- Search query
- IME Marked Text
- Secret Selection
- Drag payload
- Clipboard content
- Navigation history


次を禁止する。

- GUI State Store全体の無制限Debug dump
- Secret StateのSession保存
- Secret DraftのDiagnostic表示
- Drag payload内容の無断Telemetry送信
- Native Session handleのSerialization

### 49. Budget
GUI State Budget
GuiStateBudget {
  max-state-count,
  max-total-bytes,
  max-retained-state,
  max-per-view,
  max-per-component
}

Reconciliation Budget
ReconciliationBudget {
  max-elements,
  max-depth,
  max-key-comparisons,
  max-state-migrations,
  max-mounts,
  max-unmounts,
  max-native-operations,
  max-preparation-memory,
  max-commit-steps
}

Interaction Budget
InteractionBudget {
  max-active-interactions,
  max-buffered-events,
  max-draft-bytes,
  max-drag-payload-size,
  max-gesture-candidates,
  max-session-duration?,
  max-rebase-steps
}


Budget超過時には、半端なGUI状態をcommitせず、安全にRollbackまたはCancelする。

### 50. Failure
GUI State Failure
GuiStateFailureKind =
  DuplicateKey
  | MissingStableKey
  | StateSchemaMismatch
  | StateMigrationFailed
  | InvalidStateTransfer
  | OwnerMissing
  | FocusConflict
  | StaleTextPosition
  | RetentionBudgetExceeded
  | CleanupFailed
  | ForbiddenPersistence
  | PrivacyViolation

Reconciliation Failure
GuiReconciliationFailureKind =
  DuplicateKey
  | MissingRequiredKey
  | OwnerMismatch
  | StateSchemaMismatch
  | StateMigrationFailed
  | InvalidMove
  | CapabilityBoundaryViolation
  | FocusConflict
  | PointerCaptureConflict
  | ImeTransferInvalid
  | PrepareFailed
  | CommitFailed
  | CleanupFailed
  | BudgetExceeded

Interaction Failure
InteractionFailureKind =
  OwnerUnavailable
  | TargetUnavailable
  | RevisionConflict
  | RebaseFailed
  | FocusConflict
  | PointerCaptureConflict
  | GestureResolutionFailed
  | DropRejected
  | CapabilityDenied
  | NativeSessionFailed
  | CommitFailed
  | CleanupFailed
  | BudgetExceeded


これらはOPEN-ERR-DIAG-001に従って構造化Diagnosticへ変換する。

### 51. Defect

次は通常FailureではなくDefectとする。

- 一Pointerに複数の排他的Capture Ownerが存在する
- Focus Scope内にFocus Ownerが複数存在する
- Disposed InstanceへEventが配送された
- State Schema不一致を検査せず再利用した
- Completed Interactionを再度commitした
- Rebase失敗後に古いText Revisionへcommitした
- Secret Stateを通常Fieldへ移送した
- Modal境界外へFocusが漏れた
- Validation済みPlanに重複Ownershipがある
- Component Taskが別Ownerへ暗黙移送された


Defect Scopeに応じて、Interaction、Component Subtree、View、GUI Runtimeを隔離する。

### 52. Diagnostic

GUI Diagnosticの例：

Primary:
動的Collection内でGUI Keyが重複しています

Related:
最初の要素の生成元
二つ目の要素の生成元

Suggestion:
Document NodeのStable IDをKeyとして指定してください


State Reset、Draft喪失、Focus異常、Interaction Cancellation、Cleanup Failure等の利用者影響がある事象をDiagnosticへ投影する。

Derived Cacheの通常破棄を逐一Warningにしない。

### 53. Reconciliation Report
GuiReconciliationReport {
  plan-id,
  source-revision,
  target-revision,

  retained-instances,
  mounted-instances,
  moved-instances,
  suspended-instances,
  unmounted-instances,
  reset-states,
  migrated-states,

  focus-change,
  cancelled-interactions,
  task-scope-changes,
  cleanup-results,

  diagnostics,
  outcome
}

### 54. 実装責任境界
Document Model
- Stable Node ID
- Document Revision
- Text Revision
- Transaction Position Mapping
- Selection対象の意味identity

GUI Runtime
- State Store
- Key Validation
- Reconciliation
- Component Lifecycle
- Focus
- Interaction ownership
- Retention
- Cleanup

Component
- Stable local Key
- State Schema
- State Migration
- Lifecycle requirement
- Interaction contract

Host
- Native Window
- IME
- Pointer device
- Clipboard
- Drag-and-drop service
- Accessibility service
- Session State storage

Diagnostic基盤
- GUI Failure／Defectの構造化Report
- Privacy
- Fingerprint
- Attachment
- Lifecycle

### 55. 適合試験

少なくとも次を検査する。

- List先頭へItemを追加しても既存ItemのStateが維持される
- 並べ替え後もStable IDへStateが追従する
- 同じ位置の別Nodeへ旧Stateが移らない
- Duplicate KeyをCommit前に検出する
- Missing Keyを動的Collectionで報告する
- Widget kind変更で非互換Stateをresetする
- Secret Fieldへ通常Text Stateを移送しない
- Node MoveでStateとSelectionを維持できる
- Node Replaceで旧Interactionをcancelする
- Validation Failure時に旧GUIを維持する
- Mount TaskがCommit前に開始されない
- Commit中のEventが旧treeへ誤配送されない
- Component Unmount時にTaskとSubscriptionを終了する
- SuspendでStateを保持し可視Resourceを解放できる
- Resume時にCapabilityとRevisionを再検査する
- Focus OwnerがScope内で一つである
- Accessibility FocusとKeyboard Focusを別管理できる
- Text Insert後にCaretをAffinityどおりRebaseする
- Text Revision不一致時に古い位置へcommitしない
- IME commitがDocument Transactionになる
- IME Owner削除時にCompositionをcancelする
- Pointer Capture conflictを検出する
- Gesture競合を決定的に解決する
- Drop失敗時にDocumentを部分変更しない
- Modal中にFocusが外部へ漏れない
- Draftを黙ってEvictしない
- Secret StateをSession保存しない
- Budget超過時に半端な状態をcommitしない

### 56. 性能要件
PERF-GUI-STATE-01:
変更SubtreeだけをReconcileできる

PERF-GUI-STATE-02:
Stable Key一致時にStateとInstanceを再利用できる

PERF-GUI-STATE-03:
位置変更だけでWidget Stateを再生成しない

PERF-GUI-STATE-04:
Virtualized ItemのStateをData identityで復元できる

PERF-GUI-STATE-05:
Derived Stateを優先的にEvictできる

PERF-GUI-STATE-06:
複数更新要求を安全にcoalesceできる

PERF-GUI-STATE-07:
Commit barrierを短時間に限定する

PERF-GUI-STATE-08:
Interaction Eventを必要な範囲でcoalesceできる

PERF-GUI-STATE-09:
State Store全体を走査せずOwner単位でcleanupできる

PERF-GUI-STATE-10:
Reconciliation結果をThread schedulingへ依存させない

### 57. 最終状態

```text
OPEN-GUI-STATE-001:
RESOLVED
```

本決定により、ReciplexaのGUI State基盤は次の性質を持つ。

- Document StateとGUI Stateを分離する
- Stable KeyによってState ownerを追跡する
- 動的Collectionで位置Indexをidentityにしない
- Key、Owner、Schema、Revision、Privacyを検査してStateを継承する
- Node MoveとNode Replaceを区別する
- 別identity間のState移送を明示的に行う
- GUI更新をPlanning、Validation、Commitへ分離する
- Commit前に長寿命副作用を開始しない
- Component TaskとResourceをLifecycleへ所属させる
- SuspendとUnmountを区別する
- Focus、Selection、IME、Pointer CaptureをInteraction Sessionとして管理する
- Text PositionをTransaction後にRebaseする
- Gesture競合を決定的に解決する
- Drag-and-drop結果をDocument Transactionとしてcommitする
- Secretや一時Interactionを永続化しない
- Reconciliation Failure時に旧GUIを維持する
- Cleanup不能やRuntime不整合をDefectとして扱う

## OPEN-NATIVE-PKG-001 Native Package・実装選定・Lifecycle・Fallback・適合性
### DD-NATIVE-PKG-001 決定文書
#### 状態
Status:
RESOLVED

Scope:
- Native packageの独立単位
- Portable Contractとの関係
- Contract PackageとImplementation Package
- 実装選定Policy
- Native Implementation Descriptor
- Package・Contract・ABI Version
- Artifact discovery
- Candidate validation
- ABI negotiation
- Transactional initialization
- Native instance lifecycle
- Sharing・Multiplicity・Lifetime
- Capability・Executor・Callback
- Shutdown・Quarantine
- Failure分類
- Fallback
- Conformance Test
- Differential Test
- Conformance evidence

### 0. 基本原則

Reciplexaでは、Native implementationを言語の公開意味の正本にはしない。

公開される契約の正本は、RPXで記述されたPortable Contractとする。

Portable Contract:
型
Effect
Failure
Ownership
Resource lifetime
Concurrency
Cancellation
Determinism
Capability requirement
Semantic invariant


Native implementationは、このContractを満たす交換可能なImplementation Providerとして扱う。

Application／Library
↓
Portable Contract
↓
Runtime Profileによる実装選定
├─ Portable implementation
├─ Native implementation
├─ Worker implementation
└─ Host service


Applicationは原則として、特定のRust Library、C Library、GPU Driver、OS APIへ直接依存しない。

Native ABIや外部Library固有の型はTrusted Adapterの内部へ閉じ込める。

Rustでも、外部関数のABIはexternで明示され、extern "C"等による外部境界が区別される。また、symbol exportやcalling conventionもABI設計上の明示事項である。これは、言語内部の型・呼出し規約をそのまま外部契約にしないという本設計と整合する。

### 1. Native packageの役割

Native packageは、次のいずれかを目的として導入される。

- Portable実装より高い性能を得る
- OS固有機能を利用する
- GPUや専用Deviceを利用する
- 既存Native Libraryを再利用する
- Codec、Font、暗号等の専門実装を利用する
- Host serviceと接続する


しかし、Nativeであるという理由だけで複数機能を一つの巨大Packageへまとめてはならない。

たとえば次の構造は原則として採用しない。

reciplexa-native-runtime
├─ Font
├─ Image Codec
├─ Video
├─ PDF
├─ GPU
├─ Filesystem
├─ Clipboard
└─ Crypto


この構造では、更新、Capability、ABI、Failure、Isolation、Shutdownの境界が大きすぎる。

Native implementationは、独立して選択、Version解決、初期化、停止、隔離、Quarantineできる単位へ分割する。

### 2. Native packageの独立単位

次のいずれかが異なる場合、別Native packageとすることを原則とする。

- 提供するContract family
- 外部Library
- Native ABI
- Capability requirement
- Isolation requirement
- Failure domain
- Version lifecycle
- Platform／Device requirement
- Instance multiplicity
- Resource ownership
- Shutdown method
- Portable fallbackの有無


独立Packageの候補例は次である。

- Font shaping
- Image decode
- Video encode
- PDF emission
- GPU rendering
- Filesystem watcher
- OS clipboard
- Cryptographic signing


一方、同じContract family、ABI、Capability、Isolation、Runtime instanceを共有する操作は、一つのImplementation Packageへまとめられる。

例としてFont engineが同一instanceを共有する場合、次は一つのPackageに含められる。

Font Native Implementation
├─ Font face load
├─ Glyph metadata
├─ Text shaping
└─ Glyph outline


Native関数一つごとにPackageを分割する必要はない。

### 3. Contract PackageとImplementation Package

基本構成は次とする。

Contract Package
├─ Public type
├─ Effect contract
├─ Failure contract
├─ Ownership contract
├─ Semantic invariant
└─ Conformance suite

Implementation Packages
├─ Portable implementation
├─ Native implementation A
├─ Native implementation B
└─ Worker／Host implementation


Contract PackageとImplementation Packageは原則として分離する。

これにより、次が可能になる。

- Native実装だけを独立更新する
- Platform別実装を追加する
- Portable-only環境からNative依存を除く
- Native packageを別Processへ隔離する
- 複数実装をDifferential Testする
- 公開型をNative ABIから独立させる


十分に小さく、一体性が高く、独立選択や独立更新の利益がない場合に限って、同一Package内の別Implementation memberを許可できる。

### 4. Portable Contract

Native implementationが満たすContractを、概念的に次のように定義する。

NativeContract {
  contract-identity,
  contract-version,

  input-types,
  output-types,

  effects,
  failures,

  determinism,
  replayability,

  concurrency,
  reentrancy,
  cancellation,

  ownership,
  resource-lifetime,

  capability-requirements,
  semantic-invariants,

  equivalence-contract
}


Native implementationは、公開型、正常結果だけでなく、次についてもContractへ適合しなければならない。

- Failureの種類
- Cancellation
- Partial result
- Resource cleanup
- Callback回数
- Thread affinity
- Reentrancy
- Determinism
- Budget
- Observable Effect


Status codeだけが同じでも、Native Resourceを漏らした実装はContract適合とはみなさない。

### 5. Portable implementationの必要性

すべてのContractにPortable実装を要求することはしない。

OS固有機能、Hardware固有機能、外部Service等では、Portable実装が意味を持たない場合があるためである。

Contractごとに次を宣言する。

PortabilityClass =
  PortableRequired
  | PortablePreferred
  | NativePermitted
  | NativeRequired
  | HostSpecific

Portable Required

言語の基礎意味、再現性、検証にPortable実装が必要である。

Portable Preferred

Portable実装を持つことを強く推奨するが、特定構成では省略できる。

Native Permitted

PortableとNativeの双方を許可する。

Native Required

Native機能なしではContractを実現できない。

Host Specific

Clipboard、Window、Accessibility Service等、Hostが実装を提供する。

Portable実装がある場合でも、名目的な低品質Fallbackを用意するだけでは不十分である。Portable実装も同じContractと品質条件を満たさなければならない。

### 6. 実装選定Policy

TargetまたはRuntime Profileは、Contractごとに実装選定Policyを指定する。

ImplementationSelectionPolicy =
  RequireNative
  | PreferNative
  | PreferPortable
  | PortableOnly
  | RequireSpecificImplementation

Require Native

適合するNative実装がなければ、Profile Resolutionまたは起動を失敗させる。

Portable実装へ暗黙Fallbackしない。

Prefer Native

適合するNative実装を優先し、利用不能な場合はPortable実装を候補にできる。

Native実行途中のFailureからFallbackできるかは、別のFallback Policyで判断する。

Prefer Portable

移植性、決定性、Test容易性を優先する。

Native実装は、Portableで必要条件を満たせない場合に利用する。

Portable Only

Native codeを候補から除外する。

高信頼Sandbox、Test Runtime、Compatibility検査等で使用する。

Require Specific Implementation

特定Implementation Identityを要求する。

次のような限定用途で使用する。

- 再現可能Build
- ABI検証
- Differential Test
- 認証済み暗号実装


通常Applicationが具体実装へ直接依存する既定にはしない。

### 7. 実装選定の二段階化

Implementation選定を、Build段階とRuntime段階へ分ける。

Build／Package Resolution
↓
利用可能なImplementation PackageとArtifactを固定

Runtime Profile Resolution
↓
現在のPlatform、Device、Capabilityからinstanceを選択

Build段階
- Implementation Packageを解決
- Package Release Identityを固定
- Binary Artifactを準備
- Descriptorを検証
- Native ABI metadataを検査
- Lockfileへ記録

Runtime段階
- Device capabilityを検査
- Capability grantを検査
- Isolation条件を検査
- Conformance evidenceを検査
- Multiplicityを検査
- Resource Budgetを割り当て
- 最終Implementationを選択


Job開始時に選択結果をSnapshot化し、Job中に暗黙変更しない。

### 8. Implementation Selection Snapshot
ImplementationSelectionSnapshot {
  contract-identity,
  contract-version,

  selected-implementation,
  package-instance-id,
  implementation-version,
  native-abi-version?,

  platform-identity,
  device-identity?,

  capability-view,
  isolation-boundary,
  conformance-status,

  fallback-plan,
  snapshot-revision
}


同じJob中にDevice負荷、Filesystem順、Library discovery順によって選択を変えてはならない。

### 9. Versionの分離

次を別々のVersionとして扱う。

PackageVersion
ContractVersion
ImplementationVersion
NativeAbiVersion
AdapterContractVersion
BinaryArtifactIdentity

Package Version

Implementation Packageの配布Releaseを表す。

Contract Version

RPX上の型、Effect、Failure、Ownership、意味契約のVersionを表す。

Implementation Version

Native実装固有のVersionを表す。

Native ABI Version

Native Binaryとの低水準Binary interfaceを表す。

Adapter Contract Version

Trusted AdapterとReciplexa Runtime間のprotocol Versionを表す。

Binary Artifact Identity

具体的にBuildされたBinary contentを表す。

Package Versionが同じでも、Platform、Compiler、Linker、依存Libraryが異なれば別Binary Artifactになり得る。

### 10. Native Implementation Descriptor

Native implementationは、静的なDescriptorを持つ。

NativeImplementationDescriptor {
  implementation-identity,
  package-release-identity,

  provided-contracts,
  implementation-version,

  native-abi-range,
  adapter-contract-range,

  platform-requirements,
  device-requirements,
  capability-requirements,

  isolation-support,
  instance-multiplicity,
  instance-lifetime,
  instance-sharing,

  executor-requirement,

  determinism,
  replayability,
  equivalence-class,
  fallback-support,

  resource-budgets,
  failure-contract,
  cancellation-contract,
  shutdown-contract,

  conformance-status,
  known-limitations,
  provenance
}


Descriptorは、可能な限り実行Binaryから分離した静的Dataとして配布する。

これにより、BinaryをProcessへ読み込む前に候補を検証できる。

Binary load後にRuntime Handshake Descriptorを取得する場合、静的Descriptorとの一致を検証する。

### 11. Registryの分離

Registryを次の三層へ分ける。

Implementation Catalog
Validated Candidate Registry
Runtime Instance Registry

Implementation Catalog

利用可能かもしれない実装の静的情報を保持する。

Binary codeはまだ実行しない。

Validated Candidate Registry

Descriptor、Content identity、Platform、Trust等を検証済みの候補を保持する。

可能な限り、まだBinaryを読み込まない。

Runtime Instance Registry

ABI negotiationと初期化を完了し、Ready状態にあるinstanceだけを保持する。

Applicationへ公開できるのは、原則としてRuntime Instance Registry内のReady instanceだけである。

### 12. Artifact discovery

Native Artifactは、次の確定されたSourceから発見する。

- Lockfileで固定されたArtifact
- Content-addressed Package Store
- Toolchain built-in Artifact
- 明示されたWorkspace implementation
- Host-managed implementation


次のような偶然の探索結果だけで採用してはならない。

- Current directoryに同名Libraryがある
- Environment pathで最初に見つかった
- Filesystem列挙で先に現れた


DiscoveryとBinary loadを分離する。

Artifact Discovery
↓
Descriptor Validation
↓
Candidate Selection
↓
Binary Load

### 13. Descriptor Validation

Candidate登録前に次を検証する。

- Package Release Identity
- Implementation Identity
- Binary Content Identity
- Contract Identity
- Contract Version range
- Native ABI range
- Adapter Contract range
- Platform／Architecture
- Required system library
- Capability requirement
- Isolation requirement
- Instance multiplicity
- Shutdown contract
- Conformance evidence


結果を次へ分類する。

DescriptorValidation =
  Valid
  | ValidWithRestrictions
  | Unsupported
  | Untrusted
  | Invalid


ValidWithRestrictionsでは、利用可能条件をCandidate Selectionへ伝える。

- 特定Deviceだけで利用可能
- Sandbox内だけで利用可能
- Previewだけで利用可能
- 最大入力sizeに制限

### 14. Candidate ordering

複数候補は、次の辞書式優先順位で選択する。

1. Hard Constraintを満たす
2. Lockfile／Build Planの固定選択
3. Selection Policyへの適合
4. Security／Isolation Policyへの適合
5. Conformance Level
6. 健全な既存instanceの再利用可能性
7. Platform／Device適合度
8. Resource Cost
9. Stable Implementation Identity順


登録順、Library load順、Filesystem順、Thread完了順へ依存させない。

### 15. Binary LoadとIsolation

BinaryをProcessへ読み込むのは、Candidate選定後に限定する。

Validated Candidate
↓
Isolation Boundary準備
↓
Binary Load
↓
Handshake


Isolation方式を次へ分類する。

NativeIsolation =
  InProcess
  | DedicatedThread
  | WorkerProcess
  | SandboxProcess
  | HostService


Native implementationは対応可能方式と最低条件を宣言する。

IsolationRequirement =
  InProcessAllowed
  | DedicatedThreadRequired
  | ProcessIsolationRequired
  | SandboxRequired
  | HostManaged


未信頼入力を扱うCodec、Memory safety保証が弱いLibrary、強制停止が必要な機能は、ProcessまたはSandbox隔離を優先する。

### 16. ABI Negotiation

Binary load後、Trusted AdapterとHandshakeを行う。

AbiNegotiationInput {
  runtime-adapter-range,
  required-contracts,
  platform-identity,
  pointer-width,
  endianness,
  calling-convention,
  ownership-contract,
  callback-contract
}


結果：

AbiNegotiationResult =
  Compatible
  | VersionMismatch
  | MissingContract
  | OwnershipMismatch
  | CallbackMismatch
  | PlatformMismatch
  | InvalidHandshake


整数Version範囲が重なるだけでは互換とみなさない。

次も照合する。

- Data layout
- Alignment
- Ownership transfer
- Failure representation
- Callback lifetime
- Thread affinity
- Cancellation protocol


成功した結果はNegotiatedAbiSnapshotとしてinstance lifetime中固定する。

### 17. Transactional Initialization

Native instanceの初期化を原子的な状態遷移として扱う。

Validated Candidate
+
Runtime Profile View
+
Capability Grants
+
Budget Slice
↓
Initialization Transaction
↓
Ready Instance


基本段階：

1. Isolation Boundary準備
2. Binary Load
3. ABI Negotiation
4. Capability View作成
5. Native State確保
6. Callback Table登録
7. Health Check
8. Contract Probe
9. Ready Commit


途中Failureでは、取得済みResourceを逆順にrollbackする。

初期化中のOperationを次へ分類する。

InitializationOperationClass =
  PureValidation
  | ReversibleAllocation
  | CompensatableRegistration
  | IrreversibleExternalEffect


不可逆な外部作用を通常初期化に含めることは避ける。

### 18. Native Instance Lifecycle
NativeInstanceLifecycle =
  Discovered
  | Validated
  | Loading
  | Negotiating
  | Initializing
  | Ready
  | Degraded
  | Quiescing
  | ShuttingDown
  | Disposed
  | Quarantined
  | Defected


Runtime Instance Registryへ公開できるのは、原則としてReadyだけである。

Degradedは、Descriptorで限定動作が明示されている場合だけ利用できる。

QuarantinedになったinstanceまたはImplementationを、新しいJobへ選択しない。

### 19. Instance所有権・Lifetime・共有

Instance ownerを次へ分類する。

NativeInstanceOwner =
  Operation
  | Job
  | Document
  | RootScope
  | RuntimeInstance
  | ProcessHost
  | HostService


Instance lifetimeも同じ境界に関連付ける。

NativeInstanceLifetime =
  Operation
  | Job
  | Document
  | RootScope
  | RuntimeInstance
  | Process
  | Host


共有契約を次へ分類する。

InstanceSharing =
  Exclusive
  | SharedReadOnly
  | SharedSerialized
  | SharedConcurrent
  | Partitioned


単なるthread-safeという一語だけで、共有、Partition、Reentrancy、Orderingを表現しない。

### 20. Instance Multiplicity
InstanceMultiplicity =
  Multiple
  | SingletonPerDevice
  | SingletonPerRuntime
  | SingletonPerProcess
  | HostSingleton


例：

Image decoder:
Multiple

GPU context:
SingletonPerDevice候補

GUI event loop:
SingletonPerProcess候補

OS clipboard:
HostSingleton


Singleton制約と複数Version要求が競合する場合、Package ResolutionまたはRuntime Profile ResolutionでFailureにする。

### 21. Capability View

Native instanceへRuntime Profile全体を渡さない。

必要なCapabilityだけを縮小して渡す。

NativeCapabilityView {
  resource-capabilities,
  operation-capabilities,
  lifetime,
  owner-runtime,
  revocation-channel
}


強いCapabilityを持つinstanceを、異なる権限範囲のJobへ無条件共有しない。

Capabilityがrevokeされた場合、新Operationを拒否し、必要に応じてQuiescingへ移行する。

### 22. Executor Affinity
NativeExecutorRequirement =
  Any
  | Main
  | DedicatedSerial
  | DedicatedParallel
  | DeviceExecutor
  | HostManaged


Native OperationとCallbackは、Negotiated contractで指定されたExecutorへ従う。

誤ったExecutor上のCallbackはContract Violationとして扱う。

### 23. OperationとCallback

進行中OperationをRuntime側でも追跡する。

NativeOperationRecord {
  operation-id,
  instance-id,
  owner-scope,
  input-snapshot-id,
  cancellation-state,
  completion-state,
  callback-generation,
  resource-borrows
}


Callback受信時に次を検査する。

- Instanceが有効
- Callback generationが一致
- Owner Scopeが生存
- Operationが未完了
- Executorが正しい
- Payload validatorに適合


Completion Callbackの二重到着はDefectとする。

Shutdown後のLate Callbackは、Callbackの性質に応じて記録、無視、Quarantine、Defectへ分類する。

### 24. Shutdown

Shutdownを次の順序で行う。

1. Registryから新規取得を停止
2. 新しいOperationを拒否
3. Active operationをQuiesceまたはCancel
4. Completionを待つ
5. Callback routingを閉鎖
6. Native Resourceを解放
7. Worker／Threadを停止
8. Disposedへ移行


概念Contract：

NativeShutdownContract {
  begin-quiesce,
  cancel-operation,
  await-drain,
  unregister-callbacks,
  release-resources,
  finalize,
  deadlines,
  forced-termination
}


In-process Native codeを安全に強制停止できない場合がある。

強制停止が要求されるContractであれば、WorkerまたはSandbox隔離を要求する。

### 25. Dynamic LoadとUnload

Native implementationのLinkageを次へ分類する。

NativeLinkage =
  Static
  | Dynamic
  | WorkerExecutable
  | HostService


Instance disposalとLibrary imageの物理Unloadを分離する。

Instance disposal:
必須

Dynamic library unload:
安全が証明された場合だけ


v1では、Dynamic LibraryをProcess終了まで保持することを許可する。

Quarantined Binaryは新instance生成へ使用しない。

### 26. Hot Replacement

v1では、実行中instanceの一般的なHot replacementを採用しない。

理由：

- ABIが変わり得る
- Native Stateを移送できない
- 旧Callbackが残る
- Function pointerが旧Binaryを参照する
- Jobの再現性が崩れる


既存instanceは旧実装で終了まで動作するか、安全上必要ならCancelする。

新しいImplementationは、新しいJobまたはRuntimeから選択する。

### 27. Failure分類

Native実行結果を次へ分類する。

NativeExecutionOutcome<E, A> =
  Completed(A)
  | ProgramFailed(E)
  | Cancelled
  | ImplementationUnavailable
  | ImplementationFailed
  | Defected
  | Aborted

Program Failure

Portable Contractに定義された通常Failureである。

Native implementationが壊れたことを意味しない。

Implementation Unavailable

処理開始前に利用条件を満たせない。

PreferNativeではPortable候補を選択できる。

Implementation Failure

Runtime Trustを維持したまま発生した実装・環境側のFailureである。

CleanupとRollbackが完了すればFallback候補となる。

Contract Violation

Native実装がPortable Contractへ違反した。

Defectとして扱い、Quarantineへ接続する。

Terminal Failure

Memory safetyやRuntime健全性を保証できない。

同一Process内でPortableへFallbackしない。

### 28. Fallbackの基本原則

Fallbackは単なる例外処理ではない。

次をすべて満たす場合だけ許可する。

- Runtime Trustが維持されている
- Native instanceのCleanupが完了している
- 旧出力のRollbackが完了している
- 同じ入力を再現できる
- Operationが再実行可能
- 代替Implementationが利用可能
- 外部Effectを重複させない
- Fallback Budget内である


Job途中の内部Stateを別Implementationへ暗黙移送しない。

基本となるFallback方式は、現在Jobをrollbackし、固定Input Snapshotから別Implementationで最初から再実行する方式である。

### 29. Operation Replayability
OperationReplayability =
  PureReplayable
  | SnapshotReplayable
  | IdempotentEffect
  | TransactionallyReplayable
  | RequiresIdempotencyKey
  | NonReplayable
  | Unknown


NonReplayableまたはUnknownでは自動Fallbackしない。

Artifact emissionのようにOutput Transactionで完全Rollbackできる処理は、TransactionallyReplayableにできる。

Network送信や不可逆なDevice操作は、明示的なIdempotencyまたはTransaction契約がなければ再実行しない。

### 30. Fallback Input Snapshot

Fallback時に結果へ影響する入力を固定する。

NativeJobInputSnapshot {
  typed-input,
  document-revision?,
  resource-snapshot,

  clock-snapshot?,
  random-seed-or-stream?,

  locale?,
  font-environment?,
  color-environment?,

  output-profile?,
  capability-view,
  relevant-configuration
}


Fallback中にClock、Random、Resource、Font、Output Profileを最新状態へ変更しない。

代替Implementationへ、元Implementationより強いCapabilityを自動付与しない。

### 31. RollbackとCleanup

次を分離して検査する。

Instance Cleanup:
Native Resource、Callback、Taskの終了

Output Rollback:
File、Artifact、Upload等の部分結果破棄


Rollback状態：

RollbackStatus =
  NotNeeded
  | Completed
  | PartiallyCompleted
  | Failed
  | Unknown


自動Fallbackを許可するのは原則としてNotNeededまたはCompletedの場合だけとする。

### 32. Fallback Plan

Fallback経路をJob開始前に構築する。

NativeFallbackPlan {
  primary-implementation,
  alternative-implementations,

  eligible-failure-classes,
  replayability-requirements,

  snapshot-requirements,
  rollback-contract,
  cleanup-contract,

  restart-scope,
  maximum-attempts,
  reporting-policy
}


Fallback試行数にはBudgetを設ける。

FallbackBudget {
  max-attempts,
  max-restarts,
  max-total-cost,
  max-worker-restarts
}


Implementationを順に無制限試行しない。

### 33. Fallbackの再開始範囲
FallbackExecution =
  RestartOperation
  | RestartJob
  | RestartRootScope
  | RestartWorker
  | RestartProcess
  | NoFallback


同じOperationの途中から継続するCheckpointHandoffは、Portable Checkpoint SchemaがContract化されている特殊な場合に限る。

Worker crashではWorkerを破棄し、Jobを先頭から再実行できるか判断する。

Terminal FailureではProcessまたはHost境界からの再開始を検討する。

### 34. Quarantine

Contract Violation、Crash、Ownership不一致等が発生したImplementationまたはinstanceをQuarantineできる。

QuarantineRecord {
  implementation-identity,
  instance-id?,
  reason,
  scope,
  runtime-revision,
  evidence,
  retry-policy
}

QuarantineScope =
  Instance
  | Device
  | RuntimeInstance
  | Process
  | PackageRelease
  | BinaryArtifact


Memory safety疑い、Contract Violation、Binary改ざんでは、同じRuntime内の自動再試行を行わない。

一時的Device Failureでは、Host Policyにより別Deviceや新instanceを試せる。

### 35. 同値性

PortableとNativeの結果を比較する契約を次へ分類する。

ConformanceEquivalence =
  Exact
  | CanonicalEquivalent
  | ToleranceBounded
  | ObservationalEquivalent
  | TraceEquivalent
  | ProfileDependent

Exact

型付き結果が完全に一致する。

Canonical Equivalent

正規化後の結果が一致する。

Tolerance Bounded

規定Metricと最大誤差の範囲で一致する。

Observational Equivalent

上位Contractから観測可能な値、Failure、Effectが一致する。

Trace Equivalent

Callback、Resource、Cancellation等のProtocol traceが許容Partial order内で一致する。

### 36. FailureとCancellationの互換性

Conformanceは正常値だけを対象としない。

次を比較する。

- Failure kind
- Failure payloadの規範部分
- Retry classification
- Partial result
- Cleanup状態
- Cancellation point
- Callback closure


Native OS Errorは、AdapterによってContract-defined Failureへ変換する。

OS固有詳細はRelated Diagnosticとして保持できるが、公開Failure型へ漏らさない。

### 37. Conformance Test

Conformance Testは、ImplementationをContractへ対して検査する。

NativeConformanceSuite {
  contract-identity,
  contract-version,
  suite-version,

  deterministic-cases,
  property-tests,
  failure-cases,
  cancellation-cases,
  ownership-cases,
  concurrency-cases,
  lifecycle-cases,
  budget-cases,
  isolation-cases,

  required-platform-classes,
  equivalence-contract,
  reporting-policy
}


Portable実装が存在しなくても実行できる。

### 38. Differential Test

Differential Testは複数Implementationを互いに比較する。

同じInput Corpus
├─ Portable implementation
├─ Native implementation A
└─ Native implementation B
↓
Contract-defined equivalenceで比較


両Implementationが同じ誤りを持つ可能性があるため、Differential TestだけでContract適合を証明しない。

Conformance TestとDifferential Testを併用する。

### 39. Test対象

少なくとも次を検査する。

- 正常結果
- 境界値
- 不正入力
- Failure分類
- Cancellation
- Resource cleanup
- Ownership
- Callback回数
- Reentrancy
- Concurrency
- Shutdown race
- Budget
- Worker crash
- Panic containment


Property Testの反例はOPEN-TST-001のShrinkingとAttachment機構へ接続する。

### 40. Differential Test Corpus
DifferentialCorpus =
  NormativeCases
  | GeneratedCases
  | RegressionCases
  | RealWorldSanitizedCases
  | BoundaryCases
  | AdversarialCases


User Contentや機密文書を無断でCorpusへ保存しない。

実データから反例を採取する場合、Privacy-safeな最小反例へ縮小するか、明示Retention Policyを適用する。

### 41. Known Limitation

Contractの一部条件だけを満たすImplementationは、制限を構造化して宣言する。

KnownLimitation {
  limitation-id,
  affected-operations,
  conditions,
  behavior,
  fallback-availability,
  diagnostic-code,
  conformance-evidence
}


Known LimitationはDocumentationだけでなく、Candidate SelectionのHard Constraintとして使用する。

制限条件外でImplementationを選択してはならない。

### 42. Conformance Status
ConformanceStatus =
  Declared
  | Tested
  | Certified
  | KnownLimited
  | Regressed
  | Disabled


Certifiedは特定のCertification Policyを満たすことを意味し、絶対的な正しさを意味しない。

RegressedまたはDisabledのImplementationを通常の新Jobへ選択しない。

### 43. Conformance Evidence
ConformanceEvidence {
  implementation-identity,
  binary-artifact-identity,

  contract-version,
  suite-version,

  platform-identity,
  device-class?,
  runtime-adapter-version,

  result,
  executed-cases,
  skipped-cases,

  report-identity
}


EvidenceはBinary Artifact、Platform、Suite Versionへ結び付ける。

Sourceが同じでもBinaryが変われば、原則としてEvidenceを再評価する。

### 44. Runtime Contract Monitor

実行時にも低CostのContract検査を行える。

- Completionが高々一回
- Enum tagが有効
- Sizeが宣言上限内
- Callback generationが一致
- Owner Scopeが生存
- Resource countがBudget内


Runtime Monitorが違反を検出した場合、Program FailureではなくDefectとして扱う。

高Costな検査はTestまたはDebug Profileに限定できる。

### 45. Diagnostic

Native package関連のFailure、Fallback、Quarantine、Contract Violationは、OPEN-ERR-DIAG-001の構造化Diagnosticへ変換する。

例：

Primary:
Native image decoderを初期化できません

Cause:
Native ABIがRuntime Adapterと適合しません

Required:
Adapter Contract 3以上4未満

Provided:
Adapter Contract 2

Alternative:
Portable decoderを利用可能


Fallbackで最終的に成功しても、Contract Violation、Worker crash、Quarantineを隠してはならない。

Native backtrace、Binary Path、Device identityにはPrivacy Policyを適用する。

### 46. Reproducible Build

再現可能Buildでは、Implementation選択を原則として固定する。

- Implementation Identity
- Binary Artifact Identity
- ABI Version
- Adapter Version
- Selection Snapshot


固定Implementationが利用不能なら、Strict ProfileではFailureとする。

Fallbackを許可する場合は、実際に選ばれたImplementationとFallback経路をBuild Reportへ記録し、Build identityへ反映する。

### 47. 適合試験

少なくとも次を検査する。

- 未選択BinaryをProcessへ読み込まない
- Descriptor不正候補を登録しない
- Binary identity不一致を検出する
- ABI不適合時にinstance初期化を開始しない
- Ready前のinstanceをRegistryへ公開しない
- 初期化途中FailureでResourceをrollbackする
- Singletonの二重作成を防ぐ
- Capabilityの強いinstanceを無制限共有しない
- Owner Scope終了後にHandleを使用できない
- Completion Callbackの二重到着をDefectとする
- Shutdown後のLate Callbackを検出する
- Quarantined Implementationを再選択しない
- Dynamic instance disposalとBinary unloadを区別する
- Contract上のInvalidInputで不要なFallbackを行わない
- RequireNativeがPortableへFallbackしない
- Runtime Trust喪失後に同一ProcessでFallbackしない
- Cleanup未完了時に別実装を開始しない
- Rollback失敗時にJobを再実行しない
- Clock、Random、Resource SnapshotをFallbackで維持する
- NonReplayable Effectを自動再実行しない
- Known Limitation外でImplementationを選択しない
- Failure、Cancellation、Resource lifecycleをConformance対象にする
- Differential TestだけでCertifiedにしない
- Binary更新後に旧Evidenceを無条件再利用しない
- Regression検出後に新Jobの候補から除外する

### 48. 実装責任境界
Contract Package
- Public Contract
- Failure
- Ownership
- Equivalence
- Replayability
- Conformance Suite

Implementation Package
- Native Binary
- Static Descriptor
- Adapter metadata
- Known Limitation
- Conformance metadata

Trusted Adapter
- ABI negotiation
- ForeignValue validation
- Ownership変換
- Callback validation
- Panic／Failure変換

Runtime Profile Resolver
- Candidate Selection
- Capability View
- Isolation Plan
- Fallback Plan
- Implementation Snapshot

Native Runtime
- Transactional Initialization
- Instance Registry
- Operation Tracking
- Callback Routing
- Shutdown
- Quarantine

Test Runtime
- Conformance Test
- Differential Test
- Resource Trace
- Regression Detection

### 49. 性能要件
PERF-NATIVE-PKG-01:
未選択Binaryを読み込まない

PERF-NATIVE-PKG-02:
Validated DescriptorをCache可能にする

PERF-NATIVE-PKG-03:
健全なinstanceを安全条件下で再利用できる

PERF-NATIVE-PKG-04:
CapabilityとDevice変更時にSelectionだけを再評価できる

PERF-NATIVE-PKG-05:
Contract不変のImplementation更新でRPX型検査を再利用できる

PERF-NATIVE-PKG-06:
Worker通信をBatch化可能にする

PERF-NATIVE-PKG-07:
Runtime Contract MonitorのCostをProfile別に制御できる

PERF-NATIVE-PKG-08:
Conformance EvidenceをBinary Artifact単位で再利用できる

PERF-NATIVE-PKG-09:
Fallback試行へBudgetを設定する

PERF-NATIVE-PKG-10:
Candidate SelectionをFilesystem順やThread順へ依存させない

### 50. 最終状態

```text
OPEN-NATIVE-PKG-001:
RESOLVED
```

本決定により、ReciplexaのNative package基盤は次の性質を持つ。

- Portable Contractを公開意味の正本とする
- Native implementationを交換可能なProviderとして扱う
- ContractとImplementationを原則として別Packageにする
- Native機能を選択・更新・隔離可能な単位へ分割する
- Package、Contract、Implementation、ABI Versionを分離する
- 実装選定をBuild段階とRuntime段階へ分離する
- 未選択Binaryを不必要に読み込まない
- Descriptor、Binary、ABI、Capabilityを段階的に検証する
- Native instanceをTransactionalに初期化する
- Ready前のinstanceを公開しない
- Instanceに明確なOwner、Lifetime、Sharing契約を持たせる
- CallbackとOperationをRuntime側でも追跡する
- ShutdownをQuiesce、Drain、Callback解除、Resource解放へ分ける
- Contract ViolationをDefectとしてQuarantineする
- Job途中の暗黙実装切替を原則禁止する
- Fallback前にInput、Cleanup、Rollback、Runtime Trustを検証する
- Conformance TestとDifferential Testを分離する
- Failure、Cancellation、Ownership、Resource lifecycleも検証する
- EvidenceをBinary、Platform、Suite Versionへ関連付ける
- Native実装を利用できない環境でも、Policyに従ってPortable実装を選択できる

