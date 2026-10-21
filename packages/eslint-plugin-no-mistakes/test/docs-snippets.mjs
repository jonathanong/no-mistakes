import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { __dirname, plugin } from "./helpers.mjs";

export const docsDir = resolve(__dirname, "../../../docs/eslint-rules");

// Open PR #2406 is rewriting these call-binding docs and fixtures. Execute
// their snippets only when they already pass; do not edit those pages here.
export const recallAuditRules = new Set([
  "async-call-disposition",
  "async-try-catch-return-await",
  "await-array-methods",
  "gh-api-argument-safety",
  "module-mock-boundary",
  "module-mock-preserve-exports",
  "nextjs-static-fetch-method",
  "nextjs-static-fetch-url",
  "no-argv-main-guard",
  "no-banned-import-outside-allowed-paths",
  "no-global-fetch-outside-helper",
  "no-inline-noop-promise-catch",
  "require-options-on-imported-call",
  "server-require-nullable-fetch-wrapper",
]);

const FENCE = /```([A-Za-z0-9_-]*)[^\n]*\n([\s\S]*?)```/g;
const SOURCE_LANGS = new Set(["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"]);

export function rulePages() {
  return Object.keys(plugin.rules)
    .sort()
    .map((ruleId) => {
      const file = `${ruleId}.md`;
      return { ruleId, file, body: readFileSync(resolve(docsDir, file), "utf8") };
    });
}

export function listedDocFiles() {
  return readdirSync(docsDir)
    .filter((name) => name.endsWith(".md") && name !== "README.md")
    .sort();
}

export function headingSection(body, title) {
  const start = body.search(new RegExp(`^## ${title}\\s*$`, "m"));
  if (start < 0) return "";
  const rest = body.slice(start + 3 + title.length);
  const end = rest.search(/^## /m);
  return end < 0 ? rest : rest.slice(0, end);
}

export function sourceFences(section) {
  const fences = [];
  for (const match of section.matchAll(FENCE)) {
    const lang = (match[1] || "ts").toLowerCase();
    if (!SOURCE_LANGS.has(lang)) continue;
    fences.push({ lang, code: match[2].replace(/\n$/, "") });
  }
  return fences;
}

export function ruleOption(optionsSection, ruleId) {
  const key = `no-mistakes/${ruleId}`;
  const objects = [...optionsSection.matchAll(/```(?:js|json)[^\n]*\n([\s\S]*?)```/g)].map(
    (match) => parseObject(match[1]),
  );
  for (const value of objects) {
    if (!value || typeof value !== "object") continue;
    const fromRules = value.rules?.[key];
    const fromRoot = value[key];
    const entry = fromRules ?? fromRoot;
    if (Array.isArray(entry) && entry.length > 1 && entry[1] && typeof entry[1] === "object") {
      return entry[1];
    }
    if (looksLikeOptions(value, key)) return value;
  }
  return undefined;
}

export function snippetFilename(code, lang) {
  const named = code.match(/^\s*\/\/\s*([\w./-]+\.(?:[cm]?[jt]sx?))\s*$/m);
  if (named) return named[1];
  if (lang === "tsx" || lang === "jsx") return "fixture.tsx";
  if (lang === "mts") return "fixture.mts";
  if (lang === "cts") return "fixture.cts";
  if (lang === "mjs") return "fixture.mjs";
  if (lang === "cjs") return "fixture.cjs";
  return lang.startsWith("ts") ? "fixture.ts" : "fixture.js";
}

function parseObject(source) {
  try {
    return Function(`"use strict"; return (${source});`)();
  } catch {
    return undefined;
  }
}

function looksLikeOptions(value, key) {
  if (value.files || value.rules || value[key]) return false;
  const keys = Object.keys(value);
  return keys.length > 0 && keys.every((name) => /^[A-Za-z][A-Za-z0-9]*$/.test(name));
}
