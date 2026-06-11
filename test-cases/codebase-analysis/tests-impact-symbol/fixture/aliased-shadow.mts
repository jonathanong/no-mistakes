const parseDateImpl = (input: string) => new Date(input);

const parseDate = (input: string) => new Date(`${input}T00:00:00Z`);

export { parseDateImpl as parseDate };

export const formatShadowDate = (input: string) => parseDate(input).toISOString();
