"""Assemble the mixed query-projection benchmark from a source checkout."""

import argparse
from pathlib import Path
from shutil import copy2, copytree

import yaml


def read_config(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def write_config(path: Path, config: dict) -> None:
    path.write_text(yaml.safe_dump(config, sort_keys=False), encoding="utf-8")


def materialize(repo: Path, output: Path) -> None:
    perf = repo / "fixtures/performance"
    rules = repo / "test-cases/rules"
    mixed = output / "mixed"
    copytree(perf / "graph-gates", mixed)
    for folder, source in {
        "scan": rules / "test-no-live-repository-scan/caller-roots/large",
        "query": rules / "query-reached-per-item/stage-import-index-workload",
        "forbidden": perf / "forbidden-call-argument-scope",
        "storybook": rules / "require-storybook-stories/fixture/defaults",
        "finite": perf / "finite-set-call-site-scope",
    }.items():
        copytree(source, mixed / folder)
    deps = perf / "dependency-checks"
    copytree(deps / "packages", mixed / "packages", dirs_exist_ok=True)
    copy2(deps / "pnpm-lock.yaml", mixed / "pnpm-lock.yaml")
    copytree(repo / ".github/workflows", mixed / ".github/workflows", dirs_exist_ok=True)

    config = read_config(mixed / ".no-mistakes.yml")
    query = read_config(mixed / "query/.no-mistakes.yml")
    query["rules"][0]["include"] = ["query/src/**/*.mts"]
    forbidden = read_config(mixed / "forbidden/broad.yml")["rules"][0]
    forbidden["include"] = ["forbidden/src/**"]
    forbidden["options"]["roots"][0]["glob"] = [
        "forbidden/src/**/*.ts",
        "forbidden/src/**/*.tsx",
    ]
    finite = read_config(mixed / "finite/consuming.yml")["rules"][1]
    for entry in finite["options"]["sets"]:
        entry["file"] = f"finite/{entry['file']}"

    scan = {
        "rule": "test-no-live-repository-scan",
        "scope": "repository",
        "options": {"roots": [{"glob": ["scan/tests/**/*.test.ts"]}], "fixtureDirs": ["samples"]},
    }
    story = {
        "rule": "require-storybook-stories",
        "scope": "repository",
        "options": {
            "include": ["storybook/components/*.tsx"],
            "stories": ["storybook/storybook/*.stories.tsx"],
        },
    }
    workflows = [
        ("github-actions-step-timeouts", {"maxMinutes": 30}),
        ("github-actions-max-inline-script-lines", None),
        (
            "github-actions-job-timeouts",
            {"maxMinutes": 30, "rejectStepExceedingJob": True, "rejectJobAtOrAboveStepSum": True},
        ),
        ("github-actions-download-integrity", None),
    ]
    workflow_rules = [
        {"rule": rule, "scope": "repository", **({"options": options} if options else {})}
        for rule, options in workflows
    ]
    config["effects"] = query["effects"]
    playwright = {"rule": "playwright-unique-test-ids", "scope": "repository"}
    config["rules"] += [
        forbidden,
        *query["rules"],
        scan,
        story,
        *workflow_rules,
        finite,
        *read_config(deps / ".no-mistakes.yml")["rules"],
        playwright,
    ]
    write_config(mixed / ".no-mistakes.yml", config)

    config["projects"]["storybook"] = {
        "type": "nextjs", "root": "storybook", "include": ["storybook/**"]
    }
    del story["scope"]
    story["projects"] = ["storybook"]
    story["options"] = {"include": ["components/*.tsx"], "stories": ["storybook/*.stories.tsx"]}
    del playwright["scope"]
    playwright["projects"] = ["web"]
    write_config(output / "mixed-corrected.yml", config)
    config["rules"].remove(scan)
    write_config(output / "mixed-corrected-no-scan.yml", config)
    write_config(output / "workflows.yml", {"version": 2, "rules": workflow_rules})


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path, help="checkout that supplies the saved fixtures")
    parser.add_argument("output", type=Path, help="scratch directory for the generated workload")
    args = parser.parse_args()
    materialize(args.checkout.resolve(), args.output.resolve())


if __name__ == "__main__":
    main()
