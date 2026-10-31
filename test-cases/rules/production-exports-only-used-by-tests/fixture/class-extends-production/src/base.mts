export class Base {}
export class Child extends Base {}

export default class DefaultBase {}
export class DefaultChild extends DefaultBase {}

class ReexportBase {}
export { ReexportBase }
export class ReexportChild extends ReexportBase {}
