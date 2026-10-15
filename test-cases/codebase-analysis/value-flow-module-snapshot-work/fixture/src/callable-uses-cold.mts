function direct() {}
direct();
(direct as (() => void))();
function caller() { direct(); }
caller();
