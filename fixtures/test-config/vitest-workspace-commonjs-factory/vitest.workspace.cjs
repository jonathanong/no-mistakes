const factory = (projects) => projects

// Executing a factory is not a static workspace export.
module.exports = factory([{ test: { name: 'factory-workspace-project' } }])
