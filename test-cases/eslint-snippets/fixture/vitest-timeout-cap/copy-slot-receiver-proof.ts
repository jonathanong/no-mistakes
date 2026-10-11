const valid = { copy: {} };
delete valid.copy;
let mutable = { copy: {} };
delete mutable.copy;
const reassigned = { copy: {} };
reassigned = {};
delete reassigned.copy;
const { holder } = { holder: { copy: {} } };
delete holder.copy;
function drop(parameter) {
  delete parameter.copy;
}
delete undeclared.copy;
const cyclicLeft = cyclicRight;
const cyclicRight = cyclicLeft;
delete cyclicLeft.copy;
const dynamicReceiver = receiver();
delete dynamicReceiver.copy;
