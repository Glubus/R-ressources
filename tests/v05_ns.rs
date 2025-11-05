#[test]
fn namespaced_strings_and_refs() {
    use r_ressources::r;
    
    // Type-organized access
    assert_eq!(r_ressources::string::auth::TITLE, "Login");
    assert_eq!(
        r_ressources::string::auth::errors::INVALID_CREDENTIALS,
        "Invalid credentials"
    );
    
    // Flat access (Kotlin-style)
    assert_eq!(r::auth::TITLE, "Login");
    assert_eq!(r::auth::errors::INVALID_CREDENTIALS, "Invalid credentials");
    
    // Both should be the same
    assert_eq!(r_ressources::string::auth::TITLE, r::auth::TITLE);
    assert_eq!(
        r_ressources::string::auth::errors::INVALID_CREDENTIALS,
        r::auth::errors::INVALID_CREDENTIALS
    );
}

#[test]
fn namespaced_colors_and_dimension() {
    use r_ressources::r;
    
    // Raw string constants
    assert_eq!(r_ressources::color::ui::colors::PRIMARY, "#3366FF");
    // PRIMARY_BUTTON references PRIMARY
    assert_eq!(
        r_ressources::color::ui::colors::PRIMARY_BUTTON,
        r_ressources::color::ui::colors::PRIMARY
    );

    assert_eq!(r_ressources::dimension::ui::dimens::PADDING, "16dp");
    
    // Flat access (Kotlin-style)
    assert_eq!(r::ui::colors::PRIMARY, "#3366FF");
    assert_eq!(r::ui::dimens::PADDING, "16dp");
    
    // Both should be the same
    assert_eq!(r_ressources::color::ui::colors::PRIMARY, r::ui::colors::PRIMARY);
}

#[test]
fn namespaced_arrays() {
    use r_ressources::r;
    
    assert_eq!(
        r_ressources::string_array::lists::LANGS,
        &["en", "fr", "es"][..]
    );
    assert_eq!(r_ressources::int_array::lists::SMALL_NUMBERS, &[1, 2, 3][..]);
    assert_eq!(
        r_ressources::float_array::lists::RATIOS,
        &[0.5f64, 1.0, 2.0][..]
    );
    
    // Flat access (Kotlin-style)
    assert_eq!(r::lists::LANGS, &["en", "fr", "es"][..]);
    assert_eq!(r::lists::SMALL_NUMBERS, &[1, 2, 3][..]);
    assert_eq!(r::lists::RATIOS, &[0.5f64, 1.0, 2.0][..]);
    
    // Both should be the same
    assert_eq!(r_ressources::string_array::lists::LANGS, r::lists::LANGS);
}


