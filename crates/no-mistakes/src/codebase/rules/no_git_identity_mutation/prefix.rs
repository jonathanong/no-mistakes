use regex::Regex;

pub(super) fn shell_pattern() -> Regex {
    // Quoted arguments stay single shell words without evaluating their contents.
    // Separators must never be consumed as a global option's argument.
    let space = r"(?:[ \t]|\\\n[ \t]*)+";
    let word = r#"(?:'[^'\n]*'|"[^"\n]*"|[^\s'"$`\\;|&<>()]+)+"#;
    let valued = r"(?:-C|-c|--git-dir|--work-tree|--namespace|--config-env)";
    let flag = r"(?:--no-pager|--paginate|-p|-P|--bare|--no-replace-objects|--literal-pathspecs|--glob-pathspecs|--noglob-pathspecs|--icase-pathspecs|--no-optional-locks|--no-lazy-fetch|--no-advice)";
    let option = format!(
        r#"(?:(?:{valued}|'{valued}'|"{valued}"){space}{word}|(?:-C|-c){word}|(?:--git-dir|--work-tree|--namespace|--config-env|--exec-path|--attr-source)={word}|{flag}|'{flag}'|"{flag}")"#,
    );
    // Preserve the historical permissive suffix for existing plain commands.
    // The bounded branch comes first so new prefixes cannot swallow later writes.
    let legacy = r#"(^|[^a-zA-Z0-9_])git[ \t]+config(?:[^\n\\]|\\\n[ \t]*)*[ \t][`"']?user\.(name|email)[`"']?(?:[ \t]|\\\n[ \t]*)+[^#\s\\|&;<>]"#;
    Regex::new(&format!(
        r#"(?m)(?:(^|[^a-zA-Z0-9_])(?:git|'git'|"git"){space}(?:{option}{space})*(?P<config>config|'config'|"config")(?:{space}{word})*{space}[`"']?user\.(name|email)[`"']?(?:[ \t]|\\\n[ \t]*)+[^#\s\\|&;<>]|{legacy})"#,
    ))
    .expect("shell pattern")
}
