import { crawl, crawlUnawaited } from "./run";

async function defaultImpl(_entries: unknown, _options: unknown) {
  return undefined;
}

// The copy is a const in this file. run.ts awaits the same property.
export async function start(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await crawl(dependencies);
}

async function createDependencies(options: any) {
  return {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
}

export async function startBuilt(options: any) {
  const dependencies = await createDependencies(options);
  await crawl(dependencies);
}

function createSync(options: any) {
  return {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
}

export async function startInline(options: any) {
  await crawl(createSync(options));
}

export async function startDropped(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  crawl(dependencies);
}

export async function startUnawaitedCall(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await crawlUnawaited(dependencies);
}

export async function startMutated(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  dependencies.enqueueBulkCrawlReferralLinks = defaultImpl;
  await crawl(dependencies);
}

export async function startRhs(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: defaultImpl ?? options.enqueueBulkCrawlReferralLinks,
  };
  await crawl(dependencies);
}

export async function startUnknown(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await unknownCrawl(dependencies);
}

export function startEscape(options: any) {
  return createSync(options);
}

export async function startForgot(options: any) {
  const dependencies = createDependencies(options);
  await crawl(dependencies);
}

function createEither(options: any, flag: boolean) {
  const other = async () => 1;
  if (flag) {
    return {
      enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
      other,
    };
  }
  return {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other,
  };
}

export async function startEither(options: any, flag: boolean) {
  await crawl(createEither(options, flag));
}

export async function startAlias(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  await callback([], {});
}

export async function startAliasDropped(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  callback([], {});
}

export async function startAliasOptional(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  await callback?.([], {});
}

export async function startAliasMember(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  void callback.name;
  await callback([], {});
}

export async function startLocal(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await dependencies.enqueueBulkCrawlReferralLinks([], {});
}

export async function startRead(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    label: "crawl",
  };
  void dependencies.label;
  await crawl(dependencies);
}

export async function startSafeSibling(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other: async () => 1,
  };
  await dependencies.other();
  await crawl(dependencies);
}

export async function startChained(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  void dependencies.label.extra;
  await crawl(dependencies);
}

export async function startSibling(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other: unknownHelper,
  };
  await dependencies.other();
  await crawl(dependencies);
}

export async function startFloat(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  dependencies.enqueueBulkCrawlReferralLinks([], {});
  await crawl(dependencies);
}

export async function startMixed(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other: async () => 1,
  };
  await crawl(dependencies);
  return dependencies;
}

export async function startOptional(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await dependencies.enqueueBulkCrawlReferralLinks?.([], {});
}

export async function startProto(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  void dependencies.__proto__;
  await crawl(dependencies);
}

export async function startDynamic(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  void dependencies[dynamicKey];
  await crawl(dependencies);
}

export async function startCallObject(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await dependencies([], {});
}

export async function startOptionalCall(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await crawl?.(dependencies);
}

// The copied object is never read. Using the binding would hide that hole.
export function startUnread(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  return options;
}

export async function startVoid(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  void dependencies;
}

export async function startOnlyRead(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    label: "crawl",
  };
  void dependencies.label;
}

export function startTwice(options: any, flag: boolean) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other: async () => 1,
  };
  if (flag) return dependencies;
  return dependencies;
}

// A later await does not repair an earlier return of the same copy.
export async function startReturnFirst(options: any, flag: boolean) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  if (flag) return dependencies;
  await crawl(dependencies);
}

// `missing` is not on the copy. An optional call cannot be the copied callback.
export async function startOptionalAbsent(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  await dependencies.missing?.();
  await crawl(dependencies);
}

// Optional still invokes a sibling the copy actually has, so an unsafe one stays unproven.
export async function startOptionalUnsafe(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    other: unknownHelper,
  };
  await dependencies.other?.();
  await crawl(dependencies);
}

// The safe siblings differ by path. A key is safe only when every returned copy says so.
function createDivergent(options: any, flag: boolean) {
  if (flag) {
    return {
      enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
      other: async () => 1,
    };
  }
  return {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
    extra: async () => 1,
  };
}

export async function startDivergent(options: any, flag: boolean) {
  await crawl(createDivergent(options, flag));
}

// One use returns the copy and another awaits it. Mixed uses are unproven.
export async function startParamReturnFirst(options: any, flag: boolean) {
  if (flag) return createSync(options);
  await crawl(createSync(options));
}

// Awaiting the copy first does not excuse a later return of that same object.
export async function startParamAdoptThenReturn(options: any) {
  await crawl(createSync(options));
  return createSync(options);
}

// The await sits in a closure this function returns. Awaiting the function does not run it.
export function startReturnedClosure(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  return async () => {
    await callback([], {});
  };
}

// Same escape when the copied object, rather than the callback alias, is closed over.
export function startReturnedObjectClosure(options: any) {
  const dependencies = {
    enqueueBulkCrawlReferralLinks: options.enqueueBulkCrawlReferralLinks ?? defaultImpl,
  };
  return async () => {
    await dependencies.enqueueBulkCrawlReferralLinks([], {});
  };
}

// Invoking a stored closure later is still an escape: the await is not in this function.
export async function startStoredClosure(options: any) {
  const callback = options.enqueueBulkCrawlReferralLinks ?? defaultImpl;
  const run = async () => {
    await callback([], {});
  };
  await run();
}

declare function unknownCrawl(dependencies: unknown): Promise<void>;
declare function unknownHelper(): Promise<void>;
declare const dynamicKey: string;
