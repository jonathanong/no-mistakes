'use client';
export default function ErrorPage() {
  fetch('/api/error-data');
  return <div>Error</div>;
}
