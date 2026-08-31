import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment" || item.author.toLowerCase() !== "codecov") {
    return null;
  }

  return {
    suppress: true,
    autoResolve: true,
    reason: "Ignore automated Codecov report comments",
  };
};

export default rule;
