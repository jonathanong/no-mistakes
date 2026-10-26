import { RepoError, Factory } from './bases';
import { ExternalChild, Plain as Base } from 'closure';
export class RepositoryChild extends RepoError {}
export class DeclaredChild extends ExternalChild {}
export class OpaqueFactory extends Factory {}
namespace API {
  export class Base extends Error {}
  export class Child extends Base {}
}
export class NamespaceChild extends API.Child {}
namespace Opaque {
  const Base: any = Error;
  export class Child extends Base {}
}
export class OpaqueNamespace extends Opaque.Child {}
import { Base as JsonBase } from './opaque.json';
export class JsonAncestry extends JsonBase {}
export default class extends ExternalChild {}
