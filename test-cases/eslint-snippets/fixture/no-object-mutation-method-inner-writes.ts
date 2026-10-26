import { target } from "./state";
function dynamicApply() {
  const rows = [];
  const push = rows.push;
  push.apply(rows, (target.value = 1));
}
function spreadCall() {
  const rows = [];
  const push = rows.push;
  push.call(...(target.value = 1, []));
}
function extraArgument() {
  const rows = [];
  const push = rows.push;
  push.apply(rows, [], (target.value = 1));
}

function spreadBind() {
  const rows = [];
  const push = rows.push;
  push.bind(...(target.value = 1, []));
}
