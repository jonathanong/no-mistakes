import { Button as Save } from "./ui.jsx";
const Alias = Save;
// An undeclared identifier is not an import, even without configured globals.
const Unrelated = unknown;
<Alias />;
<Unrelated />;
