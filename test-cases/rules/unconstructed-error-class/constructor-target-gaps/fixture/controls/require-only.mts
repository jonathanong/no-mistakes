// An ESM file extension must not turn require syntax into an ESM import proof.
import Required = require("conditional-entry");
export class ImportEqualsChild extends Required.Opposite {}
const { Choice: RequiredChoice } = require("conditional-entry");
export class DestructuredRequireChild extends RequiredChoice {}
