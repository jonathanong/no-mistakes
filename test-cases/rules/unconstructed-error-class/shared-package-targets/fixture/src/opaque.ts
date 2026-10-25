import { OpaqueHelper } from 'generic-helpers';
import OpaqueValidator from 'opaque-validator';
import { Missing } from 'unavailable-package';
new OpaqueHelper();
new OpaqueValidator();
new Missing();
export class OpaqueSubclass extends OpaqueHelper {}
