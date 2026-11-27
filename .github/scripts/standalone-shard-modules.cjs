const { readFileSync, existsSync, readdirSync } = require("node:fs");
const { dirname, basename, join, normalize, relative } = require("node:path");

// #[test] attributes rustc does not compile into the library or the binary.
// Name a path only after its module is cfg-off for every shard, or after
// confirming the file has no `mod` declaration at all.
const UNCOMPILED_SRC = [
  "executable_file_tests.rs",
  "flow_query_tests/prepared_config.rs",
  "codebase/rules/tsconfig_gate_coverage/workflow/conditions/inputs/needs_tests.rs",
];

const ALL = new Set([0, 1, 2]);
let tokenFile = "";

function tokenize(text) {
  const tokens = [];
  let i = 0;
  const n = text.length;
  let line = 1;
  const push = (kind, value, startLine) => tokens.push({ kind, value, line: startLine });
  while (i < n) {
    const c = text[i];
    if (c === "\n") {
      line += 1;
      i += 1;
      continue;
    }
    if (c === " " || c === "\t" || c === "\r") {
      i += 1;
      continue;
    }
    if (c === "/" && text[i + 1] === "/") {
      i += 2;
      while (i < n && text[i] !== "\n") i += 1;
      continue;
    }
    if (c === "/" && text[i + 1] === "*") {
      i += 2;
      while (i + 1 < n && !(text[i] === "*" && text[i + 1] === "/")) {
        if (text[i] === "\n") line += 1;
        i += 1;
      }
      i = Math.min(n, i + 2);
      continue;
    }
    if (c === "#" && text[i + 1] === "[") {
      const start = line;
      let depth = 0;
      let buf = "";
      while (i < n) {
        if (text[i] === "\n") line += 1;
        buf += text[i];
        if (text[i] === "[") depth += 1;
        else if (text[i] === "]") {
          depth -= 1;
          if (depth === 0) {
            i += 1;
            break;
          }
        }
        i += 1;
      }
      if (depth !== 0) throw new Error(`unclosed attribute in ${tokenFile}`);
      push("attr", buf, start);
      continue;
    }
    if (c === "'") {
      const start = line;
      const charEnd = readCharLiteral(text, i);
      if (charEnd !== null) {
        const chunk = text.slice(i, charEnd);
        push("char", chunk, start);
        line += chunk.split("\n").length - 1;
        i = charEnd;
        continue;
      }
      const lifeEnd = readLifetime(text, i);
      if (lifeEnd !== null) {
        push("lifetime", text.slice(i, lifeEnd), start);
        i = lifeEnd;
        continue;
      }
      throw new Error(`unterminated char literal in ${tokenFile}`);
    }
    const string = readString(text, i);
    if (string) {
      const chunk = text.slice(i, string.end);
      push("str", chunk, line);
      line += chunk.split("\n").length - 1;
      i = string.end;
      continue;
    }
    if (/[A-Za-z_]/.test(c)) {
      let j = i + 1;
      while (j < n && /[A-Za-z0-9_]/.test(text[j])) j += 1;
      push("ident", text.slice(i, j), line);
      i = j;
      continue;
    }
    if ("(){}[];,".includes(c)) {
      push(c, c, line);
      i += 1;
      continue;
    }
    push("op", c, line);
    i += 1;
  }
  return tokens;
}

function readString(text, i) {
  let j = i;
  if (text.startsWith("br", j) || text.startsWith("cr", j)) j += 2;
  else if (text[j] === "b" || text[j] === "c") {
    if (text[j + 1] === "'" || text[j + 1] === '"' || text[j + 1] === "r") j += 1;
    else return null;
  }
  if (text[j] === "'") {
    const end = readCharLiteral(text, j);
    if (end === null) return null;
    return { end };
  }
  let hashes = 0;
  let raw = false;
  if (text[j] === "r") {
    raw = true;
    j += 1;
    while (text[j] === "#") {
      hashes += 1;
      j += 1;
    }
  }
  if (text[j] !== '"') return null;
  j += 1;
  if (raw) {
    const endMark = `"${"#".repeat(hashes)}`;
    const at = text.indexOf(endMark, j);
    if (at < 0) throw new Error(`unterminated raw string in ${tokenFile}`);
    return { end: at + endMark.length };
  }
  while (j < text.length) {
    if (text[j] === "\\") {
      j += 2;
      continue;
    }
    if (text[j] === '"') return { end: j + 1 };
    j += 1;
  }
  throw new Error(`unterminated string in ${tokenFile}`);
}

function readCharLiteral(text, i) {
  let j = i + 1;
  if (j >= text.length) return null;
  if (text[j] === "\\") {
    j += 1;
    if (text[j] === "u" && text[j + 1] === "{") {
      j += 2;
      const hex = j;
      while (j < text.length && /[0-9a-fA-F]/.test(text[j])) j += 1;
      if (j === hex || text[j] !== "}") return null;
      j += 1;
    } else if (text[j] === "x") {
      if (!/[0-9a-fA-F]/.test(text[j + 1] || "") || !/[0-9a-fA-F]/.test(text[j + 2] || ""))
        return null;
      j += 3;
    } else if (j < text.length) j += 1;
    else return null;
    if (text[j] !== "'") return null;
    return j + 1;
  }
  const width = text.codePointAt(j) > 0xffff ? 2 : 1;
  if (text[j + width] !== "'") return null;
  return j + width + 1;
}

function readLifetime(text, i) {
  if (!/[A-Za-z_]/.test(text[i + 1] || "")) return null;
  let j = i + 2;
  while (j < text.length && /[A-Za-z0-9_]/.test(text[j])) j += 1;
  return j;
}

function shardOfAttrs(attrs) {
  let shards = new Set(ALL);
  let explicit = null;
  let postgresSource = true;
  for (const attr of attrs) {
    const normalized = attr.replace(/\s+/g, "");
    const match = normalized.match(
      /^#\[cfg\(all\(test,any\(nm_shard_all,nm_shard_([012])(,nm_shard_postgres_source)?\)\)\)\]$/,
    );
    if (match) {
      explicit = Number(match[1]);
      shards = new Set([...shards].filter((id) => id === explicit));
      postgresSource = postgresSource && Boolean(match[2]);
      continue;
    }
    if (normalized === "#[cfg(test)]") continue;
    if (normalized.startsWith("#[cfg(not(test))"))
      return { shards: null, explicit, postgresSource: false };
    if (normalized.includes("nm_shard_")) throw new Error(`unrecognized shard attr: ${attr}`);
  }
  return { shards, explicit, postgresSource };
}

function pathFromAttrs(attrs) {
  let direct = null;
  let testPath = null;
  for (const attr of attrs) {
    const plain = attr.match(/#\[path\s*=\s*"([^"]+)"\]/);
    if (plain) direct = plain[1];
    const cfg = attr.match(/#\[cfg_attr\(([\s\S]*?),\s*path\s*=\s*"([^"]+)"\)\]/);
    if (!cfg) continue;
    const condition = cfg[1].replace(/\s+/g, "");
    if (condition.startsWith("not(test)")) continue;
    if (condition.includes("test")) testPath = cfg[2];
  }
  return testPath || direct;
}

function resolveMod(physical, childDir, name, pathAttr) {
  if (pathAttr) {
    const found = normalize(join(dirname(physical), pathAttr));
    return { file: found, childDir: dirname(found) };
  }
  const fileRs = join(childDir, `${name}.rs`);
  const modRs = join(childDir, name, "mod.rs");
  if (existsSync(fileRs)) return { file: fileRs, childDir: join(childDir, name) };
  if (existsSync(modRs)) return { file: modRs, childDir: join(childDir, name) };
  return null;
}

function initialChildDir(file) {
  const name = basename(file);
  if (name === "mod.rs" || name === "lib.rs" || name === "main.rs") return dirname(file);
  return join(dirname(file), name.replace(/\.rs$/, ""));
}

function isTestAttr(value) {
  return value.replace(/\s+/g, "") === "#[test]";
}

function skipItem(tokens, index, end) {
  let brace = 0;
  let paren = 0;
  let started = false;
  let tests = 0;
  let i = index;
  const done = (at) => ({ index: at, balanced: brace === 0 && paren === 0, tests });
  while (i < end) {
    const kind = tokens[i].kind;
    if (kind === "attr" && isTestAttr(tokens[i].value)) tests += 1;
    if (kind === "(" || kind === "[") {
      paren += 1;
      started = true;
    } else if (kind === ")" || kind === "]") paren -= 1;
    else if (kind === "{") {
      brace += 1;
      started = true;
    } else if (kind === "}") {
      if (brace === 0 && paren === 0) return done(i + 1);
      brace -= 1;
      if (brace === 0 && paren === 0) return done(i + 1);
    } else if (kind === ";" && brace === 0 && paren === 0) return done(i + 1);
    else if (started && brace === 0 && paren === 0 && kind === "ident") return done(i);
    i += 1;
  }
  return done(Math.max(i, index + 1));
}

function libraryShardCoverage(root, options = {}) {
  const nested = [];
  const droppedTests = [];
  const missingPostgresSourceTests = [];
  const unresolved = [];
  const incomplete = [];
  const seen = new Set();
  const censused = new Set();
  let testCount = 0;
  let postgresSourceTestCount = 0;
  const includeStack = [];
  const excluded = new Set(options.excluded ?? UNCOMPILED_SRC);
  const roots = options.srcDir
    ? [
        {
          crateName: "no-mistakes",
          srcDir: options.srcDir,
          entries: options.entry
            ? [options.entry]
            : [join(options.srcDir, "lib.rs"), join(options.srcDir, "main.rs")],
        },
      ]
    : [
        {
          crateName: "no-mistakes",
          srcDir: join(root, "crates/no-mistakes/src"),
          entries: [
            join(root, "crates/no-mistakes/src/lib.rs"),
            join(root, "crates/no-mistakes/src/main.rs"),
          ],
        },
        {
          crateName: "no-mistakes-invocation",
          srcDir: join(root, "crates/no-mistakes-invocation/src"),
          entries: [join(root, "crates/no-mistakes-invocation/src/lib.rs")],
        },
        {
          crateName: "no-mistakes-postgres-sql",
          srcDir: join(root, "crates/no-mistakes-postgres-sql/src"),
          entries: [join(root, "crates/no-mistakes-postgres-sql/src/lib.rs")],
        },
        {
          crateName: "no-mistakes-rule-kernel",
          srcDir: join(root, "crates/no-mistakes-rule-kernel/src"),
          entries: [join(root, "crates/no-mistakes-rule-kernel/src/lib.rs")],
        },
        {
          crateName: "no-mistakes-ts-facts",
          srcDir: join(root, "crates/no-mistakes-ts-facts/src"),
          entries: [join(root, "crates/no-mistakes-ts-facts/src/lib.rs")],
        },
      ];
  let currentCrate = "no-mistakes";
  let currentSrcDir = roots[0].srcDir;
  const relativeSrc = (file) => relative(currentSrcDir, file).split("\\").join("/");
  const countsTests = (file) =>
    currentCrate !== "no-mistakes" || !excluded.has(relativeSrc(file));
  const isPostgresSourcePath = (modPath) =>
    currentCrate === "no-mistakes-postgres-sql"
      ? modPath.length >= 1 && modPath[0] === "source"
      : modPath.length >= 3 &&
        modPath[0] === "codebase" &&
        modPath[1] === "postgres" &&
        modPath[2] === "source";

  function walkFile(file, shards, inPgSource, modPath, childDir, included) {
    if (!existsSync(file)) {
      unresolved.push(file);
      return;
    }
    tokenFile = file;
    const tokens = tokenize(readFileSync(file, "utf8"));
    const dir = childDir ?? initialChildDir(file);
    const countThis = countsTests(file) && !censused.has(file);
    if (countThis) censused.add(file);
    if (included) includeStack.push(file);
    try {
      walkTokens(
        tokens,
        0,
        tokens.length,
        new Set(shards),
        inPgSource,
        modPath,
        file,
        dir,
        countThis,
      );
    } finally {
      if (included) includeStack.pop();
    }
  }

  function walkTokens(
    tokens,
    start,
    end,
    shards,
    inPgSource,
    modPath,
    physical,
    childDir,
    countThis,
  ) {
    let i = start;
    while (i < end) {
      if (tokens[i].kind === "attr" && tokens[i].value.startsWith("#!")) {
        i += 1;
        continue;
      }
      const attrs = [];
      while (i < end && tokens[i].kind === "attr") {
        attrs.push(tokens[i].value);
        if (countThis && isTestAttr(tokens[i].value)) testCount += 1;
        i += 1;
      }
      if (i >= end) break;
      const parsed = shardOfAttrs(attrs);
      while (
        i < end &&
        tokens[i].kind === "ident" &&
        ["pub", "unsafe", "async", "const", "extern"].includes(tokens[i].value)
      ) {
        if (tokens[i].value === "pub") {
          const pubLine = tokens[i].line;
          i += 1;
          if (i < end && tokens[i].kind === "(") {
            let depth = 0;
            let closed = false;
            while (i < end) {
              if (tokens[i].kind === "(") depth += 1;
              else if (tokens[i].kind === ")") {
                depth -= 1;
                if (depth === 0) {
                  i += 1;
                  closed = true;
                  break;
                }
              }
              i += 1;
            }
            if (!closed) {
              incomplete.push(`${physical}:${pubLine}`);
              return;
            }
          }
          continue;
        }
        i += 1;
      }
      if (i >= end) break;
      const kind = tokens[i].kind === "ident" ? tokens[i].value : null;
      if (kind === "mod" && parsed.shards) {
        const nameTok = tokens[i + 1];
        const next = tokens[i + 2] ? tokens[i + 2].kind : null;
        const child = new Set([...shards].filter((id) => parsed.shards.has(id)));
        const childPgSource = inPgSource && parsed.postgresSource;
        const childPath = [...modPath, nameTok.value];
        if (parsed.explicit !== null && shards.size !== ALL.size) {
          nested.push({
            file: physical,
            line: nameTok.line,
            name: nameTok.value,
            parentShards: [...shards].sort(),
            shard: parsed.explicit,
          });
        }
        if (next === ";") {
          const resolved = resolveMod(physical, childDir, nameTok.value, pathFromAttrs(attrs));
          if (!resolved || !existsSync(resolved.file))
            unresolved.push(`${physical}:${nameTok.line}:${nameTok.value}`);
          else {
            const edge = `${resolved.file}|${[...child].sort().join("")}|${childPgSource ? 1 : 0}|${childPath.join("::")}|${resolved.childDir}`;
            if (!seen.has(edge)) {
              seen.add(edge);
              walkFile(resolved.file, child, childPgSource, childPath, resolved.childDir, false);
            }
          }
          i += 3;
          continue;
        }
        if (next === "{") {
          let depth = 0;
          let j = i + 2;
          const bodyStart = j + 1;
          let closed = false;
          while (j < end) {
            if (tokens[j].kind === "{") depth += 1;
            else if (tokens[j].kind === "}") {
              depth -= 1;
              if (depth === 0) {
                walkTokens(
                  tokens,
                  bodyStart,
                  j,
                  child,
                  childPgSource,
                  childPath,
                  physical,
                  join(childDir, nameTok.value),
                  countThis,
                );
                i = j + 1;
                closed = true;
                break;
              }
            }
            j += 1;
          }
          if (!closed) throw new Error(`unclosed mod ${nameTok.value} in ${physical}`);
          continue;
        }
      }
      if (
        kind === "include" &&
        tokens[i + 1] &&
        tokens[i + 1].kind === "op" &&
        tokens[i + 1].value === "!"
      ) {
        if (
          tokens[i + 2] &&
          tokens[i + 2].kind === "(" &&
          tokens[i + 3] &&
          tokens[i + 3].kind === "str"
        ) {
          const match = tokens[i + 3].value.match(/"([^"]+)"/);
          if (match && parsed.shards) {
            const inc = normalize(join(dirname(physical), match[1]));
            const child = new Set([...shards].filter((id) => parsed.shards.has(id)));
            const childPgSource = inPgSource && parsed.postgresSource;
            if (!includeStack.includes(inc))
              walkFile(inc, child, childPgSource, modPath, dirname(inc), true);
          }
          i += 5;
          if (i < end && tokens[i].kind === ";") i += 1;
          continue;
        }
      }
      if (kind === "fn" && parsed.shards) {
        const isTest = attrs.some((attr) => isTestAttr(attr));
        if (isTest) {
          const child = new Set([...shards].filter((id) => parsed.shards.has(id)));
          const childPgSource = inPgSource && parsed.postgresSource;
          if (child.size === 0) {
            droppedTests.push({
              file: physical,
              line: tokens[i].line,
              name: tokens[i + 1] ? tokens[i + 1].value : "?",
            });
          }
          if (isPostgresSourcePath(modPath)) {
            postgresSourceTestCount += 1;
            if (!childPgSource) {
              missingPostgresSourceTests.push({
                file: physical,
                line: tokens[i].line,
                name: tokens[i + 1] ? tokens[i + 1].value : "?",
              });
            }
          }
        }
      }
      const skipped = skipItem(tokens, i, end);
      if (!skipped.balanced) {
        incomplete.push(`${physical}:${tokens[i].line}`);
        return;
      }
      if (countThis) testCount += skipped.tests;
      i = skipped.index;
    }
  }

  let plainTestCount = 0;
  for (const crate of roots) {
    currentCrate = crate.crateName;
    currentSrcDir = crate.srcDir;
    for (const entry of crate.entries) walkFile(entry, ALL, true, [], null, false);
    plainTestCount += countPlainTests(
      crate.srcDir,
      crate.crateName === "no-mistakes" ? excluded : new Set(),
    );
  }
  return {
    nested,
    droppedTests,
    missingPostgresSourceTests,
    postgresSourceTestCount,
    unresolved,
    incomplete,
    testCount,
    plainTestCount,
  };
}

function countPlainTests(srcDir, excluded) {
  let count = 0;
  const files = [];
  const walk = (dir) => {
    if (!existsSync(dir)) return;
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(".rs")) files.push(full);
    }
  };
  walk(srcDir);
  for (const file of files) {
    const rel = relative(srcDir, file).split("\\").join("/");
    if (excluded.has(rel)) continue;
    tokenFile = file;
    for (const token of tokenize(readFileSync(file, "utf8"))) {
      if (token.kind === "attr" && isTestAttr(token.value)) count += 1;
    }
  }
  return count;
}

module.exports = { libraryShardCoverage };
