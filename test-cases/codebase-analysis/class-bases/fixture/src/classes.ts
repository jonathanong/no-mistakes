import { LibError } from "@fixture/lib";
import { Missing } from "external-package";
import * as base from "./base.js";
import { Base } from "./base.js";

export class Local extends Base {}
export class FromNamespace extends base.Base {}
export class FromWorkspace extends LibError {}
export class SameFile extends Local {}
export class Builtin extends TypeError {}
export class FromPackage extends Missing {}
export class Plain {}
export class Opaque extends mixin(Error) {}

class Private extends Error {}
export { Private as Renamed };

export default class extends Error {}
