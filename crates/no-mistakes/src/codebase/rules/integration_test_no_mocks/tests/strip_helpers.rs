use super::*;

#[test]
fn strip_helpers_preserve_offsets_for_unclosed_and_escaped_tokens() {
    assert_eq!(
        strip::comments_and_regex_literals("before /* unclosed\nvi.fn()"),
        "before            \n       "
    );
    assert_eq!(
        strip::strip(
            "const value = 'escaped\\' quote'\nconst tail = `open",
            true,
            true,
        ),
        "const value =                  \nconst tail =      "
    );
    assert_eq!(
        strip::strip("const tail = 'open\\", true, true),
        "const tail =       "
    );
    let complex_template =
        "const value = `${/* hidden */ ({ nested: `text ${vi.fn()}` }) // tail\n}`";
    let stripped = strip::strip(complex_template, true, true);
    assert!(stripped.contains("vi.fn()"), "{stripped}");
    assert!(!stripped.contains("hidden"), "{stripped}");
    assert!(!stripped.contains("text"), "{stripped}");
    assert_eq!(
        strip::strip("const value = `open\\", true, true),
        "const value =       "
    );
    assert_eq!(
        strip::strip("const value = `open\\x`", true, true),
        "const value =         "
    );
    assert_eq!(
        strip::strip("const value = `${unterminated", true, true),
        "const value =    unterminated"
    );
    assert_eq!(
        strip::comments_and_regex_literals("const value = `open\\"),
        "const value = `open\\"
    );
    let regex_stripped = strip::strip("expect(src).toMatch(/vi\\.mock\\(/gi)", true, true);
    assert!(!regex_stripped.contains("vi"), "{regex_stripped}");
    let template_regex_text = strip::strip(
        "const value = `${/vi\\.mock\\({1}\\)/.test(source)}`",
        true,
        true,
    );
    assert!(!template_regex_text.contains("vi"), "{template_regex_text}");
    let preserved_template_regex =
        strip::comments_and_regex_literals("const value = `${/from \"msw\"/.test(source)}`");
    assert!(
        !preserved_template_regex.contains("msw"),
        "{preserved_template_regex}"
    );
    assert_eq!(
        strip::comments_and_regex_literals("const value = `open\\x`"),
        "const value = `open\\x`"
    );
    assert_eq!(
        strip::comments_and_regex_literals("const value = `${'open\\}`"),
        "const value = `${'open\\}`"
    );
    assert_eq!(
        strip::comments_and_regex_literals("const value = `${'open\\"),
        "const value = `${'open\\"
    );
    let template_regex_brace = strip::strip(
        "const value = `${/\\}/.test(source) ? vi.fn() : value}`",
        true,
        true,
    );
    assert!(
        template_regex_brace.contains("vi.fn()"),
        "{template_regex_brace}"
    );
    let template_string_text = strip::strip("const value = `${'vi.fn()'}`", true, true);
    assert!(
        !template_string_text.contains("vi.fn()"),
        "{template_string_text}"
    );
    assert_eq!(strip::strip("/vi\\.mock\\(/", true, true), "            ");
    assert_eq!(
        strip::strip("return /vi\\.mock\\(/", true, true),
        "return             "
    );
    assert_eq!(
        strip::strip("const re = /[a\\/b]/g", true, true),
        "const re =          "
    );
    assert_eq!(
        strip::strip("const re = /unterminated\nvi.fn()", true, true),
        "const re =              \nvi.fn()"
    );
    assert_eq!(
        strip::strip("const re = /unterminated", true, true),
        "const re =              "
    );
    assert_eq!(
        strip::strip("throw /vi\\.fn\\(/", true, true),
        "throw           "
    );
    assert_eq!(
        strip::strip("case /vi\\.fn\\(/", true, true),
        "case           "
    );
    assert_eq!(
        strip::strip("const fn = () => /vi\\.fn\\(/", true, true),
        "const fn = () =>           "
    );
}
