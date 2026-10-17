export interface ReactComponentFacts {
  name: string;
  file: string;
  environment: "server" | "client" | "shared" | "unknown";
  hasState: boolean;
  hasProps: boolean;
  passesProps: boolean;
  usesMemo: boolean;
  usesContextProvider: boolean;
  usesSuspense: boolean;
  fetches: ReactFetchCall[];
  dependencies: string[];
  children: ReactComponentRef[];
  inheritedFromChildren?: ReactAggregatedFacts;
}

export interface ReactFetchCall {
  file: string;
  exportedName: string | null;
  shape: string | null;
}

export interface ReactComponentRef {
  name: string;
  file: string;
}

export interface ReactAggregatedFacts {
  hasState: boolean;
  hasProps: boolean;
  passesProps: boolean;
  usesMemo: boolean;
  usesContextProvider: boolean;
  usesSuspense: boolean;
  hasFetch: boolean;
}

export interface ReactViolation {
  component: string;
  file: string;
  rule: string;
  detail: string | null;
}

export interface ReactCallsite {
  file: string;
  line: number;
  component: string;
  props: string[];
  hasSpread: boolean;
}

export interface ReactUsagesReport {
  target: { file: string; symbol?: string };
  callsites: ReactCallsite[];
  /** Story files importing the target. Omitted when `props`/`tests`-only `include`. */
  stories?: string[];
  /** Test files importing the target. */
  tests?: string[];
  /** Exported prop type/interface names declared in the target file. */
  propTypes?: string[];
}
