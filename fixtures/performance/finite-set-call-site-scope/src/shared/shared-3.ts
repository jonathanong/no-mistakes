export const sharedValue3 = 3;
export const helper3 = {
  mapValue<T>(value: T, index: number, label: string) { return value; },
  record<T>(value: T, shared: number, detail: { index: number; name: string }) { return [value, shared, detail]; },
};
