# -*- coding: utf-8 -*-
from pathlib import Path
import re
import json
from collections import Counter

path = Path(r"d:/reciplexa/lang/part2-conformance.md")
text = path.read_text(encoding="utf-8")

updates = [
    # ERR core path (batch 1)
    ("L18993", "partial", "raise/handle/or-raise/as-result + Never; bracket/cleanup/defect deferred Part III"),
    ("L19159", "partial", "failure perform + handle failure; bracket/cleanup still deferred"),
    ("L19195", "ok", "raise → Perform failure; or-raise/as-result elaboration"),
    ("L19196", "ok", "(raise e) → Perform{op:failure}"),
    ("L19214", "partial", "CoreType::Never for failure perform; subtype via unify"),
    ("L19239", "ok", "existing effect perform; no separate exception runtime"),
    ("L19245", "ok", "handle failure 1-param; resume rejected at check+eval"),
    ("L19306", "partial", "failure in effect row; removed by handle failure"),
    ("L19359", "partial", "ERR §7 guideline; single failure type per boundary is policy not enforced"),
    ("L19384", "partial", "re-raise via nested handle works (deep semantics)"),
    ("L19407", "partial", "or-raise / as-result explicit conversion helpers"),
    ("L19408", "ok", "no implicit result↔failure conversion"),
    ("L19421", "partial", "(or-raise result) elaborates to match+raise"),
    ("L19441", "partial", "(as-result (fn () body)) → handle failure + ok/err variants"),
    ("L19486", "partial", "failure in effect row is explicit; no auto dual API"),
    # Part III → deferred 依存待ち
    ("L19496", "deferred", "依存待ち: bracket primitive + release guarantees (Part III runtime)"),
    ("L19497", "deferred", "依存待ち: bracket acquire/use/release (Part III runtime)"),
    ("L19515", "deferred", "依存待ち: bracket roles (Part III runtime)"),
    ("L19525", "deferred", "依存待ち: bracket release guarantees (Part III runtime)"),
    ("L19535", "deferred", "依存待ち: acquire/release registration (Part III runtime)"),
    ("L19536", "deferred", "依存待ち: acquire failure skip release (Part III runtime)"),
    ("L19540", "deferred", "依存待ち: partial acquire (Part III runtime)"),
    ("L19552", "deferred", "依存待ち: nested bracket on partial acquire (Part III runtime)"),
    ("L19558", "deferred", "依存待ち: use normal completion + release (Part III runtime)"),
    ("L19570", "deferred", "依存待ち: failure during use + release (Part III runtime)"),
    ("L19581", "deferred", "依存待ち: effect suspend + continuation cleanup (Part III runtime)"),
    ("L19582", "deferred", "依存待ち: temporary effect suspend (Part III runtime)"),
    ("L19594", "deferred", "依存待ち: resume without release (Part III runtime)"),
    ("L19598", "deferred", "依存待ち: discard continuation cleanup (Part III runtime)"),
    ("L19620", "deferred", "依存待ち: continuation escape cleanup (Part III runtime)"),
    ("L19638", "deferred", "依存待ち: cleanup LIFO order (Part III runtime)"),
    ("L19639", "deferred", "依存待ち: cleanup LIFO (Part III runtime)"),
    ("L19653", "deferred", "依存待ち: cleanup at-most-once (Part III runtime)"),
    ("L19668", "deferred", "依存待ち: single release failure (Part III runtime)"),
    ("L19676", "deferred", "依存待ち: failure during cleanup (Part III runtime)"),
    ("L19677", "deferred", "依存待ち: primary failure during cleanup (Part III runtime)"),
    ("L19681", "deferred", "依存待ち: suppressed failure (Part III runtime)"),
    ("L19702", "deferred", "依存待ち: release failure after success (Part III runtime)"),
    ("L19709", "deferred", "依存待ち: multiple suppressed failures (Part III runtime)"),
    ("L19720", "deferred", "依存待ち: suppressed failure not in handler (Part III runtime)"),
    ("L19734", "deferred", "依存待ち: finally (Part III runtime)"),
    ("L19757", "deferred", "依存待ち: cancellation cleanup (Part III runtime)"),
    ("L19761", "deferred", "依存待ち: cancellation details (Part III runtime)"),
    ("L19772", "deferred", "依存待ち: defect model (Part III outcome/runtime)"),
    ("L19773", "deferred", "依存待ち: defect definition (Part III outcome/runtime)"),
    ("L19788", "deferred", "依存待ち: defect not in effect row (Part III)"),
    ("L19794", "deferred", "依存待ち: no defect resume (Part III)"),
    ("L19798", "deferred", "依存待ち: defect not caught by handler (Part III)"),
    ("L19802", "deferred", "依存待ち: defect cleanup (Part III)"),
    ("L19808", "deferred", "依存待ち: fault boundary (Part III runtime)"),
    ("L19809", "deferred", "依存待ち: fault boundary definition (Part III)"),
    ("L19823", "deferred", "依存待ち: fault boundary not user API (Part III)"),
    ("L19827", "deferred", "依存待ち: fault boundary processing (Part III)"),
    ("L19838", "deferred", "依存待ち: fault boundary continuation (Part III)"),
    ("L19851", "deferred", "依存待ち: terminal failure (Part III)"),
    ("L19852", "deferred", "依存待ち: terminal failure definition (Part III)"),
    ("L19866", "deferred", "依存待ち: terminal failure vs handler (Part III)"),
    ("L19870", "deferred", "依存待ち: terminal failure cleanup (Part III)"),
    ("L19874", "deferred", "依存待ち: terminal failure minimal handling (Part III)"),
    ("L19887", "deferred", "依存待ち: assertion→defect wiring (Part III)"),
    ("L19897", "deferred", "依存待ち: match exhaustiveness defect (Part III)"),
    ("L19904", "deferred", "依存待ち: cast failure classification (Part III gradual)"),
    ("L19911", "deferred", "依存待ち: index access failure (Part III)"),
    ("L19918", "deferred", "依存待ち: arithmetic overflow policy (Part III)"),
    ("L19934", "deferred", "依存待ち: division by zero policy (Part III)"),
    ("L19944", "partial", "one-shot double resume rejected at eval; defect report deferred"),
    ("L19950", "deferred", "依存待ち: validator defect (Part III)"),
    # MEM lower batch
    ("L20916", "partial", "MakeClosure in mem lower; not default eval memory path"),
    ("L20940", "deferred", "依存待ち: closure identity observability (Part III / full MEM)"),
    ("L21047", "partial", "Resume/DiscardCont/Raise in mem IR+exec; not full continuation model"),
    ("L21106", "partial", "MemInstr::Raise runs RegisterCleanup LIFO in exec"),
    ("L21135", "deferred", "依存待ち: scoped resource handle + bracket (Part III)"),
    ("L21153", "deferred", "依存待ち: bracket concept type surface (Part III)"),
    ("L21182", "deferred", "依存待ち: scope closure capture rules (Part III)"),
    ("L21201", "deferred", "依存待ち: resource API surface (Part III)"),
    ("L21296", "deferred", "依存待ち: foreign ownership boundary (Part III)"),
    ("L21366", "deferred", "依存待ち: snapshot + Perceus integration (Part III)"),
]


def set_status(text: str, line_marker: str, new_status: str, new_notes: str) -> str:
    pat = re.compile(
        rf"(- \[x\] \*\*L\d+ {re.escape(line_marker)}:[^\n]*\*\* )"
        rf"[—-] `(unchecked|ok|partial|gap|deferred|meta)"
        rf"(`(?:\r?\n)  - spec: `[^\n]+`(?:\r?\n)  - notes: )([^\n]*)",
        re.M,
    )
    m = pat.search(text)
    if not m:
        print("MISS", line_marker)
        return text
    old = m.group(2)
    text = pat.sub(rf"\g<1>`{new_status}\g<3>{new_notes}", text, count=1)
    print(f"{line_marker}: {old} -> {new_status}")
    return text


for marker, status, notes in updates:
    text = set_status(text, marker, status, notes)

pat = re.compile(r"^- \[[xX]\].*?`(unchecked|ok|partial|gap|deferred|meta)`\s*$", re.M)
statuses = pat.findall(text)
c = Counter(statuses)
stats = {
    "total": sum(c.values()),
    "unchecked": c.get("unchecked", 0),
    "ok": c.get("ok", 0),
    "partial": c.get("partial", 0),
    "gap": c.get("gap", 0),
    "deferred": c.get("deferred", 0),
    "meta": c.get("meta", 0),
}
print("STATS", stats)
assert stats["total"] == 1589, stats
assert stats["unchecked"] == 0, stats


def repl_counter(name: str, val: int, text: str) -> str:
    return re.sub(rf"(- \*\*{name}\*\*: )\d+", rf"\g<1>{val}", text)


for k in ["total", "unchecked", "ok", "partial", "gap", "deferred", "meta"]:
    text = repl_counter(k, stats[k], text)

path.write_text(text, encoding="utf-8")
Path(r"d:/reciplexa/lang/part2-conformance-stats.json").write_text(
    json.dumps(stats, indent=2) + "\n", encoding="utf-8"
)

# Update part2-deferred.md dependency count
dep = Path(r"d:/reciplexa/lang/part2-deferred.md")
dep_text = dep.read_text(encoding="utf-8")
dep_count = len(re.findall(r"^- \*\*L\d+", dep_text, re.M))
# recount 依存待ち in conformance
dep_items = len(re.findall(r"依存待ち:", text))
dep_text = re.sub(r"(- \*\*依存待ち\*\*: )\d+", rf"\g<1>{dep_items}", dep_text)
dep_text = re.sub(r"(- \*\*合計\*\*: )\d+", lambda m: f"{m.group(1)}{dep_count}", dep_text)
dep.write_text(dep_text, encoding="utf-8")
print("wrote conformance + stats + deferred dep count", dep_items)
