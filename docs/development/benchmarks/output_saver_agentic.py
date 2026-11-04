#!/usr/bin/env python3
"""Agentic A/B for the output-saver block: cost, turns and success with tools on.

output_saver_ab.py runs with `--tools ""`, so it can only see prose. In a real
agent most of the bill is turns (every turn rereads the context), so this one
gives the model a small repo, lets it use Read/Edit and a short Bash allowlist, and compares arms on
what an agent run costs: provider-reported `total_cost_usd`, `num_turns`, output
tokens and whether a deterministic check passed.

Arms are text files written as the run's CLAUDE.md (`--setting-sources project`,
so no global config or trs hook leaks in). Built in: `none` and `concise`
("Answer concisely."). Add more with `--arm NAME=FILE`.

Runs are interleaved (run, task, shuffled arms) so a spend cap leaves balanced
data. It spends real usage: it prints the call count and refuses without --yes.
`--total-cap-usd` stops the sweep; `--max-budget-usd` caps one run.

Usage:  python3 docs/development/benchmarks/output_saver_agentic.py --yes --n 3
        --arm old=old.md --arm new=new.md [--model sonnet] [--effort low]
        [--tasks fix,find] [--total-cap-usd 25] [--out results.json]
"""

import argparse
import json
import os
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor

from bench_stats import boot_diff

REPO = {
    "pricing.py": (
        "def apply_discount(price, pct):\n"
        "    \"\"\"pct is a percentage, 0 to 100.\"\"\"\n"
        "    return round(price * (1 - pct), 2)\n\n\n"
        "def with_tax(price, rate=0.2):\n    return round(price * (1 + rate), 2)\n"
    ),
    "test_pricing.py": (
        "import unittest\nfrom pricing import apply_discount, with_tax\n\n\n"
        "class T(unittest.TestCase):\n"
        "    def test_discount(self):\n        self.assertEqual(apply_discount(200.0, 25), 150.0)\n"
        "    def test_zero(self):\n        self.assertEqual(apply_discount(99.5, 0), 99.5)\n"
        "    def test_tax(self):\n        self.assertEqual(with_tax(100.0), 120.0)\n\n\n"
        "if __name__ == '__main__':\n    unittest.main()\n"
    ),
    "validators.py": (
        "import re\n\n_EMAIL = re.compile(r'^[^@\\s]+@[^@\\s]+\\.[a-z]{2,}$')\n\n\n"
        "def is_valid_email(s):\n    return bool(_EMAIL.match(s))\n\n\n"
        "def is_valid_phone(s):\n    return bool(re.fullmatch(r'\\+?[0-9 ]{7,15}', s))\n"
    ),
    "test_validators.py": (
        "import unittest\nfrom validators import is_valid_email, is_valid_phone\n\n\n"
        "class V(unittest.TestCase):\n"
        "    def test_email_ok(self):\n        self.assertTrue(is_valid_email('a@b.io'))\n"
        "    def test_email_bad(self):\n        self.assertFalse(is_valid_email('a@b'))\n"
        "    def test_phone(self):\n        self.assertTrue(is_valid_phone('+34 600 123 456'))\n\n\n"
        "if __name__ == '__main__':\n    unittest.main()\n"
    ),
    "greet.py": (
        "import argparse\n\n\ndef main(argv=None):\n"
        "    p = argparse.ArgumentParser()\n    p.add_argument('name')\n"
        "    a = p.parse_args(argv)\n    print(f'Hello, {a.name}')\n\n\n"
        "if __name__ == '__main__':\n    main()\n"
    ),
    "users.py": (
        "USERS = {1: 'ana', 2: 'luis'}\n\n\ndef get_usr(uid):\n    return USERS.get(uid)\n"
    ),
    "report.py": (
        "from users import get_usr\n\n\ndef line(uid):\n    return f'user={get_usr(uid)}'\n"
    ),
    "api.py": (
        "from users import get_usr\n\n\ndef handler(uid):\n    u = get_usr(uid)\n"
        "    return {'user': u} if u else {'error': 'missing'}\n"
    ),
    "test_users.py": (
        "import unittest\nfrom users import get_usr\nfrom api import handler\n\n\n"
        "class U(unittest.TestCase):\n"
        "    def test_get(self):\n        self.assertEqual(get_usr(1), 'ana')\n"
        "    def test_api(self):\n        self.assertEqual(handler(2), {'user': 'luis'})\n\n\n"
        "if __name__ == '__main__':\n    unittest.main()\n"
    ),
    "README.md": "# demo\n\nSmall repo used to benchmark agent runs.\n",
}


# Filler modules so locating things takes a search, not one read.
for _i in range(30):
    REPO[f"mod_{_i:02d}.py"] = "".join(
        f"def f{_i}_{j}(x):\n    \"\"\"Helper {j} of module {_i}.\"\"\"\n    return (x * {j + 2}) % {_i + 7}\n\n\n"
        for j in range(12)
    )


def sh(cwd, *cmd):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=60)


def tests_pass(cwd):
    return sh(cwd, sys.executable, "-I", "-m", "unittest", "discover", "-q").returncode == 0


def v_fix(cwd, reply):
    pristine = open(os.path.join(cwd, "test_pricing.py")).read() == REPO["test_pricing.py"]
    return tests_pass(cwd) and pristine


def v_find(cwd, reply):
    return "validators.py" in reply and re.search(r"test_validators(\.py)?|test_email", reply) is not None


def v_flag(cwd, reply):
    out = sh(cwd, sys.executable, "-I", "greet.py", "--upper", "bob")
    return out.returncode == 0 and "HELLO, BOB" in out.stdout and sh(cwd, sys.executable, "-I", "greet.py", "bob").stdout.strip() == "Hello, bob"


def v_rename(cwd, reply):
    left = sh(cwd, "grep", "-rl", "get_usr", ".", "--include=*.py").stdout.strip()
    return not left and "def get_user" in open(os.path.join(cwd, "users.py")).read() and tests_pass(cwd)


TASKS = {
    "fix": ("Tests in this repo fail. Make them pass without editing the tests.", v_fix),
    "find": ("Which function validates email format and which test covers it? Give file names.", v_find),
    "flag": ("Add an --upper flag to greet.py that prints the greeting in uppercase.", v_flag),
    "rename": ("Rename get_usr to get_user everywhere in this repo and keep the tests passing.", v_rename),
}

# No bypassPermissions: edits are accepted, Bash is limited to what the fixture needs.
ALLOWED = "Read Edit Write Glob Grep Bash(python3:*) Bash(python:*) Bash(cd:*) Bash(grep:*) Bash(ls:*) Bash(cat:*) Bash(find:*) Bash(head:*) Bash(wc:*) Bash(sed:*) Bash(diff:*)"


def run_one(prompt, arm_text, check, model, effort, budget):
    cwd = tempfile.mkdtemp(prefix="agentic-")
    try:
        for name, body in REPO.items():
            open(os.path.join(cwd, name), "w").write(body)
        open(os.path.join(cwd, "CLAUDE.md"), "w").write(arm_text)
        cmd = [
            "claude", "-p", prompt, "--output-format", "json", "--setting-sources", "project",
            "--no-session-persistence", "--permission-mode", "acceptEdits",
            "--allowedTools", ALLOWED, "--max-budget-usd", str(budget),
        ]
        cmd += ["--model", model] if model else []
        cmd += ["--effort", effort] if effort else []
        t0 = time.time()
        out = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=600)
        d = json.loads(out.stdout)
        u = d.get("usage") or {}
        reply = d.get("result", "") or ""
        return {
            "cost": d.get("total_cost_usd", 0.0), "turns": d.get("num_turns", 0),
            "out_tok": u.get("output_tokens", 0), "error": bool(d.get("is_error")),
            "denied": len(d.get("permission_denials") or []), "ok": int(bool(check(cwd, reply))), "secs": round(time.time() - t0), "reply": reply[:400],
        }
    finally:
        shutil.rmtree(cwd, ignore_errors=True)


def summarize(rows, arms):
    print(f"\n{'arm':9}{'runs':>5}{'cost$':>9}{'turns':>7}{'out_tok':>9}{'success':>9}{'denied':>8}")
    by = {a: [r for r in rows if r["arm"] == a] for a in arms}
    for a, rs in by.items():
        if rs:
            m = lambda k: statistics.fmean(r[k] for r in rs)
            print(f"{a:9}{len(rs):5}{m('cost'):9.3f}{m('turns'):7.1f}{m('out_tok'):9.0f}{m('ok') * 100:8.0f}%{m('denied'):8.2f}")
    print("\nper task, mean cost$ / turns:")
    for t in sorted({r["task"] for r in rows}):
        cells = []
        for a in arms:
            rs = [r for r in by[a] if r["task"] == t]
            cells.append(f"{a} {statistics.fmean(r['cost'] for r in rs):.3f}/{statistics.fmean(r['turns'] for r in rs):.1f}" if rs else f"{a} -")
        print(f"  {t:7}" + "  ".join(cells))
    base = [a for a in ("none", "concise") if by.get(a)]
    for a in arms:
        for b in base + [x for x in arms if x not in ("none", "concise", a) and by[x]][:1]:
            if a == b or not by[a] or not by[b]:
                continue
            for k in ("cost", "turns"):
                x, y = [r[k] for r in by[b]], [r[k] for r in by[a]]
                lo, hi = boot_diff(x, y)
                print(f"{a:9} vs {b:8} {k:5} {statistics.fmean(y) - statistics.fmean(x):+.3f} (95% CI {lo:+.3f}..{hi:+.3f})")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", action="append", default=[], metavar="NAME=FILE")
    ap.add_argument("--n", type=int, default=3)
    ap.add_argument("--tasks", default=",".join(TASKS))
    ap.add_argument("--model")
    ap.add_argument("--effort")
    ap.add_argument("--max-budget-usd", type=float, default=1.0)
    ap.add_argument("--total-cap-usd", type=float, default=25.0)
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--out")
    ap.add_argument("--yes", action="store_true")
    args = ap.parse_args()

    texts = {"none": "", "concise": "Answer concisely.\n"}
    for spec in args.arm:
        name, _, path = spec.partition("=")
        texts[name] = open(path).read()
    tasks = args.tasks.split(",")
    calls = len(texts) * len(tasks) * args.n
    print(f"{len(texts)} arms x {len(tasks)} tasks x N={args.n} = {calls} runs, cap ${args.total_cap_usd}", file=sys.stderr)
    if not args.yes:
        sys.exit("this spends usage; rerun with --yes")

    rng, rows, spent, lock = random.Random(7), [], [0.0], threading.Lock()
    jobs = []
    for i in range(args.n):
        for t in tasks:
            order = list(texts)
            rng.shuffle(order)
            jobs += [(i, t, arm) for arm in order]

    def work(job):
        i, t, arm = job
        if spent[0] >= args.total_cap_usd:
            return
        prompt, check = TASKS[t]
        try:
            r = run_one(prompt, texts[arm], check, args.model, args.effort, args.max_budget_usd)
        except (subprocess.TimeoutExpired, json.JSONDecodeError) as e:
            print(f"run failed ({arm}/{t}): {type(e).__name__}", file=sys.stderr)
            return
        r.update(arm=arm, task=t, run=i)
        with lock:
            spent[0] += r["cost"]
            rows.append(r)
            print(f"  {arm:8}{t:7} run{i} ${r['cost']:.3f} turns {r['turns']} ok {r['ok']} denied {r['denied']}  (total ${spent[0]:.2f})", file=sys.stderr)
            if args.out:
                json.dump(rows, open(args.out, "w"), indent=1)

    with ThreadPoolExecutor(args.jobs) as pool:
        list(pool.map(work, jobs))
    if spent[0] >= args.total_cap_usd:
        print(f"cap reached at ${spent[0]:.2f}", file=sys.stderr)
    summarize(rows, list(texts))


if __name__ == "__main__":
    main()
