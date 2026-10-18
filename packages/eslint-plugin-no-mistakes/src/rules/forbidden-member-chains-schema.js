"use strict";

const MAX_SEGMENTS = 16;
// A segment is non-empty and is not exactly "*". The star is only a leading receiver.
const SEGMENT = "(?!\\*(?:\\.|$))[^.]+";
const OPEN_PATH = `^(?:\\*(?:\\.${SEGMENT}){1,15}|${SEGMENT}(?:\\.${SEGMENT}){0,15})$`;
// A named receiver replaces a leading "*" and still counts toward 16 segments.
const NAMED_PATH = `^(?:\\*|\\*(?:\\.${SEGMENT}){1,15}|${SEGMENT}(?:\\.${SEGMENT}){0,14})$`;

function text(maxLength, pattern) {
  return pattern
    ? { type: "string", minLength: 1, maxLength, pattern }
    : { type: "string", minLength: 1, maxLength };
}

function nonStar() {
  return text(128, "^(?!\\*$)[\\s\\S]+$");
}

// Segment rules run in assertPath so a bad path names the segment. A JSON Schema
// anyOf only says the value matched none of the alternatives.
const pathValue = {
  anyOf: [
    { type: "string", minLength: 1, maxLength: 256 },
    {
      type: "array",
      minItems: 1,
      maxItems: 32,
      items: { type: "string", minLength: 1, maxLength: 128 },
    },
  ],
};

function chain(properties, required) {
  return {
    type: "object",
    additionalProperties: false,
    required,
    properties: { message: text(512), ...properties },
  };
}

const schema = [
  {
    type: "object",
    additionalProperties: false,
    properties: {
      chains: {
        type: "array",
        maxItems: 64,
        items: {
          anyOf: [
            chain({ path: pathValue }, ["path", "message"]),
            chain(
              {
                path: pathValue,
                receiver: { type: "string", enum: ["*"] },
              },
              ["path", "message", "receiver"],
            ),
            chain(
              {
                path: pathValue,
                receiver: nonStar(),
              },
              ["path", "message", "receiver"],
            ),
          ],
        },
      },
    },
  },
];

function shownPath(path) {
  return Array.isArray(path) ? path.join(".") : String(path);
}

function assertPath(path, named) {
  const parts = Array.isArray(path) ? path : String(path).split(".");
  const shown = shownPath(path);
  const empty = parts.findIndex((part) => part === "");
  if (empty !== -1) throw new Error(`empty segment at index ${empty} in '${shown}'`);
  const star = parts.findIndex((part, index) => part === "*" && index !== 0);
  if (star !== -1) {
    throw new Error(`'*' segment at index ${star} in '${shown}'; '*' is only a leading segment`);
  }
  if (typeof path === "string") {
    const pattern = new RegExp(named ? NAMED_PATH : OPEN_PATH);
    if (pattern.test(path)) return;
    const limit = named && parts[0] !== "*" ? MAX_SEGMENTS - 1 : MAX_SEGMENTS;
    if (parts.length > limit) {
      throw new Error(`path '${shown}' has ${parts.length} segments; the maximum is ${limit}`);
    }
    throw new Error(`malformed path '${shown}'`);
  }
  const leadingStar = parts[0] === "*";
  const limit = leadingStar ? MAX_SEGMENTS : named ? MAX_SEGMENTS - 1 : MAX_SEGMENTS;
  const min = leadingStar ? (named ? 1 : 2) : 1;
  if (parts.length < min) {
    throw new Error(`path '${shown}' has ${parts.length} segments; at least ${min} are required`);
  }
  if (parts.length > limit) {
    throw new Error(`path '${shown}' has ${parts.length} segments; the maximum is ${limit}`);
  }
}

function assertChains(entries) {
  if (!Array.isArray(entries)) return;
  for (const entry of entries) {
    assertPath(entry.path, typeof entry.receiver === "string" && entry.receiver !== "*");
  }
}

function compile(entry) {
  assertPath(entry.path, typeof entry.receiver === "string" && entry.receiver !== "*");
  const parts = Array.isArray(entry.path) ? entry.path.slice() : entry.path.split(".");
  let wildcard = false;
  let body = parts;
  if (body[0] === "*") {
    wildcard = true;
    body = body.slice(1);
  }
  if (entry.receiver === "*") wildcard = true;
  else if (typeof entry.receiver === "string") {
    body = [entry.receiver, ...body];
    wildcard = false;
  }
  return { wildcard, parts: body, message: entry.message };
}

module.exports = { schema, compile, assertChains };
