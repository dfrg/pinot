use super::{ClassDef, CoverageArray, Covered, GlyphClass, Subtable};
use crate::parse_prelude::*;

/// Sequence context format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#sequence-context-format-1-simple-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct SeqContext1<'a>(pub(super) Subtable<'a>);

impl<'a> SeqContext1<'a> {
    /// Returns the rule set for the specified glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<RuleSet<'a, GlyphSeqRule<'a>>> {
        self.rule_sets().get(glyph_id.coverage_index() as usize)
    }

    /// Returns the list of chained contextual rule sets. The rule set is indexed by the
    /// coverage value of the current glyph.
    pub fn rule_sets(&self) -> RuleSetArray<'a, GlyphSeqRule<'a>> {
        RuleSetArray::new(*self.0.data(), self.0.record.offset, 4).unwrap_or_default()
    }
}

/// Sequence context format 2.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#sequence-context-format-2-class-based-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct SeqContext2<'a>(pub(super) Subtable<'a>);

impl<'a> SeqContext2<'a> {
    /// Returns the input class definitions.
    pub fn input(&self) -> ClassDef<'a> {
        classes(&self.0, 4)
    }

    /// Returns the rule set for the specified glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<RuleSet<'a, ClassSeqRule<'a>>> {
        let index = self.input().get(glyph_id.glyph_id()) as usize;
        self.rule_sets().get(index)
    }

    /// Returns the list of contextual rule sets. The rule set is indexed by the
    /// input class of the current glyph.
    pub fn rule_sets(&self) -> RuleSetArray<'a, ClassSeqRule<'a>> {
        RuleSetArray::new(*self.0.data(), self.0.record.offset, 6).unwrap_or_default()
    }
}

/// Sequence context format 3.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#sequence-context-format-3-coverage-based-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct SeqContext3<'a>(pub(super) Subtable<'a>);

impl<'a> SeqContext3<'a> {
    /// Returns the rule for the specified glyph.
    pub fn get(&self, _glyph_id: Covered) -> Option<CoverageSeqRule<'a>> {
        self.rule_impl(false)
    }

    /// Returns the contextual rule.
    pub fn rule(&self) -> Option<CoverageSeqRule<'a>> {
        self.rule_impl(true)
    }

    fn rule_impl(&self, include_first: bool) -> Option<CoverageSeqRule<'a>> {
        let data = *self.0.data();
        let base = self.0.record.offset;
        let mut c = data.cursor_at(base as usize + 2)?;
        let mut input_count = c.read::<u16>()? as usize;
        if !include_first {
            input_count -= 1;
            c.skip(2)?;
        }
        let lookup_count = c.read::<u16>()? as usize;
        let input = CoverageArray::new(data, base, c.read_slice::<u16>(input_count)?);
        let lookups = c.read_slice::<NestedLookup>(lookup_count)?;
        Some(SeqContextRule { input, lookups })
    }
}

/// Chained context format 1.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#chained-sequence-context-format-1-simple-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct ChainContext1<'a>(pub(super) Subtable<'a>);

impl<'a> ChainContext1<'a> {
    /// Returns the rule set for the specified glyph.
    pub fn get(&self, glyph_id: Covered) -> Option<RuleSet<'a, GlyphChainRule<'a>>> {
        self.rule_sets().get(glyph_id.coverage_index() as usize)
    }

    /// Returns the list of chained context rule sets. The rule set is indexed by the
    /// coverage value of the current glyph.
    pub fn rule_sets(&self) -> RuleSetArray<'a, GlyphChainRule<'a>> {
        RuleSetArray::new(*self.0.data(), self.0.record.offset, 4).unwrap_or_default()
    }
}

/// Chained context format 2.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#chained-sequence-context-format-2-class-based-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct ChainContext2<'a>(pub(super) Subtable<'a>);

impl<'a> ChainContext2<'a> {
    /// Returns the backtrack class definitions.
    pub fn backtrack(&self) -> ClassDef<'a> {
        classes(&self.0, 4)
    }

    /// Returns the input class definitions.
    pub fn input(&self) -> ClassDef<'a> {
        classes(&self.0, 6)
    }

    /// Returns the lookahead class definitions.
    pub fn lookahead(&self) -> ClassDef<'a> {
        classes(&self.0, 8)
    }

    /// Returns the rule set for the specified covered glyph.
    pub fn get(&self, covered: Covered) -> Option<RuleSet<'a, ClassChainRule<'a>>> {
        self.rule_sets().get(covered.coverage_index() as usize)
    }

    /// Returns the list of chained contextual rule sets. The rule set is indexed by the
    /// input class of the current glyph.
    pub fn rule_sets(&self) -> RuleSetArray<'a, ClassChainRule<'a>> {
        RuleSetArray::new(*self.0.data(), self.0.record.offset, 10).unwrap_or_default()
    }
}

/// Chained context format 3.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#chained-sequence-context-format-3-coverage-based-glyph-contexts>
#[derive(Copy, Clone, Debug)]
pub struct ChainContext3<'a>(pub(super) Subtable<'a>);

impl<'a> ChainContext3<'a> {
    /// Returns the rule for the specified covered glyph.
    pub fn get(&self, _covered: Covered) -> Option<CoverageChainRule<'a>> {
        self.rule_impl(false)
    }

    /// Returns the contextual rule.
    pub fn rule(&self) -> Option<CoverageChainRule<'a>> {
        self.rule_impl(true)
    }

    fn rule_impl(&self, include_first: bool) -> Option<CoverageChainRule<'a>> {
        let data = *self.0.data();
        let base = self.0.record.offset;
        let mut s = data.cursor_at(base as usize + 2)?;
        let backtrack = CoverageArray::new(data, base, s.read_slice16::<u16>()?);
        let mut input_count = s.read::<u16>()? as usize;
        if !include_first {
            input_count -= 1;
            s.skip(2)?;
        }
        let input = CoverageArray::new(data, base, s.read_slice::<u16>(input_count)?);
        let lookahead = CoverageArray::new(data, base, s.read_slice16::<u16>()?);
        let lookups = s.read_slice16::<NestedLookup>()?;
        Some(ChainContextRule {
            backtrack,
            input,
            lookahead,
            lookups,
        })
    }
}

/// Collection of rule sets for a contextual lookup.
#[derive(Copy, Clone)]
pub struct RuleSetArray<'a, R: ReadRule<'a>> {
    data: Buffer<'a>,
    base: u32,
    offsets: Slice<'a, u16>,
    _phantom: core::marker::PhantomData<R>,
}

impl<'a, R: ReadRule<'a>> Default for RuleSetArray<'a, R> {
    fn default() -> Self {
        Self {
            data: Buffer::default(),
            base: 0,
            offsets: Slice::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, R: ReadRule<'a>> RuleSetArray<'a, R> {
    fn new(data: Buffer<'a>, base: u32, offset: u32) -> Option<Self> {
        let mut c = data.cursor_at(base as usize + offset as usize)?;
        let offsets = c.read_slice16::<u16>()?;
        Some(Self {
            data,
            base,
            offsets,
            _phantom: core::marker::PhantomData,
        })
    }

    /// Returns the number of sets in the list.
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the set at the specified index.
    pub fn get(&self, index: usize) -> Option<RuleSet<'a, R>> {
        let offset = self.offsets.get(index)? as u32;
        if offset != 0 {
            let base = self.base + offset;
            let mut c = self.data.cursor_at(base as usize)?;
            let offsets = c.read_slice16::<u16>()?;
            Some(RuleSet {
                data: self.data,
                base,
                offsets,
                _phantom: core::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Returns an iterator over the sets in the list.
    pub fn iter(&self) -> impl Iterator<Item = RuleSet<'a, R>> + '_ + Clone {
        (0..self.len()).map(move |index| self.get(index).unwrap_or_default())
    }
}

/// Set of rules for a contextual lookup.
#[derive(Copy, Clone)]
pub struct RuleSet<'a, R: ReadRule<'a>> {
    data: Buffer<'a>,
    base: u32,
    offsets: Slice<'a, u16>,
    _phantom: core::marker::PhantomData<R>,
}

impl<'a, R: ReadRule<'a>> Default for RuleSet<'a, R> {
    fn default() -> Self {
        Self {
            data: Buffer::default(),
            base: 0,
            offsets: Slice::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, R: ReadRule<'a>> RuleSet<'a, R> {
    /// Returns the number of rules in the set.
    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Returns the rule at the specified index.
    pub fn get(&self, index: usize) -> Option<R> {
        let offset = self.offsets.get(index)?;
        if offset != 0 {
            let mut s = self.data.cursor_at(self.base as usize + offset as usize)?;
            R::read_rule(&mut s)
        } else {
            None
        }
    }

    /// Returns an iterator over the rules in the set.
    pub fn iter(&'a self) -> impl Iterator<Item = R> + 'a + Clone {
        (0..self.len()).filter_map(move |index| self.get(index))
    }
}

#[doc(hidden)]
pub trait ReadRule<'a>: Sized {
    fn read_rule(c: &mut Cursor<'a>) -> Option<Self>;
}

impl<'a> ReadRule<'a> for SeqContextRule<'a, Slice<'a, u16>> {
    fn read_rule(c: &mut Cursor<'a>) -> Option<Self> {
        let input_count = (c.read::<u16>()? as usize).checked_sub(1)?;
        let lookup_count = c.read::<u16>()?;
        let input = c.read_slice::<u16>(input_count)?;
        let lookups = c.read_slice::<NestedLookup>(lookup_count as usize)?;
        Some(Self { input, lookups })
    }
}

impl<'a> ReadRule<'a> for ChainContextRule<'a, Slice<'a, u16>> {
    fn read_rule(c: &mut Cursor<'a>) -> Option<Self> {
        let backtrack = c.read_slice16::<u16>()?;
        let input_count = (c.read::<u16>()? as usize).checked_sub(1)?;
        let input = c.read_slice::<u16>(input_count)?;
        let lookahead = c.read_slice16::<u16>()?;
        let lookups = c.read_slice16::<NestedLookup>()?;
        Some(Self {
            backtrack,
            input,
            lookahead,
            lookups,
        })
    }
}

/// Rule for a sequence context subtable.
#[derive(Copy, Clone)]
pub struct SeqContextRule<'a, T> {
    pub input: T,
    pub lookups: Slice<'a, NestedLookup>,
}

/// Rule for a sequence context matched by glyph identifers.
pub type GlyphSeqRule<'a> = SeqContextRule<'a, Slice<'a, GlyphId>>;

/// Rule for a sequence context matched by glyph class identifiers.
pub type ClassSeqRule<'a> = SeqContextRule<'a, Slice<'a, GlyphClass>>;

/// Rule for a sequence context matched by coverage.
pub type CoverageSeqRule<'a> = SeqContextRule<'a, CoverageArray<'a>>;

/// Rule for a chained context subtable.
#[derive(Copy, Clone)]
pub struct ChainContextRule<'a, T> {
    pub backtrack: T,
    pub input: T,
    pub lookahead: T,
    pub lookups: Slice<'a, NestedLookup>,
}

/// Rule for a chained context matched by glyph identifers.
pub type GlyphChainRule<'a> = ChainContextRule<'a, Slice<'a, GlyphId>>;

/// Rule for a chained context matched by glyph class identifiers.
pub type ClassChainRule<'a> = ChainContextRule<'a, Slice<'a, GlyphClass>>;

/// Rule for a chained context matched by coverage.
pub type CoverageChainRule<'a> = ChainContextRule<'a, CoverageArray<'a>>;

/// Lookup to be applied on a successful contextual match.
#[derive(Copy, Clone, Debug)]
pub struct NestedLookup {
    /// Index in the lookup list.
    pub lookup_index: u16,
    /// Index in the input glyph sequence where the lookup should be applied.
    pub sequence_index: u16,
}

impl ReadData for NestedLookup {
    unsafe fn read_data_unchecked(buf: &[u8], offset: usize) -> Self {
        Self {
            lookup_index: u16::read_data_unchecked(buf, offset + 2),
            sequence_index: u16::read_data_unchecked(buf, offset),
        }
    }
}

fn classes<'a>(s: &Subtable<'a>, offset: usize) -> ClassDef<'a> {
    let data = *s.data();
    let base = s.record.offset;
    let mut classes_offset = data.read::<u16>(base as usize + offset).unwrap_or(0) as u32;
    if classes_offset != 0 {
        classes_offset += base;
    }
    ClassDef::new(data, classes_offset)
}
