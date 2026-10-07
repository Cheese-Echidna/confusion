//! Safe dimensional expressions for driving sketch dimensions and extrusion depth.
//! Exports evaluate; resolves named parameters with cycle detection and returns SI lengths.
//! Supports literals mm/cm/m/in, scalar arithmetic, parentheses and parameter names.
use crate::document::schema::Design;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
#[derive(Clone, Copy)]
struct Value {
    n: f64,
    dimension: [i32; 2],
}
impl Value {
    fn finite(self) -> Result<Self, String> {
        if self.n.is_finite() {
            Ok(self)
        } else {
            Err("Expression produced a nonfinite number".into())
        }
    }
}
fn checked_dimensions(
    left: [i32; 2],
    right: [i32; 2],
    operation: fn(i32, i32) -> Option<i32>,
) -> Result<[i32; 2], String> {
    Ok([
        operation(left[0], right[0]).ok_or("Expression dimension overflow")?,
        operation(left[1], right[1]).ok_or("Expression dimension overflow")?,
    ])
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
        || v.dimension
            != if p.scalar {
                [0, 0]
            } else if p.angular {
                [0, 1]
            } else {
                [1, 0]
            }
        || !v.n.is_finite()
        || v.n.abs() > 1000.
    {
        return Err(format!(
            "{}: expected a finite {}",
            p.name,
            if p.scalar {
                "number"
            } else if p.angular {
                "angle (for example 90 deg)"
            } else {
                "length (for example 80 mm)"
            }
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
            v = v.finite()?;
        }
        Ok(v)
    }
    fn product(&mut self) -> Result<Value, String> {
        let mut v = self.power()?;
        loop {
            let divide = if self.take(b'*') {
                false
            } else if self.take(b'/') {
                true
            } else {
                break;
            };
            let r = self.power()?;
            if divide {
                if r.n == 0. {
                    return Err("Division by zero".into());
                }
                v.n /= r.n;
                v.dimension = checked_dimensions(v.dimension, r.dimension, i32::checked_sub)?;
            } else {
                v.n *= r.n;
                v.dimension = checked_dimensions(v.dimension, r.dimension, i32::checked_add)?;
            }
            v = v.finite()?;
        }
        Ok(v)
    }
    fn power(&mut self) -> Result<Value, String> {
        let mut value = self.atom()?;
        if self.take(b'^') {
            let exponent = self.atom()?;
            if exponent.dimension != [0, 0] || exponent.n.fract() != 0. || exponent.n.abs() > 16. {
                return Err("Exponent must be a bounded integer scalar".into());
            }
            value.n = value.n.powf(exponent.n);
            value.dimension =
                checked_dimensions(value.dimension, [exponent.n as i32; 2], i32::checked_mul)?;
            value = value.finite()?;
        }
        Ok(value)
    }
    fn atom(&mut self) -> Result<Value, String> {
        if self.depth >= 64 {
            return Err("Expression nesting exceeds 64".into());
        }
        self.depth += 1;
        let result = self.atom_inner();
        self.depth -= 1;
        result.and_then(Value::finite)
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
            if !n.is_finite() {
                return Err("Nonfinite number literal".into());
            }
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
                "" => {
                    return Ok(Value {
                        n,
                        dimension: [0, 0],
                    });
                }
                "mm" => 0.001,
                "cm" => 0.01,
                "m" => 1.,
                "deg" => {
                    return Ok(Value {
                        n: n.to_radians(),
                        dimension: [0, 1],
                    });
                }
                "rad" => {
                    return Ok(Value {
                        n,
                        dimension: [0, 1],
                    });
                }
                "in" => 0.0254,
                "ft" => 0.3048,
                "um" => 1e-6,
                _ => return Err(format!("Unknown unit {unit}")),
            };
            return Ok(Value {
                n: n * factor,
                dimension: [1, 0],
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
        if self.take(b'(') {
            let value = self.sum()?;
            let second = if self.take(b',') {
                Some(self.sum()?)
            } else {
                None
            };
            if !self.take(b')') {
                return Err("Missing function closing parenthesis".into());
            }
            if let Some(second) = second {
                if value.dimension != second.dimension {
                    return Err("Function arguments need matching units".into());
                }
                return match name {
                    "min" => Ok(Value {
                        n: value.n.min(second.n),
                        dimension: value.dimension,
                    }),
                    "max" => Ok(Value {
                        n: value.n.max(second.n),
                        dimension: value.dimension,
                    }),
                    _ => Err(format!("Unknown two-argument function {name}")),
                };
            }
            return match name {
                "sqrt" if value.dimension.iter().all(|p| p % 2 == 0) && value.n >= 0. => {
                    Ok(Value {
                        n: value.n.sqrt(),
                        dimension: value.dimension.map(|p| p / 2),
                    })
                }
                "abs" => Ok(Value {
                    n: value.n.abs(),
                    dimension: value.dimension,
                }),
                "sin" | "cos" | "tan" if value.dimension == [0, 1] || value.dimension == [0, 0] => {
                    Ok(Value {
                        n: match name {
                            "sin" => value.n.sin(),
                            "cos" => value.n.cos(),
                            _ => value.n.tan(),
                        },
                        dimension: [0, 0],
                    })
                }
                "asin" | "acos" | "atan" if value.dimension == [0, 0] => Ok(Value {
                    n: match name {
                        "asin" => value.n.asin(),
                        "acos" => value.n.acos(),
                        _ => value.n.atan(),
                    },
                    dimension: [0, 1],
                }),
                _ => Err(format!(
                    "Unsupported function or incompatible units for {name}"
                )),
            };
        }
        if name == "pi" && !self.design.parameters.iter().any(|p| p.name == name) {
            return Ok(Value {
                n: std::f64::consts::PI,
                dimension: [0, 0],
            });
        }
        let id = self
            .design
            .parameters
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("Unknown parameter {name}"))?
            .id;
        Ok(Value {
            n: resolve(id, self.design, self.active, self.cache)?,
            dimension: {
                let p = self.design.parameters.iter().find(|p| p.id == id).unwrap();
                if p.scalar {
                    [0, 0]
                } else if p.angular {
                    [0, 1]
                } else {
                    [1, 0]
                }
            },
        })
    }
}

/// UI defaults numeric input to millimetres/degrees; stored intent always carries units.
/// Named or explicitly unit-bearing expressions retain their dimensional semantics.
pub fn dimension_input(source: &str, angular: bool) -> String {
    if !source.trim().is_empty()
        && source
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_whitespace() || b".+-*/()".contains(&b))
    {
        format!("({source}) * 1 {}", if angular { "deg" } else { "mm" })
    } else {
        source.to_owned()
    }
}
