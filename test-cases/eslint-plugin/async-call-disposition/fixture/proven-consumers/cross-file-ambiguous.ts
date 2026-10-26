// Duplicate exports cannot establish one unambiguous implementation.
async function a(callback: any) { await callback(); }
async function b(callback: any) { callback(); }
export { a as ambiguous };
export { b as ambiguous };
