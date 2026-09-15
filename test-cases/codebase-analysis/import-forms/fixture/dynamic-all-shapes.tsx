export function Page(flag: boolean, kind: "a" | "b") {
  if (flag) {
    void import("./dynamic-if-target.mts");
  }
  switch (kind) {
    case "a":
      void import("./dynamic-switch-target.mts");
      break;
    default:
      break;
  }
  function nested() {
    void import("./dynamic-nested-target.mts");
  }
  void nested;
  return <button onClick={() => import("./dynamic-click-target.mts")} />;
}
