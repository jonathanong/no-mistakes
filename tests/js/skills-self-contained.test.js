const assert = require("node:assert/strict");
const { existsSync, readdirSync, readFileSync } = require("node:fs");
const { dirname, join, relative, resolve, sep } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const skillsDir = join(repoRoot, "skills");

// Absolute URLs (https://…, etc.) travel with the skill, so they are allowed.
// Bare relative paths do not — they only resolve in this repo, not in a consumer
// that synced the skill via `npx skills`.
const urlScheme = /^[a-z][a-z0-9+.-]*:/i;

function walkMarkdown(dir) {
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkMarkdown(full));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(full);
    }
  }
  return files;
}

// A skill's own root is `skills/<name>`; only files under it survive the sync.
function skillRootFor(file) {
  const [name] = relative(skillsDir, file).split(sep);
  return join(skillsDir, name);
}

// Pull `.md` references out of inline-code spans and markdown links — the two
// conventions these skills use to point at docs.
function extractMarkdownRefs(content) {
  const refs = [];
  for (const match of content.matchAll(/`([^`\n]+\.md)`/g)) {
    refs.push(match[1]);
  }
  for (const match of content.matchAll(/\]\(([^)\s]+\.md)(?:#[^)]*)?\)/g)) {
    refs.push(match[1]);
  }
  return refs;
}

test("skill markdown only references documents inside its own skill directory", () => {
  const violations = [];

  for (const file of walkMarkdown(skillsDir)) {
    const skillRoot = skillRootFor(file);
    const where = relative(repoRoot, file);

    for (const ref of extractMarkdownRefs(readFileSync(file, "utf8"))) {
      if (urlScheme.test(ref)) {
        continue;
      }
      const target = resolve(dirname(file), ref);
      if (target !== skillRoot && !target.startsWith(skillRoot + sep)) {
        violations.push(`${where}: \`${ref}\` resolves outside its skill directory`);
      } else if (!existsSync(target)) {
        violations.push(`${where}: \`${ref}\` does not exist within the skill`);
      }
    }
  }

  assert.deepEqual(
    violations,
    [],
    `Skill docs must be self-contained so they survive \`npx skills\` sync; ` +
      `use an absolute URL for upstream docs instead:\n${violations.join("\n")}`,
  );
});
