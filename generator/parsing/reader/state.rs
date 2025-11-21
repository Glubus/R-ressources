#[derive(Default)]
pub(super) struct ParseState {
    pub(super) current_tag: String,
    pub(super) current_name: Option<String>,
    pub(super) namespace_stack: Vec<String>,
    pub(super) current_number_type: Option<String>, // For <number type="...">
    pub(super) template_params: Vec<crate::generator::parsing::ast::TemplateParam>, // For <template><param>
    pub(super) template_text: String, // Accumulated text for templates
    pub(super) in_template: bool, // Track if we're inside a <template> tag
}
