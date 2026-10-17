// Static analyzer input only; never execute this fixture.
const reassigned = "before";
reassigned = "after";
accept(reassigned);
accept(before);
const before = "later";
let mutable = "text";
accept(mutable);
function named() {}
accept(named);
