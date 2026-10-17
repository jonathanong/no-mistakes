/** Bounded source literals; no source code is evaluated. */
export type ForbiddenCallArgumentLiteral = string | number | boolean | null;
export type ForbiddenCallUnknownArguments = "finding" | "ignore";
export type ForbiddenCallArgumentPredicate = {
  /** Zero-based argument index, from 0 through 31. */
  position: number;
  /** Optional single literal options-property name. */
  property?: string;
} & (
  | { equals: ForbiddenCallArgumentLiteral; arrayIncludes?: never; arrayExcludes?: never }
  | { arrayIncludes: ForbiddenCallArgumentLiteral; equals?: never; arrayExcludes?: never }
  | { arrayExcludes: ForbiddenCallArgumentLiteral; equals?: never; arrayIncludes?: never }
);
export interface ForbiddenCallArgumentConditions {
  /** Between one and sixteen conjunctive predicates. */
  all: ForbiddenCallArgumentPredicate[];
}
