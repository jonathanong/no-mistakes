import React from "react";
import type { ZodType } from "zod";
import "./setup.mts";
export { helper } from "@scope/pkg/helpers";

type Remote = import("remote-types").Remote;

export async function load() {
  await import("next/dynamic");
  const fs = require("node:fs");
  const resolved = require.resolve("@scope/pkg/register");
  const local = require("./local.cjs");
  return { fs, local, resolved };
}

export type { Remote, ZodType };
