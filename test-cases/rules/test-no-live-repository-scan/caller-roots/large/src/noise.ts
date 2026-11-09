const loader = process.env.SCAN_LOADER;
if (loader) require(loader);

function scan(value: string) {
  return value;
}
const scanAsValue = scan;
void scanAsValue;
