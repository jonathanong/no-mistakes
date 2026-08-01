use super::{
    is_atx_heading, is_indented_code, is_setext_heading_underline, is_thematic_break,
    looks_like_mdx_flow_boundary, starts_block_container, ContainerPrefix,
};

pub(super) fn retain_paragraph_context(
    line: &[u8],
    in_paragraph: &mut bool,
    paragraph_container: &mut Option<ContainerPrefix>,
) {
    if *in_paragraph
        && paragraph_container
            .as_ref()
            .is_none_or(|container| container.strip_line(line).is_none())
    {
        *in_paragraph = false;
        *paragraph_container = None;
    }
}

pub(super) fn update_paragraph_state(
    raw_line: &[u8],
    in_paragraph: &mut bool,
    paragraph_container: &mut Option<ContainerPrefix>,
) {
    if *in_paragraph {
        let content = paragraph_container
            .as_ref()
            .and_then(|container| container.strip_line(raw_line))
            .expect("active paragraph context was retained for this line");
        if is_non_container_paragraph_boundary(&content, true) {
            *in_paragraph = false;
            *paragraph_container = None;
            return;
        }
        let (_, nested) = ContainerPrefix::from_opening_line(&content);
        if nested.has_steps() && nested.can_interrupt_paragraph() {
            *in_paragraph = false;
            *paragraph_container = None;
            update_paragraph_state(raw_line, in_paragraph, paragraph_container);
        }
        return;
    }

    if is_non_container_paragraph_boundary(raw_line, false) {
        return;
    }
    let (content, container) = ContainerPrefix::from_opening_line(raw_line);
    if paragraph_content_is_active(content, false) {
        *in_paragraph = true;
        *paragraph_container = Some(container);
    }
}

fn is_non_container_paragraph_boundary(line: &[u8], in_paragraph: bool) -> bool {
    if is_indented_code(line) {
        return !in_paragraph;
    }
    let first = line.iter().position(|byte| !matches!(byte, b' ' | b'\t'));
    match first.map(|index| line[index]) {
        None => true,
        Some(b'<') if looks_like_mdx_flow_boundary(line) => true,
        Some(b'<') => !in_paragraph,
        Some(b'#') if is_atx_heading(line) => true,
        Some(b'*' | b'-' | b'_') if is_thematic_break(line) => true,
        Some(b'=' | b'-') if in_paragraph && is_setext_heading_underline(line) => true,
        _ => false,
    }
}

fn paragraph_content_is_active(line: &[u8], in_paragraph: bool) -> bool {
    !is_non_container_paragraph_boundary(line, in_paragraph)
        && !starts_block_container(line, in_paragraph)
}
