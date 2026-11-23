import type {
  TestFixtureSizeCapEachTableMode,
  TestFixtureSizeCapOptions,
} from "no-mistakes";

export const modes: TestFixtureSizeCapEachTableMode[] = [
  "per-element",
  "all-elements",
];

export const fixtureSizeCap: TestFixtureSizeCapOptions = {
  threshold: 1000,
  exemptions: [1024],
  eachCalls: ["it.each"],
  eachTableMode: "all-elements",
  scopes: { tests: ["**/*.test.ts"], helpers: ["helpers/**/*.ts"] },
};

// @ts-expect-error Only the supported table modes are public.
export const invalidMode: TestFixtureSizeCapEachTableMode = "everything";
