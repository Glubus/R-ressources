#[test]
fn interpolated_strings_resolve_at_build_time() {
    // From res/advanced.xml
    assert_eq!(
        r_ressources::string::WELCOME_TITLE,
        "Welcome to My Awesome App!"
    );
    assert_eq!(
        r_ressources::string::API_URL_WITH_VERSION,
        "https://api.example.com/v2"
    );
}

#[test]
fn template_functions_generate_correctly() {
    // Test greeting template with string and int parameters
    let result = r_ressources::string::greeting("Alice", 5);
    assert_eq!(result, "Hello Alice, you have 5 messages!");

    let result2 = r_ressources::string::greeting("Bob", 0);
    assert_eq!(result2, "Hello Bob, you have 0 messages!");

    // Test status template with string and bool parameters
    let result3 = r_ressources::string::status("charlie", true);
    assert_eq!(result3, "User charlie is true");

    let result4 = r_ressources::string::status("dave", false);
    assert_eq!(result4, "User dave is false");
}


