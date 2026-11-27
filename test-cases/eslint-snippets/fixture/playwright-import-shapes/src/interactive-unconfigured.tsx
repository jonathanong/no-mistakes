import * as Icons from "lucide-react";

declare const name: string;

const Icon = Icons[name];
export const Mark = () => <Icon className="h-4" />;
