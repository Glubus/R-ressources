use r_resources::include_resources;
include_resources!();
use std::str::FromStr;

fn main() {
    println!("=== R Resources Demo ===\n");

    println!("  App Name: {}", r::APP_NAME);
    println!("  Max Retries: {}", r::MAX_RETRIES);
    println!("  Timeout: {}ms", r::TIMEOUT_MS);
    println!("  Rate: {}", r::RATE);
    println!("  Tax Rate: {}%", r::TAX_RATE * 100.0);
    println!("  Debug Mode: {}", r::DEBUG_MODE);

    println!("  Auth Title: {}", r::auth::TITLE);
    println!("  Auth Error Message: {}", r::auth::error::CREDENTIALS);
    println!("  Big Number: {:?}", r::BIG_NUMBER.to_string());
    println!("  Auto Big Number: {:?}", r::AUTO_BIG_NUMBER.to_string());
    println!("  Auto Big Decimal: {:?}", r::AUTO_BIG_DECIMAL.to_string());

    println!("  Welcome Message: {}", r::welcome_message("John", r_resources::BigDecimal::from_str("10").unwrap()));
}
