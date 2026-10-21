import { baseTest as test } from "./fixtures";

test.sequential("renamed fixture", () => {});

// A parameter named test is not the imported runner.
function helper(test: { sequential(name: string, fn: () => void): void }) {
  test.sequential("nested shadow", () => {});
}
