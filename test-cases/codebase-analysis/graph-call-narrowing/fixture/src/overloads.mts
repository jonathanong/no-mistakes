export function parse(value: string): string;
export function parse(value: number): string;
export function parse(value: string | number) {
  return target();
}

function target() {}

parse("input");
