use super::data::Data;
use super::hint::*;
use super::Point;

#[derive(Copy, Clone)]
pub enum CacheSlot {
    Uncached,
    Cached(usize, usize),
}

#[derive(Default)]
pub struct Cache {
    fonts: Vec<FontEntry>,
    sizes: Vec<SizeEntry>,
    stack: Vec<i32>,
    twilight: Vec<Point>,
    twilight_tags: Vec<u8>,
    epoch: u64,
    max_entries: usize,
    uncached_font: FontEntry,
    uncached_size: SizeEntry,
}

impl Cache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            fonts: Vec::new(),
            sizes: Vec::new(),
            stack: Vec::new(),
            twilight: Vec::new(),
            twilight_tags: Vec::new(),
            epoch: 0,
            max_entries,
            uncached_font: FontEntry::default(),
            uncached_size: SizeEntry::default(),
        }
    }

    pub fn prepare(
        &mut self,
        font_id: Option<u64>,
        data: &Data,
        coords: &[i16],
        ppem: u16,
        scale: i32,
        mode: HinterMode,
    ) -> Option<CacheSlot> {
        self.epoch += 1;
        let epoch = self.epoch;
        let mut run_fpgm = false;
        let max_twilight = data.limits.max_twilight as usize + 4;
        self.twilight.resize(max_twilight * 3, Point::default());
        self.twilight_tags.resize(max_twilight, 0);
        self.stack.resize(data.limits.max_stack as usize, 0);
        let font_entry = self.find_font(font_id);
        if !font_entry.0 {
            let max_fdefs = data.limits.max_fdefs as usize;
            let max_idefs = data.limits.max_idefs as usize;
            let font = if font_entry.1 == !0 {
                &mut self.uncached_font
            } else {
                &mut self.fonts[font_entry.1]
            };
            font.font_id = font_id.unwrap_or(0);
            font.epoch = epoch;
            font.definitions.clear();
            font.definitions
                .resize(max_fdefs + max_idefs, Function::default());
            font.max_fdefs = max_fdefs;
            font.cvt_len = data.tables.cvt.len() / 2;
            run_fpgm = true;
        } else {
            self.fonts[font_entry.1].epoch = epoch;
        }
        let size_entry = self.find_size(font_id, coords, scale, mode);
        let mut run_prep = false;
        if !size_entry.0 {
            let size = if size_entry.1 == !0 {
                &mut self.uncached_size
            } else {
                &mut self.sizes[size_entry.1]
            };
            size.font_id = font_id.unwrap_or(0);
            size.epoch = epoch;
            size.state = HinterState::default();
            size.mode = mode;
            size.scale = scale;
            size.coords.clear();
            size.coords.extend_from_slice(coords);
            let cvt_len = data.tables.cvt.len() / 2;
            size.store.clear();
            size.store
                .resize(cvt_len + data.limits.max_storage as usize, 0);
            data.cvt(Some(scale), coords, &mut size.store);
            run_prep = true;
        } else {
            self.sizes[size_entry.1].epoch = epoch;
        }
        if run_fpgm | run_prep {
            let font = &mut self.fonts[font_entry.1];
            let size = &mut self.sizes[size_entry.1];
            let (cvt, store) = size.store.split_at_mut(font.cvt_len);
            let (fdefs, idefs) = font.definitions.split_at_mut(font.max_fdefs);
            let glyph = Zone::new(&mut [], &mut [], &mut [], &mut [], &[]);
            let max_twilight = self.twilight_tags.len();
            let (unscaled, rest) = self.twilight.split_at_mut(max_twilight);
            let (original, points) = rest.split_at_mut(max_twilight);
            let twilight_contours = [max_twilight as u16];
            let twilight = Zone::new(
                unscaled,
                original,
                points,
                &mut self.twilight_tags[..],
                &twilight_contours,
            );
            let mut hinter = Hinter::new(
                store,
                cvt,
                fdefs,
                idefs,
                &mut self.stack,
                twilight,
                glyph,
                coords,
                data.info.axis_count,
            );
            if run_fpgm {
                let mut state = HinterState::default();
                if !hinter.run_fpgm(&mut state, data.tables.fpgm) {
                    return None;
                }
            }
            if run_prep {
                size.state = HinterState::default();
                if !hinter.run_prep(
                    &mut size.state,
                    mode,
                    data.tables.fpgm,
                    data.tables.prep,
                    ppem,
                    scale,
                ) {
                    return None;
                }
            }
        }
        if !self.sizes[size_entry.1].state.hinting_enabled() {
            return None;
        }
        if font_entry.1 != !0 {
            Some(CacheSlot::Cached(font_entry.1, size_entry.1))
        } else {
            Some(CacheSlot::Uncached)
        }
    }

    pub fn hint(
        &mut self,
        data: &Data,
        coords: &[i16],
        slot: CacheSlot,
        unscaled: &mut [Point],
        original: &mut [Point],
        scaled: &mut [Point],
        tags: &mut [u8],
        contours: &mut [u16],
        phantom: &mut [Point],
        point_base: usize,
        contour_base: usize,
        ins: &[u8],
        is_composite: bool,
    ) {
        let (font, size) = match slot {
            CacheSlot::Uncached => (&mut self.uncached_font, &mut self.uncached_size),
            CacheSlot::Cached(font, size) => (&mut self.fonts[font], &mut self.sizes[size]),
        };
        if is_composite && point_base != 0 {
            for c in &mut contours[contour_base..] {
                *c -= point_base as u16;
            }
        }
        let glyph = Zone::new(
            unscaled,
            original,
            &mut scaled[point_base..],
            &mut tags[point_base..],
            &contours[contour_base..],
        );
        let twilight_len = self.twilight_tags.len();
        let twilight_contours = [twilight_len as u16];
        let (twilight_original, rest) = self.twilight.split_at_mut(twilight_len);
        let (twilight_unscaled, twilight_points) = rest.split_at_mut(twilight_len);
        let twilight = Zone::new(
            twilight_unscaled,
            twilight_original,
            twilight_points,
            &mut self.twilight_tags[..],
            &twilight_contours,
        );
        let (cvt, store) = size.store.split_at_mut(font.cvt_len);
        let (fdefs, idefs) = font.definitions.split_at_mut(font.max_fdefs);
        let mut hinter = Hinter::new(
            store,
            cvt,
            fdefs,
            idefs,
            &mut self.stack[..],
            twilight,
            glyph,
            coords,
            data.info.axis_count,
        );
        hinter.run(
            &mut size.state,
            data.tables.fpgm,
            data.tables.prep,
            ins,
            is_composite,
        );
        if !size.state.compat_enabled() {
            for (i, p) in (&scaled[scaled.len() - 4..]).iter().enumerate() {
                phantom[i] = *p;
            }
        }
        if is_composite && point_base != 0 {
            for c in &mut contours[contour_base..] {
                *c += point_base as u16;
            }
        }
    }

    fn find_font(&mut self, font_id: Option<u64>) -> (bool, usize) {
        let font_id = match font_id {
            Some(font_id) => font_id,
            None => return (false, !0),
        };
        let mut lowest_epoch = self.epoch;
        let mut lowest_index = 0;
        for (i, font) in self.fonts.iter().enumerate() {
            if font.font_id == font_id {
                return (true, i);
            }
            if font.epoch < lowest_epoch {
                lowest_epoch = font.epoch;
                lowest_index = i;
            }
        }
        if self.fonts.len() < self.max_entries {
            lowest_index = self.fonts.len();
            self.fonts.push(FontEntry::default());
        }
        (false, lowest_index)
    }

    fn find_size(
        &mut self,
        font_id: Option<u64>,
        coords: &[i16],
        scale: i32,
        mode: HinterMode,
    ) -> (bool, usize) {
        let font_id = match font_id {
            Some(font_id) => font_id,
            None => return (false, !0),
        };
        let mut lowest_epoch = self.epoch;
        let mut lowest_index = 0;
        let vary = !coords.is_empty();
        for (i, size) in self.sizes.iter().enumerate() {
            if size.epoch < lowest_epoch {
                lowest_epoch = size.epoch;
                lowest_index = i;
            }
            if size.font_id == font_id
                && size.scale == scale
                && size.mode == mode
                && (!vary || (coords == &size.coords[..]))
            {
                return (true, i);
            }
        }
        if self.sizes.len() < self.max_entries {
            lowest_index = self.sizes.len();
            self.sizes.push(SizeEntry::default());
        }
        (false, lowest_index)
    }
}

#[derive(Default)]
struct FontEntry {
    font_id: u64,
    definitions: Vec<Function>,
    max_fdefs: usize,
    cvt_len: usize,
    epoch: u64,
}

#[derive(Default)]
struct SizeEntry {
    font_id: u64,
    state: HinterState,
    mode: HinterMode,
    coords: Vec<i16>,
    scale: i32,
    store: Vec<i32>,
    epoch: u64,
}
