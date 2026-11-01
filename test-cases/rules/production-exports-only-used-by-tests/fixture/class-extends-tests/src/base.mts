export class Base {}
export class OnlyHidden {}
class Hidden extends OnlyHidden {}

export class Shadow {}
export namespace Box {
  class Shadow {}
  export class Child extends Shadow {}
}
