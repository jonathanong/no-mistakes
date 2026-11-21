const { readFileSync, existsSync } = require("node:fs");
const { dirname, basename, join, normalize } = require("node:path");

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
      push("attr", buf, start);
      continue;
    }
    if (c === "'") {
      const start = line;
      if (i + 1 < n && /[A-Za-z_]/.test(text[i + 1])) {
        let j = i + 2;
        while (j < n && /[A-Za-z0-9_]/.test(text[j])) j += 1;
        push("lifetime", text.slice(i, j), start);
        i = j;
        continue;
      }
      let j = i + 1;
      if (text[j] === "\\") j += 2;
      else if (j < n) j += 1;
      if (text[j] === "'") j += 1;
      push("char", text.slice(i, j), start);
      i = j;
      continue;
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
    let end = j + 1;
    if (text[end] === "\\") end += 2;
    else end += 1;
    if (text[end] === "'") end += 1;
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

function shardOfAttrs(attrs) {
  let shards = new Set(ALL);
  let explicit = null;
  for (const attr of attrs) {
    const normalized = attr.replace(/\s+/g, "");
    const match = normalized.match(/^#\[cfg\(all\(test,any\(nm_shard_all,nm_shard_([012])\)\)\)\]$/);
    if (match) {
      explicit = Number(match[1]);
      shards = new Set([...shards].filter((id) => id === explicit));
      continue;
    }
    if (normalized === "#[cfg(test)]") continue;
    if (normalized.startsWith("#[cfg(not(test))")) return { shards: null, explicit };
    if (normalized.includes("nm_shard_")) throw new Error(`unrecognized shard attr: ${attr}`);
  }
  return { shards, explicit };
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

function skipItem(tokens, index, end) {
  let brace = 0;
  let paren = 0;
  let started = false;
  let i = index;
  while (i < end) {
    const kind = tokens[i].kind;
    if (kind === "(" || kind === "[") {
      paren += 1;
      started = true;
    } else if (kind === ")" || kind === "]") paren -= 1;
    else if (kind === "{") {
      brace += 1;
      started = true;
    } else if (kind === "}") {
      if (brace === 0 && paren === 0) return Math.max(i + 1, index + 1);
      brace -= 1;
      if (brace === 0 && paren === 0) return i + 1;
    } else if (kind === ";" && brace === 0 && paren === 0) return i + 1;
    else if (started && brace === 0 && paren === 0 && kind === "ident") return i;
    i += 1;
  }
  return Math.max(i, index + 1);
}

function libraryShardCoverage(root) {
  const nested = [];
  const droppedTests = [];
  const unresolved = [];
  const seen = new Set();
  let testCount = 0;
  const includeStack = [];

  function walkFile(file, shards, childDir, included) {
    if (!existsSync(file)) {
      unresolved.push(file);
      return;
    }
    tokenFile = file;
    const tokens = tokenize(readFileSync(file, "utf8"));
    const dir = childDir ?? initialChildDir(file);
    if (included) includeStack.push(file);
    try {
      walkTokens(tokens, 0, tokens.length, new Set(shards), file, dir);
    } finally {
      if (included) includeStack.pop();
    }
  }

  function walkTokens(tokens, start, end, shards, physical, childDir) {
    let i = start;
    while (i < end) {
      if (tokens[i].kind === "attr" && tokens[i].value.startsWith("#!")) {
        i += 1;
        continue;
      }
      const attrs = [];
      while (i < end && tokens[i].kind === "attr") {
        attrs.push(tokens[i].value);
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
          i += 1;
          if (i < end && tokens[i].kind === "(") {
            let depth = 0;
            while (i < end) {
              if (tokens[i].kind === "(") depth += 1;
              else if (tokens[i].kind === ")") {
                depth -= 1;
                if (depth === 0) {
                  i += 1;
                  break;
                }
              }
              i += 1;
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
          if (!resolved || !existsSync(resolved.file)) unresolved.push(`${physical}:${nameTok.line}:${nameTok.value}`);
          else {
            const edge = `${resolved.file}|${[...child].sort().join("")}|${resolved.childDir}`;
            if (!seen.has(edge)) {
              seen.add(edge);
              walkFile(resolved.file, child, resolved.childDir, false);
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
                walkTokens(tokens, bodyStart, j, child, physical, join(childDir, nameTok.value));
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
      if (kind === "include" && tokens[i + 1] && tokens[i + 1].kind === "op" && tokens[i + 1].value === "!") {
        if (tokens[i + 2] && tokens[i + 2].kind === "(" && tokens[i + 3] && tokens[i + 3].kind === "str") {
          const match = tokens[i + 3].value.match(/"([^"]+)"/);
          if (match && parsed.shards) {
            const inc = normalize(join(dirname(physical), match[1]));
            const child = new Set([...shards].filter((id) => parsed.shards.has(id)));
            if (!includeStack.includes(inc)) walkFile(inc, child, dirname(inc), true);
          }
          i += 5;
          if (i < end && tokens[i].kind === ";") i += 1;
          continue;
        }
      }
      if (kind === "fn" && parsed.shards) {
        const isTest = attrs.some((attr) => attr.replace(/\s+/g, "") === "#[test]");
        if (isTest) {
          testCount += 1;
          const child = new Set([...shards].filter((id) => parsed.shards.has(id)));
          if (child.size === 0) {
            droppedTests.push({
              file: physical,
              line: tokens[i].line,
              name: tokens[i + 1] ? tokens[i + 1].value : "?",
            });
          }
        }
      }
      i = skipItem(tokens, i, end);
    }
  }

  walkFile(join(root, "crates/no-mistakes/src/lib.rs"), ALL, null, false);
  return { nested, droppedTests, unresolved, testCount };
}

module.exports = { libraryShardCoverage };
