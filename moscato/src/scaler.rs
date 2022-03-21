use super::geometry::{Bounds, Point, Transform};
use super::glyph::{Glyph, PathBuilder};
use super::{cache, data, truetype};
use pinot::colr::Paint;
use pinot::types::{Fixed, Tag};
use pinot::{FontRef, TableProvider};
use std::collections::{HashMap, HashSet};

pub struct Context {
    cache: super::cache::Cache,
    state: State,
    coords: Vec<i16>,
}

struct State {
    truetype: truetype::scale::Scaler,
    colr_blacklist: HashSet<u16>,
    colr_map: HashMap<u16, usize>,
}

impl State {
    fn new() -> Self {
        Self {
            truetype: truetype::scale::Scaler::new(8),
            colr_blacklist: Default::default(),
            colr_map: Default::default(),
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            cache: cache::Cache::new(8),
            state: State::new(),
            coords: vec![],
        }
    }

    pub fn new_scaler<'a>(&'a mut self, provider: &impl TableProvider<'a>) -> Builder<'a> {
        let data = data::Data::from_table_provider(provider).unwrap_or_default();
        self.coords.clear();
        Builder {
            ctx: self,
            id: None,
            font: data,
            size: 0.,
            hint: false,
        }
    }

    pub fn new_scaler_with_id<'a>(&'a mut self, font: &FontRef<'a>, font_id: u64) -> Builder<'a> {
        let data =
            data::Data::from_cached(font, &self.cache.get(font, font_id)).unwrap_or_default();
        self.coords.clear();
        Builder {
            ctx: self,
            id: Some(font_id),
            font: data,
            size: 0.,
            hint: false,
        }
    }
}

pub struct Builder<'a> {
    ctx: &'a mut Context,
    id: Option<u64>,
    font: data::Data<'a>,
    size: f32,
    hint: bool,
}

impl<'a> Builder<'a> {
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn hint(mut self, yes: bool) -> Self {
        self.hint = yes;
        self
    }

    pub fn variations<I>(self, settings: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<(Tag, f32)>,
    {
        if let Some(var) = self.font.var {
            self.ctx.coords.resize(var.fvar.num_axes() as usize, 0);
            for setting in settings {
                let (tag, value) = setting.into();
                for axis in var.fvar.axes() {
                    if axis.tag == tag {
                        let mut coord = axis.normalize(Fixed::from_f32(value));
                        if let Some(avar) = var.avar {
                            coord = avar
                                .segment_map(axis.index)
                                .map(|map| map.apply(coord))
                                .unwrap_or(coord);
                        }
                        if let Some(c) = self.ctx.coords.get_mut(axis.index as usize) {
                            *c = coord.to_f2dot14();
                        }
                    }
                }
            }
        }
        self
    }

    pub fn build(self) -> Scaler<'a> {
        let upem = self.font.info.upem;
        let scale = if self.size != 0. && upem != 0 {
            self.size / upem as f32
        } else {
            1.
        };
        Scaler {
            state: &mut self.ctx.state,
            font: self.font,
            id: self.id,
            coords: &self.ctx.coords,
            size: self.size,
            scale,
            hint: self.hint,
            truetype: None,
        }
    }
}

pub struct Scaler<'a> {
    state: &'a mut State,
    font: data::Data<'a>,
    id: Option<u64>,
    coords: &'a [i16],
    size: f32,
    scale: f32,
    hint: bool,
    truetype: Option<truetype::scale::ScalerState<'a>>,
}

impl<'a> Scaler<'a> {
    pub fn glyph(&mut self, gid: u16) -> Option<Glyph> {
        let mut glyph = Glyph::default();
        if load_glyph(self, gid, &mut glyph) {
            Some(glyph)
        } else {
            None
        }
    }

    pub fn color_glyph(&mut self, palette_index: u16, gid: u16) -> Option<Glyph> {
        if let Some(paint) = self
            .font
            .color
            .and_then(|color| color.colr.find_base_paint(gid))
        {
            let mut glyph = Glyph::default();
            if load_color(self, palette_index, &paint, &mut glyph, 0) {
                Some(glyph)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn load_color(
    scaler: &mut Scaler,
    palette: u16,
    paint: &Paint,
    glyph: &mut Glyph,
    depth: usize,
) -> bool {
    if depth > 32 {
        return false;
    }
    use super::color::*;
    use pinot::cpal::Color;
    let paint = flatten_transform(paint);
    let color_data = scaler.font.color.unwrap();
    let colr = color_data.colr;
    let cpal = color_data.cpal;
    let pal = cpal.get(palette).or_else(|| cpal.get(0)).unwrap();
    const DEFAULT_COLOR: Color = Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    match paint {
        Paint::Layers { start, end } => {
            for i in start..end {
                if let Some(layer) = colr.paint_layer(i) {
                    load_color(scaler, palette, &layer, glyph, depth + 1);
                }
            }
        }
        Paint::Glyph { paint, id } => {
            let path_index = glyph.num_paths();
            if load_glyph(scaler, id, glyph) {
                glyph.push_command(Command::PushClip(path_index));
                if let Some(paint) = paint.get() {
                    load_color(scaler, palette, &paint, glyph, depth + 1);
                }
                glyph.push_command(Command::PopClip);
            }
        }
        Paint::ColorGlyph { id } => {
            if let Some(paint) = colr.find_base_paint(id) {
                load_color(scaler, palette, &paint, glyph, depth + 1);
            }
        }
        Paint::Transform {
            paint,
            xx,
            yx,
            xy,
            yy,
            dx,
            dy,
            ..
        } => {
            if let Some(paint) = paint.get() {
                glyph.push_command(Command::PushTransform(Transform::new(&[
                    xx, yx, xy, yy, dx, dy,
                ])));
                load_color(scaler, palette, &paint, glyph, depth + 1);
                glyph.push_command(Command::PopTransform);
            }
        }
        Paint::Composite {
            source,
            mode,
            backdrop,
        } => {
            if let (Some(source), Some(backdrop)) = (source.get(), backdrop.get()) {
                glyph.push_command(Command::PushLayer(Bounds::default()));
                load_color(scaler, palette, &backdrop, glyph, depth + 1);
                glyph.push_command(Command::BeginBlend(Bounds::default(), mode));
                load_color(scaler, palette, &source, glyph, depth + 1);
                glyph.push_command(Command::EndBlend);
                glyph.push_command(Command::PopLayer);
            }
        }
        Paint::Solid {
            palette_index,
            alpha,
            ..
        } => {
            let mut color = pal
                .colors
                .get(palette_index as usize)
                .unwrap_or(DEFAULT_COLOR);
            if alpha != 1.0 {
                color.a = (color.a as f32 * alpha) as u8;
            }
            glyph.push_command(Command::Fill(Brush::Solid(color), None));
        }
        Paint::LinearGradient {
            color_line,
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            ..
        } => {
            let stops = convert_stops(&color_line, &pal);
            glyph.push_command(Command::Fill(
                Brush::LinearGradient(LinearGradient {
                    start: Point::new(x0, y0),
                    end: Point::new(x1, y1),
                    stops,
                    extend: color_line.extend(),
                }),
                None,
            ))
        }
        Paint::RadialGradient {
            color_line,
            x0,
            y0,
            radius0,
            x1,
            y1,
            radius1,
            ..
        } => {
            let stops = convert_stops(&color_line, &pal);
            glyph.push_command(Command::Fill(
                Brush::RadialGradient(RadialGradient {
                    center0: Point::new(x0, y0),
                    radius0,
                    center1: Point::new(x1, y1),
                    radius1,
                    stops,
                    extend: color_line.extend(),
                }),
                None,
            ))
        }
        _ => return true,
    }
    true
}

fn convert_stops(
    color_line: &pinot::colr::ColorLine,
    pal: &pinot::cpal::Palette,
) -> Vec<super::color::ColorStop> {
    use pinot::cpal::Color;
    const DEFAULT_COLOR: Color = Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    };
    color_line
        .stops()
        .map(|stop| {
            let mut color = pal
                .colors
                .get(stop.palette_index as usize)
                .unwrap_or(DEFAULT_COLOR);
            if stop.alpha != 1.0 {
                color.a = (color.a as f32 * stop.alpha) as u8;
            }
            super::color::ColorStop {
                offset: stop.offset,
                color,
            }
        })
        .collect()
}

fn flatten_transform<'a>(paint: &Paint<'a>) -> Paint<'a> {
    match *paint {
        Paint::Translate { paint, dx, dy, .. } => Paint::Transform {
            paint,
            xx: 1.,
            yx: 0.,
            xy: 0.,
            yy: 1.,
            dx,
            dy,
            var_index: None,
        },
        Paint::Scale {
            paint,
            scale_x,
            scale_y,
            ..
        } => Paint::Transform {
            paint,
            xx: scale_x,
            yx: 0.,
            xy: 0.,
            yy: scale_y,
            dx: 0.,
            dy: 0.,
            var_index: None,
        },
        Paint::ScaleAroundCenter {
            paint,
            scale_x,
            scale_y,
            center_x,
            center_y,
            ..
        } => {
            let x = Transform::scale(scale_x, scale_y).around_center(center_x, center_y);
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        Paint::ScaleUniform { paint, scale, .. } => Paint::Transform {
            paint,
            xx: scale,
            yx: 0.,
            xy: 0.,
            yy: scale,
            dx: 0.,
            dy: 0.,
            var_index: None,
        },
        Paint::ScaleUniformAroundCenter {
            paint,
            scale,
            center_x,
            center_y,
            ..
        } => {
            let x = Transform::scale(scale, scale).around_center(center_x, center_y);
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        Paint::Rotate { paint, angle, .. } => {
            let x = Transform::rotate((angle * 180.0).to_radians());
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        Paint::RotateAroundCenter {
            paint,
            angle,
            center_x,
            center_y,
            ..
        } => {
            let x =
                Transform::rotate((angle * 180.0).to_radians()).around_center(center_x, center_y);
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        Paint::Skew {
            paint,
            x_skew,
            y_skew,
            ..
        } => {
            let x = Transform::skew((x_skew * 180.0).to_radians(), (y_skew * 180.0).to_radians());
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        Paint::SkewAroundCenter {
            paint,
            x_skew,
            y_skew,
            center_x,
            center_y,
            ..
        } => {
            let x = Transform::skew((x_skew * 180.0).to_radians(), (y_skew * 180.0).to_radians())
                .around_center(center_x, center_y);
            Paint::Transform {
                paint,
                xx: x.xx,
                yx: x.yx,
                xy: x.xy,
                yy: x.yy,
                dx: x.dx,
                dy: x.dy,
                var_index: None,
            }
        }
        _ => *paint,
    }
}

fn load_glyph(scaler: &mut Scaler, gid: u16, glyph: &mut Glyph) -> bool {
    match scaler.font.simple {
        data::SimpleData::TrueType(data) => {
            if scaler.truetype.is_none() {
                scaler.truetype = Some(truetype::scale::ScalerState::new(
                    data,
                    scaler.id,
                    &scaler.coords,
                    scaler.size,
                    scaler.hint,
                ));
            }
            let state = scaler.truetype.as_mut().unwrap();
            if load_truetype(
                &mut scaler.state.truetype,
                state,
                gid,
                glyph,
                scaler.size != 0.,
            ) {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn load_truetype(
    scaler: &mut truetype::scale::Scaler,
    state: &mut truetype::scale::ScalerState,
    gid: u16,
    outline: &mut Glyph,
    scaled: bool,
) -> bool {
    if scaler.scale(state, gid).is_some() {
        let mut builder = PathBuilder::new(outline);
        fill_outline(
            &mut builder,
            &scaler.scaled,
            &scaler.contours,
            &scaler.tags,
            scaled,
        );
        builder.finish();
        true
    } else {
        false
    }
}

fn fill_outline(
    outline: &mut PathBuilder,
    points: &[truetype::Point],
    contours: &[u16],
    tags: &[u8],
    scaled: bool,
) -> Option<()> {
    use truetype::Point as PointI;
    #[inline(always)]
    fn conv(p: truetype::Point, s: f32) -> Point {
        Point::new(p.x as f32 * s, p.y as f32 * s)
    }
    const TAG_MASK: u8 = 0x3;
    const CONIC: u8 = 0x0;
    const ON: u8 = 0x1;
    const CUBIC: u8 = 0x2;
    let s = if scaled { 1. / 64. } else { 1. };
    for c in 0..contours.len() {
        let mut cur = if c > 0 {
            contours[c - 1] as usize + 1
        } else {
            0
        };
        let mut last = contours[c] as usize;
        if last < cur || last >= points.len() {
            return None;
        }
        let mut v_start = points[cur];
        let v_last = points[last];
        let mut tag = tags[cur] & TAG_MASK;
        if tag == CUBIC {
            return None;
        }
        let mut step_point = true;
        if tag == CONIC {
            if tags[last] & TAG_MASK == ON {
                v_start = v_last;
                last -= 1;
            } else {
                v_start.x = (v_start.x + v_last.x) / 2;
                v_start.y = (v_start.y + v_last.y) / 2;
            }
            step_point = false;
        }
        outline.move_to(conv(v_start, s));
        // let mut do_close = true;
        while cur < last {
            if step_point {
                cur += 1;
            }
            step_point = true;
            tag = tags[cur] & TAG_MASK;
            match tag {
                ON => {
                    outline.line_to(conv(points[cur], s));
                    continue;
                }
                CONIC => {
                    let mut do_close_conic = true;
                    let mut v_control = points[cur];
                    while cur < last {
                        cur += 1;
                        let point = points[cur];
                        tag = tags[cur] & TAG_MASK;
                        if tag == ON {
                            outline.quad_to(conv(v_control, s), conv(point, s));
                            do_close_conic = false;
                            break;
                        }
                        if tag != CONIC {
                            return None;
                        }
                        let v_middle =
                            PointI::new((v_control.x + point.x) / 2, (v_control.y + point.y) / 2);
                        outline.quad_to(conv(v_control, s), conv(v_middle, s));
                        v_control = point;
                    }
                    if do_close_conic {
                        outline.quad_to(conv(v_control, s), conv(v_start, s));
                        //                        do_close = false;
                        break;
                    }
                    continue;
                }
                _ => {
                    if cur + 1 > last || (tags[cur + 1] & TAG_MASK != CUBIC) {
                        return None;
                    }
                    let v1 = conv(points[cur], s);
                    let v2 = conv(points[cur + 1], s);
                    cur += 2;
                    if cur <= last {
                        outline.curve_to(v1, v2, conv(points[cur], s));
                        continue;
                    }
                    outline.curve_to(v1, v2, conv(v_start, s));
                    // do_close = false;
                    break;
                }
            }
        }
        if true {
            outline.maybe_close();
        }
    }
    Some(())
}
