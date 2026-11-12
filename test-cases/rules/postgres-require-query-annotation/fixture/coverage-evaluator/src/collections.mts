import { write } from "@app/db";
function joined() { const pieces = ["SELECT", "1"]; return pieces.join(" "); }
function comma() { const pieces = ["SELECT", 1]; return pieces.join(); }
function pushed() { const pieces = ["SELECT"]; pieces.push("2"); return pieces.join(" "); }
function mapped() { const pieces = [1, 2]; return pieces.map(value => "SELECT " + value).join(" UNION ALL "); }
function each() { const pieces = [1, 2]; pieces.forEach(value => write("SELECT " + value)); return "SELECT 3"; }
function badSeparator(separator) { const pieces = ["SELECT", "4"]; return pieces.join(separator); }
function objectElement() { const pieces = ["SELECT", {}]; return pieces.join(" "); }
function hole() { const pieces = ["SELECT", , "5"]; return pieces.join(" "); }
function removed() { const pieces = ["SELECT", "6"]; delete pieces[1]; return pieces.join(" "); }
function conditional(flag) { const pieces = ["SELECT"]; if (flag) pieces.push("7"); return pieces.join(" "); }
function noCallback() { const pieces = ["SELECT"]; return pieces.map(); }
write(joined()); // collection-join
write(comma()); // collection-default-separator
write(pushed()); // collection-push
write(mapped()); // collection-map
write(each()); // collection-for-each
write(badSeparator(unknownSeparator)); // collection-bad-separator
write(objectElement()); // collection-object
write(hole()); // collection-hole
write(removed()); // collection-deleted
write(conditional(unknownFlag)); // collection-conditional
write(noCallback()); // collection-missing-callback
