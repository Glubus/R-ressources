use r_ressources::{float, int, string, string_array, int_array, float_array};

fn main() {
    println!("=== R Resources Demo ===\n");

    // Strings
    println!("Strings:");
    println!("  App Name: {}", string::APP_NAME);
    println!("  Welcome: {}", string::WELCOME_MESSAGE);
    println!("  Error: {}", string::ERROR_NETWORK);

    // Ints
    println!("\nIntegers:");
    println!("  Max Retries: {}", int::MAX_RETRIES);
    println!("  Timeout: {}ms", int::TIMEOUT_MS);
    println!("  Cache Size: {}", int::CACHE_SIZE);

    // Floats
    println!("\nFloats:");
    println!("  Default Rate: {}", float::DEFAULT_RATE);
    println!("  Tax Rate: {}%", float::TAX_RATE * 100.0);
    println!("  Version: {}", float::VERSION);

    // Arrays
    println!("\nArrays:");
    println!("  Supported Languages: {:?}", string_array::SUPPORTED_LANGS);
    println!("  Fibonacci: {:?}", int_array::FIBONACCI);
    println!("  Prices: {:?}", float_array::PRICES);
}
