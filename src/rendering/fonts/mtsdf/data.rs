use std::collections::HashMap;
use serde::Deserialize;

use crate::math::{Rectangle, Vector2};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MTSDF {
    pub atlas: Atlas,
    pub metrics: Metrics,

    #[serde(with = "items")]
    pub glyphs: HashMap<u32, Glyph>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Atlas {
    pub distance_range: f32,
    pub size: f32,
    pub width: f32,
    pub height: f32,
    pub y_origin: YOrigin,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub em_size: f64,
    pub line_height: f32,
    pub ascender: f32,
    pub descender: f32,
    pub underline_y: f32,
    pub underline_thickness: f32,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Glyph {
    pub unicode: u32,
    pub advance: f64,

    #[serde(default)]
    pub plane_bounds: GlyphBounds,

    #[serde(default)]
    pub atlas_bounds: GlyphBounds,
}

impl Into<crate::rendering::fonts::glyph::Glyph> for Glyph {
    fn into(self) -> crate::rendering::fonts::glyph::Glyph {
        crate::rendering::fonts::glyph::Glyph {
            source_area: self.atlas_bounds.into(),
            bearing_x: self.plane_bounds.left,
            bearing_y: self.plane_bounds.top,
            width: self.plane_bounds.right - self.plane_bounds.left,
            height: self.plane_bounds.bottom - self.plane_bounds.top,
            advance_x: self.advance,
            advance_y: 0.0,
        }
    }
}

#[derive(Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub struct GlyphBounds {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Into<Rectangle<f64>> for GlyphBounds {
    fn into(self) -> Rectangle<f64> {
        Rectangle::points(
            Vector2::new(self.left, self.top),
            Vector2::new(self.right, self.bottom),
        )
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum YOrigin {
    Bottom,
    Top,
}

mod items {
    use super::Glyph;

    use std::collections::HashMap;

    //use serde::ser::Serializer;
    use serde::de::{Deserialize, Deserializer};

    /*
    pub fn serialize<S>(map: &HashMap<u32, Glyph>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.collect_seq(map.values())
    }
    */

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<u32, Glyph>, D::Error>
        where D: Deserializer<'de>
    {
        let mut map = HashMap::new();
        for item in Vec::<Glyph>::deserialize(deserializer)? {
            map.insert(item.unicode, item);
        }
        Ok(map)
    }
}
