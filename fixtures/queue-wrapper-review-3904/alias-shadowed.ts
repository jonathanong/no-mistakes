import { JOB, STRING } from "./stable";
JOB.name = "mutated-before-shadow";
STRING = "mutated-before-shadow";
export { JOB as JOB_ALIAS, STRING as STRING_ALIAS };
function shadow() {
  const JOB = { name: "unrelated-shadow" };
  const STRING = "unrelated-shadow";
}