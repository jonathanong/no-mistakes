export async function sortWrapped() {
  const xs = [1, 2];
  await (xs.sort() as number[]);
  await xs["sort" as const]();
  let ys = [1] as number[];
  await ys.sort();
  return ys;
}
