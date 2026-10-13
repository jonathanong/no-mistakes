"""Temporary same-host comparison on pinned public Vouchington trees."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


root = Path.cwd()
fixture = root / "fixture"
artifacts = Path(os.environ["RUNNER_TEMP"]) / "codegen-consumer"
output = artifacts / "measurements"
output.mkdir(parents=True, exist_ok=True)
expected_commit = os.environ["EXPECTED_COMMIT"]
expected_tree = os.environ["EXPECTED_TREE"]


def command(*args, cwd=fixture):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def digest(data):
    return hashlib.sha256(data).hexdigest()


if command("git", "rev-parse", "HEAD") != expected_commit:
    raise SystemExit("Unexpected consumer commit")
if command("git", "rev-parse", "HEAD^{tree}") != expected_tree:
    raise SystemExit("Unexpected consumer tree")

native_package = command(
    "node", "-e",
    "const{createRequire}=require('node:module');"
    "const{dirname,join}=require('node:path');"
    "const r=createRequire(join(process.cwd(),'package.json'));"
    "const p=r.resolve('no-mistakes');"
    "process.stdout.write(require.resolve(process.env.NATIVE_PACKAGE+'/package.json',"
    "{paths:[dirname(p)]}))",
)
package = json.loads(Path(native_package).read_text())
if package["version"] != "0.86.2":
    raise SystemExit(f"Unexpected installed native version: {package['version']}")
native = Path(native_package).parent / "bin" / "no-mistakes"
published = artifacts / "published-no-mistakes"
shutil.copyfile(native, published)
config = fixture / ".no-mistakes.yml"
config_hash = digest(config.read_bytes())
source_commits = {
    variant: (artifacts / variant / "source-commit.txt").read_text().strip()
    for variant in ("baseline", "pgo")
}
if len(set(source_commits.values())) != 1:
    raise SystemExit(f"Binary source mismatch: {source_commits}")

metadata = {
    "fixtureCommit": expected_commit,
    "fixtureTree": expected_tree,
    "configSha256": config_hash,
    "sourceCommits": source_commits,
    "imageOS": os.environ.get("ImageOS"),
    "imageVersion": os.environ.get("ImageVersion"),
    "runnerArch": os.environ.get("RUNNER_ARCH"),
    "cpu": command("lscpu"),
    "node": command("node", "--version"),
    "pnpm": command("pnpm", "--version"),
    "command": "pnpm exec no-mistakes --timeout 0 --lock-timeout 0 check",
    "deadlineSeconds": 180,
    "order": ["baseline", "pgo", "pgo", "baseline"],
    "binaryBuildProvenance": {
        variant: {
            "sha256": digest((artifacts / variant / "no-mistakes").read_bytes()),
            "rustc": (artifacts / variant / "rustc.txt").read_text(),
            "flags": (artifacts / variant / "environment.txt").read_text(),
            "buildTime": (artifacts / variant / "build.time").read_text(),
        }
        for variant in ("baseline", "pgo")
    },
}
(output / "provenance.json").write_text(json.dumps(metadata, indent=2))

records = []
try:
    for trial, variant in enumerate(metadata["order"], 1):
        binary = artifacts / variant / "no-mistakes"
        shutil.copyfile(binary, native)
        native.chmod(0o755)
        prefix = output / f"{variant}-{trial}"
        run = subprocess.run(
            [
                "/usr/bin/time", "-o", str(prefix) + ".time",
                "-f", "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
                "timeout", "--signal=TERM", "--kill-after=5s", "180s",
                "pnpm", "exec", "no-mistakes", "--timeout", "0",
                "--lock-timeout", "0", "check",
            ],
            cwd=fixture, capture_output=True, check=False,
        )
        (Path(str(prefix) + ".stdout")).write_bytes(run.stdout)
        (Path(str(prefix) + ".stderr")).write_bytes(run.stderr)
        fields = dict(
            line.split("=", 1)
            for line in Path(str(prefix) + ".time").read_text().splitlines()
            if "=" in line
        )
        record = {
            "variant": variant, "trial": trial, "exit": run.returncode,
            "elapsed": float(fields["elapsed"]), "user": float(fields["user"]),
            "system": float(fields["system"]), "rssKiB": int(fields["rss_kib"]),
            "binarySha256": digest(binary.read_bytes()),
            "stdoutSha256": digest(run.stdout), "stderrSha256": digest(run.stderr),
            "timeout": run.returncode in (124, 137),
        }
        records.append(record)
        (output / "samples.json").write_text(json.dumps(records, indent=2))
        print(json.dumps(record), flush=True)
finally:
    shutil.copyfile(published, native)

if digest(config.read_bytes()) != config_hash:
    raise SystemExit("Consumer configuration changed during measurements")
command("git", "diff", "--exit-code", "HEAD")
parity = {(row["exit"], row["stdoutSha256"], row["stderrSha256"]) for row in records}
if len(parity) != 1:
    raise SystemExit(f"Consumer output parity failed: {parity}")
