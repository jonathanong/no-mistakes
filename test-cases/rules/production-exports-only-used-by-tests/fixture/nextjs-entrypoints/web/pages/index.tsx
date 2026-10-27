export const pagesMarker = "pages";

export function getServerSideProps() {
  return { props: {} };
}

export default function PagesHome() {
  return pagesMarker;
}
