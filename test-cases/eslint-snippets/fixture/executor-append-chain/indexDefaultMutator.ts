import DefaultCols from "./indexFragments";

export function extend(dynamicFragment: unknown) {
  DefaultCols.append(dynamicFragment);
}
