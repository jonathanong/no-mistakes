import type { Unanalyzable } from "./traversal-types";

/** An exact callable in the prepared TS/JS catalog. No whole-file roots. */
export interface CoverageProducerFunction {
  file: string;
  symbol: string;
}
/** Binding-aware assertion or modeled dispatch identity. */
export interface CoverageAssertionImport {
  module: string;
  export: string;
}
/** Explicit trusted callback/process contract; the owner must still reach an assertion. */
export interface CoverageAssertionOwnership {
  producer: CoverageProducerFunction;
  owner: CoverageProducerFunction;
  via: CoverageAssertionImport;
}
/** Options for the opt-in coverage-producer-assertion-owner check rule. */
export interface CoverageProducerAssertionOwnerOptions {
  producers: CoverageProducerFunction[];
  assertions: CoverageAssertionImport[];
  ownership?: CoverageAssertionOwnership[];
  /** Positive canonical edge depth. Default: 64. Truncation is unanalyzable. */
  maxDepth?: number;
  /** Positive visited callable cap, including the producer. Default: 10000. */
  maxNodes?: number;
  unanalyzable?: Unanalyzable;
}
