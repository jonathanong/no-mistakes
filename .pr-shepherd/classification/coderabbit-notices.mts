import type { ClassifyRule } from "pr-shepherd/classify";

const rule: ClassifyRule = (item) => {
  if (item.kind !== "pr-comment") return null;
  const author = item.author.toLowerCase().replace(/\[bot\]$/, "");
  if (author !== "coderabbitai") return null;

  // The same mutable comment can later contain a walkthrough or findings.
  // Accept only the notice envelope, including its known UI footer blocks.
  const body = item.body
    .trim()
    .replace(/\n<!-- tips_start -->[\s\S]*?<!-- tips_end -->$/, "")
    .trim()
    .replace(/\n<!-- autopilot:start -->[\s\S]*?<!-- autopilot:end -->$/, "")
    .trim()
    .replace(/^<!-- This is an auto-generated comment: summarize by coderabbit.ai -->\s*/, "")
    .replace(/^<!-- review_stack_entry_start -->[\s\S]*?<!-- review_stack_entry_end -->\s*/, "");
  const notice = body.match(
    /^<!-- This is an auto-generated comment: (rate limited|skip review|review in progress) by coderabbit.ai -->\n([\s\S]*?)\n<!-- end of auto-generated comment: \1 by coderabbit.ai -->$/,
  );
  const quotedNotice = notice?.[2].trim();
  const onlyQuotedLines = quotedNotice
    ?.split("\n")
    .every((line) => !line.trim() || line.startsWith(">"));
  const recognizedNotice =
    quotedNotice &&
    (quotedNotice.includes("> ## Review limit reached\n") ||
      quotedNotice.includes("> ## Draft PR not reviewed\n") ||
      (quotedNotice.includes("> ## Review skipped\n") &&
        quotedNotice.includes("> Too many files!\n")) ||
      quotedNotice.includes(
        "> Currently processing new changes in this PR. This may take a few minutes, please wait...",
      ));
  const reply =
    /^<!-- This is an auto-generated reply by CodeRabbit -->\n<!-- CodeRabbit review command invocation: v2:[a-f0-9]+ -->\n<details>\n<summary>⚠️ Action not completed<\/summary>\n\n(?:Review rate limited\.|Review skipped: \d+ files exceed the limit of \d+\.)\n\n(?:> Note: CodeRabbit is an incremental review system and does not re-review already reviewed commits\. This command is applicable only when automatic reviews are paused\.|---\n\nYour included review limit is currently reached under our \[Fair Usage Limits Policy\]\(https:\/\/docs.coderabbit.ai\/management\/plans#fair-usage-limits-policy\)\. This review may still proceed through usage-based billing if eligible\. Your next included review will be available in \d+ minutes?\.)\n\n<\/details>$/.test(
      body,
    );
  if (!(onlyQuotedLines && recognizedNotice) && !reply) return null;

  return {
    suppress: true,
    reason: "Ignore CodeRabbit progress, review-limit, and draft notices",
  };
};

export default rule;
