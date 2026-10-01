pub(super) fn output_parameters(words: &[String]) -> String {
    let mut parts = Vec::new();
    let mut index = 0;
    while index < words.len() {
        if words[index] == "out" || words[index] == "inout" {
            let mut chunk = vec![words[index].as_str()];
            index += 1;
            while index < words.len()
                && !matches!(
                    words[index].as_str(),
                    "in" | "out" | "inout" | "variadic" | ","
                )
                && !super::function_clauses::ends_return(&words[index])
            {
                chunk.push(words[index].as_str());
                index += 1;
            }
            if chunk.len() > 1 {
                parts.push(chunk.join(" "));
            }
            continue;
        }
        index += 1;
    }
    parts.join(" ")
}
