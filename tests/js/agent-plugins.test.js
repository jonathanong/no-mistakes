const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const {
  PLUGIN_MANIFESTS,
} = require("../../packages/no-mistakes/scripts/sync-native-package-versions");

const repoRoot = join(__dirname, "..", "..");
const pluginRoot = join(repoRoot, "plugins", "no-mistakes");

function readJson(...parts) {
  return JSON.parse(readFileSync(join(repoRoot, ...parts), "utf8"));
}

test("Claude and Codex marketplaces point at the no-mistakes plugin directory", () => {
  const claude = readJson(".claude-plugin", "marketplace.json");
  assert.deepEqual(
    claude.plugins.map((plugin) => [plugin.name, plugin.source]),
    [["no-mistakes", "./plugins/no-mistakes"]],
  );
  const codex = readJson(".agents", "plugins", "marketplace.json");
  assert.deepEqual(
    codex.plugins.map((plugin) => [plugin.name, plugin.source]),
    [["no-mistakes", { source: "local", path: "./plugins/no-mistakes" }]],
  );
  assert.ok(existsSync(join(pluginRoot, ".claude-plugin", "plugin.json")));
  assert.ok(existsSync(join(pluginRoot, ".codex-plugin", "plugin.json")));
});

test("plugin manifests match the npm package version and ship the skill", () => {
  const { version } = readJson("packages", "no-mistakes", "package.json");
  for (const file of PLUGIN_MANIFESTS) {
    const manifest = readJson(file);
    assert.equal(manifest.name, "no-mistakes", file);
    assert.equal(manifest.version, version, file);
    assert.equal(manifest.skills, "./skills/", file);
  }
  assert.ok(existsSync(join(pluginRoot, "skills", "no-mistakes", "SKILL.md")));
});

test("release bumps and verifies the plugin manifest versions", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  for (const file of PLUGIN_MANIFESTS) {
    assert.ok(workflow.includes(`"${file.split("\\").join("/")}"`), `${file} not verified`);
  }
});
