import { check, queueRelated } from "no-mistakes";
import type {
  DependencyBoundOptions,
  FlowOptions,
  GithubActionsEnvPolicyOptions,
  QueueRelatedOptions,
  SymbolsInclude,
  SymbolsMode,
  TestNoLiveRepositoryScanAnalyzer,
  TestNoLiveRepositoryScanOptions,
  TestNoLiveRepositoryScanRuntimeAsset,
  TraversalDirection,
} from "no-mistakes";

const bounds: DependencyBoundOptions = { candidateExclude: ["generated/**"] };
const env: GithubActionsEnvPolicyOptions = { required: { SAMPLE_MODE: null } };
const mode: SymbolsMode = "signature-impact";
const include: SymbolsInclude = "both";
const direction: TraversalDirection = "deps";
const flow: FlowOptions = { target: "example.ts", direction };
const related: QueueRelatedOptions = { files: ["example.ts"], direction };

function liveScanOptions(
  options: TestNoLiveRepositoryScanOptions,
  asset: TestNoLiveRepositoryScanRuntimeAsset,
  analyzer: TestNoLiveRepositoryScanAnalyzer,
) {
  return { ...options, runtimeAssets: [asset], analyzers: [analyzer] };
}

function checkQueueRequest() {
  void queueRelated(related);
  // @ts-expect-error At least one selected file is required.
  void queueRelated({});
  // @ts-expect-error An empty selection is rejected by the native API.
  void queueRelated({ files: [] });
}

export { bounds, check, checkQueueRequest, env, flow, include, liveScanOptions, mode };
