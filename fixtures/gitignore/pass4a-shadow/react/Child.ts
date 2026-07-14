import { useState } from "react";

export default function Child() {
  const [value] = useState(1);
  return value;
}
