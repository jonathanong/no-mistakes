import { ThroughError, Plain } from 'stars';
import { Base } from 'ambiguous';
import { Missing } from 'cyclic';
import { Alias, Duplicate, Dynamic } from 'opaque';
import { Broken } from 'broken';
import * as ObjectExport from 'object-export';
export class StarError extends ThroughError {}
export class StarNonError extends Plain {}
export class AmbiguousBase extends Base {}
export class CyclicBase extends Missing {}
export class OpaqueAlias extends Alias {}
export class DuplicateExport extends Duplicate {}
export class DynamicBase extends Dynamic {}
export class BrokenBase extends Broken {}
export class ObjectAssignment extends ObjectExport.Component {}
import { CycleA, Hidden, Conflict as Duplicated } from 'opaque';
export class CyclicClasses extends CycleA {}
export class PrivateNamespaceClass extends Hidden.Private {}
export class ConflictingNamespaceClass extends Duplicated.Base {}
export class ConflictingNamespaceBase extends Duplicated.Child {}
