# Direct Native v2

**Status:** required future milestone (not optional debt)  
**Current shipping model:** Hybrid Native v1  
**Execution order:** after `lang/active-roadmap.md` Steps 1–6, as Step 7 item 2  
**Normative anchors:** `specification.md` Compiler-native package (~L17260), `KER-001`, `OPEN-NATIVE-PKG-001`

## Hybrid Native v1 (current)

Standard packages today are **Hybrid Native v1**:

- authors still `import` package APIs (`.rpi` + package path)
- hot-domain bodies are supplied as `synthetic_source` RPX
- Rust builtins / eval bridges implement the observable behavior

Absence of a checked-in `.rpx` body file is **not** Direct Native. Do not call v1 “fully native”.

## Direct Native v2 (required)

DN2 replaces synthetic RPX elaboration with a typed Rust callable bound from the package public name or BindingId.

### Complete only when all of the following hold

1. Standard package bodies are **not** supplied via `synthetic_source`.
2. Package public names or BindingIds resolve to **typed Rust callables**.
3. Authors still use the same public `import` API (`.rpi` + package path).
4. Observable behavior matches Hybrid v1 or a portable reference, proven by differential conformance tests.
5. All quality gates are green on the same HEAD.

Until then, keep DN2 **OPEN**. Do not mark native-domain work complete as DN2.

## Portable fallback

Portable fallback remains an OPEN contract (`OPEN-NATIVE-PKG-001`). Do not delete it silently. v1 may ship native-only std packages with thin `.rpi` while the fallback contract stays tracked.

## Non-goals until this milestone starts

- Changing author-visible import paths
- Exposing raw FFI as the authoring API
- Treating coverage % or `gap=0` as DN2 completion

## Gate

Same workspace gates as `lang/implementation-agent-prompt.md`. Add v1/reference differential tests before calling DN2 complete.
