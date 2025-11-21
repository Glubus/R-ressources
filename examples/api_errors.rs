use r_resources::include_resources;
include_resources!();

fn main() {
    println!("=== API Errors Demo ===\n");

    // Example: Using French error messages
    println!("🇫🇷 French Error Messages:");
    println!("  Unauthorized: {}", r::fr::api::error::UNAUTHORIZED);
    println!("  Forbidden: {}", r::fr::api::error::FORBIDDEN);
    println!("  Not Found: {}", r::fr::api::error::NOT_FOUND);
    println!("  Bad Request: {}", r::fr::api::error::BAD_REQUEST);
    println!("  Internal Server Error: {}", r::fr::api::error::INTERNAL_SERVER_ERROR);
    println!("  Rate Limit Exceeded: {}", r::fr::api::error::RATE_LIMIT_EXCEEDED);
    println!("  Validation Failed: {}", r::fr::api::error::VALIDATION_FAILED);
    println!("  Token Expired: {}", r::fr::api::error::TOKEN_EXPIRED);
    println!();

    // Example: Using English error messages
    println!("🇬🇧 English Error Messages:");
    println!("  Unauthorized: {}", r::en::api::error::UNAUTHORIZED);
    println!("  Forbidden: {}", r::en::api::error::FORBIDDEN);
    println!("  Not Found: {}", r::en::api::error::NOT_FOUND);
    println!("  Bad Request: {}", r::en::api::error::BAD_REQUEST);
    println!("  Internal Server Error: {}", r::en::api::error::INTERNAL_SERVER_ERROR);
    println!("  Rate Limit Exceeded: {}", r::en::api::error::RATE_LIMIT_EXCEEDED);
    println!("  Validation Failed: {}", r::en::api::error::VALIDATION_FAILED);
    println!("  Token Expired: {}", r::en::api::error::TOKEN_EXPIRED);
    println!();

    // Example: Using error codes
    println!("📊 HTTP Error Codes:");
    println!("  Unauthorized: {}", r::api::error::UNAUTHORIZED_CODE);
    println!("  Forbidden: {}", r::api::error::FORBIDDEN_CODE);
    println!("  Not Found: {}", r::api::error::NOT_FOUND_CODE);
    println!("  Bad Request: {}", r::api::error::BAD_REQUEST_CODE);
    println!("  Internal Server Error: {}", r::api::error::INTERNAL_SERVER_ERROR_CODE);
    println!("  Rate Limit Exceeded: {}", r::api::error::RATE_LIMIT_EXCEEDED_CODE);
    println!();

    // Example: Using error templates
    println!("📝 Error Templates:");
    let error_msg = r::error_with_details(
        "Authentication",
        "Invalid credentials provided",
        401
    );
    println!("  {}", error_msg);
    println!();

    let validation_msg = r::validation_error("email", "must be a valid email address");
    println!("  {}", validation_msg);
    println!();

    // Example: Simulating an API error handler
    println!("🔧 Simulated API Error Handler:");
    simulate_api_error("fr", "unauthorized");
    simulate_api_error("en", "not_found");
    simulate_api_error("fr", "rate_limit_exceeded");
}

fn simulate_api_error(locale: &str, error_type: &str) {
    let message = match (locale, error_type) {
        ("fr", "unauthorized") => r::fr::api::error::UNAUTHORIZED,
        ("fr", "forbidden") => r::fr::api::error::FORBIDDEN,
        ("fr", "not_found") => r::fr::api::error::NOT_FOUND,
        ("fr", "bad_request") => r::fr::api::error::BAD_REQUEST,
        ("fr", "rate_limit_exceeded") => r::fr::api::error::RATE_LIMIT_EXCEEDED,
        ("en", "unauthorized") => r::en::api::error::UNAUTHORIZED,
        ("en", "forbidden") => r::en::api::error::FORBIDDEN,
        ("en", "not_found") => r::en::api::error::NOT_FOUND,
        ("en", "bad_request") => r::en::api::error::BAD_REQUEST,
        ("en", "rate_limit_exceeded") => r::en::api::error::RATE_LIMIT_EXCEEDED,
        _ => "Unknown error",
    };

    let code = match error_type {
        "unauthorized" => r::api::error::UNAUTHORIZED_CODE,
        "forbidden" => r::api::error::FORBIDDEN_CODE,
        "not_found" => r::api::error::NOT_FOUND_CODE,
        "bad_request" => r::api::error::BAD_REQUEST_CODE,
        "rate_limit_exceeded" => r::api::error::RATE_LIMIT_EXCEEDED_CODE,
        _ => 500,
    };

    println!("  [{}] {} (HTTP {})", locale.to_uppercase(), message, code);
}

