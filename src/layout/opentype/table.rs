use super::{GlyphDef, Layout, Lookup, LookupFilter, LookupFlag, LookupKind, LookupRecord, Stage};
use crate::container::prelude::*;

impl<'a> Layout<'a> {
    /// Creates a new layout table for the specified stage, table data,
    /// and glyph definitions. The table data should contain a `GSUB` or
    /// `GPOS` table matching the specified stage.
    pub fn new(stage: Stage, data: &'a [u8], glyphs: Option<GlyphDef<'a>>) -> Self {
        Self {
            stage,
            data: Buffer::new(data),
            glyphs,
        }
    }

    /// Returns the stage.
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Returns the underlying table data.
    pub fn data(&self) -> &'a [u8] {
        self.data.data()
    }

    /// Returns the associated glyph definitions.
    pub fn glyphs(&self) -> Option<&GlyphDef<'a>> {
        self.glyphs.as_ref()
    }

    /// Returns the number of available scripts.
    pub fn num_scripts(&self) -> u16 {
        if let Some(base) = self.data.read_u16(4) {
            self.data.read_u16(base as usize).unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns the script at the specified index.
    pub fn script(&'a self, index: u16) -> Option<Script<'a>> {
        let b = &self.data;
        let list_base = b.read_u16(4)? as usize;
        let len = b.read_u16(list_base)?;
        if index >= len {
            return None;
        }
        let record_base = list_base + 2 + index as usize * 6;
        let tag = b.read_u32(record_base)?;
        let mut offset = b.read_u16(record_base + 4)? as u32;
        if offset == 0 {
            return None;
        }
        offset += list_base as u32;
        let num_languages = b.read_u16(offset as usize + 2)?;
        let record = ScriptRecord {
            tag,
            offset,
            num_languages,
        };
        Some(record.materialize(self))
    }

    /// Returns an iterator over the available scripts.
    pub fn scripts(&'a self) -> impl Iterator<Item = Script<'a>> + 'a + Clone {
        (0..self.num_scripts()).filter_map(move |index| self.script(index))
    }

    /// Returns the number of available features.
    pub fn num_features(&self) -> u16 {
        if let Some(base) = self.data.read_u16(6) {
            self.data.read_u16(base as usize).unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns the feature at the specified index.
    pub fn feature(&'a self, index: u16) -> Option<Feature<'a>> {
        let b = &self.data;
        let list_base = b.read_u16(6)? as usize;
        let len = b.read_u16(list_base)?;
        if index >= len {
            return None;
        }
        let record_base = list_base + 2 + index as usize * 6;
        let tag = b.read_u32(record_base)?;
        let offset = b.read_u16(record_base + 4)? as u32;
        if offset == 0 {
            return None;
        }
        Some(
            FeatureRecord {
                index,
                tag,
                offset: list_base as u32 + offset,
            }
            .materialize(self),
        )
    }

    /// Returns an iterator over the available features.
    pub fn features(&'a self) -> impl Iterator<Item = Feature<'a>> + 'a + Clone {
        (0..self.num_features()).filter_map(move |index| self.feature(index))
    }

    /// Returns feature variation support for the layout table.
    pub fn feature_variations(&'a self) -> Option<FeatureVariations<'a>> {
        // If minor version is >= 1, feature variations offset should be present at offset 10
        if self.data.read_u16(2) >= Some(1) {
            let offset = self.data.read_offset32(10, 0)? as usize;
            let len = self.data.read_u32(offset + 4)?;
            Some(FeatureVariations {
                layout: self,
                base: offset,
                len,
            })
        } else {
            None
        }
    }

    /// Returns the number of available lookups.
    pub fn num_lookups(&self) -> u16 {
        if let Some(base) = self.data.read_u16(8) {
            self.data.read_u16(base as usize).unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns the lookup at the specified index.
    pub fn lookup(&'a self, index: u16) -> Option<Lookup<'a>> {
        let b = &self.data;
        let list_base = b.read_u16(8)? as usize;
        let len = b.read_u16(list_base)?;
        if index >= len {
            return None;
        }
        let base = list_base + b.read_u16(list_base + 2 + index as usize * 2)? as usize;
        let mut kind = b.read_u16(base)? as u8;
        let flag = b.read_u16(base + 2)?;
        let f = flag as u8;
        let num_subtables = b.read_u16(base + 4)?;
        let mark_class = (flag >> 8) as u8;
        let ignore_marks = f & (1 << 3) != 0;
        let mut mark_check = false;
        let mut mark_set = 0;
        if !ignore_marks {
            if let Some(glyphs) = &self.glyphs {
                mark_check = mark_class != 0 && glyphs.has_mark_classes();
                mark_set = if flag & 0x10 != 0 {
                    let idx = b.read_u16(base + 6 + num_subtables as usize * 2)?;
                    mark_check = true;
                    glyphs.mark_set_offset(idx).unwrap_or(0)
                } else {
                    0
                };
            }
        }
        let is_sub = self.stage == Stage::Substitution;
        let subtables = base + 6;
        let is_extension = (is_sub && kind == 7) || (!is_sub && kind == 9);
        if is_extension && num_subtables > 0 {
            let s = base + b.read_u16(subtables)? as usize;
            kind = b.read_u16(s + 2)? as u8;
        }
        use LookupKind::*;
        let kind = if is_sub {
            match kind {
                1 => SingleSubst,
                2 => MultipleSubst,
                3 => AlternateSubst,
                4 => LigatureSubst,
                5 => SeqContext,
                6 => ChainContext,
                8 => RevChainContext,
                _ => return None,
            }
        } else {
            match kind {
                1 => SinglePos,
                2 => PairPos,
                3 => CursivePos,
                4 => MarkPos,
                5 => MarkLigaturePos,
                6 => MarkMarkPos,
                7 => SeqContext,
                8 => ChainContext,
                _ => return None,
            }
        };
        let ignored_classes = ((f as u8) & 0b1110) | 1 << 5;
        let filter = LookupFilter {
            ignored_classes,
            mask: 0,
            mark_check,
            mark_class,
            mark_set,
        };
        Some(
            LookupRecord {
                index,
                stage: self.stage,
                kind,
                flag: LookupFlag(flag),
                filter,
                is_extension,
                offset: base as u32,
                num_subtables,
            }
            .materialize(self),
        )
    }

    /// Returns an iterator over the available lookups.
    pub fn lookups(&'a self) -> impl Iterator<Item = Lookup<'a>> + 'a + Clone {
        (0..self.num_lookups()).filter_map(move |index| self.lookup(index))
    }
}

/// Information about a script.
#[derive(Copy, Clone)]
pub struct ScriptRecord {
    /// Tag that identifies the script.
    pub tag: Tag,
    /// Offset to the script table from the beginning of the associated layout
    /// table.
    pub offset: u32,
    /// Number of languages associated with the script.
    pub num_languages: u16,
}

impl ScriptRecord {
    /// Creates a new bound script for the specified layout context. The script
    /// must belong to the associated layout table.
    pub fn materialize<'a>(&self, layout: &'a Layout<'a>) -> Script<'a> {
        Script {
            layout,
            record: *self,
        }
    }
}

/// Script in a layout table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#script-table-and-language-system-record>
#[derive(Copy, Clone)]
pub struct Script<'a> {
    /// Associated layout table.
    pub layout: &'a Layout<'a>,
    /// Record for the script.
    pub record: ScriptRecord,
}

impl<'a> Script<'a> {
    /// Returns the default language for the script.
    pub fn default_language(&self) -> Option<Language<'a>> {
        let data = &self.layout.data;
        let base = self.record.offset;
        let offset = data.read_u16(base as usize)? as u32;
        if offset != 0 {
            let tag = tag::from_bytes(b"DFLT");
            let record = LanguageRecord {
                script: self.record,
                is_default: true,
                tag,
                offset: base + offset,
            };
            Some(Language {
                script: *self,
                record,
            })
        } else {
            None
        }
    }

    /// Returns the number of languages supported by the script.
    pub fn num_languages(&self) -> u16 {
        self.record.num_languages
    }

    /// Returns the language at the specified index.
    pub fn language(&self, index: u16) -> Option<Language<'a>> {
        if index >= self.record.num_languages {
            return None;
        }
        let data = &self.layout.data;
        let base = self.record.offset;
        let record_base = base as usize + 4 + index as usize * 6;
        let tag = data.read_u32(record_base)?;
        let mut offset = data.read_u16(record_base + 4)? as u32;
        if offset == 0 {
            return None;
        }
        offset += base;
        let record = LanguageRecord {
            script: self.record,
            is_default: false,
            tag,
            offset,
        };
        Some(Language {
            script: *self,
            record,
        })
    }

    /// Returns an iterator over the languages supported by the script.
    pub fn languages(self) -> impl Iterator<Item = Language<'a>> + 'a + Clone {
        (0..self.record.num_languages).filter_map(move |index| self.language(index))
    }
}

/// Information about a language.
#[derive(Copy, Clone)]
pub struct LanguageRecord {
    /// Script that contains the language.
    pub script: ScriptRecord,
    /// True for a default language.
    pub is_default: bool,
    /// Tag that identifies the language.
    pub tag: Tag,
    /// Offset to the language from the beginning of the associated layout
    /// table.
    pub offset: u32,
}

impl LanguageRecord {
    /// Returns the BCP 47 language code.
    pub fn code(&self) -> Option<&'static str> {
        // super::tag::language_tag_to_id(self.tag)
        None
    }

    /// Creates a new bound language for the specified layout context. The language
    /// must belong to the associated layout table.    
    pub fn materialize<'a>(&self, table: &'a Layout<'a>) -> Language<'a> {
        let script = self.script.materialize(table);
        Language {
            script,
            record: *self,
        }
    }
}

/// Language with an associated script.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#language-system-table>
#[derive(Copy, Clone)]
pub struct Language<'a> {
    /// Associated script.
    pub script: Script<'a>,
    /// Record for the language.
    pub record: LanguageRecord,
}

impl<'a> Language<'a> {
    /// Returns the associated layout table.
    pub fn layout(&self) -> &Layout<'a> {
        self.script.layout
    }

    /// Returns the language tag.
    pub fn tag(&self) -> u32 {
        self.record.tag
    }

    /// Returns the BCP 47 language code.
    pub fn code(&self) -> Option<&'static str> {
        self.record.code()
    }

    /// Returns the indices of the features associated with the language.
    pub fn feature_indices(&self) -> Slice<'a, u16> {
        let data = &self.layout().data;
        data.read_slice16(self.record.offset as usize + 4)
            .unwrap_or_default()
    }

    /// Returns an iterator over the features associated with the language.
    pub fn features(&'a self) -> impl Iterator<Item = Feature<'a>> + 'a + Clone {
        self.feature_indices()
            .iter()
            .filter_map(move |index| self.layout().feature(index))
    }
}

/// Information about a feature.
#[derive(Copy, Clone, Debug)]
pub struct FeatureRecord {
    /// Index of the feature.
    pub index: u16,
    /// Tag that identifies the feature.
    pub tag: Tag,
    /// Offset to the feature from the beginning of the associated layout
    /// table.
    pub offset: u32,
}

impl FeatureRecord {
    pub fn materialize<'a>(&self, layout: &'a Layout<'a>) -> Feature<'a> {
        Feature {
            layout,
            record: *self,
        }
    }
}

/// Typographic feature defined as a set of lookups.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#feature-table>
#[derive(Copy, Clone)]
pub struct Feature<'a> {
    /// Associated layout.
    pub layout: &'a Layout<'a>,
    /// Record for the feature.
    pub record: FeatureRecord,
}

impl<'a> Feature<'a> {
    /// Returns the lookup indices for the feature.
    pub fn lookup_indices(&self) -> Slice<'a, u16> {
        self.layout
            .data
            .read_slice16(self.record.offset as usize + 2)
            .unwrap_or_default()
    }

    /// Returns an iterator over the lookups for the feature.
    pub fn lookups(&'a self) -> impl Iterator<Item = Lookup<'a>> + 'a + Clone {
        self.lookup_indices()
            .iter()
            .filter_map(move |index| self.layout.lookup(index))
    }
}

/// Feature variations table.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#featurevariations-table>
#[derive(Copy, Clone)]
pub struct FeatureVariations<'a> {
    layout: &'a Layout<'a>,
    base: usize,
    len: u32,
}

impl<'a> FeatureVariations<'a> {
    /// Returns the number of condition sets.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns true if there are no condition sets.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the condition set at the specified index.
    pub fn get(&self, index: u32) -> Option<ConditionSet<'a>> {
        if index >= self.len {
            return None;
        }
        let data = &self.layout.data;
        let record_base = self.base + 8 + index as usize * 8;
        let condition_set = data.read_offset32(record_base, self.base as u32)? as usize;
        let feature_subst = data.read_offset32(record_base + 4, self.base as u32)?;
        let len = data.read_u16(condition_set)?;
        Some(ConditionSet {
            layout: self.layout,
            base: condition_set,
            feature_subst,
            len,
        })
    }

    /// Returns an iterator over the condition sets.
    pub fn iter(&'a self) -> impl Iterator<Item = ConditionSet<'a>> + 'a + Clone {
        (0..self.len).filter_map(move |index| self.get(index))
    }

    /// Returns the first condition set that is satisfied by the specified
    /// normalized variation coordinates.
    pub fn find(&'a self, coords: &[NormalizedCoord]) -> Option<ConditionSet<'a>> {
        for set in self.iter() {
            let mut satisfied = true;
            for condition in set.iter() {
                let coord = coords
                    .get(condition.axis_index as usize)
                    .copied()
                    .unwrap_or(0);
                if coord < condition.min_value || coord > condition.max_value {
                    satisfied = false;
                    break;
                }
            }
            if satisfied {
                return Some(set);
            }
        }
        None
    }
}

/// Set of conditions for selecting a feature substitution.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#conditionset-table>
#[derive(Copy, Clone)]
pub struct ConditionSet<'a> {
    layout: &'a Layout<'a>,
    base: usize,
    feature_subst: u32,
    len: u16,
}

impl<'a> ConditionSet<'a> {
    /// Returns the number of conditions in the set.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the condition at the specified index.
    pub fn get(&self, index: u16) -> Option<Condition> {
        if index >= self.len {
            return None;
        }
        let data = &self.layout.data;
        let offset =
            data.read_offset32(self.base + 2 + index as usize * 4, self.base as u32)? as usize;
        let format = data.read_u16(offset)?;
        if format != 1 {
            return None;
        }
        let axis_index = data.read_u16(offset + 2)?;
        let min_value = data.read_i16(offset + 4)?;
        let max_value = data.read_i16(offset + 6)?;
        Some(Condition {
            axis_index,
            min_value,
            max_value,
        })
    }

    /// Returns an iterator over the conditions in the set.
    pub fn iter(&'a self) -> impl Iterator<Item = Condition> + 'a + Clone {
        (0..self.len).filter_map(move |index| self.get(index))
    }

    /// Returns the associated feature substitutions.
    pub fn features(&self) -> FeatureSubst<'a> {
        let data = &self.layout.data;
        let len = data.read_u16(self.feature_subst as usize + 4).unwrap_or(0);
        FeatureSubst {
            layout: self.layout,
            base: self.feature_subst as usize,
            len,
        }
    }
}

/// Condition for selecting a feature substitution.
///
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#condition-table>
#[derive(Copy, Clone, Debug)]
pub struct Condition {
    /// Index of the axis to which the condition applies.
    pub axis_index: u16,
    /// Minimum value that satisfies the condition.
    pub min_value: NormalizedCoord,
    /// Maximum value that satisfies the condition.
    pub max_value: NormalizedCoord,
}

/// Feature substitution table.
/// 
/// <https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#featuretablesubstitution-table>
#[derive(Copy, Clone)]
pub struct FeatureSubst<'a> {
    layout: &'a Layout<'a>,
    base: usize,
    len: u16,
}

impl<'a> FeatureSubst<'a> {
    /// Returns the number of feature substitutions.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the feature substitution at the specified index.
    pub fn get(&self, index: u16) -> Option<Feature<'a>> {
        if index >= self.len() {
            return None;
        }
        let data = &self.layout.data;
        let subst_base = self.base + 6 + index as usize * 6;
        let feature_index = data.read_u16(subst_base)?;
        let offset = data.read_offset32(subst_base + 2, self.base as u32)?;
        Some(
            FeatureRecord {
                tag: 0,
                index: feature_index,
                offset,
            }
            .materialize(self.layout),
        )
    }

    /// Returns an iterator over the feature substitutions.
    pub fn iter(&'a self) -> impl Iterator<Item = Feature<'a>> + 'a + Clone {
        (0..self.len).filter_map(move |index| self.get(index))
    }

    /// Returns the substitution for the feature at the specified index.
    pub fn find(&self, feature_index: u16) -> Option<Feature<'a>> {
        for i in 0..self.len() {
            if let Some(feature) = self.get(i) {
                if feature.record.index == feature_index {
                    return Some(feature);
                }
                if feature.record.index > feature_index {
                    return None;
                }
            }
        }
        None
    }
}
