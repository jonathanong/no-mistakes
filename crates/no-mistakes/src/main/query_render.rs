use no_mistakes::cli::Format;
use serde::Serialize;

pub(crate) fn print_query_report<T: Serialize>(
    report: &T,
    paths: impl IntoIterator<Item = String>,
    format: Format,
    human: impl FnOnce(&T),
    markdown: impl FnOnce(&T),
) {
    match format {
        Format::Json => println!("{}", no_mistakes::cli::json_string(report)),
        Format::Yml => println!("{}", no_mistakes::cli::yaml_string(report)),
        Format::Md => markdown(report),
        Format::Paths => {
            for path in paths {
                println!("{path}");
            }
        }
        Format::Human => human(report),
    }
}
