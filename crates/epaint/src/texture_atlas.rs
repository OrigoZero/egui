use ecolor::Color32;
use emath::{Rect, remap_clamp};

use crate::{ColorImage, ImageDelta, TextOptions};

/// [`TextureAtlas::max_glyph_extent`] is the atlas side divided by this.
const GLYPH_EXTENT_DIVISOR: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rectu {
    /// inclusive
    min_x: usize,

    /// inclusive
    min_y: usize,

    /// exclusive
    max_x: usize,

    /// exclusive
    max_y: usize,
}

impl Rectu {
    const NOTHING: Self = Self {
        min_x: usize::MAX,
        min_y: usize::MAX,
        max_x: 0,
        max_y: 0,
    };
    const EVERYTHING: Self = Self {
        min_x: 0,
        min_y: 0,
        max_x: usize::MAX,
        max_y: usize::MAX,
    };
}

#[derive(Copy, Clone, Debug)]
struct PrerasterizedDisc {
    r: f32,
    uv: Rectu,
}

/// A pre-rasterized disc (filled circle), somewhere in the texture atlas.
#[derive(Copy, Clone, Debug)]
pub struct PreparedDisc {
    /// The radius of this disc in texels.
    pub r: f32,

    /// Width in texels.
    pub w: f32,

    /// Where in the texture atlas the disc is.
    /// Normalized in 0-1 range.
    pub uv: Rect,
}

/// Contains font data in an atlas, where each character occupied a small rectangle.
///
/// More characters can be added, possibly expanding the texture.
#[derive(Clone)]
pub struct TextureAtlas {
    image: ColorImage,

    /// What part of the image that is dirty
    dirty: Rectu,

    /// Used for when allocating new rectangles.
    cursor: (usize, usize),

    row_height: usize,

    /// Set when someone requested more space than was available.
    overflowed: bool,

    /// pre-rasterized discs of radii `2^i`, where `i` is the index.
    discs: Vec<PrerasterizedDisc>,

    /// Controls how to convert glyph coverage to alpha.
    options: TextOptions,
}

impl TextureAtlas {
    pub fn new(size: [usize; 2], options: TextOptions) -> Self {
        assert!(size[0] >= 1024, "Tiny texture atlas");
        let mut atlas = Self {
            image: ColorImage::filled(size, Color32::TRANSPARENT),
            dirty: Rectu::EVERYTHING,
            cursor: (0, 0),
            row_height: 0,
            overflowed: false,
            discs: vec![], // will be filled in below
            options,
        };

        // Make the top left pixel fully white for `WHITE_UV`, i.e. painting something with solid color:
        let (pos, image) = atlas.allocate((1, 1));
        assert_eq!(
            pos,
            (0, 0),
            "Expected the first allocation to be at (0, 0), but was at {pos:?}"
        );
        image[pos] = Color32::WHITE;

        // Allocate a series of anti-aliased discs used to render small filled circles:
        // TODO(emilk): these circles can be packed A LOT better.
        // In fact, the whole texture atlas could be packed a lot better.
        // for r in [1, 2, 4, 8, 16, 32, 64] {
        //     let w = 2 * r + 3;
        //     let hw = w as i32 / 2;
        const LARGEST_CIRCLE_RADIUS: f32 = 8.0; // keep small so that the initial texture atlas is small
        for i in 0.. {
            let r = 2.0_f32.powf(i as f32 / 2.0 - 1.0);
            if r > LARGEST_CIRCLE_RADIUS {
                break;
            }
            let hw = (r + 0.5).ceil() as i32;
            let w = (2 * hw + 1) as usize;
            let ((x, y), image) = atlas.allocate((w, w));
            for dx in -hw..=hw {
                for dy in -hw..=hw {
                    let distance_to_center = ((dx * dx + dy * dy) as f32).sqrt();
                    let coverage =
                        remap_clamp(distance_to_center, (r - 0.5)..=(r + 0.5), 1.0..=0.0);
                    image[((x as i32 + hw + dx) as usize, (y as i32 + hw + dy) as usize)] =
                        options.alpha_from_coverage.color_from_coverage(coverage);
                }
            }
            atlas.discs.push(PrerasterizedDisc {
                r,
                uv: Rectu {
                    min_x: x,
                    min_y: y,
                    max_x: x + w,
                    max_y: y + w,
                },
            });
        }

        atlas
    }

    pub fn options(&self) -> &TextOptions {
        &self.options
    }

    pub fn size(&self) -> [usize; 2] {
        self.image.size
    }

    /// Returns the locations and sizes of pre-rasterized discs (filled circles) in this atlas.
    pub fn prepared_discs(&self) -> Vec<PreparedDisc> {
        let size = self.size();
        let inv_w = 1.0 / size[0] as f32;
        let inv_h = 1.0 / size[1] as f32;
        self.discs
            .iter()
            .map(|disc| {
                let r = disc.r;
                let Rectu {
                    min_x,
                    min_y,
                    max_x,
                    max_y,
                } = disc.uv;
                let w = max_x - min_x;
                let uv = Rect::from_min_max(
                    emath::pos2(min_x as f32 * inv_w, min_y as f32 * inv_h),
                    emath::pos2(max_x as f32 * inv_w, max_y as f32 * inv_h),
                );
                PreparedDisc { r, w: w as f32, uv }
            })
            .collect()
    }

    fn max_height(&self) -> usize {
        // the initial width is set to the max size
        self.image.height().max(self.image.width())
    }

    /// When this get high, it might be time to clear and start over!
    pub fn fill_ratio(&self) -> f32 {
        if self.overflowed {
            1.0
        } else {
            (self.cursor.1 + self.row_height) as f32 / self.max_height() as f32
        }
    }

    /// The texture options suitable for a font texture
    #[inline]
    pub fn texture_options() -> crate::textures::TextureOptions {
        crate::textures::TextureOptions::LINEAR
    }

    /// The full font atlas image.
    #[inline]
    pub fn image(&self) -> &ColorImage {
        &self.image
    }

    /// Call to get the change to the image since last call.
    pub fn take_delta(&mut self) -> Option<ImageDelta> {
        let texture_options = Self::texture_options();

        let dirty = std::mem::replace(&mut self.dirty, Rectu::NOTHING);
        if dirty == Rectu::NOTHING {
            None
        } else if dirty == Rectu::EVERYTHING {
            Some(ImageDelta::full(self.image.clone(), texture_options))
        } else {
            let pos = [dirty.min_x, dirty.min_y];
            let size = [dirty.max_x - dirty.min_x, dirty.max_y - dirty.min_y];
            let region = self.image.region_by_pixels(pos, size);
            Some(ImageDelta::partial(pos, region, texture_options))
        }
    }

    /// The longest side, in texels, of a glyph the atlas places.
    ///
    /// A glyph whose rasterization is larger is rasterized at this extent and
    /// drawn magnified over its box, so text of any size draws while a page
    /// of glyphs at that extent still fits in the atlas.
    pub fn max_glyph_extent(&self) -> usize {
        self.image.width() / GLYPH_EXTENT_DIVISOR
    }

    /// The row the cursor restarts at when the atlas runs out of rows. The
    /// rows above it hold the white texel, the discs and the glyphs drawn
    /// first, which stay valid until the atlas is recreated.
    fn restart_row(&self) -> usize {
        self.max_height() / 3
    }

    /// Returns the coordinates of where the rect ended up,
    /// and invalidates the region.
    ///
    /// The rect always lies inside the image, which never grows past the side
    /// the atlas was created with. When the rows run out, the cursor restarts
    /// at [`Self::restart_row`] and the rect is placed over glyphs placed
    /// earlier; [`Self::fill_ratio`] then reads full, so the atlas is recreated
    /// before the next pass.
    pub fn allocate(&mut self, (w, h): (usize, usize)) -> ((usize, usize), &mut ColorImage) {
        /// On some low-precision GPUs (my old iPad) characters get muddled up
        /// if we don't add some empty pixels between the characters.
        /// On modern high-precision GPUs this is not needed.
        const PADDING: usize = 1;

        assert!(
            w <= self.image.width(),
            "Tried to allocate a {} wide glyph in a {} wide texture atlas",
            w,
            self.image.width()
        );
        assert!(
            h <= self.max_height() - self.restart_row(),
            "Tried to allocate a {} tall glyph in a texture atlas that places at most {} below the row it restarts at",
            h,
            self.max_height() - self.restart_row()
        );
        if self.cursor.0 + w > self.image.width() {
            // New row:
            self.cursor.0 = 0;
            self.cursor.1 += self.row_height + PADDING;
            self.row_height = 0;
        }

        self.row_height = self.row_height.max(h);

        if self.cursor.1 + self.row_height > self.max_height() {
            // This is a bad place to be - we need to start reusing space :/

            log::warn!("epaint texture atlas overflowed!");

            self.cursor = (0, self.restart_row());
            self.row_height = h;
            self.overflowed = true; // this will signal the user that we need to recreate the texture atlas next frame.
        }

        let required_height = self.cursor.1 + self.row_height;
        if self.grow_to_min_height(required_height) {
            self.dirty = Rectu::EVERYTHING;
        }

        let pos = self.cursor;
        self.cursor.0 += w + PADDING;

        self.dirty.min_x = self.dirty.min_x.min(pos.0);
        self.dirty.min_y = self.dirty.min_y.min(pos.1);
        self.dirty.max_x = self.dirty.max_x.max(pos.0 + w);
        self.dirty.max_y = self.dirty.max_y.max(pos.1 + h);

        (pos, &mut self.image)
    }

    /// Doubles the image's height until it holds `required_height` rows, at
    /// most up to [`Self::max_height`]. Returns whether the image grew.
    fn grow_to_min_height(&mut self, required_height: usize) -> bool {
        let max_height = self.max_height();
        while required_height > self.image.height() && self.image.height() < max_height {
            self.image.size[1] = (self.image.size[1] * 2).min(max_height);
        }

        if self.image.width() * self.image.height() > self.image.pixels.len() {
            self.image.pixels.resize(
                self.image.width() * self.image.height(),
                Color32::TRANSPARENT,
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rect the atlas hands out lies inside its image, and the image
    /// stays inside the side it was created with, through the rows running
    /// out and the cursor restarting, for rects up to the tallest it places.
    #[test]
    fn every_rect_lies_inside_the_image_through_an_overflow() {
        const SIDE: usize = 2048;
        let mut atlas = TextureAtlas::new([SIDE, 32], TextOptions::default());
        let tallest = SIDE - SIDE / 3;
        let sizes = [
            (40, 60),
            (300, 200),
            (SIDE, 1),
            (12, tallest),
            (700, 900),
            (5, 5),
            (SIDE, tallest),
            (256, 256),
        ];
        for round in 0..40 {
            let (w, h) = sizes[round % sizes.len()];
            let ((x, y), image) = atlas.allocate((w, h));
            let [iw, ih] = image.size;
            assert!(
                x + w <= iw && y + h <= ih,
                "round {round}: a {w}x{h} rect at ({x}, {y}) lies outside the {iw}x{ih} image"
            );
            assert!(
                iw <= SIDE && ih <= SIDE,
                "round {round}: image {iw}x{ih} past the {SIDE} side"
            );
            image[(x + w - 1, y + h - 1)] = Color32::WHITE;
        }
        assert_eq!(
            atlas.fill_ratio(),
            1.0,
            "the rows ran out, so the atlas reads full"
        );
    }

    /// Rects that fill the atlas exactly to its last row grow it to its side
    /// and no further.
    #[test]
    fn rows_ending_on_the_last_texel_keep_the_image_at_its_side() {
        const SIDE: usize = 1024;
        let mut atlas = TextureAtlas::new([SIDE, 32], TextOptions::default());
        let ((_, mut y), _) = atlas.allocate((SIDE, 1));
        let mut h = 1;
        while SIDE - (y + h + 1) > 300 {
            let (pos, _) = atlas.allocate((SIDE, 300));
            (y, h) = (pos.1, 300);
        }
        let last = SIDE - (y + h + 1);
        let ((_, y), image) = atlas.allocate((SIDE, last));
        assert_eq!(y + last, SIDE, "the rect ends on the atlas's last row");
        assert_eq!(image.size, [SIDE, SIDE]);
        assert!(atlas.fill_ratio() <= 1.0, "nothing overflowed");
    }
}
