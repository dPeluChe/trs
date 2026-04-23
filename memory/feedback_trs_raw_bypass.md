---
name: Use trs raw for bypass
description: Use `trs raw <cmd>` not `rtk proxy <cmd>` when raw output is needed
type: feedback
---

When raw (uncompressed) command output is needed in this project, use `trs raw <cmd>`, not `rtk proxy <cmd>`.

**Why:** trs is the tool we're building; using `rtk proxy` pollutes the stats history with mixed tooling and confuses the optimization analysis. `trs raw` still tracks to trs stats (0% compression), which is the correct behavior.

**How to apply:** Any time I'd normally reach for `rtk proxy grep`, `rtk proxy trs stats`, etc., use `trs raw grep`, `trs raw trs stats` etc. instead. `TRS_SKIP=1 <cmd>` is the alternative if tracking is also undesired.
