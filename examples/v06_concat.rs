fn main() {
    println!("=== String Interpolation ===");
    println!("welcome: {}", r_ressources::string::WELCOME_TITLE);
    println!(
        "api_url_with_version: {}",
        r_ressources::string::API_URL_WITH_VERSION
    );

    println!("\n=== Template Functions ===");
    println!(
        "greeting: {}",
        r_ressources::string::greeting("Alice", 5)
    );
    println!(
        "status: {}",
        r_ressources::string::status("bob", true)
    );
}


