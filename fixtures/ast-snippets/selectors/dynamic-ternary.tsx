export function Page({ cond }: { cond: boolean }) {
  const dataPw = cond ? 'option-a' : 'option-b';
  return <button data-pw={dataPw} />;
}
export function InlineTernary({ cond }: { cond: boolean }) {
  return <button data-pw={cond ? 'inline-a' : 'inline-b'} />;
}
