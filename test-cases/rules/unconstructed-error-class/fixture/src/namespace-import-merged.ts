// An import and a namespace of one name: a member access may mean either, so
// the class is never reported.
import { Imported } from "./namespace-sourced-target";

export namespace Imported {
  export class ImportMergedError extends Error {}
}
