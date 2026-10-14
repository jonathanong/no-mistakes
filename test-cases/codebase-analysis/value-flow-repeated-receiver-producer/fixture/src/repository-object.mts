export const objectReceiver = {
  get(key: string) { return key; },
  set(key: string, value: string) { return [key, value]; },
};
