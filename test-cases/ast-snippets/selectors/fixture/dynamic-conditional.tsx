export function Page({ cond }: { cond: boolean }) {
  let dataPw: string;
  if (cond) {
    dataPw = 'branch-a';
  } else {
    dataPw = 'branch-b';
  }
  return <button data-pw={dataPw} />;
}
