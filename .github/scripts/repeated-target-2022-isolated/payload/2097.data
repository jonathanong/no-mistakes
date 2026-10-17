export type FetchSourceType =
  | "page"
  | "layout"
  | "loading"
  | "error"
  | "template"
  | "route"
  | "module";

export type CacheKind =
  | "none"
  | "fetch-cache"
  | "fetch-next-revalidate"
  | "fetch-next-tags"
  | "react-cache"
  | "cache"
  | "unstable-cache";

export interface FetchOccurrence {
  path: string;
  rawPath: string;
  method: string;
  file: string;
  line: number;
  side: "server" | "client";
  rsc: boolean;
  cached: boolean;
  cacheKind: CacheKind;
  cachedFunction?: string;
  dynamic: boolean;
  unsupported: boolean;
  functionName?: string;
  conditional: boolean;
  inPromiseAll: boolean;
  errorHandled: boolean;
  sourceType: FetchSourceType;
}

export interface FetchRouteReport {
  route: string;
  file: string;
  apiCalls: FetchOccurrence[];
}

export interface FetchSummary {
  totalRoutes: number;
  routesWithApiCalls: number;
  totalApiCalls: number;
  uniqueApiCalls: number;
  duplicateApiCalls: number;
  dynamicApiCalls: number;
  cachedApiCalls: number;
  clientApiCalls: number;
  serverApiCalls: number;
  rscApiCalls: number;
  conditionalApiCalls: number;
  parallelApiCalls: number;
  errorHandledApiCalls: number;
}

export interface FetchReport {
  summary: FetchSummary;
  routes: FetchRouteReport[];
  duplicates: DuplicateApiCall[];
  unsupported: UnsupportedApiCall[];
}

export interface DuplicateApiCall {
  key: string;
  count: number;
  occurrences: ApiCallOccurrence[];
}

export interface ApiCallOccurrence {
  route: string;
  file: string;
  line: number;
}

export interface UnsupportedApiCall {
  route: string;
  file: string;
  line: number;
  reason: string;
  rawPath: string;
}
