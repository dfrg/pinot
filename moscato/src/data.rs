use super::truetype;

use pinot::avar::{Avar, AVAR};
use pinot::colr::{Colr, COLR};
use pinot::cpal::{Cpal, CPAL};
use pinot::fvar::{Fvar, FVAR};
use pinot::{FontRef, TableProvider};

/// General font information necessary for scaling.
#[derive(Copy, Clone, Default)]
pub struct FontInfo {
    pub upem: u16,
    pub glyph_count: u16,
    pub axis_count: u16,
    pub loca_fmt: u8,
    pub hmetric_count: u16,
}

impl FontInfo {
    /// Creates new configuration data from the specified table provider.
    pub fn new<'a>(provider: &impl TableProvider<'a>) -> Self {
        let (upem, loca_fmt) = provider
            .head()
            .map(|head| (head.units_per_em(), head.index_to_location_format() as u8))
            .unwrap_or((1, 0));
        let glyph_count = provider
            .maxp()
            .map(|maxp| maxp.num_glyphs())
            .unwrap_or_default();
        let axis_count = provider
            .fvar()
            .map(|fvar| fvar.num_axes())
            .unwrap_or_default();
        let hmetric_count = provider
            .hhea()
            .map(|hhea| hhea.num_long_metrics())
            .unwrap_or_default();
        Self {
            upem,
            glyph_count,
            loca_fmt,
            axis_count,
            hmetric_count,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Cached {
    pub simple: SimpleCached,
    pub color: Option<ColorCached>,
    pub var: Option<VarCached>,
    pub info: FontInfo,
}

impl Cached {
    pub fn new(font: &FontRef) -> Self {
        let simple = match truetype::data::Cached::new(font) {
            Some(cached) => SimpleCached::TrueType(cached),
            _ => SimpleCached::None,
        };
        let color = match (font.find_record(COLR), font.find_record(CPAL)) {
            (Some(colr), Some(cpal)) => Some(ColorCached {
                colr: colr.offset,
                cpal: cpal.offset,
            }),
            _ => None,
        };
        let var = if let Some(fvar) = font.find_record(FVAR) {
            let avar = font.find_record(AVAR).map(|r| r.offset).unwrap_or(0);
            Some(VarCached {
                fvar: fvar.offset,
                avar,
            })
        } else {
            None
        };
        let info = FontInfo::new(font);
        Self {
            simple,
            color,
            var,
            info,
        }
    }
}

#[derive(Copy, Clone)]
pub enum SimpleCached {
    None,
    TrueType(truetype::data::Cached),
}

#[derive(Copy, Clone)]
pub struct ColorCached {
    pub colr: u32,
    pub cpal: u32,
}

#[derive(Copy, Clone, Default)]
pub struct VarCached {
    fvar: u32,
    avar: u32,
}

#[derive(Copy, Clone)]
pub struct Data<'a> {
    pub simple: SimpleData<'a>,
    pub color: Option<ColorData<'a>>,
    pub var: Option<VarData<'a>>,
    pub info: FontInfo,
}

impl<'a> Data<'a> {
    pub fn from_cached(font: &FontRef<'a>, cached: &Cached) -> Option<Self> {
        let simple = match cached.simple {
            SimpleCached::TrueType(x) => {
                SimpleData::TrueType(truetype::data::Data::from_cached(font, &x, cached.info)?)
            }
            _ => SimpleData::None,
        };
        let color = cached.color.map(|color| ColorData {
            colr: Colr::new(font.data.get(color.colr as usize..).unwrap()),
            cpal: Cpal::new(font.data.get(color.cpal as usize..).unwrap()),
        });
        let var = cached.var.map(|var| VarData {
            fvar: Fvar::new(font.data.get(var.fvar as usize..).unwrap()),
            avar: if var.avar != 0 {
                Some(Avar::new(font.data.get(var.avar as usize..).unwrap()))
            } else {
                None
            },
        });
        Some(Self {
            simple,
            color,
            var,
            info: cached.info,
        })
    }

    pub fn from_table_provider(provider: &impl TableProvider<'a>) -> Option<Self> {
        let info = FontInfo::new(provider);
        let simple = if let Some(data) = truetype::data::Data::from_table_provider(provider, info) {
            SimpleData::TrueType(data)
        } else {
            SimpleData::None
        };
        let color = match (provider.colr(), provider.cpal()) {
            (Some(colr), Some(cpal)) => Some(ColorData { colr, cpal }),
            _ => None,
        };
        let var = if let Some(fvar) = provider.fvar() {
            Some(VarData {
                fvar,
                avar: provider.avar(),
            })
        } else {
            None
        };
        Some(Self {
            simple,
            color,
            var,
            info,
        })
    }
}

impl Default for Data<'_> {
    fn default() -> Self {
        Self {
            simple: SimpleData::None,
            color: None,
            var: None,
            info: FontInfo {
                upem: 1,
                ..Default::default()
            },
        }
    }
}

#[derive(Copy, Clone)]
pub enum SimpleData<'a> {
    None,
    TrueType(truetype::data::Data<'a>),
}

#[derive(Copy, Clone)]
pub struct ColorData<'a> {
    pub colr: Colr<'a>,
    pub cpal: Cpal<'a>,
}

#[derive(Copy, Clone)]
pub struct VarData<'a> {
    pub fvar: Fvar<'a>,
    pub avar: Option<Avar<'a>>,
}
