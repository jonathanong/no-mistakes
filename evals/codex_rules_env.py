#!/usr/bin/env python3
"""Isolated Codex homes for the rules-plugin eval.

The two arms share one config, one workspace copy, and one exec plan.
The with arm also installs plugins/no-mistakes-rules through a marketplace
that lists that plugin and nothing else. Neither arm reads the caller's
home directory.
"""
import json
import os
import shutil
from pathlib import Path

MARKETPLACE = "rules-eval"
PLUGIN = "no-mistakes-rules"
MODEL = "gpt-6.2-sol"
REASONING = "medium"
SANDBOX = "read-only"
APPROVAL = "never"
REPO = Path(__file__).resolve().parents[1]
PLUGIN_SRC = REPO / "plugins" / PLUGIN

_INSTRUCTION_NAMES = {"agents.md", "claude.md", "agents.override.md"}
_RULES_WORD = "rules"


def _shared_config() -> str:
    return (
        f'model = "{MODEL}"\n'
        f'model_reasoning_effort = "{REASONING}"\n'
        f'sandbox_mode = "{SANDBOX}"\n'
        f'approval_policy = "{APPROVAL}"\n'
        "project_doc_max_bytes = 0\n"
        "project_doc_fallback_filenames = []\n"
        "\n"
        "[features]\n"
        "plugins = true\n"
        "hooks = false\n"
    )


def _mentions_rules(text: str) -> bool:
    folded = text.lower()
    if "no-mistakes-rules" in folded:
        return True
    for token in folded.replace("/", " ").replace("\\", " ").split():
        bare = token.strip(".,;:\"'`()[]{}")
        stem = Path(bare).stem
        if bare == _RULES_WORD or stem == _RULES_WORD or stem.endswith("-rules"):
            return True
    return False


def _copy_tree(src: Path, dest: Path) -> None:
    dest.mkdir(parents=True, exist_ok=True)
    for child in src.iterdir():
        if child.name == ".git" or child.is_symlink():
            continue
        target = dest / child.name
        if child.is_dir():
            _copy_tree(child, target)
        elif child.is_file():
            target.write_bytes(child.read_bytes())


def _plugin_version(plugin_src: Path) -> str:
    manifest = plugin_src / ".codex-plugin" / "plugin.json"
    return json.loads(manifest.read_text())["version"]


def _write_marketplace(root: Path, plugin_src: Path) -> Path:
    market = root / "marketplace"
    _copy_tree(plugin_src, market / "plugins" / PLUGIN)
    catalog = {
        "name": MARKETPLACE,
        "plugins": [
            {
                "name": PLUGIN,
                "source": {"source": "local", "path": f"./plugins/{PLUGIN}"},
                "policy": {
                    "installation": "AVAILABLE",
                    "authentication": "ON_INSTALL",
                },
                "category": "Coding",
            }
        ],
    }
    path = market / ".agents" / "plugins" / "marketplace.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(catalog, indent=2) + "\n")
    return market


def installed_plugin_root(codex_home: Path, plugin_src: Path = PLUGIN_SRC) -> Path:
    version = _plugin_version(plugin_src)
    return codex_home / "plugins" / "cache" / MARKETPLACE / PLUGIN / version


def _install_cache(codex_home: Path, plugin_src: Path) -> None:
    _copy_tree(plugin_src, installed_plugin_root(codex_home, plugin_src))


def copy_workspace(src: Path | None, dest: Path) -> None:
    """Copy a fixture, dropping instruction files and anything that names the rules."""
    dest.mkdir(parents=True, exist_ok=True)
    if src is None:
        return
    for path in _files(src):
        if path.is_symlink():
            continue
        rel = path.relative_to(src)
        if any(part.lower() in _INSTRUCTION_NAMES or _mentions_rules(part) for part in rel.parts):
            continue
        text = path.read_bytes().decode("utf-8", errors="replace")
        if _mentions_rules(text):
            continue
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(path.read_bytes())


def _files(root: Path):
    stack = [root]
    while stack:
        current = stack.pop()
        for child in current.iterdir():
            if child.is_dir() and not child.is_symlink():
                stack.append(child)
            elif child.is_file() or child.is_symlink():
                yield child


def reject_output(path: Path, user_home: Path, repo: Path) -> None:
    """Keep a report directory out of the repo and out of ~/.codex or ~/.claude."""
    resolved = path.resolve()
    root = repo.resolve()
    if resolved == root or _is_inside(resolved, root):
        raise SystemExit(f"refusing --output inside the repository: {resolved}")
    reject_user_path(path, user_home, "--output")


def reject_user_path(path: Path, user_home: Path, label: str) -> None:
    """Refuse auth or config copied out of the caller's Codex or Claude home."""
    resolved = path.resolve()
    home = user_home.resolve()
    for parent in (home / ".codex", home / ".claude"):
        if resolved == parent or _is_inside(resolved, parent):
            raise SystemExit(f"refusing {label} inside {parent}")
    for name in ("AGENTS.md", "CLAUDE.md"):
        if resolved == home / name:
            raise SystemExit(f"refusing {label} from {home / name}")


def _is_inside(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
    except ValueError:
        return False
    return True


def _assert_isolated(root: Path, user_home: Path, repo: Path) -> None:
    resolved = root.resolve()
    if _is_inside(resolved, user_home.resolve()) or _is_inside(resolved, repo.resolve()):
        raise SystemExit(f"attempt {resolved} must sit outside the user home and the repository")
    for banned in (user_home / ".codex", user_home / ".claude"):
        if _is_inside(resolved, banned.resolve()):
            raise SystemExit(f"attempt {resolved} inherits {banned}")


def build_arm(
    root: Path,
    arm: str,
    *,
    plugin_src: Path = PLUGIN_SRC,
    fixture: Path | None = None,
    user_home: Path,
    repo: Path = REPO,
    auth_file: Path | None = None,
    api_key: str | None = None,
) -> dict:
    """Create one attempt directory. `arm` is `with` or `without`."""
    if arm not in ("with", "without"):
        raise SystemExit(f"unknown arm {arm}")
    if auth_file is not None:
        reject_user_path(auth_file, user_home, "CODEX_AUTH_FILE")
    root.mkdir(parents=True, exist_ok=True)
    _assert_isolated(root, user_home, repo)
    home = root / "home"
    codex_home = root / "codex-home"
    workspace = root / "workspace"
    home.mkdir()
    codex_home.mkdir()
    copy_workspace(fixture, workspace)
    config = _shared_config()
    installed = None
    if arm == "with":
        market = _write_marketplace(root, plugin_src)
        installed = installed_plugin_root(codex_home, plugin_src)
        _install_cache(codex_home, plugin_src)
        config += (
            f"\n[marketplaces.{MARKETPLACE}]\n"
            'source_type = "local"\n'
            f'source = "{market.resolve()}"\n'
            f'\n[plugins."{PLUGIN}@{MARKETPLACE}"]\n'
            "enabled = true\n"
        )
    (codex_home / "config.toml").write_text(config)
    if auth_file is not None:
        shutil.copyfile(auth_file, codex_home / "auth.json")
    env = {
        "PATH": os.environ.get("PATH", ""),
        "HOME": str(home),
        "CODEX_HOME": str(codex_home),
        "LANG": "C.UTF-8",
    }
    if api_key:
        env["CODEX_API_KEY"] = api_key
    return {
        "arm": arm,
        "root": root,
        "home": home,
        "codex_home": codex_home,
        "workspace": workspace,
        "env": env,
        "config": config,
        "plugin_root": installed,
    }


def exec_argv(codex_bin: str, workspace: Path, last_message: Path, prompt: str) -> list:
    return [
        codex_bin,
        "exec",
        "--json",
        "--ephemeral",
        "--skip-git-repo-check",
        "--ignore-rules",
        "--sandbox",
        SANDBOX,
        "--model",
        MODEL,
        "-c",
        f'model_reasoning_effort="{REASONING}"',
        "-c",
        f'approval_policy="{APPROVAL}"',
        "-c",
        "project_doc_max_bytes=0",
        "-c",
        "project_doc_fallback_filenames=[]",
        "-c",
        "features.plugins=true",
        "-c",
        "features.hooks=false",
        "-C",
        str(workspace),
        "-o",
        str(last_message),
        prompt,
    ]


def _canonical(path: Path) -> str:
    return os.path.realpath(path)


def exec_plan(built: dict, prompt: str, timeout_seconds: int, codex_bin: str = "codex") -> dict:
    last_message = built["root"] / "last-message.txt"
    plugin_root = None
    marketplace_root = None
    if built["arm"] == "with":
        plugin_root = _canonical(built["plugin_root"])
        marketplace_root = _canonical(built["root"] / "marketplace" / "plugins" / PLUGIN)
    return {
        "arm": built["arm"],
        "argv": exec_argv(codex_bin, built["workspace"], last_message, prompt),
        "env": built["env"],
        "cwd": _canonical(built["workspace"]),
        "timeout_seconds": timeout_seconds,
        "model": MODEL,
        "reasoning": REASONING,
        "sandbox": SANDBOX,
        "approval": APPROVAL,
        "prompt": prompt,
        "last_message": str(last_message),
        "workspace": str(built["workspace"]),
        "codex_home": str(built["codex_home"]),
        "plugin_root": plugin_root,
        "marketplace_root": marketplace_root,
        "attempt_root": _canonical(built["root"]),
    }
