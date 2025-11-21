use quick_xml::events::{BytesStart, BytesText};

pub(super) fn attr_value(
    e: &BytesStart<'_>,
    name: &[u8],
) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name {
            return Some(to_string(attr.value.as_ref()));
        }
    }
    None
}

pub(super) fn to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

pub(super) fn text_to_string(text: &BytesText<'_>) -> String {
    String::from_utf8_lossy(text.as_ref()).to_string()
}
