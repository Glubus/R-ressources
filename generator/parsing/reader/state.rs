#[derive(Default)]
pub(super) struct ParseState {
    pub(super) current_tag: String,
    pub(super) current_name: Option<String>,
    pub(super) namespace_stack: Vec<String>,
    pub(super) current_number_type: Option<String>, // For <number type="...">
}
