use super::color::Command;
use super::geometry::*;
use core::ops::Range;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Verb {
    MoveTo,
    LineTo,
    QuadTo,
    CurveTo,
    Close,
}

#[derive(Copy, Clone, Debug)]
pub enum Element {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CurveTo(Point, Point, Point),
    Close,
}

#[derive(Clone, Default, Debug)]
pub struct Glyph {
    points: Vec<Point>,
    verbs: Vec<Verb>,
    paths: Vec<PathData>,
    commands: Vec<Command>,
}

impl Glyph {
    pub fn num_paths(&self) -> usize {
        self.paths.len()
    }

    pub fn path(&self, index: usize) -> Option<Path> {
        let data = self.paths.get(index)?;
        Some(Path {
            points: self.points.get(data.points.clone())?,
            verbs: self.verbs.get(data.verbs.clone())?,
            bounds: &data.bounds,
        })
    }

    pub fn is_simple(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.verbs.clear();
        self.paths.clear();
        self.commands.clear();
    }
}

impl Glyph {
    pub(crate) fn push_command(&mut self, command: Command) {
        self.commands.push(command);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Path<'a> {
    pub points: &'a [Point],
    pub verbs: &'a [Verb],
    pub bounds: &'a Bounds,
}

impl<'a> Path<'a> {
    pub fn elements(&self) -> impl Iterator<Item = Element> + 'a + Clone {
        let mut i = 0;
        let copy = *self;
        copy.verbs.iter().map(move |verb| match verb {
            Verb::MoveTo => {
                let p0 = copy.points[i];
                i += 1;
                Element::MoveTo(p0)
            }
            Verb::LineTo => {
                let p0 = copy.points[i];
                i += 1;
                Element::LineTo(p0)
            }
            Verb::QuadTo => {
                let p0 = copy.points[i];
                let p1 = copy.points[i + 1];
                i += 2;
                Element::QuadTo(p0, p1)
            }
            Verb::CurveTo => {
                let p0 = copy.points[i];
                let p1 = copy.points[i + 1];
                let p2 = copy.points[i + 2];
                i += 3;
                Element::CurveTo(p0, p1, p2)
            }
            Verb::Close => Element::Close,
        })
    }
}

#[derive(Clone, Default, Debug)]
struct PathData {
    points: Range<usize>,
    verbs: Range<usize>,
    bounds: Bounds,
}

pub(super) struct PathBuilder<'a> {
    inner: &'a mut Glyph,
    path: PathData,
}

impl<'a> PathBuilder<'a> {
    pub fn new(inner: &'a mut Glyph) -> Self {
        let path = PathData {
            points: inner.points.len()..inner.points.len(),
            verbs: inner.verbs.len()..inner.verbs.len(),
            bounds: Bounds::default(),
        };
        Self { inner, path }
    }

    pub fn move_to(&mut self, p: Point) {
        self.maybe_close();
        self.inner.points.push(p);
        self.inner.verbs.push(Verb::MoveTo);
    }

    pub fn line_to(&mut self, p: Point) {
        self.inner.points.push(p);
        self.inner.verbs.push(Verb::LineTo);
    }

    pub fn quad_to(&mut self, p0: Point, p1: Point) {
        self.inner.points.push(p0);
        self.inner.points.push(p1);
        self.inner.verbs.push(Verb::QuadTo);
    }

    pub fn curve_to(&mut self, p0: Point, p1: Point, p2: Point) {
        self.inner.points.push(p0);
        self.inner.points.push(p1);
        self.inner.points.push(p2);
        self.inner.verbs.push(Verb::CurveTo);
    }

    pub fn close(&mut self) {
        self.inner.verbs.push(Verb::Close);
    }

    pub fn maybe_close(&mut self) {
        if self.inner.verbs.len() > self.path.verbs.start
            && self.inner.verbs.last() != Some(&Verb::Close)
        {
            self.close();
        }
    }

    pub fn finish(mut self) {
        self.path.points.end = self.inner.points.len();
        self.path.verbs.end = self.inner.verbs.len();
        self.path.bounds = Bounds::from_points(&self.inner.points[self.path.points.clone()]);
        self.inner.paths.push(self.path);
    }
}
