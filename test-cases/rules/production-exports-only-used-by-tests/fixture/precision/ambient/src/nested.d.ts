declare namespace Outer {
  namespace Inner {
    interface State { ready: boolean }
  }
}
declare module "overloads" {
  export function choose(value: string): string;
  export function choose(value: number): number;
}
