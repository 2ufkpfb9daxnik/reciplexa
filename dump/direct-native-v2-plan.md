# Direct Native v2

**Status:** complete (DN2-0〜DN2-7); portable fallback remains `OPEN-NATIVE-PKG-001`  
**Current shipping model:** Direct Native v2 (std packages)  
**Execution order:** [`active-roadmap.md`](active-roadmap.md) Step 7 item 2（package local/offline slice の後）
**Normative anchors:** `specification.md` Compiler-native package (~L17260), `KER-001`, `OPEN-NATIVE-PKG-001`

## Hybrid Native v1 (reference / differential only)

Standard packages **shipped** Hybrid Native v1 until DN2-6:

- authors still `import` package APIs (`.rpi` + package path)
- hot-domain bodies were supplied as synthesized RPX (`*_source()` helpers remain for differential tests)
- Rust builtins / eval bridges implemented the observable behavior

DN2-6 removed `DomainNativeModule.synthetic_source`; production load uses DN2 stubs + typed callables.

## Direct Native v2 (current for std packages)

DN2 replaces synthetic RPX elaboration with a typed Rust callable bound from the package public name or BindingId.

## Historical Hybrid v1 path (differential harness)

```text
author import
→ LocalPackageIndex / DomainNativeRegistry
→ hybrid_reference_source (differential only)
→ RPX parse + elaborate
→ Core typecheck
→ eval_expr + primitive_env
→ RuntimeValue
→ graphics / math / document bridge
```

`BindingId` は source resolve 用であり、永続 ABI identity ではない。DN2 の registry key は
`package/module/export` とし、本番 `eval_package_entry_main` / `typecheck_with_packages`
は stub を language binder で解決したうえで compilation `BindingId` を割り当て、
`DomainNativeBindMap::op_for` で typed callable を引ける。

## Architecture

DN2 の native export は少なくとも次を持つ。

- package/module/export の完全修飾名
- 引数型、戻り型、effect 契約
- Rust callable
- Failure の構造化変換
- portable / Hybrid reference との対応情報

Domain package export を言語 builtinへ昇格させない。author API は引き続き `.rpi` と
通常の `import` であり、raw FFI や Rust symbol を公開しない。

型検査は synthetic body の推論へ依存せず、native export の宣言型を import environment
へ供給する。評価時だけ native callable を解決する設計にはしない。

## Ordered implementation slices

### DN2-0 — dual-path infrastructure

- `DomainNativeModule` に typed export table を追加する。
- export lookup を `package/module/export` で行い、bind 時に `BindingId` へ対応付ける。
- native module は body elaborationを省略し、宣言型をCore typecheckへ渡す。
- eval に domain native callable 経路を追加する。kernel `BuiltinOp` とは分離する。
- module単位で Hybrid `synthetic_source` とDN2を切替可能にする。
- 同じconsumerを両経路で評価する differential harness を追加する。

### DN2-1 — pilot `length/units`

- 最小でpureな `length/units` を最初のtyped callableへ移す。
- Hybrid v1とDN2の `RuntimeValue`、型、Failureを比較する。
- 差分試験と対象crate gateがgreenになるまで `synthetic_source` を削除しない。

### DN2-2 — pure constructors

順序:

1. `color/srgb`
2. `graphics/color`
3. `graphics/page`
4. `graphics/shapes`

product Vertical Slice のscene、保存、再読込、PDF/SVG E2Eを維持する。

### DN2-3 — math

`math/atoms` から各math moduleを順次移す。record shape、layout style、Failureを
Hybrid referenceと比較する。OpenType MATHの本格実装はDN2の範囲外。

### DN2-4 — japanese

`classes`、`linebreak`、`kihon`、`markup` の順を基本とする。
`japanese/linebreak` はsynthetic RPX内の規則を単純移植せず、
`reciplexa-std` の正本へ直接接続し、既存parity testを差分試験へ拡張する。
font-backed JLReqはDN2の範囲外。

### DN2-5 — `document/page`

doc record constructorsをtyped callableへ移し、live-layout / document bridgeとの
観測同値性を固定する。doc-*作者編集はこのmilestoneの非対象。

### DN2-6 — remove Hybrid standard bodies — **done**

- 全標準packageでtyped callable経路を使用する。
- `DomainNativeModule.synthetic_source` を削除した。
- standard moduleについてsynthetic RPX fallbackへ暗黙に戻らないことを試験した（`dn2_std_modules_use_stub_not_synthetic_fallback`）。
- 現状正本を更新した。

### DN2-7 — portable fallback contract — **done**

- 非native packageの通常 `.rpx` loadを試験で固定した。
- 標準packageのportable fallback、ABI/version negotiation、native unavailable時の挙動を
  `OPEN-NATIVE-PKG-001` としてコードと現状正本に明示追跡した（実装は defer）。
- 標準moduleが disk `.rpx` へ暗黙 fallback しないことを DN2-6 と合わせて固定した。

## First implementation increment

最初のatomic incrementは **DN2-0 + `length/units` dual-path** とする。
`japanese/linebreak` や product-criticalな `graphics/shapes` から開始しない。

### Complete only when all of the following hold

1. Standard package bodies are **not** supplied via `synthetic_source`.
2. Package public names or BindingIds resolve to **typed Rust callables**.
3. Authors still use the same public `import` API (`.rpi` + package path).
4. Observable behavior matches Hybrid v1 or a portable reference, proven by differential conformance tests.
5. All quality gates are green on the same HEAD.

Until DN2-6, keep incremental migration **OPEN**. DN2-6/7 plus production stub bind,
isolated hybrid overrides, type/Failure differential coverage, and workspace gates
complete the required **shipping** milestone; portable fallback stays `OPEN-NATIVE-PKG-001` only.

### DN2-8 — observational hardening (before product typesetting)

Shipping DN2 is in production. Before font-backed JLReq / OpenType MATH, close the
remaining observational gaps without treating 300 exports as a coverage target:

1. Differential eval on deeper existing examples (`pkg_graphics_shapes`, `pkg_columns`, `pkg_math`).
2. `japanese/linebreak` dynamic exports: predicates and `classify-sample` where Hybrid and std agree; `break-between` **kind** on Hybrid-covered pairs. Notes and the full pair matrix follow `reciplexa_std` (DN2-4), not the Hybrid nested-if stub.
3. Structured Failure conversion for `length/units` arity/type (`package/arity`, `package/type` on the eval abort; not yet a `failure` effect). Other std modules still use unstructured `EvalError` strings.
4. Skip native stub body elaboration (later increment).

## Portable fallback

Portable fallback remains an OPEN contract (`OPEN-NATIVE-PKG-001`). Do not delete it silently. v1 may ship native-only std packages with thin `.rpi` while the fallback contract stays tracked.

## Non-goals until this milestone starts

- Changing author-visible import paths
- Exposing raw FFI as the authoring API
- Treating coverage % or `gap=0` as DN2 completion

## Gate

Same workspace gates as [`implementation-agent-prompt.md`](implementation-agent-prompt.md).
各moduleの移行commitにv1/reference differential testを含め、DN2-6完了時に
workspace全gateとGUI/CLI host smokeを同じHEADで実行する。
