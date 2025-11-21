use r_resources::include_resources;
include_resources!();

fn main() {
    println!("=== R Resources Demo ===\n");

    println!("  App Name: {}", r::APP_NAME);
    println!("  Max Retries: {}", r::MAX_RETRIES);
    println!("  Timeout: {}ms", r::TIMEOUT_MS);
    println!("  Rate: {}", r::RATE);
    println!("  Tax Rate: {}%", r::TAX_RATE * 100.0);
    println!("  Debug Mode: {}", r::DEBUG_MODE);

    println!("  Auth Title: {}", r::auth::TITLE);
    println!("  Auth Error Message: {}", r::auth::ERROR_MESSAGE);
}
