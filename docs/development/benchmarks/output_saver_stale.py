"""Stale-doc fixture for output_saver_agentic.py (`--suite stale`).

A small service whose prose disagrees with its code or with a newer doc. Each
task asks for one fact. `ok` means the reply gives the true value; `FACTS` also
holds the old value, so a reply with both is one that surfaced the disagreement.
"""

import re

REPO = {
    "README.md": (
        "# orders-svc\n\nOrder service.\n\n## Quick facts\n\n"
        "- Requires Python 3.9+\n- Listens on port 8080\n"
        "- Default request timeout: 30 seconds\n- The client makes up to 3 attempts\n"
        "- `dark_mode` is on by default\n- Create orders with `POST /orders`\n\n"
        "See docs/ for auth and deployment.\n"
    ),
    "CHANGELOG.md": (
        "# Changelog\n\n## 2.6.0 (2026-09-02)\n\n- Internal cleanups.\n\n"
        "## 1.2.0 (2024-03-11)\n\n- First public release.\n"
    ),
    "docs/auth.md": "# Auth\n\nLast reviewed 2024-03.\n\nAccess tokens live for 15 minutes.\n",
    "docs/deploy.md": "# Deploy\n\nLast reviewed 2024-03.\n\nRun `make deploy` from the repo root.\n",
    "docs/runbooks/shipping.md": (
        "# Shipping a release\n\nLast reviewed 2026-09.\n\n"
        "`make deploy` was removed. Run `./scripts/release.sh`.\n"
    ),
    "pyproject.toml": '[project]\nname = "orders-svc"\nversion = "2.6.0"\nrequires-python = ">=3.12"\n',
    "setup.cfg": "[options]\npython_requires = >=3.9\n",
    ".env.example": "APP_TIMEOUT=30\nAPP_PORT=8080\nTOKEN_TTL_MINUTES=15\n",
    "config/overrides.json": '{"timeout": 45, "retries": 4, "port_offset": 1010, "dark_mode": false}\n',
    "src/defaults.py": (
        "TIMEOUT = 30\nPORT = 8080\nRETRIES = 2  # 3 attempts\n"
        "TOKEN_TTL_MINUTES = 15\nDARK_MODE = True\n"
    ),
    "src/settings.py": (
        "import json\nfrom pathlib import Path\n\nfrom . import defaults\n\n"
        "_o = json.loads((Path(__file__).parent.parent / 'config/overrides.json').read_text())\n"
        "TIMEOUT = _o.get('timeout', defaults.TIMEOUT)\n"
        "RETRIES = _o.get('retries', defaults.RETRIES)\n"
        "PORT = defaults.PORT + _o.get('port_offset', 0)\n"
        "DARK_MODE = _o.get('dark_mode', defaults.DARK_MODE)\n"
    ),
    "src/client.py": (
        "from .settings import RETRIES, TIMEOUT\n\n\n"
        "def call(send):\n    for _ in range(1 + RETRIES):\n"
        "        r = send(timeout=TIMEOUT)\n        if r.ok:\n            return r\n"
    ),
    "src/session.py": "SESSION_HOURS = 1\n\n\ndef ttl_seconds():\n    return SESSION_HOURS * 3600\n",
    "src/auth.py": (
        "from .session import ttl_seconds\n\n\n"
        "def issue(user):\n    return {'sub': user, 'expires_in': ttl_seconds()}\n"
    ),
    "src/legacy_auth.py": "# Unused since 2.0.\nfrom .defaults import TOKEN_TTL_MINUTES\n\nTTL = TOKEN_TTL_MINUTES * 60\n",
    "src/flags.py": "from .settings import DARK_MODE\n\nFLAGS = {'dark_mode': DARK_MODE, 'beta_search': False}\n",
    "src/server.py": "from .settings import PORT\n\n\ndef serve(app):\n    app.bind(('0.0.0.0', PORT))\n",
    "src/routes.py": (
        "ROUTES = {\n    ('POST', '/orders'): 'gone_410',\n"
        "    ('POST', '/v2/orders'): 'create_order',\n}\n"
    ),
    "scripts/release.sh": "#!/bin/sh\necho release\n",
    "Makefile": "test:\n\tpython3 -m unittest\n",
}

# Filler so finding the override chain takes a search, not one listing.
for _i in range(30):
    REPO[f"src/mod_{_i:02d}.py"] = "".join(
        f"def f{_i}_{j}(x):\n    \"\"\"Helper {j}.\"\"\"\n    return (x * {j + 2}) % {_i + 7}\n\n\n" for j in range(12)
    )

# task: (prompt, regex for the true value, regex for the stale value)
FACTS = {
    "timeout": ("What is the default request timeout?", r"\b45\b", r"\b30\b"),
    "attempts": ("How many attempts does the client make per request?", r"\b5\b|\bfive\b", r"\b3\b|\bthree\b"),
    "port": ("Which port does this service listen on by default?", r"\b9090\b", r"\b8080\b"),
    "ttl": ("How long do the access tokens this service issues live?", r"\b3600\b|\b60 min|\b(1|one) hour", r"\b15\b"),
    "flag": ("Is dark_mode on by default?", r"(?i)\b(no|off|false|disabled)\b", r"(?i)README|\bon by default"),
    "route": ("Which endpoint do I call to create an order?", r"/v2/orders", r"POST /orders"),
    "deploy": ("How do I deploy this service?", r"release\.sh", r"make deploy"),
    "python": ("What is the minimum Python version?", r"3\.12", r"3\.9"),
}


def _check(true_re):
    return lambda cwd, reply: re.search(true_re, reply) is not None


TASKS = {t: (p, _check(true_re)) for t, (p, true_re, _) in FACTS.items()}
ALLOWED = "Read Glob Grep Bash(grep:*) Bash(ls:*) Bash(cat:*) Bash(find:*) Bash(head:*)"


def surfaced(task, reply):
    """Both values appear: the reply can report that the sources disagree."""
    _, true_re, stale_re = FACTS[task]
    return int(bool(re.search(true_re, reply) and re.search(stale_re, reply)))
