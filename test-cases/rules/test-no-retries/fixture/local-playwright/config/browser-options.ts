export function createBrowserOptions() {
  function unrelatedOptions() {
    return { retries: 4 };
  }
  unrelatedOptions();
  return { retries: process.env.CI ? 1 : 0 };
}

export const localOptions = () => ({
  retries: 0,
  projects: [{ name: "secondary", retries: Number(process.env.TEST_RETRIES) }],
});

export const unchangedOptions = function () {
  return { retries: 0 };
};

export default () => {
  if (process.env.CI) return;
  return { retries: 0 };
};
