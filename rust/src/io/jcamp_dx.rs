use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, line_ending, not_line_ending, one_of},
    combinator::{consumed, opt, peek, value},
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, terminated},
    IResult,
};

/// A parser for JCAMP-DX files.
///
/// This parser is based on the JCAMP-DX specification, defined
/// [here](http://www.jcamp-dx.org/protocols/dxir01.pdf) and
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)
/// TODO: stuff
#[derive(Clone, Debug)]
pub struct Parser {}

pub struct JcampDx;

#[derive(Debug, PartialEq, Eq, Clone)]
struct DataLabel(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct Text(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct StringDataSet(String);

#[derive(Debug, PartialEq, Copy, Clone)]
struct AffnFloatDataSet(f64);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct AffnIntDataSet(i64);

#[derive(Debug, PartialEq, Eq, Clone)]
struct AsdfDataSet(String);

enum Value {
    Text(String),
    String(String),
    Float(f64),
    Int(i64),
    IntList(Vec<i64>),
    FloatList(Vec<f64>),
}

fn data_label_name(input: &str) -> IResult<&str, String> {
    let (remaining, (_, output)) =
        consumed(many1(terminated(alphanumeric1, many0(one_of(" -/\\_")))))(input)?;
    Ok((remaining, output.join("").to_uppercase()))
}

fn data_label(input: &str) -> IResult<&str, DataLabel> {
    let (remaining, output) =
        delimited(pair(tag("##"), opt(tag("."))), data_label_name, tag("="))(input)?;
    Ok((remaining, DataLabel(output)))
}

fn inline_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("$$"), not_line_ending))(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many_till(value((), anychar), peek(value((), data_label))),
    )(input)
}

fn text_data_set(input: &str) -> IResult<&str, Text> {
    let (remaining, (output, _)) = many_till(anychar, peek(line_ending))(input)?;
    Ok((remaining, Text(String::from_iter(output))))
}

fn string_data_set(input: &str) -> IResult<&str, StringDataSet> {
    let (remaining, output) = alphanumeric1(input)?;
    Ok((remaining, StringDataSet(output.into())))
}

fn affn_float_data_set(input: &str) -> IResult<&str, AffnFloatDataSet> {
    todo!()
}

// fn affn_int_data_set(input: &str) -> IResult<&str, AffnIntDataSet> {
//     pair(one_of("+-."), digit1)
// }

fn asdf_data_set(input: &str) -> IResult<&str, AsdfDataSet> {
    todo!()
}

impl Parser {
    fn new() -> Self {
        Self {}
    }

    fn parse(input: &str) -> HashMap<String, Value> {
        todo!()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_label() {
        let (remaining, output) = data_label("##O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "SOLID_ANODE");
        assert_eq!(output, DataLabel("OBSERVATION232TYPE".into()));

        let (remaining, output) = data_label("##.O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "SOLID_ANODE");
        assert_eq!(output, DataLabel("OBSERVATION232TYPE".into()));
    }

    #[test]
    fn test_text_data_set() {
        let (remaining, output) = text_data_set("asd\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(output, Text("asd".into()));
    }

    #[test]
    fn test_string_data_set() {
        let (remaining, output) = string_data_set("asd\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(output, StringDataSet("asd".into()));
    }

    #[test]
    fn test_inline_comment() {
        let (remaining, _) = inline_comment("$$SOME COMMENT\n").unwrap();
        assert_eq!(remaining, "\n");
    }

    #[test]
    fn test_multiline_comment() {
        let (remaining, _) = multi_line_comment(
            "##=this is a commnt
            comment
            comment
            ##TITLE= ",
        )
        .unwrap();
        assert_eq!(remaining, "##TITLE= ")
    }
}
