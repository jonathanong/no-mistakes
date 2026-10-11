// Parser recovery must not turn invalid Annex B heads into eager mock helpers.
for (var [key] = require('./destructured.cjs') in {}) {}
for (var first = require('./first.cjs'), second in {}) {}
