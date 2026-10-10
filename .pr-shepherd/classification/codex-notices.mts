import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment") return null;
  const author = item.author.toLowerCase().replace(/\[bot\]$/, "");
  if (author !== "chatgpt-codex-connector") return null;

  const quotaNotice =
    /^You have reached your Codex usage limits for (?:code|security) reviews\./.test(item.body);
  const activitySummary =
    item.body.startsWith("<!-- codex-pull-request-review-summary -->\n") &&
    item.body.includes("\n## Codex Review Summary\n") &&
    item.body.includes("This comment shows the latest Codex review activity on this pull request.");
  if (!quotaNotice && !activitySummary) return null;

  return {
    suppress: true,
    reason: "Ignore Codex quota and review activity notices",
  };
};

export default rule;
