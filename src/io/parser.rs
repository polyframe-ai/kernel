// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2025 Polyframe Inc.

//! OpenSCAD parser using pest

use crate::ast::{Node, NodeKind, TransformOp, Vec3};
use anyhow::{anyhow, Context, Result};
use nalgebra::Vector3;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "io/scad.pest"]
struct ScadParser;

/// Parse OpenSCAD source code into an AST
pub fn parse_scad(source: &str) -> Result<Node> {
    let mut pairs =
        ScadParser::parse(Rule::program, source).context("Failed to parse SCAD source")?;

    let mut statements = Vec::new();

    // Get the program node and iterate over its children
    if let Some(program) = pairs.next() {
        for pair in program.into_inner() {
            match pair.as_rule() {
                Rule::statement => {
                    if let Some(node) = parse_statement(pair)? {
                        statements.push(node);
                    }
                }
                Rule::EOI => {}
                _ => {}
            }
        }
    }

    // If single statement, return it directly
    if statements.len() == 1 {
        Ok(statements.into_iter().next().unwrap())
    } else if statements.is_empty() {
        Ok(Node::new(NodeKind::Empty))
    } else {
        // Multiple statements become a union
        Ok(Node::new(NodeKind::Union(statements)))
    }
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Option<Node>> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| anyhow!("Empty statement"))?;

    match inner.as_rule() {
        Rule::primitive_stmt => parse_primitive(inner),
        Rule::transform_stmt => parse_transform(inner),
        Rule::boolean_stmt => parse_boolean(inner),
        Rule::module_call => Ok(Some(Node::new(NodeKind::Empty))), // Ignore for now
        _ => Ok(None),
    }
}

fn parse_primitive(pair: pest::iterators::Pair<Rule>) -> Result<Option<Node>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::cube_stmt => {
            let params = parse_params(inner)?;
            let size = params
                .get_vector("size")
                .or_else(|| params.get_positional_vector(0))
                .unwrap_or(Vector3::new(1.0, 1.0, 1.0));
            let center = params.get_boolean("center").unwrap_or(false);
            Ok(Some(Node::new(NodeKind::Cube { size, center })))
        }
        Rule::sphere_stmt => {
            let params = parse_params(inner)?;
            let r = params
                .get_number("r")
                .or_else(|| params.get_positional_number(0))
                .unwrap_or(1.0);
            let fn_ = params.get_number("$fn").map(|v| v as u32).unwrap_or(32);
            Ok(Some(Node::new(NodeKind::Sphere { r, fn_ })))
        }
        Rule::cylinder_stmt => {
            let params = parse_params(inner)?;
            let h = params
                .get_number("h")
                .or_else(|| params.get_positional_number(0))
                .unwrap_or(1.0);
            let r = params
                .get_number("r")
                .or_else(|| params.get_positional_number(1))
                .unwrap_or(1.0);
            let fn_ = params.get_number("$fn").map(|v| v as u32).unwrap_or(32);
            Ok(Some(Node::new(NodeKind::Cylinder { h, r, fn_ })))
        }
        _ => Ok(None),
    }
}

fn parse_transform(pair: pest::iterators::Pair<Rule>) -> Result<Option<Node>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::translate_stmt => {
            let mut inner_pairs = inner.into_inner();
            let params = parse_params_from_list(inner_pairs.next().unwrap())?;
            let children = parse_block_or_stmt(inner_pairs.next().unwrap())?;

            let v = params
                .get_vector("v")
                .or_else(|| params.get_positional_vector(0))
                .unwrap_or(Vector3::zeros());

            Ok(Some(Node::new(NodeKind::Transform {
                op: TransformOp::Translate(v),
                children,
            })))
        }
        Rule::rotate_stmt => {
            let mut inner_pairs = inner.into_inner();
            let params = parse_params_from_list(inner_pairs.next().unwrap())?;
            let children = parse_block_or_stmt(inner_pairs.next().unwrap())?;

            let a = params
                .get_vector("a")
                .or_else(|| params.get_positional_vector(0))
                .unwrap_or(Vector3::zeros());

            Ok(Some(Node::new(NodeKind::Transform {
                op: TransformOp::Rotate(a),
                children,
            })))
        }
        Rule::scale_stmt => {
            let mut inner_pairs = inner.into_inner();
            let params = parse_params_from_list(inner_pairs.next().unwrap())?;
            let children = parse_block_or_stmt(inner_pairs.next().unwrap())?;

            let v = params
                .get_vector("v")
                .or_else(|| params.get_positional_vector(0))
                .unwrap_or(Vector3::new(1.0, 1.0, 1.0));

            Ok(Some(Node::new(NodeKind::Transform {
                op: TransformOp::Scale(v),
                children,
            })))
        }
        _ => Ok(None),
    }
}

fn parse_boolean(pair: pest::iterators::Pair<Rule>) -> Result<Option<Node>> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::union_stmt => {
            let block = inner.into_inner().next().unwrap();
            let children = parse_block(block)?;
            Ok(Some(Node::new(NodeKind::Union(children))))
        }
        Rule::difference_stmt => {
            let block = inner.into_inner().next().unwrap();
            let children = parse_block(block)?;
            Ok(Some(Node::new(NodeKind::Difference(children))))
        }
        Rule::intersection_stmt => {
            let block = inner.into_inner().next().unwrap();
            let children = parse_block(block)?;
            Ok(Some(Node::new(NodeKind::Intersection(children))))
        }
        _ => Ok(None),
    }
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Node>> {
    let mut nodes = Vec::new();

    for stmt in pair.into_inner() {
        if let Some(node) = parse_statement(stmt)? {
            nodes.push(node);
        }
    }

    Ok(nodes)
}

fn parse_block_or_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Node>> {
    match pair.as_rule() {
        Rule::block => parse_block(pair),
        Rule::statement => {
            if let Some(node) = parse_statement(pair)? {
                Ok(vec![node])
            } else {
                Ok(vec![])
            }
        }
        _ => Ok(vec![]),
    }
}

// Parameter parsing helpers
struct Params {
    named: std::collections::HashMap<String, Value>,
    positional: Vec<Value>,
}

#[derive(Clone)]
enum Value {
    Number(f32),
    Vector(Vec3),
    #[allow(dead_code)]
    String(String),
    #[allow(dead_code)]
    Boolean(bool),
}

impl Params {
    fn new() -> Self {
        Self {
            named: std::collections::HashMap::new(),
            positional: Vec::new(),
        }
    }

    fn get_number(&self, name: &str) -> Option<f32> {
        self.named.get(name).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
    }

    fn get_vector(&self, name: &str) -> Option<Vec3> {
        self.named.get(name).and_then(|v| match v {
            Value::Vector(v) => Some(*v),
            _ => None,
        })
    }

    fn get_boolean(&self, name: &str) -> Option<bool> {
        self.named.get(name).and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            _ => None,
        })
    }

    fn get_positional_number(&self, idx: usize) -> Option<f32> {
        self.positional.get(idx).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
    }

    fn get_positional_vector(&self, idx: usize) -> Option<Vec3> {
        self.positional.get(idx).and_then(|v| match v {
            Value::Vector(v) => Some(*v),
            _ => None,
        })
    }
}

fn parse_params(pair: pest::iterators::Pair<Rule>) -> Result<Params> {
    for inner in pair.into_inner() {
        if let Rule::param_list = inner.as_rule() {
            return parse_params_from_list(inner);
        }
    }
    Ok(Params::new())
}

fn parse_params_from_list(pair: pest::iterators::Pair<Rule>) -> Result<Params> {
    let mut params = Params::new();

    for param in pair.into_inner() {
        let mut param_inner = param.into_inner();
        let first = param_inner.next().unwrap();

        if let Rule::ident = first.as_rule() {
            // Named parameter
            let name = first.as_str().to_string();
            let expr = param_inner.next().unwrap();
            let value = parse_expr(expr)?;
            params.named.insert(name, value);
        } else {
            // Positional parameter
            let value = parse_expr(first)?;
            params.positional.push(value);
        }
    }

    Ok(params)
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Result<Value> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::number => {
            let num: f32 = inner.as_str().parse()?;
            Ok(Value::Number(num))
        }
        Rule::vector => {
            let mut values = Vec::new();
            if let Some(expr_list) = inner.into_inner().next() {
                for expr in expr_list.into_inner() {
                    if let Value::Number(n) = parse_expr(expr)? {
                        values.push(n);
                    }
                }
            }

            let v = match values.len() {
                1 => Vector3::new(values[0], values[0], values[0]),
                2 => Vector3::new(values[0], values[1], 0.0),
                3 => Vector3::new(values[0], values[1], values[2]),
                _ => Vector3::zeros(),
            };

            Ok(Value::Vector(v))
        }
        Rule::boolean => {
            let b = inner.as_str() == "true";
            Ok(Value::Boolean(b))
        }
        Rule::string => {
            let s = inner.into_inner().next().unwrap().as_str().to_string();
            Ok(Value::String(s))
        }
        _ => Ok(Value::Number(0.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cube() {
        let result = parse_scad("cube([10, 10, 10]);");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_transform() {
        let result = parse_scad("translate([5, 0, 0]) cube([10, 10, 10]);");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_boolean() {
        let result = parse_scad("difference() { cube(10); sphere(8); }");
        assert!(result.is_ok());
    }
}
