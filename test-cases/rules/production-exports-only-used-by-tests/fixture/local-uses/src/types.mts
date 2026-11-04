export type UsedType = { value: number };
const value: UsedType = { value: 1 };
export interface UsedInterface { value: number }
type Shape = UsedInterface;
export type ExportOnlyType = string;
export type ShadowedType = number;
function local<ShadowedType>(value: ShadowedType) { return value; }
