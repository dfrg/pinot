use super::{ClassDef, Coverage, Covered, GlyphClass, MarkAttachClass, Subtable};
use crate::container::prelude::*;
use crate::variation::item::Store;

/// Single position adjustment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#single-adjustment-positioning-format-1-single-positioning-value>
#[derive(Copy, Clone, Debug)]
pub struct SinglePos1<'a>(pub(super) Subtable<'a>);

impl<'a> SinglePos1<'a> {
    /// Returns the value format.
    pub fn value_format(&self) -> ValueFormat {
        ValueFormat(
            self.0
                .data()
                .read_u16(self.0.record.offset as usize + 4)
                .unwrap_or(0),
        )
    }

    /// Returns the single position adjustment value.
    pub fn value(&self) -> Option<Value> {
        let base = self.0.record.offset as usize;
        let data = self.0.data();
        self.0
            .read_position(base, base + 6, ValueFormat(data.read_u16(base + 4)?), true)
    }

    /// Returns the position adjustment for the specified covered glyph.
    pub fn get(self, _glyph_id: Covered) -> Option<Value> {
        self.value()
    }

    /// Invokes the specified closure with all position adjustments in the subtable.
    pub fn values_with(&self, mut f: impl FnMut(GlyphId, Value) -> bool) -> Option<bool> {
        let value = self.value()?;
        self.0.coverage().indices_with(|glyph_id, _| {
            if !f(glyph_id, value) {
                return false;
            }
            true
        })
    }
}

/// Single position adjustment format 2.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#single-adjustment-positioning-format-2-array-of-positioning-values>
#[derive(Copy, Clone, Debug)]
pub struct SinglePos2<'a>(pub(super) Subtable<'a>);

impl<'a> SinglePos2<'a> {
    /// Returns the value format.
    pub fn value_format(&self) -> ValueFormat {
        ValueFormat(
            self.0
                .data()
                .read_u16(self.0.record.offset as usize + 4)
                .unwrap_or(0),
        )
    }

    /// Returns the position adjustment for the specified covered glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<Value> {
        self.get_impl(glyph_id.coverage_index())
    }

    /// Invokes the specified closure with all position adjustments in the subtable.
    pub fn values_with(&self, mut f: impl FnMut(GlyphId, Value) -> bool) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(pos) = self.get_impl(coverage) {
                if !f(glyph_id, pos) {
                    return false;
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16) -> Option<Value> {
        let base = self.0.record.offset as usize;
        let data = self.0.data();
        let value_format = ValueFormat(data.read_u16(base + 4)?);
        let len = value_format.size();
        self.0
            .read_position(base, base + 8 + coverage as usize * len, value_format, true)
    }
}

/// Pair position adjustment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#pair-adjustment-positioning-format-1-adjustments-for-glyph-pairs>
#[derive(Copy, Clone, Debug)]
pub struct PairPos1<'a>(pub(super) Subtable<'a>);

impl<'a> PairPos1<'a> {
    /// Returns the value formats.
    pub fn value_formats(&self) -> [ValueFormat; 2] {
        let (data, base) = self.0.data_and_offset();
        [
            ValueFormat(data.read_u16(base + 4).unwrap_or(0)),
            ValueFormat(data.read_u16(base + 6).unwrap_or(0)),
        ]
    }

    /// Returns the position adjustments for the specified covered glyph
    /// and the following glyph.
    pub fn get(&self, glyph_id: Covered, next_id: GlyphId) -> Option<[Option<Value>; 2]> {
        self.get_impl(glyph_id.coverage_index(), next_id)
    }

    /// Invokes the specified closure with all position adjustments in the subtable.
    pub fn pairs_with(
        &self,
        mut f: impl FnMut(GlyphId, GlyphId, [Option<Value>; 2]) -> bool,
    ) -> Option<bool> {
        let (data, base) = self.0.data_and_offset();
        let [value_format1, value_format2] = self.value_formats();
        let len1 = value_format1.size();
        let step = len1 + value_format2.size() + 2;
        self.0.coverage().indices_with(|glyph_id, coverage| {
            let mut proc = || {
                let set_base = base + data.read_u16(base + 10 + coverage as usize * 2)? as usize;
                let count = data.read_u16(set_base)? as usize;
                let value_base = set_base + 2;
                for i in 0..count {
                    let v = value_base + i * step;
                    let g2 = data.read_u16(v)?;
                    let mut pos: [Option<Value>; 2] = Default::default();
                    if value_format1.0 != 0 {
                        pos[0] = self.0.read_position(set_base, v + 2, value_format1, true);
                    }
                    if value_format2.0 != 0 {
                        pos[1] = self
                            .0
                            .read_position(set_base, v + 2 + len1, value_format2, true);
                    }
                    if !f(glyph_id, g2, pos) {
                        return Some(false);
                    }
                }
                Some(true)
            };
            if proc() != Some(true) {
                return false;
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16, next_id: u16) -> Option<[Option<Value>; 2]> {
        let (data, base) = self.0.data_and_offset();
        let [value_format1, value_format2] = self.value_formats();
        let len1 = value_format1.size();
        let step = len1 + value_format2.size() + 2;
        let set_base = base + data.read_u16(base + 10 + coverage as usize * 2)? as usize;
        let count = data.read_u16(set_base)? as usize;
        let value_base = set_base + 2;
        let mut lo = 0;
        let mut hi = count;
        while lo < hi {
            use core::cmp::Ordering::*;
            let i = (lo + hi) / 2;
            let v = value_base + i * step;
            let g2 = data.read_u16(v)?;
            match next_id.cmp(&g2) {
                Greater => lo = i + 1,
                Less => hi = i,
                Equal => {
                    let mut pos: [Option<Value>; 2] = Default::default();
                    if value_format1.0 != 0 {
                        pos[0] = self.0.read_position(set_base, v + 2, value_format1, true);
                    }
                    if value_format2.0 != 0 {
                        pos[1] = self
                            .0
                            .read_position(set_base, v + 2 + len1, value_format2, true);
                    }
                    return Some(pos);
                }
            }
        }
        None
    }
}

/// Pair position adjustment format 2.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#pair-adjustment-positioning-format-2-class-pair-adjustment>
#[derive(Copy, Clone, Debug)]
pub struct PairPos2<'a>(pub(super) Subtable<'a>);

impl<'a> PairPos2<'a> {
    /// Returns the value formats for the first and second glyph.
    pub fn value_formats(&self) -> [ValueFormat; 2] {
        let (data, base) = self.0.data_and_offset();
        [
            ValueFormat(data.read_u16(base + 4).unwrap_or(0)),
            ValueFormat(data.read_u16(base + 6).unwrap_or(0)),
        ]
    }

    /// Returns the number of classes for the first and second glyph.
    pub fn num_classes(&self) -> [u16; 2] {
        let (data, base) = self.0.data_and_offset();
        [
            data.read_u16(base + 12).unwrap_or(0),
            data.read_u16(base + 14).unwrap_or(0),
        ]
    }

    /// Returns the class definitions for the first and second glyph.
    pub fn classes(&self) -> [ClassDef<'a>; 2] {
        let (data, base) = self.0.data_and_offset();
        let offset1 = data.read_offset16(base + 8, base as u32).unwrap_or(0);
        let offset2 = data.read_offset16(base + 10, base as u32).unwrap_or(0);
        [ClassDef::new(*data, offset1), ClassDef::new(*data, offset2)]
    }

    /// Returns the position adjustments for the specified covered glyph
    /// and the following glyph.
    pub fn get(&self, glyph_id: Covered, next_id: GlyphId) -> Option<[Option<Value>; 2]> {
        self.get_impl(glyph_id.coverage_index(), next_id)
    }

    /// Returns the position adjustments for the specified glyph class pair.
    pub fn get_by_classes(
        &self,
        class1: GlyphClass,
        class2: GlyphClass,
    ) -> Option<[Option<Value>; 2]> {
        let (data, base) = self.0.data_and_offset();
        let [value_format1, value_format2] = self.value_formats();
        let len1 = value_format1.size();
        let step = len1 + value_format2.size();
        let class2_count = data.read_u16(base + 14)? as usize;
        let v = base + 16 + (class1 as usize * step * class2_count) + (class2 as usize * step);
        let mut pos: [Option<Value>; 2] = Default::default();
        if value_format1.0 != 0 {
            pos[0] = self.0.read_position(base, v, value_format1, true);
        }
        if value_format2.0 != 0 {
            pos[1] = self.0.read_position(base, v + len1, value_format2, true);
        }
        Some(pos)
    }

    /// Invokes the specified closure with all pair position adjustments in the
    /// subtable.
    pub fn pairs_with(
        &self,
        mut f: impl FnMut(GlyphId, GlyphId, [Option<Value>; 2]) -> bool,
    ) -> Option<bool> {
        let [classes1, classes2] = self.classes();
        classes1.classes_with(|glyph_id, class1| {
            classes2.classes_with(|next_id, class2| {
                if let Some(pos) = self.get_by_classes(class1, class2) {
                    let all_zero = pos[0].map(|p| p.all_zero()).unwrap_or(true)
                        && pos[1].map(|p| p.all_zero()).unwrap_or(true);
                    if !all_zero && !f(glyph_id, next_id, pos) {
                        return false;
                    }
                }
                true
            }) == Some(true)
        })
    }

    /// Invokes the specified closure with all pair position adjustments in the
    /// subtable by classes.
    pub fn pairs_by_class_with(
        &self,
        mut f: impl FnMut(GlyphClass, GlyphClass, [Option<Value>; 2]) -> bool,
    ) -> Option<bool> {
        let [count1, count2] = self.num_classes();
        for class1 in 0..count1 {
            for class2 in 0..count2 {
                if let Some(pos) = self.get_by_classes(class1, class2) {
                    let all_zero = pos[0].map(|p| p.all_zero()).unwrap_or(true)
                        && pos[1].map(|p| p.all_zero()).unwrap_or(true);
                    if !all_zero && !f(class1, class2, pos) {
                        return Some(false);
                    }
                }
            }
        }
        Some(true)
    }

    fn get_impl(&self, glyph_id: u16, next_id: u16) -> Option<[Option<Value>; 2]> {
        let (data, base) = self.0.data_and_offset();
        let [value_format1, value_format2] = self.value_formats();
        let len1 = value_format1.size();
        let step = len1 + value_format2.size();
        let [classes1, classes2] = self.classes();
        let class2_count = data.read_u16(base + 14)? as usize;
        let class1 = classes1.get(glyph_id) as usize;
        let class2 = classes2.get(next_id) as usize;
        let v = base + 16 + (class1 * step * class2_count) + (class2 * step);
        let mut pos: [Option<Value>; 2] = Default::default();
        if value_format1.0 != 0 {
            pos[0] = self.0.read_position(base, v, value_format1, true);
        }
        if value_format2.0 != 0 {
            pos[1] = self.0.read_position(base, v + len1, value_format2, true);
        }
        Some(pos)
    }
}

/// Cursive attachment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#cursive-attachment-positioning-format1-cursive-attachment>
#[derive(Copy, Clone, Debug)]
pub struct CursivePos1<'a>(pub(super) Subtable<'a>);

impl<'a> CursivePos1<'a> {
    const ENTRY: usize = 0;
    const EXIT: usize = 2;

    /// Returns the entry and exit anchors for the specified covered glyph.
    pub fn get(&self, covered: Covered) -> Option<[Option<Anchor>; 2]> {
        let index = covered.coverage_index();
        Some([
            self.get_impl(index, Self::ENTRY),
            self.get_impl(index, Self::EXIT),
        ])
    }

    /// Returns the entry anchor for the specified covered glyph.
    pub fn entry(&self, glyph_id: Covered) -> Option<Anchor> {
        self.get_impl(glyph_id.coverage_index(), Self::ENTRY)
    }

    /// Returns the exit anchor for the specified covered glyph.
    pub fn exit(&self, glyph_id: Covered) -> Option<Anchor> {
        let index = glyph_id.coverage_index();
        self.get_impl(index, Self::EXIT)
    }

    /// Invokes the specified closure with all entry and exit anchors in the subtable.
    pub fn anchors_with(
        &self,
        mut f: impl FnMut(GlyphId, [Option<Anchor>; 2]) -> bool,
    ) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(anchors) = {
                Some([
                    self.get_impl(coverage, Self::ENTRY),
                    self.get_impl(coverage, Self::EXIT),
                ])
            } {
                if !f(glyph_id, anchors) {
                    return false;
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16, which: usize) -> Option<Anchor> {
        let (data, base) = self.0.data_and_offset();
        let offset =
            data.read_offset16(base + 6 + coverage as usize * 4 + which, base as u32)? as usize;
        self.0.read_anchor(offset, true)
    }
}

/// Mark to base attachment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-base-attachment-positioning-format-1-mark-to-base-attachment-point>
#[derive(Copy, Clone, Debug)]
pub struct MarkPos1<'a>(pub(super) Subtable<'a>);

impl<'a> MarkPos1<'a> {
    /// Returns the base and mark anchors for the specified base glyph and covered
    /// mark glyph.
    pub fn get(&self, base_id: GlyphId, mark_id: Covered) -> Option<[Anchor; 2]> {
        self.get_impl(mark_id.coverage_index(), base_id)
    }

    /// Returns the number of mark classes.
    pub fn num_mark_classes(&self) -> u16 {
        let (data, base) = self.0.data_and_offset();
        data.read_u16(base + 6).unwrap_or(0)
    }

    /// Returns the base glyph coverage table.
    pub fn base_coverage(&self) -> Coverage<'a> {
        let (data, base) = self.0.data_and_offset();
        let offset = data.read_offset16(base + 4, base as u32).unwrap_or(0);
        Coverage::new(*data, offset)
    }

    /// Returns the base anchor for the specified glyph and mark class.
    pub fn base_anchor(&self, base_id: GlyphId, mark_class: MarkAttachClass) -> Option<Anchor> {
        let base_coverage = self.base_coverage().get(base_id)? as usize;
        let (data, base) = self.0.data_and_offset();
        let class_count = data.read_u16(base + 6)? as usize;
        let bases_base = data.read_offset16(base + 10, base as u32)? as usize;
        let count = data.read_u16(bases_base)? as usize * class_count;
        let index = class_count * base_coverage + mark_class as usize;
        if index >= count {
            return None;
        }
        let anchor_base =
            data.read_offset16(bases_base + 2 + index * 2, bases_base as u32)? as usize;
        self.0.read_anchor(anchor_base, true)
    }

    /// Returns the mark class and anchor for the specified covered glyph.
    pub fn mark_anchor(&self, mark_id: Covered) -> Option<(MarkAttachClass, Anchor)> {
        let (data, base) = self.0.data_and_offset();
        let marks_base = data.read_offset16(base + 8, base as u32)? as usize;
        self.0
            .read_mark_anchor(marks_base, mark_id.coverage_index(), true)
    }

    /// Invokes the specified closure with all base/mark anchor pairs in the subtable.
    pub fn pairs_with(
        &self,
        mut f: impl FnMut(GlyphId, GlyphId, [Anchor; 2]) -> bool,
    ) -> Option<bool> {
        let base_coverage = self.base_coverage();
        base_coverage.indices_with(|base_id, _| {
            self.0.coverage().indices_with(|mark_id, coverage| {
                if let Some(anchors) = self.get_impl(coverage, base_id) {
                    if !f(base_id, mark_id, anchors) {
                        return false;
                    }
                }
                true
            }) == Some(true)
        })
    }

    /// Invokes the specified closure with the glyph, mark class and anchor of all
    /// marks in the subtable.
    pub fn marks_with(
        &self,
        mut f: impl FnMut(GlyphId, (MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(anchor) = self.mark_anchor(Covered::new(glyph_id, coverage)) {
                if !f(glyph_id, anchor) {
                    return false;
                }
            }
            true
        })
    }

    /// Invokes the specified closure with the glyph, mark class and anchor of all
    /// bases in the subtable.
    pub fn bases_with(
        &self,
        mut f: impl FnMut(GlyphId, (MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        let num_classes = self.num_mark_classes();
        self.base_coverage().indices_with(|glyph_id, _| {
            for i in 0..num_classes {
                if let Some(anchor) = self.base_anchor(glyph_id, i) {
                    if !f(glyph_id, (i, anchor)) {
                        return false;
                    }
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16, base_id: u16) -> Option<[Anchor; 2]> {
        let base_coverage = self.base_coverage().get(base_id)? as usize;
        let (data, base) = self.0.data_and_offset();
        let marks_base = data.read_offset16(base + 8, base as u32)? as usize;
        let (mark_class, mark_anchor) = self.0.read_mark_anchor(marks_base, coverage, true)?;
        let base_anchor = {
            let class_count = data.read_u16(base + 6)? as usize;
            let bases_base = data.read_offset16(base + 10, base as u32)? as usize;
            let count = data.read_u16(bases_base)? as usize * class_count;
            let index = class_count * base_coverage + mark_class as usize;
            if index >= count {
                return None;
            }
            let anchor_base =
                data.read_offset16(bases_base + 2 + index * 2, bases_base as u32)? as usize;
            self.0.read_anchor(anchor_base, true)?
        };
        Some([base_anchor, mark_anchor])
    }
}

/// Mark to ligature attachment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-ligature-attachment-positioning-format-1-mark-to-ligature-attachment>
#[derive(Copy, Clone, Debug)]
pub struct MarkLigaturePos1<'a>(pub(super) Subtable<'a>);

impl<'a> MarkLigaturePos1<'a> {
    /// Returns the base and mark anchors for the specified ligature glyph,
    /// component index and covered mark glyph.
    pub fn get(
        &self,
        ligature_id: GlyphId,
        component_index: u16,
        mark_id: Covered,
    ) -> Option<[Anchor; 2]> {
        self.get_impl(mark_id.coverage_index(), ligature_id, component_index)
    }

    /// Returns the ligature glyph coverage table.
    pub fn ligature_coverage(&self) -> Coverage<'a> {
        MarkPos1(self.0).base_coverage()
    }

    /// Returns the number of mark classes.
    pub fn num_mark_classes(&self) -> u16 {
        MarkPos1(self.0).num_mark_classes()
    }

    /// Returns the number of ligature components for the specified glyph.
    pub fn num_ligature_components(&self, ligature_id: GlyphId) -> u16 {
        self.ligature_coverage()
            .get(ligature_id)
            .and_then(|coverage| self.ligature_offset(coverage))
            .map(|offset| self.0.data().read_u16(offset))
            .flatten()
            .unwrap_or(0)
    }

    /// Returns the mark class and anchor for the specified glyph.
    pub fn mark_anchor(&self, mark_id: Covered) -> Option<(MarkAttachClass, Anchor)> {
        MarkPos1(self.0).mark_anchor(mark_id)
    }

    /// Returns the component anchor for the specified ligature, component index, and mark class.
    pub fn component_anchor(
        &self,
        ligature_id: GlyphId,
        component_index: u16,
        mark_class: MarkAttachClass,
    ) -> Option<Anchor> {
        let base_coverage = self.ligature_coverage().get(ligature_id)?;
        let ligature_base = self.ligature_offset(base_coverage)?;
        let (data, base) = self.0.data_and_offset();
        let component_count = data.read_u16(ligature_base)? as usize;
        if component_index as usize >= component_count {
            return None;
        }
        let class_count = data.read_u16(base + 6)? as usize;
        let count = component_count * class_count;
        let index = class_count * component_index as usize + mark_class as usize;
        if index >= count {
            return None;
        }
        let anchor_base =
            data.read_offset16(ligature_base + 2 + index * 2, ligature_base as u32)? as usize;
        self.0.read_anchor(anchor_base, true)
    }

    /// Invokes the specified closure with the glyph, mark class and anchor of all
    /// marks in the subtable.
    pub fn marks_with(
        self,
        f: impl FnMut(GlyphId, (MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        MarkPos1(self.0).marks_with(f)
    }

    /// Invokes the specified closure with the glyph, component index, mark class and anchor of
    /// all ligature components in the subtable.
    pub fn components_with(
        &self,
        mut f: impl FnMut(GlyphId, (u16, MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        let num_classes = self.num_mark_classes();
        self.ligature_coverage().indices_with(|glyph_id, _| {
            let count = self.num_ligature_components(glyph_id);
            for i in 0..count {
                for j in 0..num_classes {
                    if let Some(anchor) = self.component_anchor(glyph_id, i, j) {
                        if !f(glyph_id, (i, j, anchor)) {
                            return false;
                        }
                    }
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16, ligature_id: u16, component: u16) -> Option<[Anchor; 2]> {
        let base_coverage = self.ligature_coverage().get(ligature_id)?;
        let ligature_base = self.ligature_offset(base_coverage)?;
        let (data, base) = self.0.data_and_offset();
        let component_count = data.read_u16(ligature_base)? as usize;
        if component as usize >= component_count {
            return None;
        }
        let marks_base = data.read_offset16(base + 8, base as u32)? as usize;
        let (mark_class, mark_anchor) = self.0.read_mark_anchor(marks_base, coverage, true)?;
        let base_anchor = {
            let class_count = data.read_u16(base + 6)? as usize;
            let count = component_count * class_count;
            let index = class_count * component as usize + mark_class as usize;
            if index >= count {
                return None;
            }
            let anchor_base =
                data.read_offset16(ligature_base + 2 + index * 2, ligature_base as u32)? as usize;
            self.0.read_anchor(anchor_base, true)?
        };
        Some([base_anchor, mark_anchor])
    }

    fn ligature_offset(self, coverage: u16) -> Option<usize> {
        let (data, base) = self.0.data_and_offset();
        let bases_base = data.read_offset16(base + 10, base as u32)? as usize;
        let offsets = data.read_slice16::<u16>(bases_base)?;
        let offset = offsets.get(coverage as usize)? as usize;
        if offset != 0 {
            Some(bases_base + offset)
        } else {
            None
        }
    }
}

/// Mark to mark attachment format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-mark-attachment-positioning-format-1-mark-to-mark-attachment>
#[derive(Copy, Clone, Debug)]
pub struct MarkMarkPos1<'a>(pub(super) Subtable<'a>);

impl<'a> MarkMarkPos1<'a> {
    /// Returns the base and mark anchors for the specified base glyph and covered
    /// mark glyph.
    pub fn get(&self, base_id: GlyphId, mark_id: Covered) -> Option<[Anchor; 2]> {
        MarkPos1(self.0).get(base_id, mark_id)
    }

    /// Returns the number of mark classes.
    pub fn num_mark_classes(&self) -> u16 {
        MarkPos1(self.0).num_mark_classes()
    }

    /// Returns the base glyph coverage table.
    pub fn base_coverage(&self) -> Coverage<'a> {
        MarkPos1(self.0).base_coverage()
    }

    /// Returns the base anchor for the specified glyph and mark class.
    pub fn base_anchor(&self, base_id: GlyphId, mark_class: MarkAttachClass) -> Option<Anchor> {
        MarkPos1(self.0).base_anchor(base_id, mark_class)
    }

    /// Returns the mark class and anchor for the specified covered glyph.
    pub fn mark_anchor(&self, mark_id: Covered) -> Option<(MarkAttachClass, Anchor)> {
        MarkPos1(self.0).mark_anchor(mark_id)
    }

    /// Invokes the specified closure with all base/mark anchor pairs in the subtable.
    pub fn pairs_with(&self, f: impl FnMut(GlyphId, GlyphId, [Anchor; 2]) -> bool) -> Option<bool> {
        MarkPos1(self.0).pairs_with(f)
    }

    /// Invokes the specified closure with the glyph, mark class and anchor of all
    /// marks in the subtable.
    pub fn marks_with(
        &self,
        f: impl FnMut(GlyphId, (MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        MarkPos1(self.0).marks_with(f)
    }

    /// Invokes the specified closure with the glyph, mark class and anchor of all
    /// bases in the subtable.
    pub fn bases_with(
        &self,
        f: impl FnMut(GlyphId, (MarkAttachClass, Anchor)) -> bool,
    ) -> Option<bool> {
        MarkPos1(self.0).bases_with(f)
    }
}

/// Format of a value record.
#[derive(Copy, Clone, Default, Debug)]
pub struct ValueFormat(pub u16);

impl ValueFormat {
    /// Returns the number of entries in the value.
    pub fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Returns the size of the value in bytes.
    pub fn size(self) -> usize {
        self.0.count_ones() as usize * 2
    }

    /// Returns true if the value has an X offset.
    pub fn has_x(self) -> bool {
        self.0 & 1 != 0
    }

    /// Returns true if the value has a Y offset.
    pub fn has_y(self) -> bool {
        self.0 & 2 != 0
    }

    /// Returns true if the value has an X advance.
    pub fn has_x_advance(self) -> bool {
        self.0 & 4 != 0
    }

    /// Returns true if the value has a Y advance.
    pub fn has_y_advance(self) -> bool {
        self.0 & 8 != 0
    }

    /// Returns true if the format has deltas.
    pub fn has_deltas(self) -> bool {
        self.0 & (0x10 | 0x20 | 0x40 | 0x80) != 0
    }

    /// Returns true if the value has an X delta.
    pub fn has_x_delta(self) -> bool {
        self.0 & 0x10 != 0
    }

    /// Returns true if the value has a Y delta.
    pub fn has_y_delta(self) -> bool {
        self.0 & 0x20 != 0
    }

    /// Returns true if the value has an X advance delta.
    pub fn has_x_advance_delta(self) -> bool {
        self.0 & 0x40 != 0
    }

    /// Returns true if the value has a Y advance delta.
    pub fn has_y_advance_delta(self) -> bool {
        self.0 & 0x80 != 0
    }
}

/// Component of a value or anchor.
#[derive(Copy, Clone, Default, Debug)]
pub struct Component {
    /// Raw value in font units.
    pub value: FWord,
    /// Outer and inner indices for the item variation store.
    pub delta_indices: Option<[u16; 2]>,
}

impl Component {
    /// Returns true if the value element has a variation delta.
    pub fn has_delta(&self) -> bool {
        self.delta_indices.is_some()
    }

    /// Returns the delta for the value according to the specified
    /// item variation store and normalized variation coordinates.
    pub fn delta(&self, store: &Store, coords: &[NormalizedCoord]) -> Option<Fixed> {
        self.delta_indices
            .map(|[outer, inner]| store.delta(outer, inner, coords))
    }
}

/// Raw value of a position adjustment in font units.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#value-record>
#[derive(Copy, Clone, Default, Debug)]
pub struct Value {
    /// Format of the value.
    pub format: ValueFormat,
    /// Horizontal offset.
    pub x: Component,
    /// Vertical offset.
    pub y: Component,
    /// Horizontal advance.
    pub x_advance: Component,
    /// Vertical advance.
    pub y_advance: Component,
}

impl Value {
    /// Returns true if all values are zero.
    fn all_zero(&self) -> bool {
        self.x.value == 0
            && self.y.value == 0
            && self.x_advance.value == 0
            && self.y_advance.value == 0
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "(")?;
        let mut space = "";
        if self.format.has_x() {
            write!(f, "x: {}", self.x.value)?;
            space = " ";
        }
        if self.format.has_y() {
            write!(f, "{}y: {}", space, self.y.value)?;
            space = " ";
        }
        if self.format.has_x_advance() {
            write!(f, "{}x_advance: {}", space, self.x_advance.value)?;
            space = " ";
        }
        if self.format.has_y_advance() {
            write!(f, "{}y_advance: {}", space, self.y_advance.value)?;
        }
        write!(f, ")")
    }
}

/// Raw value of an anchor in font units.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#anchor-tables>
#[derive(Copy, Clone, Default, Debug)]
pub struct Anchor {
    /// Horizontal position.
    pub x: Component,
    /// Vertical position.
    pub y: Component,
}

impl<'a> Subtable<'a> {
    fn read_position(
        &self,
        parent_offset: usize,
        mut offset: usize,
        format: ValueFormat,
        do_var: bool,
    ) -> Option<Value> {
        let d = self.data();
        let mut pos =  Value { format, ..Default::default() };
        if format.0 == 4 {
            pos.x_advance.value = d.read_i16(offset)?;
            return Some(pos);
        }
        if format.has_x() {
            pos.x.value = d.read::<i16>(offset)?;
            offset += 2;
        }
        if format.has_y() {
            pos.y.value = d.read::<i16>(offset)?;
            offset += 2;
        }
        if format.has_x_advance() {
            pos.x_advance.value = d.read::<i16>(offset)?;
            offset += 2;
        }
        if format.has_y_advance() {
            pos.y_advance.value = d.read::<i16>(offset)?;
            offset += 2;
        }
        if do_var && format.has_deltas() {
            if format.has_x_delta() {
                pos.x.delta_indices =
                    self.read_delta_indices(parent_offset, d.read::<u16>(offset)?);
                offset += 2;
            }
            if format.has_y_delta() {
                pos.y.delta_indices =
                    self.read_delta_indices(parent_offset, d.read::<u16>(offset)?);
                offset += 2;
            }
            if format.has_x_advance_delta() {
                pos.x_advance.delta_indices =
                    self.read_delta_indices(parent_offset, d.read::<u16>(offset)?);
                offset += 2;
            }
            if format.has_y_advance_delta() {
                pos.y_advance.delta_indices =
                    self.read_delta_indices(parent_offset, d.read::<u16>(offset)?);
            }
        }
        Some(pos)
    }

    fn read_delta_indices(&self, parent_offset: usize, offset: u16) -> Option<[u16; 2]> {
        if offset == 0 {
            return None;
        }
        let b = self.data();
        let offset = parent_offset + offset as usize;
        let format = b.read::<u16>(offset + 4)?;
        if format != 0x8000 {
            return None;
        }
        let outer = b.read::<u16>(offset)?;
        let inner = b.read::<u16>(offset + 2)?;
        Some([outer, inner])
    }

    fn read_anchor(&self, offset: usize, do_var: bool) -> Option<Anchor> {
        let mut anchor = Anchor::default();
        let d = self.data();
        let format = d.read::<u16>(offset)?;
        anchor.x.value = d.read::<i16>(offset + 2)?;
        anchor.y.value = d.read::<i16>(offset + 4)?;
        if format == 3 && do_var {
            anchor.x.delta_indices = self.read_delta_indices(offset, d.read::<u16>(offset + 6)?);
            anchor.y.delta_indices = self.read_delta_indices(offset, d.read::<u16>(offset + 8)?);
        }
        Some(anchor)
    }

    fn read_mark_anchor(&self, marks: usize, index: u16, do_var: bool) -> Option<(u16, Anchor)> {
        let d = self.data();
        if index >= d.read::<u16>(marks)? {
            return None;
        }
        let rec = marks + 2 + index as usize * 4;
        let class = d.read::<u16>(rec)?;
        let offset = d.read::<u16>(rec + 2)? as usize;
        if offset == 0 {
            return None;
        }
        Some((class, self.read_anchor(marks + offset, do_var)?))
    }
}
