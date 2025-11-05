/// Tests for v0.2.0 new resource types
use r_ressources::{bool, color, dimension, int, r, string, url};

#[test]
fn test_bool_resources() {
    // Touch constants (avoids constant assertions under clippy strict)
    std::hint::black_box(bool::DEBUG_MODE);
    std::hint::black_box(bool::ENABLE_ANALYTICS);
    std::hint::black_box(bool::SHOW_TUTORIAL);
    
    // Test flat access
    std::hint::black_box(r::DEBUG_MODE);
    std::hint::black_box(r::ENABLE_ANALYTICS);
}

#[test]
fn test_color_resources() {
    assert_eq!(color::PRIMARY, "#FF5722");
    assert_eq!(color::SECONDARY, "#03A9F4");
    assert_eq!(color::BACKGROUND, "#FFFFFF");
    assert_eq!(color::TEXT_DARK, "#212121");
    
    // Test flat access
    assert_eq!(r::PRIMARY, "#FF5722");
}

#[test]
fn test_url_resources() {
    assert_eq!(url::API_BASE, "https://api.example.com");
    assert_eq!(url::WEBSITE, "https://example.com");
    assert_eq!(url::DOCS, "https://docs.example.com");
    
    // Test flat access
    assert_eq!(r::API_BASE, "https://api.example.com");
}

#[test]
fn test_dimension_resources() {
    assert_eq!(dimension::PADDING_SMALL, "8dp");
    assert_eq!(dimension::PADDING_MEDIUM, "16dp");
    assert_eq!(dimension::PADDING_LARGE, "24dp");
    assert_eq!(dimension::FONT_SIZE, "14sp");
    
    // Test flat access
    assert_eq!(r::PADDING_SMALL, "8dp");
}

#[test]
fn test_multi_file_loading() {
    // Resources from values.xml
    assert_eq!(string::APP_NAME, "My Awesome App");
    assert_eq!(int::MAX_RETRIES, 3);
    
    // Resources from config.xml
    std::hint::black_box(bool::DEBUG_MODE);
    assert_eq!(color::PRIMARY, "#FF5722");
    assert_eq!(url::API_BASE, "https://api.example.com");
    assert_eq!(dimension::PADDING_SMALL, "8dp");
    
    // All accessible via r:: module
    assert_eq!(r::APP_NAME, "My Awesome App");
    std::hint::black_box(r::DEBUG_MODE);
    assert_eq!(r::PRIMARY, "#FF5722");
}

