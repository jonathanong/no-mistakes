mod property_strings;
mod regex_literals;

pub(crate) use property_strings::extract_property_strings;
use regex_literals::extract_test_regex_literals;

pub(crate) fn extract_test_regexes(source: &str) -> Vec<String> {
    let mut regexes = extract_property_strings(source, "testRegex");
    regexes.extend(extract_test_regex_literals(source));
    regexes
}
