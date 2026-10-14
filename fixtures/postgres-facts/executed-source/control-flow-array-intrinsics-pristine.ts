import { read } from '@example/db';
export function run() {
  const parts = ['/* intrinsics/pristine */ SELECT 1'];
  parts.push('');
  parts.forEach(() => 0);
  read(parts.map(value => value).join(''));
}
