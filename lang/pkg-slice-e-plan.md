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

## Non-goals (Slice E itself)
Full typed `package-resource` / resource effects; real registry network; content hashes.

**Status:** complete (E0–E5)

## Follow-on (PKG leftover)

| ID | Unit | Status |
|----|------|--------|
| R0 | Package API `resolve_package_resource` under `resource_root` + listed resources | **done** |
| R1 | Document language `(resource …)` still OPEN / deferred | **done** (historical note) |
| R2 | Language light `(resource "rel")` → deferred `package-resource` record | **done** |
| R3 | Host `materialize_package_resource` / `resolve_resource_value` when root known | **done** |

Language `(resource "path")` light surface is **done** (tagged record + R0 when package root available). Host materialize helper is **done**. Full typed handle / effects / auto load-time rewrite remain OPEN.
