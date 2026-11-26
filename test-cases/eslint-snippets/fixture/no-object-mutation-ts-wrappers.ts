let moduleBox: { value: number } = { value: 0 };

export function update(
  input: { value: number; items: number[]; [key: string]: unknown },
  value: number,
  key: string,
) {
  input.value = value;
  (input.value as number) = value;
  input.value! = value;
  (<number>input.value) = value;
  ((input.value as number)!) = value;
  (input[key] as number) = value;
  ({ value: input.value } = { value });
  ({ value: (input.value as number) } = { value });
  ({ value: input.value! } = { value });
  ({ value: (<number>input.value) } = { value });
  ({ value: (input[key] as number) } = { value });
  ({ value: (input.value as number) = 0 } = {});
  ({ value: input.value! = 0 } = {});
  [input.value!, (input.value as number)] = [value, value];
  [...input.items!] = [value];
  for (input.value! of [value]) {
    void value;
  }
  for ((input.value as number) of [value]) {
    void value;
  }
}

export function escapeThroughWrappedBinding(value: number) {
  const local = { value };
  (moduleBox as { value: number }) = local;
  local.value = value + 1;
}

export function buildLocal(value: number) {
  const local: { value?: number; other?: number; items?: number[] } = {};
  (local.value as number) = value;
  local.other! = value;
  (<number>local.value) = value;
  ({ value: (local.value as number), other: local.other! } = { value, other: value });
  [...local.items!] = [value];
  return local;
}

export function suppressed(input: { value: number }, value: number) {
  // no-mistakes-disable-next-line no-object-mutation -- wrapper suppression
  (input.value as number) = value;
  // no-mistakes-disable-next-line no-object-mutation -- wrapper suppression
  ({ value: input.value! } = { value });
}
