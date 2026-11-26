const label = "Save";
const items = [];

export const emptyButton = <button type="button" />;
export const booleanChildrenProp = <button type="button" children />;
export const outerExpression = <div>{label}</div>;
export const outerSpread = <div>{...items}</div>;
export const siblings = (
  <div>
    <a href="/items">Items</a>
    <button type="button">Save</button>
  </div>
);
