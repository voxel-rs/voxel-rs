extern crate texture_packer;
extern crate image;

use gfx;
use std::collections::HashMap;
use self::texture_packer::TexturePackerConfig;

/// List of loaded textures and their position in the atlas.
pub struct TextureRegistry {
    textures: HashMap<String, TextureRect>,
}

/// Texture position in the atlas.
#[derive(Clone)]
pub struct TextureRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

const TEXTURE_PACKER_CONFIG: TexturePackerConfig = TexturePackerConfig {
    max_width: MAX_TEXTURE_SIZE,
    max_height: MAX_TEXTURE_SIZE,
    allow_rotation: false,
    border_padding: 0,
    texture_padding: 0,
    trim: false,
    texture_outlines: false,
};

pub fn load_textures<F, R>(factory: &mut F) -> (gfx::handle::ShaderResourceView<R, [f32; 4]>, TextureRegistry)
    where F: gfx::Factory<R>, R: gfx::Resources
{
    use self::image::{ImageBuffer, GenericImage};
    use self::texture_packer::TexturePacker;
    use self::texture_packer::importer::ImageImporter;
    use self::texture_packer::exporter::ImageExporter;
    use std::path::Path;

    let mut packer = TexturePacker::new_skyline(TEXTURE_PACKER_CONFIG);
    let mut registry = TextureRegistry::new();
    let textures = [
        "dirt",
        "grass_side",
        "grass_top",
        "wood_side",
        "wood_top",
        "leaves",
    ];
    for &tex in &textures {
        let path = format!("assets/{}.png", tex);
        packer.pack_own(String::from(tex), ImageImporter::import_from_file(&Path::new(&path)).unwrap());
    }
    for (name, frame) in packer.get_frames() {
        let frame = frame.frame;
        registry.add_texture(name, TextureRect {
            x: (frame.x as f32)/MAX_TEXTURE_SIZE_F,
            y: (frame.y as f32)/MAX_TEXTURE_SIZE_F,
            w: (frame.w as f32)/MAX_TEXTURE_SIZE_F,
            h: (frame.h as f32)/MAX_TEXTURE_SIZE_F,
        });
    }
    let mut buffer = ImageBuffer::new(MAX_TEXTURE_SIZE, MAX_TEXTURE_SIZE);
    buffer.copy_from(&ImageExporter::export(&packer).unwrap(), 0, 0);
    let kind = gfx::texture::Kind::D2(MAX_TEXTURE_SIZE as u16, MAX_TEXTURE_SIZE as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Rgba8>(kind, &[&buffer]).unwrap();
    (view, registry)
}

const MAX_TEXTURE_SIZE: u32 = 1024;
const MAX_TEXTURE_SIZE_F: f32 = MAX_TEXTURE_SIZE as f32;

impl TextureRegistry {
    pub fn new() -> Self {
        TextureRegistry {
            textures: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, name: &str, rect: TextureRect) {
        self.textures.insert(String::from(name), rect);
    }

    pub fn get_position(&self, name: &str) -> TextureRect {
        self.textures.get(name).unwrap().clone()
    }
}

impl TextureRect {
    pub fn get_pos(&self, uv: (f32, f32)) -> (f32, f32) {
        assert!(0. <= uv.0 && uv.0 <= 1.);
        assert!(0. <= uv.1 && uv.1 <= 1.);
        (self.x + self.w * uv.0, self.y + self.h * uv.1)
    }
}