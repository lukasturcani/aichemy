use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, line_ending, not_line_ending, one_of, space0},
    combinator::{consumed, opt, peek, value},
    multi::{many0, many1, many_till},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
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

#[derive(Debug, PartialEq, Clone)]
enum Value {
    Text(String),
    Number(f64),
}

fn data_label_name(input: &str) -> IResult<&str, String> {
    let (remaining, (dollar, (_, label_name))) = pair(
        opt(tag("$")),
        consumed(many1(terminated(alphanumeric1, many0(one_of(" -/\\_"))))),
    )(input)?;
    Ok((
        remaining,
        format!(
            "{}{}",
            dollar.unwrap_or(""),
            label_name.join("").to_uppercase()
        ),
    ))
}

fn data_label(input: &str) -> IResult<&str, String> {
    delimited(pair(tag("##"), opt(tag("."))), data_label_name, tag("="))(input)
}

fn labeled_data_record(input: &str) -> IResult<&str, (String, Value)> {
    let (remaining, (label, value)) = separated_pair(
        data_label,
        space0,
        alt((
            affn_number_data_set,
            multi_line_text_data_set,
            text_data_set,
        )),
    )(input)?;
    Ok((remaining, (label, value)))
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

fn text_data_set(input: &str) -> IResult<&str, Value> {
    let (remaining, (output, _)) = many_till(anychar, peek(pair(space0, line_ending)))(input)?;
    Ok((remaining, Value::Text(String::from_iter(output))))
}

fn multi_line_text_data_set(input: &str) -> IResult<&str, Value> {
    let (remaining, (output, _)) = preceded(tag("<"), many_till(anychar, tag(">")))(input)?;
    Ok((remaining, Value::Text(String::from_iter(output))))
}

fn affn_number_data_set(input: &str) -> IResult<&str, Value> {
    let (remaning, output) = double(input)?;
    Ok((remaning, Value::Number(output)))
}

// fn asdf_data_set_value(input: &str) -> IResult<&str, Value> {
//     todo!()
// }

// fn asdf_data_set(input: &str) -> IResult<&str, Value> {
//     let (remaining, output) = separated_pair(
//         tag("X++(Y..Y)"),
//         pair(space0, line_ending),
//         asdf_data_set_value,
//     )?;
//     todo!()
// }

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
        assert_eq!(output, "OBSERVATION232TYPE".to_string());

        let (remaining, output) = data_label("##.O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "SOLID_ANODE");
        assert_eq!(output, "OBSERVATION232TYPE".to_string());

        let (remaining, output) = data_label("##$O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "SOLID_ANODE");
        assert_eq!(output, "$OBSERVATION232TYPE".to_string());
    }

    #[test]
    fn test_text_data_set() {
        let (remaining, output) = text_data_set("asd\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(output, Value::Text("asd".into()));
    }

    #[test]
    fn test_multi_line_text_data_set() {
        let (remaining, output) = multi_line_text_data_set("<asd\n  asd>  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(output, Value::Text("asd\n  asd".into()));
    }

    #[test]
    fn test_labeled_data_record() {
        let (remaining, (label, value)) =
            labeled_data_record("##.OBSERVATION232TYPE= SOLID_ANODE\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(label, "OBSERVATION232TYPE");
        assert_eq!(value, Value::Text("SOLID_ANODE".into()));

        let (remaining, (label, value)) =
            labeled_data_record("##OBSERVATION232TYPE=SOLID_ANODE\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(label, "OBSERVATION232TYPE");
        assert_eq!(value, Value::Text("SOLID_ANODE".into()));

        let (remaining, (label, value)) =
            labeled_data_record("##$OBSERVATION232TYPE=     SOLID_ANODE     \n").unwrap();
        assert_eq!(remaining, "     \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Text("SOLID_ANODE".into()));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= 123e32  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Number(123e32));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= 123.32  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Number(123.32));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= .32  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Number(0.32));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= 32  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Number(32.));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= -32  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Number(-32.));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= <hello world>  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Text("hello world".into()));

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= <hello\n  world>  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Text("hello\n  world".into()));
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
