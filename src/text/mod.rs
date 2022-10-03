use std::{collections::HashMap, ffi::OsStr};

use anyhow::*;

use crate::texture;

pub mod mesh;

const FONTS_DIR: &'static str = "C:/Windows/Fonts";
macro_rules! font_path {
    ($font:literal) => {{
        use crate::text::FONTS_DIR;
        const_format::concatcp!(FONTS_DIR, "/", $font)
    }};
}

pub trait PixelBitSize {
    fn get_size(&self) -> u32;
}

impl PixelBitSize for freetype::bitmap::PixelMode {
    fn get_size(&self) -> u32 {
        use freetype::bitmap::PixelMode;
        match self {
            PixelMode::None => 0,
            PixelMode::Mono => 1,
            PixelMode::Gray => 8,
            PixelMode::Gray2 => 2,
            PixelMode::Gray4 => 4,
            PixelMode::Lcd => 8,
            PixelMode::LcdV => 8,
            PixelMode::Bgra => 32,
        }
    }
}

#[derive(Debug)]
pub struct GlyphRect {
    pub tl: (u32, u32),
    // pub bl: f32,
    pub br: (u32, u32),
    // pub tr: f32,
}

impl GlyphRect {
    pub fn new(tl: (u32, u32), br: (u32, u32)) -> Self {
        Self { tl, br }
    }

    pub fn normalized(&self, h: u32, w: u32) -> ((f32, f32), (f32, f32)) {
        (
            (self.tl.0 as f32 / w as f32, self.tl.1 as f32 / h as f32),
            (self.br.0 as f32 / w as f32, self.br.1 as f32 / h as f32),
        )
    }
    // pub fn new(tl: f32, bl: f32, br: f32, tr: f32) -> Self {
    //     Self { tl, bl, br, tr }
    // }

    // pub fn as_arr(&self) -> [f32; 4] {
    //     [ self.tl, self.bl, self.br, self.tr ]
    // }
}

#[derive(Clone, Debug)]
pub struct GlyphDesc {
    x_start: usize,
    h: i32,
    w: i32,
    pitch: i32, // row stride, add this to go down one row
    bearing_x: i32,
    bearing_y: i32,
    advance: i32, // in 1/64 pixels
}

pub struct LinearTextAtlas {
    sum_pitch: usize,
    max_y_max: usize,
    max_y_min: usize,
    pixel_mode: freetype::bitmap::PixelMode,
    descriptors: Vec<GlyphDesc>,
    bytes: Vec<u8>,
}

impl LinearTextAtlas {
    fn create(face: &freetype::face::Face) -> Result<Self> {
        const COUNT: usize = 128;

        let mut descriptors = Vec::with_capacity(COUNT);
        let mut bytes = Vec::new();

        let mut sum_pitch = 0;
        let (mut max_y_max, mut max_y_min) = (0, 0);

        let mut stride = 0;
        let mut pixel_mode = None;
        for ch in 0..COUNT {
            face.set_char_size(30 * 64, 0, 0, 0).unwrap();
            face.load_char(ch, freetype::face::LoadFlag::RENDER)
                .unwrap();
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            bytes.extend(bitmap.buffer());

            pixel_mode = Some(bitmap.pixel_mode().unwrap());
            dbg!(&pixel_mode);

            let desc = GlyphDesc {
                x_start: stride,
                h: bitmap.rows(),
                w: bitmap.width(),
                // TODO: what if pitch is negative
                // NOTE: do not support for now and produce garbage
                pitch: bitmap.pitch(),
                bearing_x: glyph.bitmap_left(),
                bearing_y: glyph.bitmap_top(),
                advance: glyph.advance().x,
            };
            sum_pitch += desc.pitch;
            max_y_max = max_y_max.max(desc.bearing_y);
            max_y_min = max_y_min.max(desc.h - desc.bearing_y);
            stride += (desc.h * desc.pitch) as usize;

            descriptors.push(desc);
        }

        Ok(Self {
            sum_pitch: sum_pitch as usize,
            max_y_max: max_y_max as usize,
            max_y_min: max_y_min as usize,
            pixel_mode: pixel_mode.unwrap(),
            descriptors,
            bytes,
        })
    }

    pub fn get_glyph_texture(&self, ch: usize) -> (&GlyphDesc, &[u8]) {
        let desc = &self.descriptors[ch];
        let stride = desc.x_start;
        let size = (desc.h * desc.pitch) as usize;

        (desc, &self.bytes[stride..stride + size])
    }
}

pub struct TextAtlas {
    pub descriptors: Vec<GlyphDesc>,
    pub rects: Vec<GlyphRect>,
    pub w: usize,
    pub h: usize,
    pub stride: usize,
    pub bytes: Vec<u8>,
}

impl TextAtlas {
    // TODO: Bearings can be zero
    pub fn create(linear_atlas: &LinearTextAtlas) -> Self {
        const COUNT: usize = 128;

        let fit_w = linear_atlas.sum_pitch;
        let fit_h = linear_atlas.max_y_max + linear_atlas.max_y_min;
        let zero = linear_atlas.max_y_max as i32;

        let descriptors = linear_atlas.descriptors.clone();
        let mut rects = Vec::with_capacity(descriptors.len());
        let mut bytes = vec![0; fit_h * fit_w];

        // bytes[zero-bearing_y..zero-bearing_y+h, x0..x1] =
        // linear_atlas.bytes[stride..stride+size].as_2d(h, pitch);

        let mut x_start = 0;
        for ch in 0..COUNT {
            let (desc, texture) = linear_atlas.get_glyph_texture(ch);
            dbg!(ch, desc);

            // let by = desc.bearing_y as usize;
            // dbg!(zero, by);
            // let (tl, bl) = (
            //     zero - desc.bearing_y,
            //     zero - desc.bearing_y + desc.h - 1,
            // );
            // let (br, tr) = (
            //     tl + desc.pitch - 1,
            //     bl + desc.pitch - 1,
            // );
            let tl = (x_start as u32, zero as u32 - desc.bearing_y as u32);
            let br = (tl.0 + desc.w as u32 - 1, tl.1 + desc.h as u32 - 1);

            for i in 0..desc.h as usize {
                // bytes[...] = texture[pitch*i .. pitch*(i+1)];
                // (
                //     zero - desc.bearing_y as usize + i .. zero - desc.bearing_y as usize + (i+1),
                //     x_start .. x_start + desc.pitch
                // );
                let offset_factor_2d = (tl.1 as usize + i) * fit_w;
                let offset = offset_factor_2d + x_start;
                bytes[offset..offset + desc.pitch as usize]
                    .as_mut()
                    .clone_from_slice(
                        &texture[desc.pitch as usize * i..desc.pitch as usize * (i + 1)],
                    );
            }

            rects.push(GlyphRect::new(tl, br));

            x_start += desc.pitch as usize;
        }

        Self {
            descriptors,
            rects,
            h: fit_h,
            w: fit_w / (linear_atlas.pixel_mode.get_size() / 8) as usize,
            stride: fit_w,
            bytes,
        }
    }
}

pub struct FontContainer {
    face: freetype::face::Face,
    linear_atlas: LinearTextAtlas,
    pub atlas: TextAtlas,
}

impl FontContainer {
    pub fn new(library: &freetype::Library, font_path: &str, face_index: isize) -> Result<Self> {
        let face = library.new_face(font_path, face_index).unwrap();
        let linear_atlas = LinearTextAtlas::create(&face).unwrap();
        let atlas = TextAtlas::create(&linear_atlas);
        Ok(Self {
            face,
            linear_atlas,
            atlas,
        })
    }

    pub fn get_glyph_texture(&self, ch: usize) -> (&GlyphDesc, &[u8]) {
        self.linear_atlas.get_glyph_texture(ch)
    }
}

pub struct TextMap {
    library: freetype::Library,
    pub fonts: HashMap<String, FontContainer>,
}

impl TextMap {
    pub fn new() -> Self {
        Self {
            library: freetype::Library::init().unwrap(),
            fonts: Default::default(),
        }
    }

    pub fn generate_from_path(
        &mut self,
        font: String,
        path: &str,
        face_index: isize,
    ) -> Result<()> {
        self.fonts
            .insert(font, FontContainer::new(&self.library, path, face_index)?);
        Ok(())
    }

    pub fn generate(&mut self, font: String, face_index: isize) -> Result<()> {
        let path = format!("{}/{}", FONTS_DIR, &font);
        self.generate_from_path(font, &path, face_index)
    }
}

#[cfg(test)]
mod tests {
    use super::{FontContainer, TextAtlas};

    #[test]
    fn create_atlas() {
        let library = freetype::Library::init().unwrap();
        let fontc = FontContainer::new(&library, font_path!("arial.ttf"), 0).unwrap();

        let atlas = TextAtlas::create(&fontc.linear_atlas);
        dbg!(&atlas.descriptors[32]);
        dbg!(&atlas.rects[32]);
        image::save_buffer(
            "save/text_atlas.png",
            &atlas.bytes,
            atlas.w as u32,
            atlas.h as u32,
            image::ColorType::L8,
        )
        .unwrap();
    }
}
