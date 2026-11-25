import {
  RateLimitError as ImportedRateLimit,
  Worker as ImportedWorker,
} from "./barrel-inner";
import * as queueMod from "bullmq";

export {
  RateLimitError,
  InnerPause as RenamedPause,
  Worker,
} from "./barrel-inner";
export { ImportedRateLimit as LocalReexport };
export const ConstAlias = ImportedRateLimit;
export const MemberAlias = ImportedWorker.RateLimitError;
export const NsMemberAlias = queueMod.RateLimitError;
export const NsWorkerMemberAlias = queueMod.Worker.RateLimitError;
export * as queueNs from "./barrel-inner";
export default ImportedRateLimit;
