export default function parseDateDefault(value: string) {
  return new Date(value);
}

export const formatDefaultDate = (value: string) => parseDateDefault(value).toISOString();
