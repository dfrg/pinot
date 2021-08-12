use super::context::*;
use super::pos::*;
use super::shared::get_coverage;
use super::sub::*;
use super::{Coverage, Covered, Layout, Stage};
use crate::container::parse::Buffer;
use core::fmt;

/// Kind of a lookup.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LookupKind {
    /// One to one substitution.
    SingleSubst,
    /// One to many substitution.
    MultipleSubst,
    /// One to one substitution from a list of candidates.
    AlternateSubst,
    /// Many to one substitution.
    LigatureSubst,
    /// Single position adjustment.
    SinglePos,
    /// Positition adjustment between a pair.
    PairPos,
    /// Cursive attachment.
    CursivePos,
    /// Mark to base attachment.
    MarkPos,
    /// Mark to ligature component attachment.
    MarkLigaturePos,
    /// Mark to mark attachment.
    MarkMarkPos,
    /// Contextual lookup.
    SeqContext,
    /// Contextual lookup with backtrack and/or lookahead sequences.
    ChainContext,
    /// Contextual lookup that operates in reverse logical order.
    RevChainContext,
}

/// Filter for a lookup.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct LookupFilter {
    /// Bit flags for ignored classes.
    pub ignored_classes: u8,
    /// User provided bit mask.
    pub mask: u8,
    /// Specifies whether additional mark tests are necessary.
    pub mark_check: bool,
    /// Mark class for the lookup.
    pub mark_class: u8,
    /// Mark filtering set for the lookup.
    pub mark_set: u32,
}

/// Information about a lookup.
#[derive(Copy, Clone, Debug)]
pub struct LookupRecord {
    /// Stage that contains the lookup.
    pub stage: Stage,
    /// Kind of the lookup.
    pub kind: LookupKind,
    /// True if the lookup is an extension.
    pub is_extension: bool,
    /// Index of the lookup in the layout table.
    pub index: u16,
    /// Lookup qualifiers.
    pub flag: LookupFlag,
    /// Lookup filter.
    pub filter: LookupFilter,
    /// Offset to the lookup from the beginning of the associated
    /// layout table.
    pub offset: u32,
    /// Number of subtables for the lookup.
    pub num_subtables: u16,
}

impl LookupRecord {
    /// Creates a new bound lookup for the specified layout context. The lookup must
    /// belong to the associated layout table.
    pub fn materialize<'a>(&self, layout: &'a Layout<'a>) -> Lookup<'a> {
        Lookup {
            layout,
            record: *self,
        }
    }
}

/// Lookup qualifiers.
#[derive(Copy, Clone, Debug)]
pub struct LookupFlag(pub u16);

impl LookupFlag {
    /// Returns true if cursive attachments should be processed in right-to-left order.
    pub fn is_rtl(self) -> bool {
        self.0 & 1 != 0
    }

    /// Returns true if bases should be ignored.
    pub fn ignore_bases(self) -> bool {
        self.0 & 2 != 0
    }

    /// Returns true if ligatures should be ignored.
    pub fn ignore_ligatures(self) -> bool {
        self.0 & 4 != 0
    }

    /// Returns true if marks should be ignored.
    pub fn ignore_marks(self) -> bool {
        self.0 & 8 != 0
    }

    /// Returns true if a mark filtering set should be used.
    pub fn use_mark_set(self) -> bool {
        self.0 & 0x10 != 0
    }

    /// Returns the mark class filter.
    pub fn mark_class(self) -> Option<u16> {
        let ty = (self.0 & 0xFF00) >> 8;
        if ty != 0 {
            Some(ty)
        } else {
            None
        }
    }
}

/// Single lookup with an associated layout table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#lookup-table>
#[derive(Copy, Clone)]
pub struct Lookup<'a> {
    /// Associated layout table.
    pub layout: &'a Layout<'a>,
    /// Record for the lookup.
    pub record: LookupRecord,
}

impl<'a> Lookup<'a> {
    /// Returns the number of subtables.
    pub fn num_subtables(&self) -> u16 {
        self.record.num_subtables
    }

    /// Returns the subtable at the specified index.
    pub fn subtable(&'a self, index: u16) -> Option<Subtable<'a>> {
        Some(Subtable {
            lookup: self,
            record: self.subtable_record(index)?,
        })
    }

    /// Returns an iterator over the subtables.
    pub fn subtables(&'a self) -> impl Iterator<Item = Subtable<'a>> + 'a + Clone {
        (0..self.record.num_subtables).filter_map(move |index| self.subtable(index))
    }

    fn subtable_record(&self, index: u16) -> Option<SubtableRecord> {
        if index >= self.record.num_subtables {
            return None;
        }
        let b = &self.layout.data;
        let lookup_base = self.record.offset as usize;
        let subtable_base = lookup_base + 6;
        let mut base = lookup_base + b.read::<u16>(subtable_base + index as usize * 2)? as usize;
        if self.record.is_extension {
            base = base + b.read::<u32>(base + 4)? as usize;
        }
        let fmt = b.read::<u16>(base)?;
        fn cov(b: &Buffer, base: usize, offset: usize) -> Option<u16> {
            use super::shared::validate_coverage;
            let c = b.read::<u16>(base as usize + offset)?;
            validate_coverage(b, base as u32 + c as u32)?;
            Some(c)
        }
        let offset = base as u32;
        use LookupKind::*;
        match self.record.kind {
            SingleSubst => {
                let kind = match fmt {
                    1 => SubtableFormat::SingleSubst1,
                    2 => SubtableFormat::SingleSubst2,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            MultipleSubst => {
                let kind = match fmt {
                    1 => SubtableFormat::MultipleSubst1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            AlternateSubst => {
                let kind = match fmt {
                    1 => SubtableFormat::AlternateSubst1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            LigatureSubst => {
                let kind = match fmt {
                    1 => SubtableFormat::LigatureSubst1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            SinglePos => {
                let kind = match fmt {
                    1 => SubtableFormat::SinglePos1,
                    2 => SubtableFormat::SinglePos2,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            PairPos => {
                let kind = match fmt {
                    1 => SubtableFormat::PairPos1,
                    2 => SubtableFormat::PairPos2,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            CursivePos => {
                let kind = match fmt {
                    1 => SubtableFormat::CursivePos1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            MarkPos => {
                let kind = match fmt {
                    1 => SubtableFormat::MarkPos1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            MarkLigaturePos => {
                let kind = match fmt {
                    1 => SubtableFormat::MarkLigaturePos1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            MarkMarkPos => {
                let kind = match fmt {
                    1 => SubtableFormat::MarkMarkPos1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
            SeqContext => match fmt {
                1 | 2 => {
                    let kind = if fmt == 1 {
                        SubtableFormat::SeqContext1
                    } else {
                        SubtableFormat::SeqContext2
                    };
                    let coverage_offset = cov(b, base, 2)?;
                    Some(SubtableRecord {
                        kind,
                        offset,
                        coverage_offset,
                    })
                }
                3 => {
                    let coverage_offset = cov(b, base, 6)?;
                    Some(SubtableRecord {
                        kind: SubtableFormat::SeqContext3,
                        offset,
                        coverage_offset,
                    })
                }
                _ => None,
            },
            ChainContext => match fmt {
                1 | 2 => {
                    let kind = if fmt == 1 {
                        SubtableFormat::ChainContext1
                    } else {
                        SubtableFormat::ChainContext2
                    };
                    let coverage_offset = cov(b, base, 2)?;
                    Some(SubtableRecord {
                        kind,
                        offset,
                        coverage_offset,
                    })
                }
                3 => {
                    let backtrack_len = b.read::<u16>(base + 2)? as usize * 2;
                    let input_len = b.read::<u16>(base + backtrack_len + 4)?;
                    if input_len == 0 {
                        return None;
                    }
                    let coverage_offset = cov(b, base, backtrack_len + 6)?;
                    Some(SubtableRecord {
                        kind: SubtableFormat::ChainContext3,
                        offset,
                        coverage_offset,
                    })
                }
                _ => None,
            },
            RevChainContext => {
                let kind = match fmt {
                    1 => SubtableFormat::RevChainContext1,
                    _ => return None,
                };
                let coverage_offset = cov(b, base, 2)?;
                Some(SubtableRecord {
                    kind,
                    offset,
                    coverage_offset,
                })
            }
        }
    }
}

/// Internal subtable format.
#[derive(Copy, Clone, Debug)]
enum SubtableFormat {
    SingleSubst1,
    SingleSubst2,
    MultipleSubst1,
    AlternateSubst1,
    LigatureSubst1,
    SinglePos1,
    SinglePos2,
    PairPos1,
    PairPos2,
    CursivePos1,
    MarkPos1,
    MarkLigaturePos1,
    MarkMarkPos1,
    SeqContext1,
    SeqContext2,
    SeqContext3,
    ChainContext1,
    ChainContext2,
    ChainContext3,
    RevChainContext1,
}

/// Information about a lookup subtable.
#[derive(Copy, Clone, Debug)]
pub struct SubtableRecord {
    /// Offset to the subtable from the beginning of the associated
    /// layout table.
    pub offset: u32,
    /// Offset to the primary coverage table from the beginning of the subtable.
    pub coverage_offset: u16,
    /// Internal format of the subtable.
    kind: SubtableFormat,
}

impl SubtableRecord {
    /// Creates a new bound subtable for the specified lookup. The subtable
    /// must belong to the lookup.
    pub fn materialize<'a>(&self, lookup: &'a Lookup<'a>) -> Subtable<'a> {
        Subtable {
            lookup,
            record: *self,
        }
    }
}

/// Single subtable with an associated lookup.
#[derive(Copy, Clone)]
pub struct Subtable<'a> {
    /// Associated lookup.
    pub lookup: &'a Lookup<'a>,
    /// Record for the subtable.
    pub record: SubtableRecord,
}

impl<'a> Subtable<'a> {
    /// Returns the associated layout table.
    pub fn layout(&self) -> &Layout<'a> {
        &self.lookup.layout
    }

    /// Returns the kind of the subtable.
    pub fn kind(self) -> SubtableKind<'a> {
        match self.record.kind {
            SubtableFormat::SingleSubst1 => SubtableKind::SingleSubst1(SingleSubst1(self)),
            SubtableFormat::SingleSubst2 => SubtableKind::SingleSubst2(SingleSubst2(self)),
            SubtableFormat::MultipleSubst1 => SubtableKind::MultipleSubst1(MultipleSubst1(self)),
            SubtableFormat::AlternateSubst1 => SubtableKind::AlternateSubst1(AlternateSubst1(self)),
            SubtableFormat::LigatureSubst1 => SubtableKind::LigatureSubst1(LigatureSubst1(self)),
            SubtableFormat::SinglePos1 => SubtableKind::SinglePos1(SinglePos1(self)),
            SubtableFormat::SinglePos2 => SubtableKind::SinglePos2(SinglePos2(self)),
            SubtableFormat::PairPos1 => SubtableKind::PairPos1(PairPos1(self)),
            SubtableFormat::PairPos2 => SubtableKind::PairPos2(PairPos2(self)),
            SubtableFormat::CursivePos1 => SubtableKind::CursivePos1(CursivePos1(self)),
            SubtableFormat::MarkPos1 => SubtableKind::MarkPos1(MarkPos1(self)),
            SubtableFormat::MarkLigaturePos1 => {
                SubtableKind::MarkLigaturePos1(MarkLigaturePos1(self))
            }
            SubtableFormat::MarkMarkPos1 => SubtableKind::MarkMarkPos1(MarkMarkPos1(self)),
            SubtableFormat::SeqContext1 => SubtableKind::SeqContext1(SeqContext1(self)),
            SubtableFormat::SeqContext2 => SubtableKind::SeqContext2(SeqContext2(self)),
            SubtableFormat::SeqContext3 => SubtableKind::SeqContext3(SeqContext3(self)),
            SubtableFormat::ChainContext1 => SubtableKind::ChainContext1(ChainContext1(self)),
            SubtableFormat::ChainContext2 => SubtableKind::ChainContext2(ChainContext2(self)),
            SubtableFormat::ChainContext3 => SubtableKind::ChainContext3(ChainContext3(self)),
            SubtableFormat::RevChainContext1 => {
                SubtableKind::RevChainContext1(RevChainContext1(self))
            }
        }
    }

    /// Returns the primary coverage table.
    pub fn coverage(&self) -> Coverage<'a> {
        Coverage::new(
            *self.data(),
            self.record.offset + self.record.coverage_offset as u32,
        )
    }

    /// Returns the coverage index for the specified glyph.
    pub fn coverage_index(&self, glyph_id: u16) -> Option<u16> {
        get_coverage(
            self.data(),
            self.record.offset + self.record.coverage_offset as u32,
            glyph_id,
        )
    }

    /// Returns coverage for the glyph if it is actionable for the subtable.
    pub fn covered(&self, glyph_id: u16) -> Option<Covered> {
        Some(Covered::new(glyph_id, self.coverage_index(glyph_id)?))
    }

    pub(super) fn data(&self) -> &Buffer<'a> {
        &self.lookup.layout.data
    }

    pub(super) fn data_and_offset(&self) -> (&Buffer<'a>, usize) {
        (&self.lookup.layout.data, self.record.offset as usize)
    }
}

impl fmt::Debug for Subtable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.record)
    }
}

/// Kind of a subtable.
#[derive(Copy, Clone)]
pub enum SubtableKind<'a> {
    /// Single substitution format 1.
    SingleSubst1(SingleSubst1<'a>),
    /// Single substitution format 2.
    SingleSubst2(SingleSubst2<'a>),
    /// Multiple substitution format 1.
    MultipleSubst1(MultipleSubst1<'a>),
    /// Alternate substitution format 1.
    AlternateSubst1(AlternateSubst1<'a>),
    /// Ligature substitution format 1.
    LigatureSubst1(LigatureSubst1<'a>),
    /// Single position adjustment format 1.
    SinglePos1(SinglePos1<'a>),
    /// Single position adjustment format 2.
    SinglePos2(SinglePos2<'a>),
    /// Pair position adjustment format 1.
    PairPos1(PairPos1<'a>),
    /// Pair position adjustment format 2.
    PairPos2(PairPos2<'a>),
    /// Cursive attachment format 1.
    CursivePos1(CursivePos1<'a>),
    /// Mark to base attachment format 1.
    MarkPos1(MarkPos1<'a>),
    /// Mark to ligature attachment format 1.
    MarkLigaturePos1(MarkLigaturePos1<'a>),
    /// Mark to mark attachment format 1.
    MarkMarkPos1(MarkMarkPos1<'a>),
    /// Sequence context lookup format 1.
    SeqContext1(SeqContext1<'a>),
    /// Sequence context lookup format 2.
    SeqContext2(SeqContext2<'a>),
    /// Sequence context lookup format 3.
    SeqContext3(SeqContext3<'a>),
    /// Chained context lookup format 1.
    ChainContext1(ChainContext1<'a>),
    /// Chained context lookup format 2.
    ChainContext2(ChainContext2<'a>),
    /// Chained context lookup format 3.
    ChainContext3(ChainContext3<'a>),
    /// Reverse chained context lookup format 1.
    RevChainContext1(RevChainContext1<'a>),
}
