export class Base {}
export class Child extends Base {}

export default class DefaultBase {}
export class DefaultChild extends DefaultBase {}

class ReexportBase {}
export { ReexportBase }
export class ReexportChild extends ReexportBase {}

export class GlobalChild extends Error {}

export class Constructed {
  constructor() {}
}
export class Sub extends Constructed {}
