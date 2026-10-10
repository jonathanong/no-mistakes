import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment") return null;
  const author = item.author.toLowerCase().replace(/\[bot\]$/, "");
  if (author !== "coderabbitai") return null;

  const rateLimited =
    item.body.includes(
      "<!-- This is an auto-generated comment: rate limited by coderabbit.ai -->",
    ) && item.body.includes("> ## Review limit reached\n");
  const draftSkipped =
    item.body.includes(
      "<!-- This is an auto-generated comment: skip review by coderabbit.ai -->",
    ) && item.body.includes("> ## Draft PR not reviewed\n");
  if (!rateLimited && !draftSkipped) return null;

  return {
    suppress: true,
    reason: "Ignore CodeRabbit rate-limit and draft-review notices",
  };
};

export default rule;
