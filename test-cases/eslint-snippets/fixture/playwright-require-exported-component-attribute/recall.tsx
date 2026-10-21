function SaveButton() {
  return <button>Save</button>;
}

export const Save = SaveButton;

export const Memoed = React["memo"](function Memoed() {
  return <button>Memo</button>;
});

export const Forwarded = React.memo.call(React, function Forwarded() {
  return <button>Call</button>;
});
