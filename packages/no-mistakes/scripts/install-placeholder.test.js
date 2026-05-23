const assert = require("node:assert/strict");
const { mkdtemp, rm, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const core = require("./install/index");

test("detects placeholder files before skipping downloads", async () => {
  const root = await mkdtemp(join(tmpdir(), "no-mistakes-placeholder-"));
  const placeholder = join(root, "placeholder-bin");
  const binary = join(root, "native-bin");
  try {
    await writeFile(placeholder, "Native binary placeholder.\n");
    await writeFile(binary, "already here");
    assert.deepEqual(
      [
        core.isPlaceholder(placeholder),
        core.isPlaceholder(binary),
        core.isPlaceholder(join(root, "missing-bin")),
      ],
      [true, false, false],
    );
    assert.throws(() => core.isPlaceholder(root), /Failed to inspect native binary placeholder/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
