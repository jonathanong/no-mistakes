function deco(_target: unknown, _context: unknown) {}

export class Box {
  @deco
  accessor value = 1;
  accessor typed: number = 2;
  accessor empty;
}
