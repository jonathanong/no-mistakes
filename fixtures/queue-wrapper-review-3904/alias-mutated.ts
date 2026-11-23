import { JOB, STRING } from "./stable";
const alias = JOB;
alias.name = "after-import-alias-write";
STRING = "after-import-string-write";
export { JOB as JOB_ALIAS, STRING as STRING_ALIAS };
