async function consumer(callback: () => Promise<unknown>) { await callback(); }
export { consumer as renamed };
