const key = "unstub" + "Envs";
export default { test: { [key]: true, ["include"]: ["tests/**/*.test.ts"] } };
export const labels = { [`include`]: ["tests/**/*.test.ts"], [/unstub/]: true };
