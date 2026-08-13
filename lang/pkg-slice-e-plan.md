# PKG Slice E — workspace lock / resources / OPEN stubs

**Status:** active  
**Anchors:** PKG-001 §13.10 / §20–21; `lang/package-plan.md` Slice E.

## Units

| ID | Unit | Done when |
|----|------|-----------|
| E0 | Parse `(resources …)` + reject escaping paths (PKG-10) | manifest stores list; `..` / absolute rejected | ✅ |
| E1 | Workspace member discovery | load members’ `package.rpxm`; unique names; no nested workspace | ✅ |
| E2 | Shared workspace root `rpx.lock` | build/read one lock; reject member-local lock | ✅ |
| E3 | Member resolve uses root lock | walk to workspace root for lock | ✅ |
| E4 | Prefer workspace members when resolving | path-free dep → member; DAG; registry → stub error | ✅ |
| E5 | OPEN registry / listed-resource existence stub | stable refusal; optional FS check under resource_root |

## Non-goals
Language `(resource …)` / `package-resource` type; real registry network; content hashes.
