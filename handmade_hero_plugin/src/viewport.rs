use crate::dimensions::Dimensions;
use crate::world::World;
use handmade_hero_interface::rectangle::Rectangle;
use handmade_hero_interface::units::si::length::Length;
use handmade_hero_interface::units::si::linear_number_density::LinearNumberDensity;
use uom::si::ratio::ratio;

/// Maps world space, measured in meters with y growing upward, onto the back buffer, measured
/// in pixels with y growing downward.
///
/// How many pixels a meter spans depends on the size of the buffer, so it is a value fitted at
/// runtime rather than a unit. The world itself never changes size; resizing the buffer only
/// changes how much of the screen each meter covers.
#[derive(Debug, Copy, Clone)]
pub struct Viewport {
    pixels_per_meter: LinearNumberDensity,
    x_offset_px: f32,
    y_offset_px: f32,
    width_px: f32,
    height_px: f32,
}

impl Viewport {
    /// Scales the world so the tile grid covers the whole buffer, then centers the grid on
    /// whichever axis overflows.
    #[must_use]
    #[expect(clippy::cast_precision_loss)]
    pub fn fit(world: &World, resolution_width_px: usize, resolution_height_px: usize) -> Self {
        let width_px = resolution_width_px as f32;
        let height_px = resolution_height_px as f32;
        let tile_width_px = width_px / f32::from(World::TILE_COLUMNS);
        let tile_height_px = height_px / f32::from(World::TILE_ROWS);
        let tile_size_px = tile_height_px.max(tile_width_px);
        // Dividing a count by a length yields a plain 1/length quantity. `LinearNumberDensity`
        // is tagged with uom's concentration kind, so the conversion has to be spelled out.
        let pixels_per_meter = (tile_size_px / world.tile_size()).into();
        let x_growth_px = (tile_height_px - tile_width_px) * f32::from(World::TILE_COLUMNS);
        let x_offset_px = x_growth_px.max(0.0) / -2.0;
        let y_growth_px = (tile_width_px - tile_height_px) * f32::from(World::TILE_ROWS);
        let y_offset_px = y_growth_px.max(0.0) / -2.0;
        Self {
            pixels_per_meter,
            x_offset_px,
            y_offset_px,
            width_px,
            height_px,
        }
    }

    #[inline]
    #[must_use]
    pub fn bounds(&self) -> Dimensions {
        let width = (self.width_px / self.pixels_per_meter).into();
        let height = (self.height_px / self.pixels_per_meter).into();
        Dimensions { width, height }
    }

    /// The whole back buffer, in pixels.
    #[must_use]
    pub fn bounds_px(&self) -> Rectangle<f32> {
        Rectangle::new(0.0, 0.0, self.height_px, self.width_px)
    }

    #[must_use]
    pub fn to_pixels(self, length: Length) -> f32 {
        (length * self.pixels_per_meter).get::<ratio>()
    }

    /// Converts a rectangle in world space into the buffer's pixel space.
    #[must_use]
    pub fn to_buffer_rectangle(self, rectangle: &Rectangle<Length>) -> Rectangle<f32> {
        let bottom_px = self.to_pixels(rectangle.bottom());
        let left_px = self.to_pixels(rectangle.left());
        let top_px = self.to_pixels(rectangle.top());
        let right_px = self.to_pixels(rectangle.right());
        let height_px = top_px - bottom_px;
        let width_px = right_px - left_px;
        let scaled = Rectangle::new(bottom_px, left_px, height_px, width_px);
        self.flip_to_buffer(&scaled)
    }

    /// Converts a rectangle already measured in pixels, with y growing upward, into the
    /// buffer's pixel space.
    #[must_use]
    pub fn flip_to_buffer(&self, rectangle: &Rectangle<f32>) -> Rectangle<f32> {
        let x = rectangle.left();
        let y = self.height_px - rectangle.top();
        let moved = rectangle.moved_to(x, y);
        moved.shifted(self.x_offset_px, -self.y_offset_px)
    }
}
