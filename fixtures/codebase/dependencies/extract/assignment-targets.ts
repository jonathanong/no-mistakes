declare const obj: Record<string, number>;
declare const arr: number[];
declare const key: string;

obj[key] = 1;
({ x: { y, ...inner } } = obj);
({
  x: [a, { b, ...nested } = obj, ...rest] = arr,
} = { x: arr });
