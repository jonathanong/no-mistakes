const assert = require("node:assert/strict");
const { execFile } = require("node:child_process");
const { cp, mkdtemp, readFile, readdir, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { promisify } = require("node:util");
const test = globalThis.test || require("node:test").test;
const execute = promisify(execFile);
const root = join(__dirname, "../..");

async function files(directory, prefix = "") {
  const entries = await readdir(join(directory, prefix), { withFileTypes: true });
  const result = [];
  for (const entry of entries) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) result.push(...(await files(directory, relative)));
    else result.push(relative);
  }
  return result.sort();
}

function withoutDescription(text) {
  const end = text.indexOf("\n---", 4);
  assert.ok(end > 0, "skill has frontmatter");
  return (
    text.slice(0, end).replace(/^description: .*$/m, "description: <variant>") + text.slice(end)
  );
}

test("rules trigger variants preserve cases, graders, and every rule body", async () => {
  const temporary = await mkdtemp(join(tmpdir(), "rules-trigger-variants-"));
  try {
    const source = join(root, "plugins/no-mistakes-rules");
    const copied = join(temporary, "plugins/no-mistakes-rules");
    await cp(source, copied, { recursive: true });
    for (const name of ["generate.py", "rules_cases.py", "rules_variants.py"]) {
      await cp(join(root, "evals", name), join(temporary, "evals", name), { recursive: true });
    }
    const queue = "skills/queue-rules/SKILL.md";
    const before = join(root, "fixtures/evals/rules-trigger-variants/queue-before.md");
    await cp(before, join(copied, queue));
    for (const variant of ["control", "describe"]) {
      await execute("python3", ["evals/generate.py", "--rules-variant", variant], {
        cwd: temporary,
        timeout: 10000,
      });
    }
    const control = join(temporary, "evals-variants/_plugins/control");
    const describe = join(temporary, "evals-variants/_plugins/describe");
    const pluginFiles = await files(control);
    assert.ok(pluginFiles.length > 0);
    assert.deepEqual(pluginFiles, await files(copied));
    assert.deepEqual(await files(describe), pluginFiles);
    for (const file of pluginFiles) {
      const original = await readFile(join(control, file), "utf8");
      const candidate = await readFile(join(describe, file), "utf8");
      assert.equal(original, await readFile(join(copied, file), "utf8"), file);
      if (file.endsWith("/SKILL.md"))
        assert.equal(withoutDescription(candidate), withoutDescription(original), file);
      else assert.equal(candidate, original, file);
    }
    assert.notEqual(
      await readFile(join(control, queue), "utf8"),
      await readFile(join(describe, queue), "utf8"),
      "synthetic description was rewritten",
    );
    const controlCases = join(temporary, "evals-variants/rules-control");
    const candidateCases = join(temporary, "evals-variants/rules-describe");
    const caseFiles = await files(controlCases);
    assert.ok(caseFiles.length > 0);
    assert.deepEqual(await files(candidateCases), caseFiles);
    for (const file of caseFiles) {
      const original = await readFile(join(controlCases, file), "utf8");
      const candidate = await readFile(join(candidateCases, file), "utf8");
      const normalized = file.endsWith("/prompt.md")
        ? candidate.replace(
            "plugins: [../../_plugins/describe]",
            "plugins: [../../_plugins/control]",
          )
        : candidate;
      assert.equal(normalized, original, file);
    }
    const caseNames = caseFiles
      .filter((file) => file.endsWith("/prompt.md"))
      .map((file) => file.slice(0, -"/prompt.md".length));
    const expectedCases = [];
    for (const entry of await readdir(join(root, "evals"), { withFileTypes: true })) {
      if (
        entry.isDirectory() &&
        entry.name.startsWith("rules-") &&
        (await readdir(join(root, "evals", entry.name))).includes("prompt.md")
      ) {
        expectedCases.push(entry.name);
      }
    }
    assert.ok(expectedCases.length > 0);
    assert.deepEqual(caseNames, expectedCases.sort());
    for (const name of caseNames) {
      const original = await readFile(join(root, "evals", name, "prompt.md"), "utf8");
      const generated = await readFile(join(controlCases, name, "prompt.md"), "utf8");
      assert.equal(
        generated.replace(
          "plugins: [../../_plugins/control]",
          "plugins: [../../plugins/no-mistakes-rules]",
        ),
        original,
      );
      const graders = await files(join(root, "evals", name, "graders"));
      assert.ok(graders.length > 0);
      assert.deepEqual(await files(join(controlCases, name, "graders")), graders);
      for (const grader of graders) {
        assert.equal(
          await readFile(join(controlCases, name, "graders", grader), "utf8"),
          await readFile(join(root, "evals", name, "graders", grader), "utf8"),
        );
      }
    }
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
});
