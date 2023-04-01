// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{hash::Hash, ops::Range};

pub use fontdb::{Family, Stretch, Style, Weight};
use rangemap::RangeMap;

/// Text color
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Color(pub u32);

impl Color {
    /// Create new color with red, green, and blue components
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 0xFF)
    }

    /// Create new color with red, green, blue, and alpha components
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    /// Get the red component
    #[inline]
    pub fn r(&self) -> u8 {
        ((self.0 & 0x00_FF_00_00) >> 16) as u8
    }

    /// Get the green component
    #[inline]
    pub fn g(&self) -> u8 {
        ((self.0 & 0x00_00_FF_00) >> 8) as u8
    }

    /// Get the blue component
    #[inline]
    pub fn b(&self) -> u8 {
        (self.0 & 0x00_00_00_FF) as u8
    }

    /// Get the alpha component
    #[inline]
    pub fn a(&self) -> u8 {
        ((self.0 & 0xFF_00_00_00) >> 24) as u8
    }
}

/// An owned version of [`Family`]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FamilyOwned {
    Name(String),
    Serif,
    SansSerif,
    Cursive,
    Fantasy,
    Monospace,
}

impl FamilyOwned {
    pub fn new(family: Family) -> Self {
        match family {
            Family::Name(name) => FamilyOwned::Name(name.to_string()),
            Family::Serif => FamilyOwned::Serif,
            Family::SansSerif => FamilyOwned::SansSerif,
            Family::Cursive => FamilyOwned::Cursive,
            Family::Fantasy => FamilyOwned::Fantasy,
            Family::Monospace => FamilyOwned::Monospace,
        }
    }

    pub fn as_family(&self) -> Family {
        match self {
            FamilyOwned::Name(name) => Family::Name(name),
            FamilyOwned::Serif => Family::Serif,
            FamilyOwned::SansSerif => Family::SansSerif,
            FamilyOwned::Cursive => Family::Cursive,
            FamilyOwned::Fantasy => Family::Fantasy,
            FamilyOwned::Monospace => Family::Monospace,
        }
    }
}

/// Text attributes
#[derive(Clone, Copy, Debug)]
pub struct Attrs<'a> {
    //TODO: should this be an option?
    pub color_opt: Option<Color>,
    pub family: Family<'a>,
    pub stretch: Stretch,
    pub style: Style,
    pub weight: Weight,
    pub scaling: f32,
    pub metadata: usize,
}

impl PartialEq for Attrs<'_> {
    fn eq(&self, other: &Self) -> bool {
        // compile error if new fields are added
        let Self {
            color_opt,
            family,
            stretch,
            style,
            weight,
            scaling,
            metadata,
        } = self;

        *color_opt == other.color_opt
            && *family == other.family
            && *stretch == other.stretch
            && *style == other.style
            && *weight == other.weight
            && f32::total_cmp(scaling, &other.scaling).is_eq()
            && *metadata == other.metadata
    }
}
impl Eq for Attrs<'_> {}
impl Hash for Attrs<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // compile error if new fields are added
        let Self {
            color_opt,
            family,
            stretch,
            style,
            weight,
            scaling,
            metadata,
        } = self;

        color_opt.hash(state);
        family.hash(state);
        stretch.hash(state);
        style.hash(state);
        weight.hash(state);
        scaling.to_bits().hash(state);
        metadata.hash(state);
    }
}

impl<'a> Attrs<'a> {
    /// Create a new set of attributes with sane defaults
    ///
    /// This defaults to a regular Sans-Serif font.
    pub fn new() -> Self {
        Self {
            color_opt: None,
            family: Family::SansSerif,
            stretch: Stretch::Normal,
            style: Style::Normal,
            weight: Weight::NORMAL,
            scaling: 1.0,
            metadata: 0,
        }
    }

    /// Set [Color]
    pub fn color(mut self, color: Color) -> Self {
        self.color_opt = Some(color);
        self
    }

    /// Set [Family]
    pub fn family(mut self, family: Family<'a>) -> Self {
        self.family = family;
        self
    }

    /// Set [Stretch]
    pub fn stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    /// Set [Style]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set [Weight]
    pub fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    /// Set scaling
    pub fn scaling(mut self, scaling: f32) -> Self {
        self.scaling = scaling;
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: usize) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if font matches
    pub fn matches(&self, face: &fontdb::FaceInfo) -> bool {
        //TODO: smarter way of including emoji
        face.post_script_name.contains("Emoji")
            || (face.style == self.style
                && face.weight == self.weight
                && face.stretch == self.stretch)
    }

    /// Check if this set of attributes can be shaped with another
    pub fn compatible(&self, other: &Self) -> bool {
        self.family == other.family
            && self.stretch == other.stretch
            && self.style == other.style
            && self.weight == other.weight
            && self.scaling.total_cmp(&other.scaling).is_eq()
    }
}

/// An owned version of [`Attrs`]
#[derive(Clone, Debug)]
pub struct AttrsOwned {
    //TODO: should this be an option?
    pub color_opt: Option<Color>,
    pub family_owned: FamilyOwned,
    pub stretch: Stretch,
    pub style: Style,
    pub weight: Weight,
    pub scaling: f32,
    pub metadata: usize,
}

impl AttrsOwned {
    pub fn new(attrs: Attrs) -> Self {
        Self {
            color_opt: attrs.color_opt,
            family_owned: FamilyOwned::new(attrs.family),
            stretch: attrs.stretch,
            style: attrs.style,
            weight: attrs.weight,
            scaling: attrs.scaling,
            metadata: attrs.metadata,
        }
    }

    pub fn as_attrs(&self) -> Attrs {
        Attrs {
            color_opt: self.color_opt,
            family: self.family_owned.as_family(),
            stretch: self.stretch,
            style: self.style,
            weight: self.weight,
            scaling: self.scaling,
            metadata: self.metadata,
        }
    }
}

impl PartialEq for AttrsOwned {
    fn eq(&self, other: &Self) -> bool {
        // compile error if new fields are added
        let Self {
            color_opt,
            family_owned,
            stretch,
            style,
            weight,
            scaling,
            metadata,
        } = self;

        *color_opt == other.color_opt
            && *family_owned == other.family_owned
            && *stretch == other.stretch
            && *style == other.style
            && *weight == other.weight
            && f32::total_cmp(scaling, &other.scaling).is_eq()
            && *metadata == other.metadata
    }
}
impl Eq for AttrsOwned {}
impl Hash for AttrsOwned {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // compile error if new fields are added
        let Self {
            color_opt,
            family_owned,
            stretch,
            style,
            weight,
            scaling,
            metadata,
        } = self;

        color_opt.hash(state);
        family_owned.hash(state);
        stretch.hash(state);
        style.hash(state);
        weight.hash(state);
        scaling.to_bits().hash(state);
        metadata.hash(state);
    }
}

/// List of text attributes to apply to a line
//TODO: have this clean up the spans when changes are made
#[derive(Eq, PartialEq)]
pub struct AttrsList {
    defaults: AttrsOwned,
    spans: RangeMap<usize, AttrsOwned>,
}

impl AttrsList {
    /// Create a new attributes list with a set of default [Attrs]
    pub fn new(defaults: Attrs) -> Self {
        Self {
            defaults: AttrsOwned::new(defaults),
            spans: RangeMap::new(),
        }
    }

    /// Get the default [Attrs]
    pub fn defaults(&self) -> Attrs {
        self.defaults.as_attrs()
    }

    /// Get the current attribute spans
    pub fn spans(&self) -> Vec<(&Range<usize>, &AttrsOwned)> {
        self.spans.iter().collect()
    }

    /// Clear the current attribute spans
    pub fn clear_spans(&mut self) {
        self.spans.clear();
    }

    /// Add an attribute span, removes any previous matching parts of spans
    pub fn add_span(&mut self, range: Range<usize>, attrs: Attrs) {
        //do not support 1..1 even if by accident.
        if range.start == range.end {
            return;
        }

        self.spans.insert(range, AttrsOwned::new(attrs));
    }

    /// Get the attribute span for an index
    ///
    /// This returns a span that contains the index
    pub fn get_span(&self, index: usize) -> Attrs {
        self.spans
            .get(&index)
            .map(|v| v.as_attrs())
            .unwrap_or(self.defaults.as_attrs())
    }

    /// Split attributes list at an offset
    pub fn split_off(&mut self, index: usize) -> Self {
        let mut new = Self::new(self.defaults.as_attrs());
        let mut removes = Vec::new();

        //get the keys we need to remove or fix.
        for span in self.spans.iter() {
            if span.0.end <= index {
                continue;
            } else if span.0.start >= index {
                removes.push((span.0.clone(), false));
            } else {
                removes.push((span.0.clone(), true));
            }
        }

        for (key, resize) in removes {
            let (range, attrs) = self
                .spans
                .get_key_value(&key.start)
                .map(|v| (v.0.clone(), v.1.clone()))
                .expect("attrs span not found");
            self.spans.remove(key);

            if resize {
                new.spans.insert(0..range.end - index, attrs.clone());
                self.spans.insert(range.start..index, attrs);
            } else {
                new.spans
                    .insert(range.start - index..range.end - index, attrs);
            }
        }
        new
    }
}
