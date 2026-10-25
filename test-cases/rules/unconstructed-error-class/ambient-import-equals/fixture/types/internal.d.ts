declare namespace LocalTypes { class Context {} }
// An internal namespace alias does not make this script an external module.
import Context = LocalTypes.Context;
declare module 'fixture:script' {
  export abstract class Base<Env = unknown> {}
}
