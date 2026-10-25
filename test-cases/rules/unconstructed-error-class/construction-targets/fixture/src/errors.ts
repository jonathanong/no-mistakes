import { Pool as BarrelPool } from '@app/barrel';
export class UsedError extends Error {}
export class DeadError extends Error {}
// A proven non-Error global must not make this class uncertain.
export class IntlUnusedError extends Error {
  format() { return new Intl.NumberFormat(); }
}
// Knowing the constructed class is enough even when its external base is opaque.
export class KnownCtorUnusedError extends Error {
  make() { return new BarrelPool(); }
}
