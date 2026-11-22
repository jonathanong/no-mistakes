import { read } from "@app/db"
const input0 = { left: "left", right: "right" }
const input1 = { left: input0, right: input0 }
const input2 = { left: input1, right: input1 }
const input3 = { left: input2, right: input2 }
const input4 = { left: input3, right: input3 }
const input5 = { left: input4, right: input4 }
const input6 = { left: input5, right: input5 }
const input7 = { left: input6, right: input6 }
const input8 = { left: input7, right: input7 }
const input9 = { left: input8, right: input8 }
const input10 = { left: input9, right: input9 }
const input11 = { left: input10, right: input10 }
const input12 = { left: input11, right: input11 }
const input13 = { left: input12, right: input12 }
const input14 = { left: input13, right: input13 }
const input15 = { left: input14, right: input14 }
const input16 = { left: input15, right: input15 }
const input17 = { left: input16, right: input16 }
const input18 = { left: input17, right: input17 }
const input19 = { left: input18, right: input18 }
const input20 = { left: input19, right: input19 }
read("SELECT id FROM widgets", input20)
