export const example = { retries: 2 };

export default function browserOptions() {
  const unrelated = () => {
    return { retries: 3 };
  };
  unrelated();
  if (process.env.NO_TESTS) return null;
  return { retries: 2, projects: [{ name: "secondary" }] };
}
