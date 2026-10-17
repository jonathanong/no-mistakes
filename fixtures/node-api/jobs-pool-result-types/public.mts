import type {
  JobsPoolWarning, WithInvocationResult,
} from "../../../packages/no-mistakes/index.js";
import { check, dependencies, ciEnv, testsWhy, writePlanningImpactArtifacts,
  parsePostgresSql, queueEdges, version } from "../../../packages/no-mistakes/index.js";

// Object APIs expose metadata; arrays and scalars retain their exact shapes.
async function contracts() {
  const warning: JobsPoolWarning | undefined = (await check()).jobsPoolWarning;
  const dependencyWarning: JobsPoolWarning | undefined =
    (await dependencies({ files: ["a.mts"] })).jobsPoolWarning;
  const ciWarning: JobsPoolWarning | undefined = (await ciEnv({ root: ".", var: "NODE_ENV" })).jobsPoolWarning;
  const whyWarning: JobsPoolWarning | undefined =
    (await testsWhy({ test: "a.test.mts" })).jobsPoolWarning;
  const planningWarning: JobsPoolWarning | undefined = (await writePlanningImpactArtifacts({
    root: ".", changedFilesManifest: "changes.txt", outputDirectory: "output",
  })).jobsPoolWarning;
  const edges = await queueEdges();
  // @ts-expect-error Array results are not decorated.
  edges.jobsPoolWarning;
  const parsed = await parsePostgresSql({ sql: "select 1" });
  // @ts-expect-error Pure source parsing does not initialize or warn about the pool.
  parsed.jobsPoolWarning;
  const text: WithInvocationResult<string> = await version();
  const array: WithInvocationResult<number[]> = [1];
  const nullable: WithInvocationResult<null> = null;
  return { warning, dependencyWarning, ciWarning, whyWarning, planningWarning, text, array, nullable };
}
void contracts;
