---
name: Hook migration plan
description: Switch Claude Code hook from rtk to trs after v0.5.10 ships
type: project
---

Plan to replace rtk with trs as the sole Claude Code hook.

**Why:** Both rtk and trs hooks are active simultaneously, causing double-wrapping and `rtk proxy` usage that pollutes trs stats. Goal is trs-only workflow.

**How to apply:** After v0.5.10 is released and installed:
1. Remove `@RTK.md` from `~/.claude/CLAUDE.md`
2. Run `trs init --all --global`
3. Replace any `rtk proxy <cmd>` usage with `trs raw <cmd>`

Do NOT do the switch before v0.5.10 ships — the hook would pin to the older installed binary and miss all the v0.5.10 improvements.
