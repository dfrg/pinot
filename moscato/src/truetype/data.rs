use super::{mul, var, Point};
use crate::data::FontInfo;
use pinot::parse::{Buffer, Slice};
use pinot::types::{Fixed, Tag};
use pinot::{hmtx::*, hvar::*, FontRef, TableProvider};

const GLYF: Tag = Tag::new(b"glyf");
const LOCA: Tag = Tag::new(b"loca");
const CVT: Tag = Tag::new(b"cvt ");
const FPGM: Tag = Tag::new(b"fpgm");
const PREP: Tag = Tag::new(b"prep");
const CVAR: Tag = Tag::new(b"CVAR");
const GVAR: Tag = Tag::new(b"GVAR");

/// Byte offsets or ranges for tables within a linear font buffer. Missing
/// tables are represented by zero offsets.
#[derive(Copy, Clone, Default)]
pub struct TableOffsets {
    pub glyf: u32,
    pub loca: u32,
    pub cvt: (u32, u32),
    pub fpgm: (u32, u32),
    pub prep: (u32, u32),
    pub cvar: u32,
    pub gvar: u32,
    pub hmtx: u32,
    pub hvar: u32,
}

impl TableOffsets {
    /// Creates table offsets from the specified font.
    pub fn new(font: &FontRef) -> Option<Self> {
        fn find_range(font: &FontRef, tag: Tag) -> (u32, u32) {
            match font.find_record(tag) {
                Some(record) => (record.offset, record.offset + record.len),
                None => (0, 0),
            }
        }
        Some(Self {
            glyf: font.find_record(GLYF)?.offset,
            loca: font.find_record(LOCA)?.offset,
            cvt: find_range(font, CVT),
            fpgm: find_range(font, FPGM),
            prep: find_range(font, PREP),
            cvar: font.find_record(CVAR).map(|r| r.offset).unwrap_or_default(),
            gvar: font.find_record(GVAR).map(|r| r.offset).unwrap_or_default(),
            hmtx: font.find_record(HMTX)?.offset,
            hvar: font.find_record(HVAR).map(|r| r.offset).unwrap_or_default(),
        })
    }
}

/// Byte slices for the tables that are relevant to TrueType
/// outline processing. Missing tables are represented by
/// empty slices.
#[derive(Copy, Clone, Default)]
pub struct TableData<'a> {
    pub glyf: &'a [u8],
    pub loca: &'a [u8],
    pub cvt: &'a [u8],
    pub fpgm: &'a [u8],
    pub prep: &'a [u8],
    pub cvar: &'a [u8],
    pub gvar: &'a [u8],
    pub hmtx: &'a [u8],
    pub hvar: &'a [u8],
}

impl<'a> TableData<'a> {
    /// Creates table data from the specified font and offsets.
    pub fn from_font(font: &FontRef<'a>, offsets: &TableOffsets) -> Option<Self> {
        fn get<'b>(font: &FontRef<'b>, offset: u32) -> Option<&'b [u8]> {
            if offset == 0 {
                None
            } else {
                font.data.get(offset as usize..)
            }
        }
        fn get_range<'b>(font: &FontRef<'b>, range: (u32, u32)) -> Option<&'b [u8]> {
            if range.0 == 0 {
                None
            } else {
                font.data.get(range.0 as usize..range.1 as usize)
            }
        }
        Some(Self {
            glyf: get(font, offsets.glyf)?,
            loca: get(font, offsets.loca)?,
            cvt: get_range(font, offsets.cvt).unwrap_or_default(),
            fpgm: get_range(font, offsets.fpgm).unwrap_or_default(),
            prep: get_range(font, offsets.prep).unwrap_or_default(),
            cvar: get(font, offsets.cvar).unwrap_or_default(),
            gvar: get(font, offsets.gvar).unwrap_or_default(),
            hmtx: get(font, offsets.hmtx)?,
            hvar: get(font, offsets.hvar).unwrap_or_default(),
        })
    }

    /// Creates table data from the specified table provider.
    pub fn from_table_provider(provider: &impl TableProvider<'a>) -> Option<Self> {
        Some(Self {
            glyf: provider.table_data(GLYF)?,
            loca: provider.table_data(LOCA)?,
            cvt: provider.table_data(CVT).unwrap_or_default(),
            fpgm: provider.table_data(FPGM).unwrap_or_default(),
            prep: provider.table_data(PREP).unwrap_or_default(),
            cvar: provider.table_data(CVAR).unwrap_or_default(),
            gvar: provider.table_data(GVAR).unwrap_or_default(),
            hmtx: provider.table_data(HMTX).unwrap_or_default(),
            hvar: provider.table_data(HVAR).unwrap_or_default(),
        })
    }
}

/// Limits for hinting storage areas.
#[derive(Copy, Clone, Default)]
pub struct HinterLimits {
    pub max_storage: u16,
    pub max_stack: u16,
    pub max_fdefs: u16,
    pub max_idefs: u16,
    pub max_twilight: u16,
}

impl HinterLimits {
    /// Creates a new set of hinter limits from the specified table provider.
    pub fn new<'a>(provider: &impl TableProvider<'a>) -> Self {
        if let Some(maxp) = provider.maxp() {
            Self {
                max_storage: maxp.max_storage(),
                max_stack: maxp.max_stack_depth(),
                max_fdefs: maxp.max_function_defs(),
                max_idefs: maxp.max_instruction_defs(),
                max_twilight: maxp.max_twilight_points(),
            }
        } else {
            Self::default()
        }
    }
}

/// TrueType metadata for a font cache entry.
#[derive(Copy, Clone, Default)]
pub struct Cached {
    pub offsets: TableOffsets,
    pub limits: HinterLimits,
}

impl Cached {
    /// Creates new cached metadata from the specified font.
    pub fn new(font: &FontRef) -> Option<Self> {
        Some(Self {
            offsets: TableOffsets::new(font)?,
            limits: HinterLimits::new(font),
        })
    }
}

/// Expanded TrueType data referencing font data.
#[derive(Copy, Clone, Default)]
pub struct Data<'a> {
    pub tables: TableData<'a>,
    pub limits: HinterLimits,
    pub info: FontInfo,
}

impl<'a> Data<'a> {
    /// Creates data from a font and cached metadata.
    pub fn from_cached(font: &FontRef<'a>, cached: &Cached, info: FontInfo) -> Option<Self> {
        Some(Self {
            tables: TableData::from_font(font, &cached.offsets)?,
            limits: cached.limits,
            info,
        })
    }

    /// Creates data from the specified table provider.
    pub fn from_table_provider(provider: &impl TableProvider<'a>, info: FontInfo) -> Option<Self> {
        Some(Self {
            tables: TableData::from_table_provider(provider)?,
            limits: HinterLimits::new(provider),
            info,
        })
    }

    /// Returns the glyph outline data for the specified glyph identifier.
    pub fn get_glyph(&self, gid: u16) -> Option<&'a [u8]> {
        let range = {
            let b = Buffer::new(self.tables.loca);
            let (start, end) = if self.info.loca_fmt == 0 {
                let offset = gid as usize * 2;
                let start = b.read::<u16>(offset)? as usize * 2;
                let end = b.read::<u16>(offset + 2)? as usize * 2;
                (start, end)
            } else if self.info.loca_fmt == 1 {
                let offset = gid as usize * 4;
                let start = b.read::<u32>(offset)? as usize;
                let end = b.read::<u32>(offset + 4)? as usize;
                (start, end)
            } else {
                return None;
            };
            if end < start {
                return None;
            }
            start..end
        };
        self.tables.glyf.get(range)
    }

    /// Returns the advance width for the specified glyph identifier and
    /// variation coordinates.
    pub fn advance_width(&self, gid: u16, coords: &[i16]) -> i32 {
        let hmtx = Hmtx::new(
            self.tables.hmtx,
            self.info.glyph_count,
            self.info.hmetric_count,
        );
        let hmetrics = hmtx.hmetrics();
        if hmetrics.is_empty() {
            return 0;
        }
        let mut advance = hmetrics
            .get(gid as usize)
            .or_else(|| hmetrics.get(hmetrics.len() - 1))
            .unwrap()
            .advance_width as i32;
        if !coords.is_empty() && !self.tables.hvar.is_empty() {
            use pinot::var::item::Index;
            let hvar = Hvar::new(self.tables.hvar);
            if let Some(ivs) = hvar.ivs() {
                let mut index = Index::new(0, gid);
                if let Some(dsim) = hvar.advance_mapping() {
                    index = dsim.get(gid as u32).unwrap_or(index);
                }
                advance = (Fixed::from_i32(advance) + ivs.delta(index, coords)).to_i32();
            }
        }
        advance
    }

    /// Returns the left side-bearing for the specified glyph identifier and
    /// variation coordinates.
    pub fn lsb(&self, gid: u16, coords: &[i16]) -> i32 {
        let hmtx = Hmtx::new(
            self.tables.hmtx,
            self.info.glyph_count,
            self.info.hmetric_count,
        );
        let hmetrics = hmtx.hmetrics();
        let mut lsb = hmetrics
            .get(gid as usize)
            .map(|m| m.lsb)
            .or_else(|| hmtx.lsbs().get(gid as usize - hmetrics.len()))
            .unwrap_or(0) as i32;
        if !coords.is_empty() && !self.tables.hvar.is_empty() {
            let hvar = Hvar::new(self.tables.hvar);
            if let Some(ivs) = hvar.ivs() {
                match hvar.lsb_mapping().and_then(|m| m.get(gid as u32)) {
                    Some(index) => {
                        lsb = (Fixed::from_i32(lsb) + ivs.delta(index, coords)).to_i32();
                    }
                    None => {}
                }
            }
        }
        lsb
    }

    /// Loads, scales and applies deltas to entries in the control value table.
    pub fn cvt(&self, scale: Option<i32>, coords: &[i16], values: &mut Vec<i32>) -> Option<()> {
        let cvt = self.tables.cvt;
        if cvt.is_empty() {
            return Some(());
        }
        let entries = Slice::<i16>::new(cvt);
        let len = entries.len();
        if values.len() < len {
            values.resize(len, 0);
        }
        for (a, b) in entries.iter().zip(values.iter_mut()) {
            *b = a as i32
        }
        if !coords.is_empty() && !self.tables.cvar.is_empty() {
            if let Some(tuples) =
                var::cvar_tuples(self.tables.cvar, 0, coords, self.info.axis_count)
            {
                for deltas in tuples {
                    for (index, delta, _) in deltas {
                        if let Some(value) = values.get_mut(index) {
                            *value += delta.to_i32();
                        }
                    }
                }
            }
        }
        if let Some(scale) = scale {
            for v in values.iter_mut() {
                *v = mul(*v, scale);
            }
        }
        Some(())
    }

    /// Computes the set of variation deltas for a simple glyph.
    pub fn deltas(
        &self,
        coords: &[i16],
        glyph_id: u16,
        points: &[Point],
        tags: &mut [u8],
        contours: &[u16],
        accum: &mut [Point],
        deltas: &mut [Point],
    ) -> bool {
        if self.tables.gvar.is_empty() {
            return false;
        }
        const HAS_DELTA_TAG: u8 = 4;
        if let Some(tuples) = var::gvar_tuples(self.tables.gvar, 0, coords, glyph_id) {
            let len = points.len();
            if len > tags.len() || len > deltas.len() || len > accum.len() {
                return false;
            }
            let tags = &mut tags[..len];
            let accum = &mut accum[..len];
            let deltas = &mut deltas[..len];
            for (d, t) in deltas.iter_mut().zip(tags.iter_mut()) {
                *d = Point::default();
                *t &= !HAS_DELTA_TAG;
            }
            for tuple_deltas in tuples {
                if tuple_deltas.full_coverage() {
                    for (index, x, y) in tuple_deltas {
                        if let Some(point) = deltas.get_mut(index) {
                            point.x += x.0;
                            point.y += y.0;
                        }
                    }
                } else {
                    for p in accum.iter_mut() {
                        *p = Point::default();
                    }
                    for (index, x, y) in tuple_deltas {
                        if let Some(tag) = tags.get_mut(index) {
                            *tag |= HAS_DELTA_TAG;
                        }
                        if let Some(point) = accum.get_mut(index) {
                            point.x += x.0;
                            point.y += y.0;
                        }
                    }
                    let mut next_start = 0;
                    for end in contours.iter() {
                        let start = next_start;
                        let end = *end as usize;
                        next_start = end + 1;
                        if start >= len || end >= len {
                            continue;
                        }
                        let mut idx = start;
                        while idx <= end && tags[idx] & HAS_DELTA_TAG == 0 {
                            idx += 1;
                        }
                        if idx <= end {
                            let first_delta = idx;
                            let mut cur_delta = idx;
                            idx += 1;
                            while idx <= end {
                                if tags[idx] & HAS_DELTA_TAG != 0 {
                                    interpolate(
                                        cur_delta + 1,
                                        idx - 1,
                                        cur_delta,
                                        idx,
                                        points,
                                        accum,
                                    );
                                    cur_delta = idx;
                                }
                                idx += 1;
                            }
                            if cur_delta == first_delta {
                                let d = accum[cur_delta];
                                for a in accum[start..=end].iter_mut() {
                                    *a = d;
                                }
                            } else {
                                interpolate(
                                    cur_delta + 1,
                                    end,
                                    cur_delta,
                                    first_delta,
                                    points,
                                    accum,
                                );
                                if first_delta > 0 {
                                    interpolate(
                                        start,
                                        first_delta - 1,
                                        cur_delta,
                                        first_delta,
                                        points,
                                        accum,
                                    );
                                }
                            }
                        }
                    }
                    for ((d, t), a) in deltas.iter_mut().zip(tags.iter_mut()).zip(accum.iter()) {
                        *t &= !HAS_DELTA_TAG;
                        d.x += a.x;
                        d.y += a.y;
                    }
                }
            }
            for d in deltas.iter_mut() {
                d.x = Fixed(d.x).round().to_i32();
                d.y = Fixed(d.y).round().to_i32();
            }
            return true;
        }
        false
    }

    /// Computes the set of variation deltas for a composite glyph.
    pub fn composite_deltas(&self, coords: &[i16], glyph_id: u16, deltas: &mut [Point]) -> bool {
        if self.tables.gvar.is_empty() {
            return false;
        }
        if let Some(tuples) = var::gvar_tuples(self.tables.gvar, 0, coords, glyph_id) {
            for delta in deltas.iter_mut() {
                *delta = Point::default();
            }
            for tuple_deltas in tuples {
                for (index, x, y) in tuple_deltas {
                    if let Some(point) = deltas.get_mut(index) {
                        point.x += x.round().to_i32();
                        point.y += y.round().to_i32();
                    }
                }
            }
            return true;
        }
        false
    }
}

fn interpolate(
    p1: usize,
    p2: usize,
    ref1: usize,
    ref2: usize,
    points: &[Point],
    deltas: &mut [Point],
) {
    if p1 > p2 {
        return;
    }
    let (ref1, ref2) = if points[ref1].x > points[ref2].x {
        (ref2, ref1)
    } else {
        (ref1, ref2)
    };
    let in1 = Fixed::from_i32(points[ref1].x);
    let in2 = Fixed::from_i32(points[ref2].x);
    let out1 = Fixed(deltas[ref1].x);
    let out2 = Fixed(deltas[ref2].x);
    if in1 == in2 {
        if out1 == out2 {
            for delta in deltas[p1..=p2].iter_mut() {
                delta.x = out1.0;
            }
        } else {
            for delta in deltas[p1..=p2].iter_mut() {
                delta.x = 0;
            }
        }
    } else {
        for p in p1..=p2 {
            let t = Fixed::from_i32(points[p].x);
            if t <= in1 {
                deltas[p].x = out1.0;
            } else if t >= in2 {
                deltas[p].x = out2.0;
            } else {
                let f = (t - in1) / (in2 - in1);
                deltas[p].x = ((Fixed::ONE - f) * out1 + f * out2).0;
            }
        }
    }
    // Repeat for y
    let (ref1, ref2) = if points[ref1].y > points[ref2].y {
        (ref2, ref1)
    } else {
        (ref1, ref2)
    };
    let in1 = Fixed::from_i32(points[ref1].y);
    let in2 = Fixed::from_i32(points[ref2].y);
    let out1 = Fixed(deltas[ref1].y);
    let out2 = Fixed(deltas[ref2].y);
    if in1 == in2 {
        if out1 == out2 {
            for delta in deltas[p1..=p2].iter_mut() {
                delta.y = out1.0;
            }
        } else {
            for delta in deltas[p1..=p2].iter_mut() {
                delta.y = 0;
            }
        }
    } else {
        for p in p1..=p2 {
            let t = Fixed::from_i32(points[p].y);
            if t <= in1 {
                deltas[p].y = out1.0;
            } else if t >= in2 {
                deltas[p].y = out2.0;
            } else {
                let f = (t - in1) / (in2 - in1);
                deltas[p].y = ((Fixed::ONE - f) * out1 + f * out2).0;
            }
        }
    }
}
