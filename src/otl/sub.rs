use super::{CoverageArray, Covered, Subtable};
use crate::parse_prelude::*;

/// Single substitution format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#lookuptype-1-single-substitution-subtable>
#[derive(Copy, Clone, Debug)]
pub struct SingleSubst1<'a>(pub(super) Subtable<'a>);

impl<'a> SingleSubst1<'a> {
    /// Returns the replacement for the specified covered glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<u16> {
        self.get_impl(glyph_id.glyph_id())
    }

    /// Single delta value applied to all glyphs covered by the subtable.
    pub fn delta(&self) -> Option<i16> {
        let (data, offset) = self.0.data_and_offset();
        data.read_i16(offset + 4)
    }

    /// Invokes the specified closure with all substitutions in the subtable.
    pub fn substs_with(&self, mut f: impl FnMut(GlyphId, GlyphId) -> bool) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, _| {
            if let Some(subst) = self.get_impl(glyph_id) {
                if !f(glyph_id, subst) {
                    return false;
                }
            }
            true
        })
    }

    fn get_impl(&self, glyph_id: u16) -> Option<u16> {
        let delta = self.delta()? as i32;
        Some((glyph_id as i32 + delta) as u16)
    }
}

/// Single substitution format 2.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#12-single-substitution-format-2>
#[derive(Copy, Clone, Debug)]
pub struct SingleSubst2<'a>(pub(super) Subtable<'a>);

impl<'a> SingleSubst2<'a> {
    /// Returns the replacement for the specified covered glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<u16> {
        self.get_impl(glyph_id.coverage_index())
    }

    /// Invokes the specified closure with all substitutions in the subtable.
    pub fn substs_with(&self, mut f: impl FnMut(GlyphId, GlyphId) -> bool) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(subst) = self.get_impl(coverage) {
                if !f(glyph_id, subst) {
                    return false;
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16) -> Option<u16> {
        let array_base = self.0.record.offset as usize + 6;
        self.0
            .data()
            .read::<u16>(array_base + coverage as usize * 2)
    }
}

/// Multiple substitution format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#21-multiple-substitution-format-1>
#[derive(Copy, Clone, Debug)]
pub struct MultipleSubst1<'a>(pub(super) Subtable<'a>);

impl<'a> MultipleSubst1<'a> {
    /// Returns the replacement sequence for the specified covered glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<Slice<'a, GlyphId>> {
        self.get_impl(glyph_id.coverage_index())
    }

    /// Invokes the specified closure with all substitutions in the subtable.
    pub fn substs_with(
        &self,
        mut f: impl FnMut(GlyphId, Slice<'a, GlyphId>) -> bool,
    ) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(subst) = self.get_impl(coverage) {
                if !f(glyph_id, subst) {
                    return false;
                }
            }
            true
        })
    }

    fn get_impl(&self, coverage: u16) -> Option<Slice<'a, GlyphId>> {
        let data = self.0.data();
        let base = self.0.record.offset as usize;
        let array_base = base + data.read::<u16>(base + 6 + coverage as usize * 2)? as usize;
        let array_len = data.read::<u16>(array_base)? as usize;
        data.read_slice::<u16>(array_base + 2, array_len)
    }
}

/// Alternate substitution format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#lookuptype-3-alternate-substitution-subtable>
#[derive(Copy, Clone, Debug)]
pub struct AlternateSubst1<'a>(pub(super) Subtable<'a>);

impl<'a> AlternateSubst1<'a> {
    /// Returns the replacement candidates for the specified covered glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<Slice<'a, GlyphId>> {
        MultipleSubst1(self.0).get(glyph_id)
    }

    /// Invokes the specified closure with all alternates in the subtable.
    pub fn alternates_with(
        &self,
        f: impl FnMut(GlyphId, Slice<'a, GlyphId>) -> bool,
    ) -> Option<bool> {
        MultipleSubst1(self.0).substs_with(f)
    }
}

/// Ligature substitution format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#41-ligature-substitution-format-1>
#[derive(Copy, Clone, Debug)]
pub struct LigatureSubst1<'a>(pub(super) Subtable<'a>);

impl<'a> LigatureSubst1<'a> {
    /// Returns an iterator over the ligatures beginning with the specified covered glyph.
    pub fn get(&self, covered: Covered) -> Option<Ligatures<'a>> {
        self.get_impl(covered.glyph_id(), covered.coverage_index())
    }

    /// Invokes the specified closure with all ligatures in the subtable.
    pub fn ligatures_with(&self, mut f: impl FnMut(Ligature<'a>) -> bool) -> Option<bool> {
        self.0.coverage().indices_with(|glyph_id, coverage| {
            if let Some(ligatures) = self.get_impl(glyph_id, coverage) {
                for ligature in ligatures {
                    if !f(ligature) {
                        return false;
                    }
                }
            }
            true
        })
    }

    fn get_impl(&self, glyph_id: u16, coverage: u16) -> Option<Ligatures<'a>> {
        let data = self.0.data();
        let base = self.0.record.offset as usize;
        let set_base = base + data.read::<u16>(base + 6 + coverage as usize * 2)? as usize;
        let len = data.read::<u16>(set_base)? as usize;
        Some(Ligatures {
            data: Buffer::with_offset(data.data(), set_base)?,
            first_component: glyph_id,
            len,
            pos: 0,
        })
    }
}

/// Iterator over the set of ligatures for an initial glyph.
#[derive(Copy, Clone)]
pub struct Ligatures<'a> {
    data: Buffer<'a>,
    first_component: u16,
    len: usize,
    pos: usize,
}

impl<'a> Iterator for Ligatures<'a> {
    type Item = Ligature<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let pos = self.pos;
        self.pos += 1;
        let base = self.data.read::<u16>(2 + pos * 2)? as usize;
        let ligature = self.data.read::<u16>(base)?;
        let len = (self.data.read::<u16>(base + 2)? as usize).saturating_sub(1);
        let trailing_components = self.data.read_slice::<u16>(base + 4, len)?;
        Some(Ligature {
            first_component: self.first_component,
            trailing_components,
            ligature,
        })
    }
}

/// Ligature glyph and components.
#[derive(Copy, Clone)]
pub struct Ligature<'a> {
    /// Initial component of the ligature.
    pub first_component: GlyphId,
    /// Sequence of identifers that represent the trailing component glyphs.
    pub trailing_components: Slice<'a, GlyphId>,
    /// Identifier of the ligature glyph.
    pub ligature: GlyphId,
}

/// Reverse chained context format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#lookuptype-8-reverse-chaining-contextual-single-substitution-subtable>
#[derive(Copy, Clone, Debug)]
pub struct RevChainContext1<'a>(pub(super) Subtable<'a>);

impl<'a> RevChainContext1<'a> {
    pub fn rule(&self) -> Option<RevChainContextRule<'a>> {
        let data = *self.0.data();
        let base = self.0.record.offset;
        let mut s = data.cursor_at(base as usize + 4)?;
        let backtrack_count = s.read::<u16>()? as usize;
        let backtrack = CoverageArray::new(data, base, s.read_slice::<u16>(backtrack_count)?);
        let lookahead_count = s.read::<u16>()? as usize;
        let lookahead = CoverageArray::new(data, base, s.read_slice::<u16>(lookahead_count)?);
        let substitution_count = s.read::<u16>()? as usize;
        let substitutions = s.read_slice::<u16>(substitution_count)?;
        Some(RevChainContextRule {
            backtrack,
            lookahead,
            substitutions,
        })
    }
}

/// Rule for a reverse chained context subtable.
#[derive(Copy, Clone)]
pub struct RevChainContextRule<'a> {
    /// Backtrack coverage sequence.
    pub backtrack: CoverageArray<'a>,
    /// Lookahead coverage sequence.
    pub lookahead: CoverageArray<'a>,
    /// Substitution glyph array, ordered by primary coverage index.
    pub substitutions: Slice<'a, GlyphId>,
}
