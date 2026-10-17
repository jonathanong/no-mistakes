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

function pathString(pattern) {
  return text(256, pattern);
}

function segments(maxItems) {
  return { type: "array", minItems: 1, maxItems, items: nonStar() };
}

function starThen(minItems, maxItems) {
  return {
    type: "array",
    minItems,
    maxItems,
    items: [{ enum: ["*"] }],
    additionalItems: nonStar(),
  };
}

function pathSchema(pattern, starMin, plainMax, starMax) {
  return {
    anyOf: [pathString(pattern), segments(plainMax), starThen(starMin, starMax)],
  };
}

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
            chain({ path: pathSchema(OPEN_PATH, 2, MAX_SEGMENTS, MAX_SEGMENTS) }, [
              "path",
              "message",
            ]),
            chain(
              {
                path: pathSchema(OPEN_PATH, 2, MAX_SEGMENTS, MAX_SEGMENTS),
                receiver: { type: "string", enum: ["*"] },
              },
              ["path", "message", "receiver"],
            ),
            chain(
              {
                path: pathSchema(NAMED_PATH, 1, MAX_SEGMENTS - 1, MAX_SEGMENTS),
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

function compile(entry) {
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

module.exports = { schema, compile };
