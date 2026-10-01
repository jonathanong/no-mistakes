// An import and a namespace of one name: a member access may mean either.
import { Imported } from "./sourced-target";

export namespace Imported {
  export class ImportMergedClass extends Error {}
}
