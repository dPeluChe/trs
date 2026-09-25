#!/usr/bin/env python3
"""trs against similar tools: same commands, same repo state, one scoring rule.

Each hook-based tool runs through its OWN agent hook, the way an agent would
hit it: the hook decides (timed apart, it is paid on every call), then the
shell runs whatever the hook returned. claw-compactor has no hook; its text
API compresses the command's output. Scoring comes from ../truth.py: o200k
tokens, anchors from the raw output that survive, pipelines that still give
the raw answer.

    bash docs/development/benchmarks/compare/setup.sh     # once
    python3 docs/development/benchmarks/compare/compare.py [--reps 3] [--only trs,rtk]

Run from the trs repo root with a release build in target/release.
"""
import argparse, importlib.util, json, os, shutil, statistics, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "../../../.."))
spec = importlib.util.spec_from_file_location("truth", os.path.join(HERE, "..", "truth.py"))
truth = importlib.util.module_from_spec(spec)
spec.loader.exec_module(truth)
TOK_NAME, count = truth.tokenizer()

CACHE = os.environ.get("TRS_BENCH_DIR", os.path.expanduser("~/.cache/trs-bench"))
CLONES = os.environ.get("TRS_BENCH_REPOS") or os.path.join(
    subprocess.run(["spark", "config"], capture_output=True, text=True).stdout.split("repos_root")[1].split("=")[1].split()[0],
    "github.com")
TRS = os.path.join(REPO, "target/release/trs")
SQ = os.path.join(CACHE, "squeez-target/release/squeez")
TS = os.path.join(CLONES, "ppgranger/token-saver")
TO = os.path.join(CLONES, "alexgreensh/token-optimizer/skills/token-optimizer/scripts/bash_hook.py")
BASE = dict(os.environ, GIT_PAGER="cat", PAGER="cat", PATH=os.path.dirname(TRS) + ":" + os.environ["PATH"])
TO_ENV = {k: v for k, v in BASE.items() if k not in ("CLAUDE_PLUGIN_ROOT", "CLAUDE_PLUGIN_DATA", "CODEX_HOME")}
TO_ENV.update(CLAUDE_CONFIG_DIR=os.path.join(CACHE, "to-home"), TOKEN_OPTIMIZER_RUNTIME="claude",
              TOKEN_OPTIMIZER_SNAPSHOT_DIR=os.path.join(CACHE, "to-home/snapshots"))


def payload(cmd, **extra):
    return json.dumps({"tool_name": "Bash", "tool_input": {"command": cmd}, **extra})


def updated(out):
    try:
        return json.loads(out)["hookSpecificOutput"]["updatedInput"]["command"]
    except Exception:
        return None


def ts_python():
    own = os.path.join(TS, "venv/bin/python3")
    return own if os.access(own, os.X_OK) else os.path.join(CACHE, "token-saver-venv/bin/python3")


# tool -> (hook argv, hook stdin, how to read the hook's answer, env for the run)
HOOKS = {
    "trs": lambda c: ([TRS, "rewrite"], payload(c), updated, BASE),
    "rtk": lambda c: (["rtk", "rewrite", c], None, lambda o: o.strip() or None, BASE),
    "token-saver": lambda c: ([ts_python(), os.path.join(TS, "scripts/hook_pretool.py")], payload(c), updated, BASE),
    "squeez": lambda c: ([SQ, "should-wrap", c], None, None, BASE),
    "token-optimizer": lambda c: (["python3", TO], payload(c, session_id="bench"), updated, TO_ENV),
}


def decide(tool, cmd):
    argv, stdin, read, env = HOOKS[tool](cmd)
    t0 = time.perf_counter()
    r = subprocess.run(argv, input=stdin, capture_output=True, text=True, env=env)
    ms = (time.perf_counter() - t0) * 1000
    if tool == "squeez":
        return (f"{SQ} wrap {json.dumps(cmd)}" if r.returncode == 0 else None), ms, env
    return read(r.stdout), ms, env


def run(cmd, env, cwd):
    # squeez dedups repeats within a session dir: a fresh one measures a first call.
    env = dict(env, SQUEEZ_DIR=tempfile.mkdtemp()) if "squeez" in cmd else env
    t0 = time.perf_counter()
    r = subprocess.run(["bash", "-c", cmd], capture_output=True, text=True, errors="replace", env=env, cwd=cwd)
    return r.stdout + r.stderr, (time.perf_counter() - t0) * 1000


def claw(text):
    py = os.path.join(CACHE, "claw-venv/bin/python3")
    site = subprocess.run([py, "-c", "import sysconfig;print(sysconfig.get_paths()['purelib'])"],
                          capture_output=True, text=True).stdout.strip()
    # Installed, the engine is `claw_compactor.fusion`; `scripts/` on the path
    # is what its Abbrev stage imports (`compressed_context`), else it is off.
    code = ("import sys;from claw_compactor.fusion.engine import FusionEngine;"
            "sys.stdout.write(FusionEngine().compress(sys.stdin.read(), role='tool')['compressed'])")
    env = dict(os.environ, PYTHONPATH=os.path.join(site, "scripts") + ":" + site)
    return subprocess.run([py, "-c", code], input=text, capture_output=True, text=True, env=env).stdout


def corpus():
    subs = {"{repo}": REPO, "{fixtures}": os.path.join(CACHE, "fixtures")}
    docker_up = subprocess.run(["docker", "info"], capture_output=True).returncode == 0 \
        if shutil.which("docker") else False
    for line in open(os.path.join(HERE, "corpus.txt")):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[docker]"):
            if not docker_up:
                print(f"  skipped (docker not running): {line}", file=sys.stderr)
                continue
            line = line[len("[docker]"):].strip()
        cwd, _, cmd = line.partition("::")
        for k, v in subs.items():
            cwd, cmd = cwd.replace(k, v), cmd.replace(k, v)
        yield cwd, cmd


def score(raw, out, found):
    rt, ot = count(raw), count(out)
    n = len(found)
    return {
        "tok": ot,
        "cut": round((rt - ot) / rt * 100, 1) if rt else 0.0,
        "kept": round(truth.kept(found, out) * 100 / n) if n else 100,
        "tail": round(truth.kept(found, truth.naive_tail(raw, len(out))) * 100 / n) if n else 100,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--only", default="")
    ap.add_argument("--out", default=os.path.join(CACHE, "compare.json"))
    a = ap.parse_args()
    tools = [t for t in [*HOOKS, "claw-compactor"] if not a.only or t in a.only.split(",")]
    rows = []
    for cwd, cmd in corpus():
        raws = [run(cmd, BASE, cwd) for _ in range(a.reps)]
        raw = raws[-1][0]
        found = truth.anchors(raw)
        row = {"cmd": cmd, "raw_tok": count(raw), "raw_ms": statistics.median(ms for _, ms in raws),
               "anchors": len(found), "tools": {}}
        for tool in tools:
            if tool == "claw-compactor":
                row["tools"][tool] = score(raw, claw(raw), found)
                continue
            new, _, env = decide(tool, cmd)
            hook_ms = statistics.median(decide(tool, cmd)[1] for _ in range(5))
            runs = [run(new or cmd, env, cwd) for _ in range(a.reps)]
            s = score(raw, runs[-1][0], found)
            s.update(rewrote=bool(new), hook_ms=round(hook_ms, 1),
                     exec_ms=round(statistics.median(ms for _, ms in runs), 1))
            row["tools"][tool] = s
        rows.append(row)
        print(f"  {cmd[:60]}", file=sys.stderr, flush=True)
    pipes = {}
    for p in truth.PIPELINES:
        raw = " ".join(run(p, BASE, REPO)[0].split())
        pipes[p] = {t: " ".join(run(decide(t, p)[0] or p, decide(t, p)[2], REPO)[0].split()) == raw
                    for t in tools if t != "claw-compactor"}
    json.dump({"tokenizer": TOK_NAME, "rows": rows, "pipelines": pipes}, open(a.out, "w"), indent=1)
    report(rows, pipes, tools)


def report(rows, pipes, tools):
    def agg(t):
        rs = [r for r in rows if t in r["tools"]]
        raw = sum(r["raw_tok"] for r in rs)
        out = sum(r["tools"][t]["tok"] for r in rs)
        an = sum(r["anchors"] for r in rs)
        kept = sum(r["tools"][t]["kept"] * r["anchors"] / 100 for r in rs)
        risk = sum(r["tools"][t]["cut"] >= 90 and r["tools"][t]["kept"] < 70 for r in rs if r["anchors"])
        grew = sum(r["tools"][t]["cut"] < 0 for r in rs)
        return raw, out, an, kept, risk, grew
    print(f"\n{len(rows)} commands, tokens: {TOK_NAME}\n")
    print("| tool | tokens cut | anchors kept | RISK | grew | pipelines | hook ms | exec +ms |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|")
    for t in tools:
        raw, out, an, kept, risk, grew = agg(t)
        hooks = [r["tools"][t].get("hook_ms") for r in rows if r["tools"][t].get("hook_ms") is not None]
        over = [r["tools"][t]["exec_ms"] - r["raw_ms"] for r in rows if r["tools"][t].get("rewrote")]
        p = sum(v[t] for v in pipes.values()) if t in next(iter(pipes.values()), {}) else None
        print(f"| {t} | {(raw - out) / raw * 100:.0f}% | {kept / an * 100:.0f}% | {risk} | {grew} | "
              f"{f'{p}/{len(pipes)}' if p is not None else 'n/a'} | "
              f"{statistics.median(hooks) if hooks else 0:.1f} | {statistics.median(over) if over else 0:.1f} |")
    print("\nPer command, `cut / anchors kept`:\n")
    print("| command | raw tok | " + " | ".join(tools) + " |")
    print("|---|---:|" + "---:|" * len(tools))
    for r in rows:
        cells = [f"{r['tools'][t]['cut']:.0f}% / {r['tools'][t]['kept']}%" for t in tools]
        print(f"| `{os.path.basename(r['cmd']) if '/' in r['cmd'].split()[0] else r['cmd']}` | {r['raw_tok']} | " + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()
