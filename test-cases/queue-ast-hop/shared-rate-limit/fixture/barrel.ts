import {
  RateLimitError as ImportedRateLimit,
  Worker as ImportedWorker,
} from "./barrel-inner";
import * as queueMod from "bullmq";

export {
  Queue,
  RateLimitError,
  InnerPause as RenamedPause,
  Worker,
} from "./barrel-inner";
export { ImportedRateLimit as LocalReexport, queueMod as ReexportedNs };
export const ConstAlias = ImportedRateLimit;
export const MemberAlias = ImportedWorker.RateLimitError;
export const NsMemberAlias = queueMod.RateLimitError;
export const NsWorkerMemberAlias = queueMod.Worker.RateLimitError;
export * as queueNs from "./barrel-inner";
export default ImportedRateLimit;
