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

export const Applied = React.memo.apply(React, [
  function Applied() {
    return <button>Apply</button>;
  },
]);

export const Plain = () => <div data-pw="plain" />,
  Other = () => <button />;
