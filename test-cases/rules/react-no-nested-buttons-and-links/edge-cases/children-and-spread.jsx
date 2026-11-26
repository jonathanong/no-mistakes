const props = {};

export const bareElementChildrenProp = (
  <button type="button" children=<a href="/items">Items</a> />
);
export const bareFragmentChildrenProp = (
  <button type="button" children=<><a href="/items">Items</a></> />
);
export const nestedControlSpreadNoChildren = (
  <a href="/items">
    <button type="button" {...props} />
  </a>
);
