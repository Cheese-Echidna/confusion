//! Safe dimensional expressions for driving sketch dimensions and extrusion depth.
//! Exports evaluate; resolves named parameters with cycle detection and returns SI lengths.
//! Supports literals mm/cm/m/in, scalar arithmetic, parentheses and parameter names.
use crate::document::schema::Design;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
#[derive(Clone, Copy)]
struct Value {
    n: f64,
    dimension: i32,
}
struct Parser<'a> {
    source: &'a [u8],
    offset: usize,
    design: &'a Design,
    active: &'a mut HashSet<Uuid>,
    cache: &'a mut HashMap<Uuid, f64>,
    depth: usize,
}
pub fn evaluate(design: &Design) -> Result<HashMap<Uuid, f64>, String> {
    design.validate()?;
    let mut cache = HashMap::new();
    let mut active = HashSet::new();
    for p in &design.parameters {
        resolve(p.id, design, &mut active, &mut cache)?;
    }
    Ok(cache)
}
fn resolve(
    id: Uuid,
    d: &Design,
    active: &mut HashSet<Uuid>,
    cache: &mut HashMap<Uuid, f64>,
) -> Result<f64, String> {
    if let Some(v) = cache.get(&id) {
        return Ok(*v);
    }
    if active.len() >= 32 {
        return Err("Parameter dependency nesting exceeds 32".into());
    }
    if !active.insert(id) {
        return Err("Cyclic parameter expression".into());
    }
    let p = d
        .parameters
        .iter()
        .find(|p| p.id == id)
        .ok_or("Unknown parameter")?;
    let mut parser = Parser {
        source: p.expression.as_bytes(),
        offset: 0,
        design: d,
        active,
        cache,
        depth: 0,
    };
    let v = parser.sum()?;
    parser.space();
    if parser.offset != parser.source.len()
        || v.dimension != 1
        || !v.n.is_finite()
        || v.n.abs() > 1000.
    {
        return Err(format!(
            "{}: expected a finite length (for example 80 mm)",
            p.name
        ));
    }
    active.remove(&id);
    cache.insert(id, v.n);
    Ok(v.n)
}
impl Parser<'_> {
    fn space(&mut self) {
        while self
            .source
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
    }
    fn take(&mut self, b: u8) -> bool {
        self.space();
        if self.source.get(self.offset) == Some(&b) {
            self.offset += 1;
            true
        } else {
            false
        }
    }
    fn sum(&mut self) -> Result<Value, String> {
        let mut v = self.product()?;
        loop {
            let sign = if self.take(b'+') {
                1.
            } else if self.take(b'-') {
                -1.
            } else {
                break;
            };
            let r = self.product()?;
            if v.dimension != r.dimension {
                return Err("Cannot add lengths and scalars".into());
            }
            v.n += sign * r.n;
        }
        Ok(v)
    }
    fn product(&mut self) -> Result<Value, String> {
        let mut v = self.atom()?;
        loop {
            let divide = if self.take(b'*') {
                false
            } else if self.take(b'/') {
                true
            } else {
                break;
            };
            let r = self.atom()?;
            if divide {
                v.n /= r.n;
                v.dimension -= r.dimension;
            } else {
                v.n *= r.n;
                v.dimension += r.dimension;
            }
        }
        Ok(v)
    }
    fn atom(&mut self) -> Result<Value, String> {
        if self.depth >= 64 {
            return Err("Expression nesting exceeds 64".into());
        }
        self.depth += 1;
        let result = self.atom_inner();
        self.depth -= 1;
        result
    }
    fn atom_inner(&mut self) -> Result<Value, String> {
        if self.take(b'-') {
            let mut v = self.atom()?;
            v.n = -v.n;
            return Ok(v);
        }
        if self.take(b'+') {
            return self.atom();
        }
        if self.take(b'(') {
            let v = self.sum()?;
            if !self.take(b')') {
                return Err("Missing closing parenthesis".into());
            }
            return Ok(v);
        }
        self.space();
        let start = self.offset;
        if self
            .source
            .get(start)
            .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
        {
            while self
                .source
                .get(self.offset)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
            {
                self.offset += 1;
            }
            let n = std::str::from_utf8(&self.source[start..self.offset])
                .unwrap()
                .parse::<f64>()
                .map_err(|_| "Invalid number")?;
            self.space();
            let unit_start = self.offset;
            while self
                .source
                .get(self.offset)
                .is_some_and(u8::is_ascii_alphabetic)
            {
                self.offset += 1;
            }
            let unit = std::str::from_utf8(&self.source[unit_start..self.offset]).unwrap();
            let factor = match unit {
                "" => return Ok(Value { n, dimension: 0 }),
                "mm" => 0.001,
                "cm" => 0.01,
                "m" => 1.,
                "in" => 0.0254,
                _ => return Err(format!("Unknown unit {unit}")),
            };
            return Ok(Value {
                n: n * factor,
                dimension: 1,
            });
        }
        while self
            .source
            .get(self.offset)
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        {
            self.offset += 1;
        }
        if self.offset == start {
            return Err("Expected a number or parameter".into());
        }
        let name =
            std::str::from_utf8(&self.source[start..self.offset]).map_err(|_| "Invalid name")?;
        let id = self
            .design
            .parameters
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("Unknown parameter {name}"))?
            .id;
        Ok(Value {
            n: resolve(id, self.design, self.active, self.cache)?,
            dimension: 1,
        })
    }
}
