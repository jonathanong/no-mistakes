import test from "./fx";

test.sequential("default import", () => {});

// A parameter named test is not the default import.
function helper(test: { sequential(name: string, fn: () => void): void }) {
  test.sequential("nested shadow", () => {});
}
