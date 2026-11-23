import path = require("node:path");
import { default as os } from "node:os";
import filesystem = require("node:fs");
import type typeOnlyPath from "path";
import { type Join } from "node:path";

const alias = path;
const { ["join"]: combine } = alias;
const { join: defaultedJoin = fallback } = path;
const { resolve: removed, ...rest } = path;
const key = "join";
const { [key]: computedJoin } = path;

namespace Helpers {
  export const filesystem = { mkdtempSync: (prefix: string) => prefix };
}

import localFilesystem = Helpers.filesystem;

export const joined = alias.join(os.tmpdir(), "alias.json");
export const computed = combine(os.tmpdir(), "computed.json");
export const unique = filesystem.mkdtempSync(combine(os.tmpdir(), "example-"));
export const literal = localFilesystem.mkdtempSync("/tmp/example.json");
export const tagged = String.raw`/tmp/template-input`;

declare const fallback: typeof combine;

defaultedJoin(os.tmpdir(), "unresolved-default.json");
rest.join(os.tmpdir(), "unresolved-rest.json");
computedJoin(os.tmpdir(), "unresolved-key.json");
typeOnlyPath.join(os.tmpdir(), "unresolved-type.json");
Join(os.tmpdir(), "unresolved-specifier.json");
unknownJoin(os.tmpdir(), "unresolved-join.json");
unknownMaker("/tmp/unresolved-maker.json");
