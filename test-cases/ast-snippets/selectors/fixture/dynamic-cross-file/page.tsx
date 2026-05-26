import { getSelector, CONST_SELECTOR, selectorMap } from './selectors';

export function FnPage() {
  const dataPw = getSelector();
  return <button data-pw={dataPw} />;
}
export function ConstPage() {
  return <button data-pw={CONST_SELECTOR} />;
}
export function ObjPage({ key }: { key: string }) {
  const dataPw = selectorMap[key as keyof typeof selectorMap];
  return <button data-pw={dataPw} />;
}
