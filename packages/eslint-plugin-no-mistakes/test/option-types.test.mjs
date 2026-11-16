import { resolve } from "node:path";
import ts from "typescript";
import { expect, it } from "vitest";
import { __dirname } from "./helpers.mjs";

it("checks the exported executor adapter mode and options", () => {
  const program = ts.createProgram([resolve(__dirname, "option-types.ts")], {
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    target: ts.ScriptTarget.ES2022,
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  expect(
    diagnostics.map((entry) => ts.flattenDiagnosticMessageText(entry.messageText, "\n")),
  ).toEqual([]);
});
