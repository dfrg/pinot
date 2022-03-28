use super::geometry::{Bounds, Point, Transform};

#[doc(inline)]
pub use pinot::colr::{CompositeMode, Extend as ExtendMode};

#[doc(inline)]
pub use pinot::cpal::Color;

pub type PathIndex = usize;

#[derive(Clone, Debug)]
pub enum Command {
    PushTransform(Transform),
    PopTransform,
    PushClip(PathIndex),
    PopClip,
    SimpleFill(PathIndex, Brush, Option<Transform>),
    Fill(Brush, Option<Transform>),
    BeginBlend(Bounds, CompositeMode),
    EndBlend,
    PushLayer(Bounds),
    PopLayer,
}

#[derive(Copy, Clone, Debug)]
pub struct ColorStop {
    pub offset: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub stops: Vec<ColorStop>,
    pub extend: ExtendMode,
}

#[derive(Clone, Debug)]
pub struct RadialGradient {
    pub center0: Point,
    pub radius0: f32,
    pub center1: Point,
    pub radius1: f32,
    pub stops: Vec<ColorStop>,
    pub extend: ExtendMode,
}

#[derive(Clone, Debug)]
pub enum Brush {
    Solid(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}
