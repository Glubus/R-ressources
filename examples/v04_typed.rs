use r_ressources::{color_t, url_t};

fn main() {
    println!("=== r-ressources v0.4.0 (typed resources) ===\n");

    // Color typed from resources (build-generated)
    let c = color_t::ACCENT;
    println!("Color rgba=({}, {}, {}, {}) rgba_u32=0x{:08X}", c.r(), c.g(), c.b(), c.a(), c.to_rgba_u32());

    // URL typed from resources (build-generated)
    let api = url_t::API_BASE;
    println!("Url: {}://{}{}", api.scheme(), api.host(), api.path());
}


