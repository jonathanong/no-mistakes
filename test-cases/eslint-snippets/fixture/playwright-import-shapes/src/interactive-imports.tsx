const { Button: ActionA } = require("./ui");
export const A = () => <ActionA />;

const UI = require("./ui");
const ActionB = UI.Button;
export const B = () => <ActionB />;

import UIEq = require("./ui");
const ActionC = UIEq.Button;
export const C = () => <ActionC />;

import { "Button" as ActionD } from "./ui";
export const D = () => <ActionD />;
