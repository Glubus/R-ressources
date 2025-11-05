use r_ressources::{Color, LatLng, Position, UrlParts};

#[test]
fn test_color_api() {
    let c = Color::new(0x12, 0x34, 0x56, 0x78);
    assert_eq!(c.r(), 0x12);
    assert_eq!(c.g(), 0x34);
    assert_eq!(c.b(), 0x56);
    assert_eq!(c.a(), 0x78);
    assert_eq!(c.to_rgb_tuple(), (0x12, 0x34, 0x56));
    assert_eq!(c.to_rgba_u32(), 0x7812_3456);
}

#[test]
fn test_urlparts_api() {
    let u = UrlParts::new("https", "example.com", "/docs");
    assert_eq!(u.scheme(), "https");
    assert_eq!(u.host(), "example.com");
    assert_eq!(u.path(), "/docs");
}

#[test]
fn test_position_api() {
    let p1 = Position::new(0.0, 0.0);
    let p2 = Position::new(3.0, 4.0);
    assert!((p1.distance_to(&p2) - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_latlng_api() {
    let ll = LatLng::new(48.8566, 2.3522);
    assert!((ll.lat() - 48.8566).abs() < f64::EPSILON);
    assert!((ll.lng() - 2.3522).abs() < f64::EPSILON);
}


