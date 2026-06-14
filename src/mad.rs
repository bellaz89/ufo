use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::{
    Aperture, Cavity, Drift, Element, Lattice, Line, LineItem, Marker, Multipole, Octupole,
    Quadrupole, Rbend, Result, Sbend, Sextupole, UfoError, Wire,
};

#[derive(Parser)]
#[grammar = "mad.pest"]
struct MadParser;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    Vector(Vec<f64>),
    String(String),
}

pub fn load_mad_file(path: impl AsRef<Path>) -> Result<Lattice> {
    let path = path.as_ref();
    let input = load_mad_source(path, &mut Vec::new())?;
    parse_mad(&input)
}

fn load_mad_source(path: &Path, stack: &mut Vec<PathBuf>) -> Result<String> {
    let input = fs::read_to_string(path).map_err(|source| UfoError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let canonical = path.canonicalize().map_err(|source| UfoError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    if stack.contains(&canonical) {
        return Err(UfoError::Parse(format!(
            "recursive CALL include `{}`",
            path.display()
        )));
    }
    stack.push(canonical);

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut expanded = String::new();
    let input = strip_mad_comments(&input);
    for statement in split_statements(&input) {
        if let Some(include) = call_file(statement) {
            let include_path = base.join(include);
            expanded.push_str(&load_mad_source(&include_path, stack)?);
            expanded.push('\n');
        } else {
            expanded.push_str(statement);
            expanded.push_str(";\n");
        }
    }

    stack.pop();
    Ok(expanded)
}

pub fn parse_mad(input: &str) -> Result<Lattice> {
    let mut lattice = Lattice::new();
    let parsed = MadParser::parse(Rule::file, input).map_err(|e| UfoError::Parse(e.to_string()))?;
    for pair in parsed {
        parse_pair(pair, &mut lattice)?;
    }
    Ok(lattice)
}

fn split_statements(input: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut in_string = false;

    for (idx, ch) in input.char_indices() {
        if ch == '"' {
            in_string = !in_string;
        } else if ch == ';' && !in_string {
            let statement = input[start..idx].trim();
            if !statement.is_empty() {
                statements.push(statement);
            }
            start = idx + ch.len_utf8();
        }
    }

    let trailing = input[start..].trim();
    if !trailing.is_empty() {
        statements.push(trailing);
    }
    statements
}

fn strip_mad_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut in_comment = false;

    for ch in input.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
                out.push(ch);
            }
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
        } else if ch == '!' && !in_string {
            in_comment = true;
        } else {
            out.push(ch);
        }
    }

    out
}

fn call_file(statement: &str) -> Option<&str> {
    let mut parts = statement.splitn(2, ',');
    let command = parts.next()?.trim();
    if !command.eq_ignore_ascii_case("CALL") {
        return None;
    }

    for param in parts.next()?.split(',') {
        let mut key_value = param.splitn(2, '=');
        let key = key_value.next()?.trim();
        let value = key_value.next()?.trim();
        if key.eq_ignore_ascii_case("FILE") {
            return value.strip_prefix('"')?.strip_suffix('"');
        }
    }
    None
}

fn parse_pair(pair: Pair<'_, Rule>, lattice: &mut Lattice) -> Result<()> {
    if pair.as_rule() == Rule::definition {
        parse_definition(pair, lattice)?;
        return Ok(());
    }
    for child in pair.into_inner() {
        parse_pair(child, lattice)?;
    }
    Ok(())
}

fn parse_definition(pair: Pair<'_, Rule>, lattice: &mut Lattice) -> Result<()> {
    let mut inner = pair.into_inner();
    let label = inner.next().unwrap().as_str().to_string();
    let kind = inner.next().unwrap().as_str().to_uppercase();
    let body = inner.next();

    if kind == "LINE" {
        let mut line = Line::new(label);
        if let Some(body) = body {
            for item in parse_line_body(body, lattice)? {
                line.items.push(item);
            }
        }
        lattice.insert_line(line);
        return Ok(());
    }

    let params = body.map(parse_params).transpose()?.unwrap_or_default();
    let element = build_element(&label, &kind, &params)?;
    lattice.insert_element(element);
    Ok(())
}

fn parse_line_body(pair: Pair<'_, Rule>, lattice: &Lattice) -> Result<Vec<LineItem>> {
    let mut out = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() != Rule::ident_list {
            continue;
        }
        for ident in child.into_inner() {
            let label = ident.as_str().to_string();
            if lattice.elements.contains_key(&label) {
                out.push(LineItem::Element(label));
            } else if lattice.lines.contains_key(&label) {
                out.push(LineItem::Line(label));
            } else {
                return Err(UfoError::UnknownReference(label));
            }
        }
    }
    Ok(out)
}

fn parse_params(pair: Pair<'_, Rule>) -> Result<BTreeMap<String, Value>> {
    let mut params = BTreeMap::new();
    if pair.as_rule() == Rule::param_list {
        parse_param_list(pair, &mut params)?;
        return Ok(params);
    }
    for child in pair.into_inner() {
        if child.as_rule() == Rule::param_list {
            parse_param_list(child, &mut params)?;
        }
    }
    Ok(params)
}

fn parse_param_list(pair: Pair<'_, Rule>, params: &mut BTreeMap<String, Value>) -> Result<()> {
    for param in pair.into_inner() {
        let mut p = param.into_inner();
        let name = p.next().unwrap().as_str().to_uppercase();
        let value = parse_value(p.next().unwrap())?;
        params.insert(name, value);
    }
    Ok(())
}

fn parse_value(pair: Pair<'_, Rule>) -> Result<Value> {
    let child = pair.into_inner().next().unwrap();
    match child.as_rule() {
        Rule::number => parse_number(child.as_str()).map(Value::Number),
        Rule::vector => {
            let mut values = Vec::new();
            for list in child.into_inner() {
                for number in list.into_inner() {
                    values.push(number.as_str().parse().map_err(|_| {
                        UfoError::Parse(format!("invalid number `{}`", number.as_str()))
                    })?);
                }
            }
            Ok(Value::Vector(values))
        }
        Rule::string => Ok(Value::String(child.as_str().trim_matches('"').to_string())),
        _ => Err(UfoError::Parse(format!(
            "unexpected value token {:?}",
            child.as_rule()
        ))),
    }
}

fn parse_number(input: &str) -> Result<f64> {
    input
        .replace(['d', 'D'], "e")
        .parse()
        .map_err(|_| UfoError::Parse(format!("invalid number `{input}`")))
}

fn number(params: &BTreeMap<String, Value>, kind: &str, name: &str, default: f64) -> Result<f64> {
    match params.get(name) {
        None => Ok(default),
        Some(Value::Number(v)) => Ok(*v),
        Some(_) => Err(UfoError::ExpectedNumber {
            kind: kind.to_string(),
            parameter: name.to_string(),
        }),
    }
}

fn vector(params: &BTreeMap<String, Value>, kind: &str, name: &str) -> Result<Vec<f64>> {
    match params.get(name) {
        None => Ok(Vec::new()),
        Some(Value::Vector(v)) => Ok(v.clone()),
        Some(_) => Err(UfoError::ExpectedVector {
            kind: kind.to_string(),
            parameter: name.to_string(),
        }),
    }
}

fn field_errors(params: &BTreeMap<String, Value>, kind: &str) -> Result<(Vec<f64>, Vec<f64>)> {
    let mut dkn = vector(params, kind, "DKN")?;
    let mut dks = vector(params, kind, "DKS")?;

    for (name, value) in params {
        if let Some(order) = name
            .strip_prefix("DK")
            .and_then(|s| s.parse::<usize>().ok())
        {
            set_field_error(&mut dkn, order, numeric_value(value, kind, name)?);
        } else if let Some(order) = name
            .strip_prefix("DK")
            .and_then(|s| s.strip_suffix('S'))
            .and_then(|s| s.parse::<usize>().ok())
        {
            set_field_error(&mut dks, order, numeric_value(value, kind, name)?);
        }
    }

    Ok((dkn, dks))
}

fn set_field_error(values: &mut Vec<f64>, order: usize, value: f64) {
    values.resize(values.len().max(order + 1), 0.0);
    values[order] = value;
}

fn numeric_value(value: &Value, kind: &str, name: &str) -> Result<f64> {
    match value {
        Value::Number(v) => Ok(*v),
        _ => Err(UfoError::ExpectedNumber {
            kind: kind.to_string(),
            parameter: name.to_string(),
        }),
    }
}

fn string(
    params: &BTreeMap<String, Value>,
    kind: &str,
    name: &str,
    default: &str,
) -> Result<String> {
    match params.get(name) {
        None => Ok(default.to_string()),
        Some(Value::String(v)) => Ok(v.clone()),
        Some(Value::Number(v)) => Ok(v.to_string()),
        Some(_) => Err(UfoError::ExpectedString {
            kind: kind.to_string(),
            parameter: name.to_string(),
        }),
    }
}

fn build_element(label: &str, kind: &str, params: &BTreeMap<String, Value>) -> Result<Element> {
    let element = match kind {
        "MARKER" => Element::Marker(Marker {
            label: label.to_string(),
        }),
        "DRIFT" => Element::Drift(Drift {
            label: label.to_string(),
            length: number(params, kind, "L", 0.0)?,
        }),
        "MULTIPOLE" => Element::Multipole(Multipole {
            label: label.to_string(),
            knl: vector(params, kind, "KNL")?,
            ksl: vector(params, kind, "KSL")?,
            dx: number(params, kind, "DX", 0.0)?,
            dy: number(params, kind, "DY", 0.0)?,
        }),
        "QUADRUPOLE" => {
            let mut q = Quadrupole::new(
                label,
                number(params, kind, "L", 0.0)?,
                number(params, kind, "K1", 0.0)?,
            );
            q.dx = number(params, kind, "DX", 0.0)?;
            q.dy = number(params, kind, "DY", 0.0)?;
            (q.dkn, q.dks) = field_errors(params, kind)?;
            Element::Quadrupole(q)
        }
        "SBEND" => {
            let mut b = Sbend::new(
                label,
                number(params, kind, "L", 0.0)?,
                number(params, kind, "ANGLE", 0.0)?,
                number(params, kind, "K1", 0.0)?,
            );
            b.e1 = number(params, kind, "E1", 0.0)?;
            b.e2 = number(params, kind, "E2", 0.0)?;
            b.hgap = number(params, kind, "HGAP", 0.0)?;
            b.fint = number(params, kind, "FINT", 0.0)?;
            b.dx = number(params, kind, "DX", 0.0)?;
            b.dy = number(params, kind, "DY", 0.0)?;
            (b.dkn, b.dks) = field_errors(params, kind)?;
            Element::Sbend(b)
        }
        "RBEND" => {
            let (dkn, dks) = field_errors(params, kind)?;
            Element::Rbend(Rbend {
                label: label.to_string(),
                slices: crate::DEFAULT_BEND_SLICES,
                length: number(params, kind, "L", 0.0)?,
                angle: number(params, kind, "ANGLE", 0.0)?,
                k1: number(params, kind, "K1", 0.0)?,
                e1: number(params, kind, "E1", 0.0)?,
                e2: number(params, kind, "E2", 0.0)?,
                hgap: number(params, kind, "HGAP", 0.0)?,
                fint: number(params, kind, "FINT", 0.0)?,
                dx: number(params, kind, "DX", 0.0)?,
                dy: number(params, kind, "DY", 0.0)?,
                dkn,
                dks,
            })
        }
        "SEXTUPOLE" => {
            let mut s = Sextupole::new(
                label,
                number(params, kind, "L", 0.0)?,
                number(params, kind, "K2", 0.0)?,
                number(params, kind, "K2S", 0.0)?,
            );
            s.dx = number(params, kind, "DX", 0.0)?;
            s.dy = number(params, kind, "DY", 0.0)?;
            (s.dkn, s.dks) = field_errors(params, kind)?;
            Element::Sextupole(s)
        }
        "OCTUPOLE" => {
            let mut o = Octupole::new(
                label,
                number(params, kind, "L", 0.0)?,
                number(params, kind, "K3", 0.0)?,
                number(params, kind, "K3S", 0.0)?,
            );
            o.dx = number(params, kind, "DX", 0.0)?;
            o.dy = number(params, kind, "DY", 0.0)?;
            (o.dkn, o.dks) = field_errors(params, kind)?;
            Element::Octupole(o)
        }
        "WIRE" => Element::Wire(Wire {
            label: label.to_string(),
            x: number(params, kind, "X", 0.0)?,
            y: number(params, kind, "Y", 0.0)?,
            k: number(params, kind, "K", 0.0)?,
        }),
        "CAVITY" => Element::Cavity(Cavity {
            label: label.to_string(),
            field: number(params, kind, "FIELD", 0.0)?,
            omega: number(params, kind, "OMEGA", 0.0)?,
            lag: number(params, kind, "LAG", 0.0)?,
        }),
        "APERTURE" => Element::Aperture(Aperture {
            label: label.to_string(),
            window: string(params, kind, "WINDOW", "0")?,
            radius: match params.get("RADIUS") {
                None => None,
                Some(Value::Number(v)) => Some(*v),
                Some(_) => {
                    return Err(UfoError::ExpectedNumber {
                        kind: kind.to_string(),
                        parameter: "RADIUS".to_string(),
                    });
                }
            },
        }),
        _ => return Err(UfoError::UnknownElementKind(kind.to_string())),
    };
    Ok(element)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_line_dialect() {
        let lattice =
            parse_mad("d1: DRIFT, L=1.5;\nq1: QUADRUPOLE, L=2, K1=-0.3;\nring: LINE=(d1, q1);\n")
                .unwrap();
        assert_eq!(lattice.elements.len(), 2);
        assert_eq!(
            lattice
                .line("ring")
                .unwrap()
                .flatten(&lattice)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn parses_comments_and_fortran_exponents() {
        let lattice = parse_mad(
            r#"
! Full-line comment
d1: DRIFT, L=1.5D+0; ! inline comment
q1: QUADRUPOLE, L=2d0, K1=-3.0D-1;
ring: LINE=(d1, q1);
"#,
        )
        .unwrap();
        let ring = lattice.line("ring").unwrap();
        assert_eq!(ring.length(&lattice).unwrap(), 3.5);
        let q1 = lattice.elements.get("q1").unwrap();
        match q1 {
            Element::Quadrupole(value) => assert_eq!(value.k1, -0.3),
            other => panic!("unexpected element: {other:?}"),
        }
    }
}
