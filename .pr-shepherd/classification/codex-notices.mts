import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment") return null;
  const author = item.author.toLowerCase().replace(/\[bot\]$/, "");
  if (author !== "chatgpt-codex-connector") return null;

  const body = item.body.trim();
  const quotaNotice =
    /^You have reached your Codex usage limits for security reviews\. Please try again later\.$/.test(
      body,
    ) ||
    /^You have reached your Codex usage limits for code reviews\. You can see your limits in the \[Codex usage dashboard\]\(https:\/\/chatgpt.com\/codex\/cloud\/settings\/usage\)\.\nTo continue using code reviews, add credits to your account and enable them for code reviews in your \[settings\]\(https:\/\/chatgpt.com\/codex\/cloud\/settings\/code-review\)\.$/.test(
      body,
    );
  // Strip only the trailing About block; extra findings after it stay visible.
  const content = body
    .replace(
      /\n\n<details> <summary>ℹ️ About Codex (?:in GitHub|security reviews in GitHub)<\/summary>[\s\S]*?<\/details>$/,
      "",
    )
    .trim();
  const activitySummary =
    /^<!-- codex-pull-request-review-summary -->\n(?:<!-- codex-security-review:v1 \{[^\n]*\} -->\n)?\n?## Codex Review Summary\n\nThis comment shows the latest Codex review activity on this pull request\.\n\n\| Review \| Status \| Commit \| Review trigger \|\n\| --- \| --- \| --- \| --- \|\n(?:\|[^\n]+\|\n?)+$/.test(
      content,
    );
  const cleanReview =
    /^Codex Review: Didn't find any major issues\. (?:Bravo\.|Breezy!|Hooray!|:\+1:)\n\n\*\*Reviewed commit:\*\* `[a-f0-9]+`$/.test(
      content,
    );
  const cleanSecurity =
    /^### 🛡️ Codex Security Review(?: · _Automatically triggered_)?\n\nSecurity review completed\. No security issues were found in this pull request\.\n\n\*\*Reviewed commit:\*\* `[a-f0-9]+`\n\n\[View security finding report\]\(https:\/\/chatgpt.com\/codex\/cloud\/tasks\/[a-zA-Z0-9_]+\)\n\n_Only the user who started this review can view the report in Codex\._$/.test(
      content,
    );
  if (!quotaNotice && !activitySummary && !cleanReview && !cleanSecurity) return null;

  return {
    suppress: true,
    reason: "Ignore Codex quota, review activity, and no-findings notices",
  };
};

export default rule;
