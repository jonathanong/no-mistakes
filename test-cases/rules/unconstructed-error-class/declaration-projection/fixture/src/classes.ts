import { Base, Plain } from './base';
export class Derived extends Base {
  // A static initializer callback is not another extends base.
  static values = [].map(Plain as unknown as () => unknown);
}
