use std::collections::BTreeMap;

use crate::{Element, Result, UfoError};

#[derive(Clone, Debug, PartialEq)]
pub enum LineItem {
    Element(String),
    Line(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub label: String,
    pub items: Vec<LineItem>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurveyPoint {
    pub position: [f64; 2],
    pub alpha: f64,
}

impl Line {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
        }
    }

    pub fn flatten<'a>(&'a self, lattice: &'a Lattice) -> Result<Vec<&'a Element>> {
        let mut out = Vec::new();
        self.flatten_into(lattice, &mut out)?;
        Ok(out)
    }

    pub fn find<F>(&self, lattice: &Lattice, mut predicate: F) -> Result<Vec<usize>>
    where
        F: FnMut(&Element) -> bool,
    {
        Ok(self
            .flatten(lattice)?
            .into_iter()
            .enumerate()
            .filter_map(|(idx, element)| predicate(element).then_some(idx))
            .collect())
    }

    pub fn locate(&self, lattice: &Lattice, element: f64) -> Result<Option<f64>> {
        let flat = self.flatten(lattice)?;
        if element == -1.0 {
            return Ok(Some(flat.iter().map(|e| e.length()).sum()));
        }
        if element < 0.0 {
            return Ok(None);
        }

        let target = element.trunc() as usize;
        let fraction = element.fract();
        let mut s = 0.0;
        for (idx, item) in flat.iter().enumerate() {
            if idx == target {
                return Ok(Some(s + fraction * item.length()));
            }
            s += item.length();
        }
        Ok(None)
    }

    fn flatten_into<'a>(&'a self, lattice: &'a Lattice, out: &mut Vec<&'a Element>) -> Result<()> {
        for item in &self.items {
            match item {
                LineItem::Element(label) => out.push(
                    lattice
                        .elements
                        .get(label)
                        .ok_or_else(|| UfoError::UnknownReference(label.clone()))?,
                ),
                LineItem::Line(label) => lattice
                    .lines
                    .get(label)
                    .ok_or_else(|| UfoError::UnknownReference(label.clone()))?
                    .flatten_into(lattice, out)?,
            }
        }
        Ok(())
    }

    pub fn length(&self, lattice: &Lattice) -> Result<f64> {
        Ok(self.flatten(lattice)?.iter().map(|e| e.length()).sum())
    }

    pub fn angle(&self, lattice: &Lattice) -> Result<f64> {
        Ok(self.flatten(lattice)?.iter().map(|e| e.angle()).sum())
    }

    pub fn count(&self, lattice: &Lattice) -> Result<usize> {
        Ok(self.flatten(lattice)?.len())
    }

    pub fn survey(
        &self,
        lattice: &Lattice,
        position: [f64; 2],
        alpha: f64,
    ) -> Result<Vec<SurveyPoint>> {
        let mut position = position;
        let mut alpha = alpha;
        let mut out = Vec::new();
        for element in self.flatten(lattice)? {
            (position, alpha) = element.survey_step(position, alpha);
            out.push(SurveyPoint { position, alpha });
        }
        Ok(out)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lattice {
    pub elements: BTreeMap<String, Element>,
    pub lines: BTreeMap<String, Line>,
}

impl Lattice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_element(&mut self, element: Element) {
        self.elements.insert(element.label().to_string(), element);
    }

    pub fn insert_line(&mut self, line: Line) {
        self.lines.insert(line.label.clone(), line);
    }

    pub fn line(&self, label: &str) -> Result<&Line> {
        self.lines
            .get(label)
            .ok_or_else(|| UfoError::UnknownReference(label.to_string()))
    }
}
