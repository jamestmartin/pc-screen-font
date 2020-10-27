#![no_std]
extern crate alloc;

use alloc::vec::Vec;

pub struct Font {
    /// The width in pixels of this font's bounding box.
    width: usize,
    /// The height in pixels of this font's bounding box.
    height: usize,
    /// Every single glyph in the font.
    glyphs: Vec<Glyph>,
    // TODO: Replace this with a proper associative structure.
    /// A map between unicode characters and indexes into the glyph vec.
    unicode: Vec<UnicodeMap>,
}

/// Associates a unicode character and a glyph.
struct UnicodeMap {
    c: char,
    // The index of the glyph.
    i: usize,
}

pub struct Glyph {
    /// A set bit indicates that a pixel should be drawn for this glyph.
    bitmap: Vec<u8>,
    /// The number of bytes in the bitmap taken up by each line of pixels in this glyph.
    line_size: usize,
    /// See the docs for `Glyph::width`.
    width: usize,
    /// See the docs for `Glyph::height`.
    height: usize,
}

impl Glyph {
    /// The width in pixels of this individual glyph.
    ///
    /// Although each PSF has a nominal width in pixels,
    /// the length of each line of pixels stored in bits is actually rounded to the next byte.
    /// Some fonts actually use this storage to make glyphs which are slightly wider
    /// than the nominal width of the font, because it makes the glyph look better.
    /// For example, Cozette does this for the unicode heart glyph.
    pub fn width(&self) -> usize { self.width }
    /// The height in pixels of this glyph. This will always be the same as the height of the font.
    pub fn height(&self) -> usize { self.height }

    /// Check whether an individual pixel of this glyph is set.
    /// This will return `None` if `x` or `y` is outside the width or height of this glyph.
    pub fn get(&self, x: usize, y: usize) -> Option<bool> {
        if x > self.width || y > self.height {
            return None
        }

        let (line_byte_index, bit_index) = num_integer::div_rem(x, 8);
        let mask = 0b10000000 >> bit_index;
        let byte = self.bitmap[(y * self.line_size + line_byte_index) as usize];
        Some(byte & mask > 0)
    }
}

impl Font {
    /// The width in pixels of this font's bounding box.
    pub fn width(&self) -> usize { self.width }
    /// The height in pixels of this font's bounding box.
    pub fn height(&self) -> usize { self.height }
    /// The width and height in pixels of this font's bounding box.
    pub fn bounding_box(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get the glyph associated with a particular unicode character,
    /// or `None` if it is not present in this font.
    pub fn lookup<'a>(&'a self, c: char) -> Option<&'a Glyph> {
        self.index_of(c).map(|i| &self.glyphs[i])
    }

    /// The index of the glyph associated with a particular unicode character.
    fn index_of(&self, c: char) -> Option<usize> {
        for entry in &self.unicode {
            if entry.c == c {
                return Some(entry.i);
            }
        }
        None
    }

    /// Parse a version 2 PC screen font from its bytes.
    pub fn parse(font: &[u8]) -> Font {
        use core::convert::TryInto;

        // The number of glyphs in this font.
        let length = u32::from_le_bytes(font[16..20].try_into().unwrap()) as usize;
        // The size in bytes of a single glyph.
        let charsize = u32::from_le_bytes(font[20..24].try_into().unwrap()) as usize;
        // The height in pixels of this font's bounding box.
        let height = u32::from_le_bytes(font[24..28].try_into().unwrap()) as usize;
        // The width in pixels of this font's bounding box.
        let width = u32::from_le_bytes(font[28..32].try_into().unwrap()) as usize;
        // The size in bytes of a single row of pixels in a glyph.
        let line_size = num_integer::div_ceil(width, 8);

        let glyphs_offset = 32; // the size of the header
        let glyphs_size = length * charsize;
        let unicode_offset = glyphs_offset + glyphs_size;

        let mut glyphs = Vec::with_capacity(length);

        for i in 0..length {
            let mut bitmap = Vec::with_capacity(charsize);
            let bitmap_begin = glyphs_offset + charsize * i;
            let bitmap_end = bitmap_begin + charsize;
            bitmap.extend_from_slice(&font[bitmap_begin..bitmap_end]);

            glyphs.push(Glyph {
                bitmap,
                line_size,
                // Glyphs may overflow the font's nominal resolution in the padding bytes of the line!
                // This trick only works for the width because there is no vertical padding.
                // TODO: Pre-compute widths and bounding box offsets of individual glyphs.
                width: line_size * 8,
                height,
            });
        }

        // HACK: This unicode map parser is still a mess.
        let mut unicode_map = Vec::new();
        let unicode_info = &font[unicode_offset..];
        let mut glyph = 0;
        let mut i = 0;
        while i < unicode_info.len() {
            let mut nc = unicode_info[i];

            while nc != 0xFE && nc != 0xFF {
                let ch_bytes = nc.leading_ones().max(1) as usize;
                let st = core::str::from_utf8(&unicode_info[i..i + ch_bytes]).expect("Invalid character");
                let ch = st.chars().next().unwrap();
                unicode_map.push(UnicodeMap { c: ch, i: glyph });
                i += ch_bytes;
                nc = unicode_info[i];
            }

            // TODO: Support multi-codepoint spellings of characters.
            while nc != 0xFF {
                i += 1;
                nc = unicode_info[i];
            }

            i += 1;
            glyph += 1;
        }

        Font {
            width,
            height,
            glyphs,
            unicode: unicode_map,
        }
    }
}
