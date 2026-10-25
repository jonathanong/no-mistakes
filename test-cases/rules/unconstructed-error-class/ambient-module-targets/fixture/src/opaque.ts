import { Base as Opaque } from 'fixture:opaque';
import { Base as Ambiguous } from 'fixture:duplicate';
import { Base as OtherProject } from 'fixture:other-project';
import { Base as Missing } from 'fixture:missing';
export class OpaqueChild extends Opaque {}
export class AmbiguousChild extends Ambiguous {}
export class OtherProjectChild extends OtherProject {}
export class MissingChild extends Missing {}
