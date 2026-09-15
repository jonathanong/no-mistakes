export function Button() {
  return <button onClick={() => import("./target.mts")} />;
}
