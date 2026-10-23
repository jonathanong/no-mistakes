export const sharedValue0 = 0;
export const helper0 = {
  mapValue<T>(value: T, index: number, label: string) { return value; },
  record<T>(value: T, shared: number, detail: { index: number; name: string }) { return [value, shared, detail]; },
};
