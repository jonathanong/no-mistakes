const assert = require("node:assert/strict");
const { existsSync, readdirSync, readFileSync } = require("node:fs");
const { basename, dirname, join } = require("node:path");

const {
  PLUGIN_MANIFESTS,
} = require("../../packages/no-mistakes/scripts/sync-native-package-versions");

const repoRoot = join(__dirname, "..", "..");
const PLUGINS = ["no-mistakes", "no-mistakes-rules"];

function readJson(...parts) {
  return JSON.parse(readFileSync(join(repoRoot, ...parts), "utf8"));
}

test("Claude and Codex marketplaces point at each plugin directory", () => {
  const claude = readJson(".claude-plugin", "marketplace.json");
  assert.deepEqual(
    claude.plugins.map((plugin) => [plugin.name, plugin.source]),
    PLUGINS.map((name) => [name, `./plugins/${name}`]),
  );
  const codex = readJson(".agents", "plugins", "marketplace.json");
  assert.deepEqual(
    codex.plugins.map((plugin) => [plugin.name, plugin.source]),
    PLUGINS.map((name) => [name, { source: "local", path: `./plugins/${name}` }]),
  );
  for (const name of PLUGINS) {
    assert.ok(existsSync(join(repoRoot, "plugins", name, ".claude-plugin", "plugin.json")));
    assert.ok(existsSync(join(repoRoot, "plugins", name, ".codex-plugin", "plugin.json")));
  }
});

test("plugin manifests match the npm package version and ship their skills", () => {
  const { version } = readJson("packages", "no-mistakes", "package.json");
  for (const file of PLUGIN_MANIFESTS) {
    const manifest = readJson(file);
    assert.equal(manifest.name, basename(dirname(dirname(file))), file);
    assert.equal(manifest.version, version, file);
    assert.equal(manifest.skills, "./skills/", file);
  }
  assert.ok(
    existsSync(join(repoRoot, "plugins", "no-mistakes", "skills", "no-mistakes", "SKILL.md")),
  );
  const rulesSkills = join(repoRoot, "plugins", "no-mistakes-rules", "skills");
  const names = readdirSync(rulesSkills);
  assert.ok(names.length > 0);
  for (const name of names) {
    assert.ok(existsSync(join(rulesSkills, name, "SKILL.md")), name);
  }
});

test("release bumps and verifies the plugin manifest versions", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  for (const file of PLUGIN_MANIFESTS) {
    assert.ok(workflow.includes(`"${file.split("\\").join("/")}"`), `${file} not verified`);
  }
});
