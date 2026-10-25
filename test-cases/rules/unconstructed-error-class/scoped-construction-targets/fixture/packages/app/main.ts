import { Plain, UsedError } from '@local/classes';
const Wrapped = Plain;
new Wrapped<string>();
new UsedError();
export function shadow(Plain: any) { return new Plain(); }
