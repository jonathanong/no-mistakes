export const sharedValue2 = 2;
export const helper2 = {
  mapValue<T>(value: T, index: number, label: string) { return value; },
  record<T>(value: T, shared: number, detail: { index: number; name: string }) { return [value, shared, detail]; },
};
