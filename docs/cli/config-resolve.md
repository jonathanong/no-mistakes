# config resolve

Print the effective configuration after discovery, Next.js app resolution, and
test-plan trigger expansion. Use this to snapshot consumer config instead of
string-locking YAML.

## Usage

```bash
no-mistakes config resolve [--root <dir>] [--config <path>]
```

## Options

| Flag | Description |
|------|-------------|
| `--root` | Project root directory (default: current directory). |
| `--config` | Path to `.no-mistakes.yml` (auto-discovered when omitted). |

Output is JSON on stdout. Named `fullSuiteTriggers` list entries appear under
`vitestFullSuiteTriggers` with `source: "triggers"`. Deprecated project-keyed
triggers use `source: "projects"`.
