export default function RootLayout({ children }) {
  fetch('/api/layout-data');
  return <html><body>{children}</body></html>;
}
