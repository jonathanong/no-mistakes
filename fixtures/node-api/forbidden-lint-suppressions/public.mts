import type {
  ForbiddenLintSuppressionDirective,
  ForbiddenLintSuppressionsOptions,
} from "no-mistakes";

const directives: ForbiddenLintSuppressionDirective[] = [
  "eslint-disable",
  "eslint-disable-line",
  "eslint-disable-next-line",
  "oxlint-disable",
  "oxlint-disable-line",
  "oxlint-disable-next-line",
  "@ts-ignore",
  "@ts-expect-error",
];
const options: ForbiddenLintSuppressionsOptions = {
  files: ["src/**/*.tsx"],
  ruleIds: ["react/purity"],
  rulePrefixes: ["react-hooks-js/"],
  directives,
  banBlanketDisable: true,
  message: "Keep compiler checks enabled.",
};
void options;

// Missing scope must stay a declaration error; there is no repository-wide default.
// @ts-expect-error files is required
const unscoped: ForbiddenLintSuppressionsOptions = { ruleIds: ["react/purity"] };
// @ts-expect-error enable directives are not suppressions selectable by this check
const unsupported: ForbiddenLintSuppressionDirective = "eslint-enable";
void [unscoped, unsupported];
