import { SaveButton as Save } from "./ui";

const Alias = Save;

<pw.SaveButton />;
<Save />;
<Alias />;

function Local() {
  const Save = () => <span />;
  return <Save />;
}

function Wrapper(Save) {
  const Shadow = Save;
  return <Shadow />;
}
