// The later callback has synthetic offset zero; it must not hide this call.
unknown[method]();
(() => {});
