exports.replacedNonobjectSetups = '../shared-setup/replaced-nonobject-old.ts'
const replacement = './not-a-static-setup-object.ts'
// A non-object replacement also shadows the earlier named export.
module.exports = replacement
// An unrelated assignment after the replacement must not become an export.
let ignoredAssignment
ignoredAssignment = true
