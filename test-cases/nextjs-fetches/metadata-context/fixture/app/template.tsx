export default function Template({ children }) {
  fetch('/api/template-data');
  return <div>{children}</div>;
}
