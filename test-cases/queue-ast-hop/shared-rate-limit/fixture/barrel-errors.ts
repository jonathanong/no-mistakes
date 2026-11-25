import { Worker } from "bullmq";
import DefaultRateLimit, {
  ConstAlias,
  LocalReexport,
  MemberAlias,
  NsMemberAlias,
  NsWorkerMemberAlias,
  Queue as QueueFromBarrel,
  RateLimitError,
  ReexportedNs,
  RenamedPause,
  Worker as BarrelWorker,
  queueNs,
} from "./barrel";
import * as barrelNs from "./barrel";
import { RateLimitError as StarRateLimitError } from "./star-barrel";
import DefaultCustomError, {
  CustomDeclaredError,
  CustomFnRateLimitError,
  CustomHelper,
  ExprClassError,
  RateLimitError as LocalCustomRateLimitError,
} from "./custom-error";
import DefaultCustomFn, {
  RateLimitError as StarCustomRateLimitError,
} from "./star-custom-barrel";
import {
  ComputedHelper,
  ExternalReexport,
  OpaquePropHelper,
  RateLimitError as OpaqueRateLimitError,
  SpreadHelper,
} from "./opaque-barrel";
import { RateLimitError as CyclicRateLimitError } from "./cycle-barrel";
import { NotFoundError } from "http-errors";
import { RateLimitError as UnknownPkgError } from "unknown-rate-limit-pkg";

declare const ambientHelper: { RateLimitError(): Error };
declare const GlobalUnknown: { RateLimitError(): Error };
declare const GlobalNs: { Worker: { RateLimitError(): Error } };

class LocalBareError {}
class LocalSubError extends LocalBareError {}
const LocalExprClass = class extends LocalBareError {};
const LocalAliasError = RateLimitError;
const LocalFnHelper = { RateLimitError: () => new Error("local") };
const LocalEmptyHelper = {};
const LocalSpreadHelper = { ...ambientHelper };

export const barrelWorker = new Worker("widgets", async () => {
  if (Math.random()) throw new RateLimitError();
  if (Math.random()) throw new RenamedPause();
  if (Math.random()) throw new LocalReexport();
  if (Math.random()) throw new ConstAlias();
  if (Math.random()) throw MemberAlias();
  if (Math.random()) throw NsMemberAlias();
  if (Math.random()) throw NsWorkerMemberAlias();
  if (Math.random()) throw new DefaultRateLimit();
  if (Math.random()) throw BarrelWorker.RateLimitError();
  if (Math.random()) throw queueNs.RateLimitError();
  if (Math.random()) throw queueNs.Worker.RateLimitError();
  if (Math.random()) throw barrelNs.RateLimitError();
  if (Math.random()) throw barrelNs.Worker.RateLimitError();
  if (Math.random()) throw ReexportedNs.RateLimitError();
  if (Math.random()) throw new LocalAliasError();
  throw new StarRateLimitError();
});

export const customAndUnrelatedWorker = new Worker("widgets", async () => {
  if (Math.random()) throw new LocalCustomRateLimitError();
  if (Math.random()) throw CustomFnRateLimitError();
  if (Math.random()) throw CustomDeclaredError();
  if (Math.random()) throw new DefaultCustomError();
  if (Math.random()) throw new ExprClassError();
  if (Math.random()) throw new CustomHelper.RateLimitError();
  if (Math.random()) throw new StarCustomRateLimitError();
  if (Math.random()) throw DefaultCustomFn();
  if (Math.random()) throw new QueueFromBarrel("q");
  if (Math.random()) throw new LocalSubError();
  if (Math.random()) throw new LocalExprClass();
  if (Math.random()) throw LocalFnHelper.RateLimitError();
  if (Math.random()) throw (LocalEmptyHelper as { RateLimitError(): Error }).RateLimitError();
  throw new NotFoundError();
});

export const untraceableWorker = new Worker("items", async () => {
  if (Math.random()) throw new OpaqueRateLimitError();
  if (Math.random()) throw new ExternalReexport();
  if (Math.random()) throw new UnknownPkgError();
  if (Math.random()) throw new CyclicRateLimitError();
  if (Math.random()) throw SpreadHelper.RateLimitError();
  if (Math.random()) throw ComputedHelper.RateLimitError();
  if (Math.random()) throw OpaquePropHelper.RateLimitError();
  if (Math.random()) throw LocalSpreadHelper.RateLimitError();
  if (Math.random()) throw GlobalUnknown.RateLimitError();
  if (Math.random()) throw GlobalNs.Worker.RateLimitError();
  throw ambientHelper.RateLimitError();
});
