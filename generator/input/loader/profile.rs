/// Environment/profile preprocessing for resources
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fmt::Write as _;

/// Preprocess XML: remove any element that has a profile attribute not matching the current profile
/// This runs before parsing, so the parser receives only relevant nodes
pub fn preprocess_xml(xml: &str, current_profile: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut out = String::new();

    // Track whether we are currently skipping a subtree due to mismatched profile
    let mut skip_depth: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                handle_start_event(
                    &e,
                    current_profile,
                    &mut skip_depth,
                    &mut out,
                );
            }
            Ok(Event::Empty(e)) => {
                handle_empty_event(
                    &e,
                    current_profile,
                    skip_depth,
                    &mut out,
                );
            }
            Ok(Event::Text(e)) => {
                handle_text_event(&e, skip_depth, &mut out)
            }
            Ok(Event::End(e)) => {
                handle_end_event(&e, &mut skip_depth, &mut out)
            }
            Ok(Event::Eof) => break,
            Ok(
                Event::Comment(_)
                | Event::Decl(_)
                | Event::CData(_)
                | Event::PI(_)
                | Event::DocType(_)
                | Event::GeneralRef(_),
            ) => {
                // ignore
            }
            Err(_) => {
                // If preprocessing fails, return original xml to avoid hard failure
                return xml.to_string();
            }
        }
        buf.clear();
    }
    out
}

#[inline]
fn extract_profile_attr_start(
    e: &quick_xml::events::BytesStart,
) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"profile" {
            return Some(
                String::from_utf8_lossy(&attr.value).to_string(),
            );
        }
    }
    None
}

#[inline]
fn extract_profile_attr_empty(
    e: &quick_xml::events::BytesStart,
) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"profile" {
            return Some(
                String::from_utf8_lossy(&attr.value).to_string(),
            );
        }
    }
    None
}

#[inline]
fn handle_start_event(
    e: &quick_xml::events::BytesStart,
    current_profile: &str,
    skip_depth: &mut usize,
    out: &mut String,
) {
    let profile_attr = extract_profile_attr_start(e);
    if *skip_depth > 0 {
        *skip_depth += 1;
    } else if let Some(p) = profile_attr.as_deref() {
        if p != current_profile {
            *skip_depth = 1; // start skipping this subtree
        }
    }
    if *skip_depth == 0 {
        write_start_tag(out, e);
    }
}

#[inline]
fn handle_empty_event(
    e: &quick_xml::events::BytesStart,
    current_profile: &str,
    skip_depth: usize,
    out: &mut String,
) {
    if skip_depth == 0 {
        let profile_attr = extract_profile_attr_empty(e);
        if let Some(p) = profile_attr.as_deref() {
            if p == current_profile {
                write_empty_tag(out, e);
            } else {
                // skip this empty element
            }
        } else {
            write_empty_tag(out, e);
        }
    }
}

#[inline]
fn handle_text_event(
    e: &quick_xml::events::BytesText,
    skip_depth: usize,
    out: &mut String,
) {
    if skip_depth == 0 {
        out.push_str(&String::from_utf8_lossy(e));
    }
}

#[inline]
fn handle_end_event(
    e: &quick_xml::events::BytesEnd,
    skip_depth: &mut usize,
    out: &mut String,
) {
    if *skip_depth > 0 {
        *skip_depth -= 1;
    } else {
        write_end_tag(out, e);
    }
}

fn write_start_tag(
    out: &mut String,
    e: &quick_xml::events::BytesStart,
) {
    let name_binding = e.name();
    let name = String::from_utf8_lossy(name_binding.as_ref());
    let _ = write!(out, "<{name}");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let val = String::from_utf8_lossy(&attr.value);
        let _ = write!(out, " {key}=\"{val}\"");
    }
    out.push('>');
}

fn write_empty_tag(
    out: &mut String,
    e: &quick_xml::events::BytesStart,
) {
    let name_binding = e.name();
    let name = String::from_utf8_lossy(name_binding.as_ref());
    let _ = write!(out, "<{name}");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        let val = String::from_utf8_lossy(&attr.value);
        let _ = write!(out, " {key}=\"{val}\"");
    }
    out.push_str("/>");
}

fn write_end_tag(out: &mut String, e: &quick_xml::events::BytesEnd) {
    let name_binding = e.name();
    let name = String::from_utf8_lossy(name_binding.as_ref());
    let _ = write!(out, "</{name}>");
}
