use rust_embed::{EmbeddedFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "data/"]
#[exclude = "img/*"]
pub struct HelloData;

// Get the `EmbeddedFile` for the given path.
pub fn get(file_path: &str) -> Option<EmbeddedFile> {
    HelloData::get(file_path)
}
