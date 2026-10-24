import { ErrorBase as Base } from 'errors';
namespace Split {
  // A nonambient namespace keeps this class private to its declaration block.
  class Base {}
}
namespace Split {
  export class Child extends Base {}
}
export class MergedPrivateError extends Split.Child {}
