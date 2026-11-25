import { Worker } from "bullmq";
import DefaultRateLimit, {
  ConstAlias,
  LocalReexport,
  MemberAlias,
  NsMemberAlias,
  NsWorkerMemberAlias,
  RateLimitError,
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
  RateLimitError as LocalCustomRateLimitError,
} from "./custom-error";
import {
  ExternalReexport,
  RateLimitError as OpaqueRateLimitError,
} from "./opaque-barrel";
import { RateLimitError as CyclicRateLimitError } from "./cycle-barrel";
import { NotFoundError } from "http-errors";
import { RateLimitError as UnknownPkgError } from "unknown-rate-limit-pkg";

declare const ambientHelper: { RateLimitError(): Error };

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
  throw new StarRateLimitError();
});

export const customAndUnrelatedWorker = new Worker("widgets", async () => {
  if (Math.random()) throw new LocalCustomRateLimitError();
  if (Math.random()) throw CustomFnRateLimitError();
  if (Math.random()) throw CustomDeclaredError();
  if (Math.random()) throw new DefaultCustomError();
  if (Math.random()) throw new CustomHelper.RateLimitError();
  throw new NotFoundError();
});

export const untraceableWorker = new Worker("items", async () => {
  if (Math.random()) throw new OpaqueRateLimitError();
  if (Math.random()) throw new ExternalReexport();
  if (Math.random()) throw new UnknownPkgError();
  if (Math.random()) throw new CyclicRateLimitError();
  throw ambientHelper.RateLimitError();
});
