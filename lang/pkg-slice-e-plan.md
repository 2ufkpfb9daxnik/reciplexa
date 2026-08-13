# PKG Slice E — workspace lock / resources / OPEN stubs

**Status:** complete  
**Anchors:** PKG-001 §13.10 / §20–21; `lang/package-plan.md` Slice E.

## Units

| ID | Unit | Done when |
|----|------|-----------|
| E0 | Parse `(resources …)` + reject escaping paths (PKG-10) | manifest stores list; `..` / absolute rejected | ✅ |
| E1 | Workspace member discovery | load members’ `package.rpxm`; unique names; no nested workspace | ✅ |
| E2 | Shared workspace root `rpx.lock` | build/read one lock; reject member-local lock | ✅ |
| E3 | Member resolve uses root lock | walk to workspace root for lock | ✅ |
| E4 | Prefer workspace members when resolving | path-free dep → member; DAG; registry → stub error | ✅ |
| E5 | OPEN registry / listed-resource existence stub | stable refusal; optional FS check under resource_root | ✅ |

## Non-goals
Language `(resource …)` / `package-resource` type; real registry network; content hashes.

**Status:** complete (E0–E5)

## Follow-on (PKG leftover)

| ID | Unit | Status |
|----|------|--------|
| R0 | Package API `resolve_package_resource` under `resource_root` + listed resources | **done** |
| R1 | Document language `(resource …)` still OPEN / deferred | **done** (this note + `package-plan.md`) |

Language `(resource "path")` parse/elaborate/host value remains **OPEN** — R0 is host/manifest only.
