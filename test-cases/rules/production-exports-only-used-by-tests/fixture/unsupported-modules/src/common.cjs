// Mutating the export object is unsupported CommonJS and must produce uncertainty, never a clean check.
Object.assign(module.exports, { value: 1 });
