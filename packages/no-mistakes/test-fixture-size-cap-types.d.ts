export type TestFixtureSizeCapEachTableMode = "per-element" | "all-elements";

export interface TestFixtureSizeCapOptions {
  threshold?: number;
  exemptions?: number[];
  eachCalls?: string[];
  eachTableMode?: TestFixtureSizeCapEachTableMode;
  scopes?: {
    tests?: string[];
    helpers?: string[];
  };
}
