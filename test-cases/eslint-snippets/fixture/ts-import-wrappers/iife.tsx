export const C = () => (
  <div>
    {(() => <span />)()}
    {((() => <span />) as () => JSX.Element)()}
    {((function <T>() { return <span />; })<string>)()}
  </div>
);
