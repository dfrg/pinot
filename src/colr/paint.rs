use super::{f2dot14_to_f32, fixed_to_f32};
use crate::parse_prelude::*;
use core::fmt;

/// Compositing modes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CompositeMode {
    Clear,
    Src,
    Dest,
    SrcOver,
    DestOver,
    SrcIn,
    DestIn,
    SrcOut,
    DestOut,
    SrcAtop,
    DestAtop,
    Xor,
    Plus,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    HslHue,
    HslSaturation,
    HslColor,
    HslLuminosity,
}

impl CompositeMode {
    fn from_raw(value: u8) -> Option<Self> {
        use CompositeMode::*;
        Some(match value {
            0 => Clear,
            1 => Src,
            2 => Dest,
            3 => SrcOver,
            4 => DestOver,
            5 => SrcIn,
            6 => DestIn,
            7 => SrcOut,
            8 => DestOut,
            9 => SrcAtop,
            10 => DestAtop,
            11 => Xor,
            12 => Plus,
            13 => Screen,
            14 => Overlay,
            15 => Darken,
            16 => Lighten,
            17 => ColorDodge,
            18 => ColorBurn,
            19 => HardLight,
            20 => SoftLight,
            21 => Difference,
            22 => Exclusion,
            23 => Multiply,
            24 => HslHue,
            25 => HslSaturation,
            26 => HslColor,
            27 => HslLuminosity,
            _ => return None,
        })
    }
}

/// Extension mode for gradients.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Extend {
    Pad,
    Repeat,
    Reflect,
}

impl Extend {
    fn from_raw(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::Pad,
            1 => Self::Repeat,
            2 => Self::Reflect,
            _ => return None,
        })
    }
}

/// Clip box for a color outline.
#[derive(Copy, Clone, Default, Debug)]
pub struct ClipBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub var_index: Option<u32>,
}

/// Single color stop for a gradient.
#[derive(Copy, Clone, Default, Debug)]
pub struct ColorStop {
    pub offset: f32,
    pub palette_index: u16,
    pub alpha: f32,
    pub var_index: Option<u32>,
}

/// Collection of color stops that define a gradient.
#[derive(Copy, Clone)]
pub struct ColorLine<'a> {
    data: Buffer<'a>,
    extend: Extend,
    is_var: bool,
    len: u16,
}

impl<'a> ColorLine<'a> {
    /// Returns the extension mode of the color line.
    pub fn extend(&self) -> Extend {
        self.extend
    }

    /// Returns the number of color stops.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns true if the color line doesn't contain any stops.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the color stop at the specified index.
    pub fn get(&self, index: u16) -> Option<ColorStop> {
        if index >= self.len {
            return None;
        }
        let element_size = if self.is_var { 10 } else { 6 };
        let base = index as usize * element_size + 3;
        let offset = f2dot14_to_f32(self.data.read_i16(base)?);
        let palette_index = self.data.read_u16(base + 2)?;
        let alpha = f2dot14_to_f32(self.data.read_i16(base + 4)?);
        let var_index = if self.is_var {
            Some(self.data.read_u32(base + 6)?)
        } else {
            None
        };
        Some(ColorStop {
            offset,
            palette_index,
            alpha,
            var_index,
        })
    }

    /// Returns an iterator over the color stops.
    pub fn stops(&self) -> impl Iterator<Item = ColorStop> + 'a + Clone {
        let copy = *self;
        (0..self.len).filter_map(move |i| copy.get(i))
    }
}

impl fmt::Debug for ColorLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.stops()).finish()
    }
}

/// Node in a paint graph for a color outline.
#[derive(Copy, Clone, Debug)]
pub enum Paint<'a> {
    Layers {
        start: u32,
        end: u32,
    },
    Solid {
        palette_index: u16,
        alpha: f32,
        var_index: Option<u32>,
    },
    LinearGradient {
        color_line: ColorLine<'a>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        var_index: Option<u32>,
    },
    RadialGradient {
        color_line: ColorLine<'a>,
        x0: f32,
        y0: f32,
        radius0: f32,
        x1: f32,
        y1: f32,
        radius1: f32,
        var_index: Option<u32>,
    },
    SweepGradient {
        color_line: ColorLine<'a>,
        center_x: f32,
        center_y: f32,
        start_angle: f32,
        end_angle: f32,
        var_index: Option<u32>,
    },
    Glyph {
        paint: PaintRef<'a>,
        id: GlyphId,
    },
    ColorGlyph {
        id: GlyphId,
    },
    Transform {
        paint: PaintRef<'a>,
        xx: f32,
        yx: f32,
        xy: f32,
        yy: f32,
        dx: f32,
        dy: f32,
        var_index: Option<u32>,
    },
    Translate {
        paint: PaintRef<'a>,
        dx: f32,
        dy: f32,
        var_index: Option<u32>,
    },
    Scale {
        paint: PaintRef<'a>,
        scale_x: f32,
        scale_y: f32,
        var_index: Option<u32>,
    },
    ScaleAroundCenter {
        paint: PaintRef<'a>,
        scale_x: f32,
        scale_y: f32,
        center_x: f32,
        center_y: f32,
        var_index: Option<u32>,
    },
    ScaleUniform {
        paint: PaintRef<'a>,
        scale: f32,
        var_index: Option<u32>,
    },
    ScaleUniformAroundCenter {
        paint: PaintRef<'a>,
        scale: f32,
        center_x: f32,
        center_y: f32,
        var_index: Option<u32>,
    },
    Rotate {
        paint: PaintRef<'a>,
        angle: f32,
        var_index: Option<u32>,
    },
    RotateAroundCenter {
        paint: PaintRef<'a>,
        angle: f32,
        center_x: f32,
        center_y: f32,
        var_index: Option<u32>,
    },
    Skew {
        paint: PaintRef<'a>,
        x_skew: f32,
        y_skew: f32,
        var_index: Option<u32>,
    },
    SkewAroundCenter {
        paint: PaintRef<'a>,
        x_skew: f32,
        y_skew: f32,
        center_x: f32,
        center_y: f32,
        var_index: Option<u32>,
    },
    Composite {
        source: PaintRef<'a>,
        mode: CompositeMode,
        backdrop: PaintRef<'a>,
    },
}

/// Reference to a paint graph node.
#[derive(Copy, Clone)]
pub struct PaintRef<'a> {
    data: Buffer<'a>,
}

impl<'a> PaintRef<'a> {
    pub(super) fn new(data: Buffer<'a>, offset: u32) -> Option<Self> {
        Some(Self {
            data: Buffer::with_offset(data.data(), offset as usize)?,
        })
    }

    /// Returns the underlying paint.
    pub fn get(&self) -> Option<Paint<'a>> {
        let d = &self.data;
        let mut c = d.cursor_at(0)?;
        let format = c.read_u8()?;
        Some(match format {
            1 => {
                let count = c.read_u8()? as u32;
                let start = c.read_u32()?;
                Paint::Layers {
                    start,
                    end: start + count,
                }
            }
            2 | 3 => {
                let palette_index = c.read_u16()?;
                let alpha = f2dot14_to_f32(c.read_i16()?);
                let var_index = if format == 3 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Solid {
                    palette_index,
                    alpha,
                    var_index,
                }
            }
            4 | 5 => {
                let color_line_offset = c.read_u24()?;
                let x0 = c.read_i16()? as f32;
                let y0 = c.read_i16()? as f32;
                let x1 = c.read_i16()? as f32;
                let y1 = c.read_i16()? as f32;
                let x2 = c.read_i16()? as f32;
                let y2 = c.read_i16()? as f32;
                let var_index = if format == 5 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                let color_line_data =
                    Buffer::with_offset(self.data.data(), color_line_offset as usize)?;
                let extend = Extend::from_raw(color_line_data.read_u8(0)?)?;
                let len = color_line_data.read_u16(1)?;
                let color_line = ColorLine {
                    data: color_line_data,
                    extend,
                    is_var: var_index.is_some(),
                    len,
                };
                Paint::LinearGradient {
                    color_line,
                    x0,
                    y0,
                    x1,
                    y1,
                    x2,
                    y2,
                    var_index,
                }
            }
            6 | 7 => {
                let color_line_offset = c.read_u24()?;
                let x0 = c.read_i16()? as f32;
                let y0 = c.read_i16()? as f32;
                let radius0 = c.read_i16()? as f32;
                let x1 = c.read_i16()? as f32;
                let y1 = c.read_i16()? as f32;
                let radius1 = c.read_i16()? as f32;
                let var_index = if format == 7 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                let color_line_data =
                    Buffer::with_offset(self.data.data(), color_line_offset as usize)?;
                let extend = Extend::from_raw(color_line_data.read_u8(0)?)?;
                let len = color_line_data.read_u16(1)?;
                let color_line = ColorLine {
                    data: color_line_data,
                    extend,
                    is_var: var_index.is_some(),
                    len,
                };
                Paint::RadialGradient {
                    color_line,
                    x0,
                    y0,
                    radius0,
                    x1,
                    y1,
                    radius1,
                    var_index,
                }
            }
            8 | 9 => {
                let color_line_offset = c.read_u24()?;
                let center_x = c.read_i16()? as f32;
                let center_y = c.read_i16()? as f32;
                let start_angle = f2dot14_to_f32(c.read_i16()?);
                let end_angle = f2dot14_to_f32(c.read_i16()?);
                let var_index = if format == 9 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                let color_line_data =
                    Buffer::with_offset(self.data.data(), color_line_offset as usize)?;
                let extend = Extend::from_raw(color_line_data.read_u8(0)?)?;
                let len = color_line_data.read_u16(1)?;
                let color_line = ColorLine {
                    data: color_line_data,
                    extend,
                    is_var: var_index.is_some(),
                    len,
                };
                Paint::SweepGradient {
                    color_line,
                    center_x,
                    center_y,
                    start_angle,
                    end_angle,
                    var_index,
                }
            }
            10 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let id = c.read_u16()?;
                Paint::Glyph { paint, id }
            }
            11 => Paint::ColorGlyph { id: c.read_u16()? },
            12 | 13 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let mat_offset = c.read_u24()?;
                let mut c = Cursor::with_offset(self.data.data(), mat_offset as usize)?;
                let s = 1f32 / 65536.;
                let xx = c.read_i32()? as f32 * s;
                let yx = c.read_i32()? as f32 * s;
                let xy = c.read_i32()? as f32 * s;
                let yy = c.read_i32()? as f32 * s;
                let dx = c.read_i32()? as f32 * s;
                let dy = c.read_i32()? as f32 * s;
                let var_index = if format == 13 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Transform {
                    paint,
                    xx,
                    yx,
                    xy,
                    yy,
                    dx,
                    dy,
                    var_index,
                }
            }
            14 | 15 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let dx = fixed_to_f32(c.read_i32()?);
                let dy = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 15 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Translate {
                    paint,
                    dx,
                    dy,
                    var_index,
                }
            }
            16 | 17 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let scale_x = fixed_to_f32(c.read_i32()?);
                let scale_y = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 17 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Scale {
                    paint,
                    scale_x,
                    scale_y,
                    var_index,
                }
            }
            18 | 19 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let scale_x = fixed_to_f32(c.read_i32()?);
                let scale_y = fixed_to_f32(c.read_i32()?);
                let center_x = fixed_to_f32(c.read_i32()?);
                let center_y = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 19 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::ScaleAroundCenter {
                    paint,
                    scale_x,
                    scale_y,
                    center_x,
                    center_y,
                    var_index,
                }
            }
            20 | 21 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let scale = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 21 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::ScaleUniform {
                    paint,
                    scale,
                    var_index,
                }
            }
            22 | 23 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let scale = fixed_to_f32(c.read_i32()?);
                let center_x = fixed_to_f32(c.read_i32()?);
                let center_y = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 23 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::ScaleUniformAroundCenter {
                    paint,
                    scale,
                    center_x,
                    center_y,
                    var_index,
                }
            }
            24 | 25 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let angle = f2dot14_to_f32(c.read_i16()?);
                let var_index = if format == 25 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Rotate {
                    paint,
                    angle,
                    var_index,
                }
            }
            26 | 27 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let angle = f2dot14_to_f32(c.read_i16()?);
                let center_x = fixed_to_f32(c.read_i32()?);
                let center_y = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 27 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::RotateAroundCenter {
                    paint,
                    angle,
                    center_x,
                    center_y,
                    var_index,
                }
            }
            28 | 29 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let x_skew = f2dot14_to_f32(c.read_i16()?);
                let y_skew = f2dot14_to_f32(c.read_i16()?);
                let var_index = if format == 29 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::Skew {
                    paint,
                    x_skew,
                    y_skew,
                    var_index,
                }
            }
            30 | 31 => {
                let paint = PaintRef::new(self.data, c.read_u24()?)?;
                let x_skew = f2dot14_to_f32(c.read_i16()?);
                let y_skew = f2dot14_to_f32(c.read_i16()?);
                let center_x = fixed_to_f32(c.read_i32()?);
                let center_y = fixed_to_f32(c.read_i32()?);
                let var_index = if format == 31 {
                    Some(c.read_u32()?)
                } else {
                    None
                };
                Paint::SkewAroundCenter {
                    paint,
                    x_skew,
                    y_skew,
                    center_x,
                    center_y,
                    var_index,
                }
            }
            32 => {
                let source = PaintRef::new(self.data, c.read_u24()?)?;
                let mode = CompositeMode::from_raw(c.read_u8()?).unwrap_or(CompositeMode::Clear);
                let backdrop = PaintRef::new(self.data, c.read_u24()?)?;
                Paint::Composite {
                    source,
                    mode,
                    backdrop,
                }
            }
            _ => return None,
        })
    }
}

impl fmt::Debug for PaintRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get() {
            Some(node) => write!(f, "{:?}", node),
            _ => write!(f, "(null)"),
        }
    }
}
