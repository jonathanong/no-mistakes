/** A reviewed allowance in repository-relative source coordinates. */
export interface QueryReachedPerItemAllowEntry {
  file: string;
  callee?: string;
  line?: number;
}

/** Options in a `query-reached-per-item` rule application selected by `config`. */
export interface QueryReachedPerItemOptions {
  effects: string[];
  allow?: QueryReachedPerItemAllowEntry[];
  /** Exempt proven page reads in counted cursor loops. Defaults to false. */
  exemptCappedCursorPageReads?: boolean;
}
