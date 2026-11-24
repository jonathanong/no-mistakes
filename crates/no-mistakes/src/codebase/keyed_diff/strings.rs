use super::catalogs::insert_unique;
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

pub(super) fn entries(mut source: &str) -> Result<BTreeMap<String, String>> {
    let mut entries = BTreeMap::new();
    while !skip_trivia(&mut source)?.is_empty() {
        let key = quoted(&mut source)?;
        expect(&mut source, '=')?;
        let value = quoted(&mut source)?;
        expect(&mut source, ';')?;
        insert_unique(&mut entries, key, value)?;
    }
    Ok(entries)
}

fn skip_trivia<'a>(source: &mut &'a str) -> Result<&'a str> {
    loop {
        *source = source.trim_start();
        if let Some(comment) = source.strip_prefix("//") {
            *source = comment
                .split_once(['\r', '\n'])
                .map_or("", |(_, rest)| rest);
        } else if let Some(comment) = source.strip_prefix("/*") {
            *source = comment
                .split_once("*/")
                .context("unclosed strings comment")?
                .1;
        } else {
            return Ok(source);
        }
    }
}

fn expect(source: &mut &str, token: char) -> Result<()> {
    *source = skip_trivia(source)?
        .strip_prefix(token)
        .with_context(|| format!("expected {token:?} in strings entry"))?;
    Ok(())
}

fn quoted(source: &mut &str) -> Result<String> {
    expect(source, '"')?;
    let mut chars = source.chars();
    let mut units = Vec::new();
    while let Some(character) = chars.next() {
        match character {
            '"' => {
                *source = chars.as_str();
                return String::from_utf16(&units)
                    .context("invalid Unicode escape in strings entry");
            }
            '\\' => escape(&mut chars, &mut units)?,
            literal => units.extend_from_slice(literal.encode_utf16(&mut [0; 2])),
        }
    }
    anyhow::bail!("unclosed quoted strings entry")
}

fn escape(chars: &mut std::str::Chars<'_>, units: &mut Vec<u16>) -> Result<()> {
    let escaped = chars.next().context("incomplete strings escape")?;
    let code = match escaped {
        'a' => 0x07,
        'b' => 0x08,
        'f' => 0x0c,
        'n' => 0x0a,
        'r' => 0x0d,
        't' => 0x09,
        'v' => 0x0b,
        'U' => {
            let mut value = 0;
            for _ in 0..4 {
                let digit = chars
                    .next()
                    .and_then(|c| c.to_digit(16))
                    .context("invalid strings Unicode escape")?;
                value = value * 16 + digit as u16;
            }
            value
        }
        '0'..='7' => {
            let mut value = escaped as u16 - u16::from(b'0');
            for _ in 0..2 {
                let Some(digit) = chars.clone().next().and_then(|c| c.to_digit(8)) else {
                    break;
                };
                chars.next();
                value = value * 8 + digit as u16;
            }
            ensure!(value <= 127, "non-ASCII strings octal escape");
            value
        }
        literal => {
            units.extend_from_slice(literal.encode_utf16(&mut [0; 2]));
            return Ok(());
        }
    };
    units.push(code);
    Ok(())
}
