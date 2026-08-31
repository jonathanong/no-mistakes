import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment") {
    return null;
  }

  const author = item.author.toLowerCase().replace(/\[bot\]$/, "");
  if (author !== "codecov") {
    return null;
  }

  return {
    suppress: true,
    autoResolve: true,
    reason: "Ignore automated Codecov report comments",
  };
};

export default rule;
