fn main() {
    use r_ressources::r;
    
    println!("=== Namespaced Resources (v0.5.0+) ===\n");
    
    // Access namespaced strings via type-organized modules
    println!("Type-organized access:");
    println!("  auth title: {}", r_ressources::string::auth::TITLE);
    println!(
        "  auth invalid: {}",
        r_ressources::string::auth::errors::INVALID_CREDENTIALS
    );

    // Access via flat r:: module (Kotlin-style: r::auth::title)
    println!("\nFlat access (Kotlin-style):");
    println!("  auth title: {}", r::auth::TITLE);
    println!("  auth invalid: {}", r::auth::errors::INVALID_CREDENTIALS);

    // Colors with reference inside namespace
    println!("\nColors:");
    println!("  primary: {}", r_ressources::color::ui::colors::PRIMARY);
    println!("  primary (via r::): {}", r::ui::colors::PRIMARY);
    println!(
        "  primary button: {}",
        r_ressources::color::ui::colors::PRIMARY_BUTTON
    );

    // Dimension
    println!("\nDimensions:");
    println!("  padding: {}", r_ressources::dimension::ui::dimens::PADDING);
    println!("  padding (via r::): {}", r::ui::dimens::PADDING);

    // Arrays
    println!("\nArrays:");
    println!("  langs: {:?}", r_ressources::string_array::lists::LANGS);
    println!("  langs (via r::): {:?}", r::lists::LANGS);
    println!(
        "  small_numbers: {:?}",
        r_ressources::int_array::lists::SMALL_NUMBERS
    );
    println!("  ratios: {:?}", r_ressources::float_array::lists::RATIOS);
}


