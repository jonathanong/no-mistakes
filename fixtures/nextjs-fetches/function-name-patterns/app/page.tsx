const getData = async () => {
  return fetch('/api/arrow-fn');
};

async function loadItems() {
  return fetch('/api/named-fn');
}

export default function Page() {
  fetch('/api/component');
  getData();
  loadItems();
  return <div>Page</div>;
}
