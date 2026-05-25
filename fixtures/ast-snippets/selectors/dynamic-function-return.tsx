function getSelector(cond: boolean) {
  if (cond) return 'fn-a';
  return 'fn-b';
}
export function Page({ cond }: { cond: boolean }) {
  const dataPw = getSelector(cond);
  return <button data-pw={dataPw} />;
}
