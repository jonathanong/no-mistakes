export function Page({ key }: { key: string }) {
  const map = { x: 'val-a', y: 'val-b', z: 'val-c' };
  const dataPw = map[key as keyof typeof map];
  return <button data-pw={dataPw} />;
}
