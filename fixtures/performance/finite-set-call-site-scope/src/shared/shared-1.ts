export const sharedValue1 = 1;
export const helper1 = {
  mapValue<T>(value: T, index: number, label: string) { return value; },
  record<T>(value: T, shared: number, detail: { index: number; name: string }) { return [value, shared, detail]; },
};
