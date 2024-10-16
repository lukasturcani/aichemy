use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{alphanumeric1, one_of},
    combinator::{consumed, recognize, value},
    multi::{many0, many1},
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
struct UntypedDataLabel(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct TypedDataLabel(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct TextDataSet(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct StringDataSet(String);

#[derive(Debug, PartialEq, Copy, Clone)]
struct AffnFloatDataSet(f64);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct AffnIntDataSet(i64);

#[derive(Debug, PartialEq, Eq, Clone)]
struct AsdfDataSet(String);

fn data_label_name(input: &str) -> IResult<&str, String> {
    let (remaining, (_, output)) =
        consumed(many1(terminated(alphanumeric1, many0(one_of(" -/\\_")))))(input)?;
    Ok((remaining, output.join("")))
}

fn untyped_data_label(input: &str) -> IResult<&str, UntypedDataLabel> {
    let (remaining, output) = delimited(tag("##"), data_label_name, tag("="))(input)?;
    Ok((remaining, UntypedDataLabel(output.to_uppercase())))
}

fn typed_data_label(input: &str) -> IResult<&str, TypedDataLabel> {
    let (remaining, output) = delimited(tag("##."), data_label_name, tag("="))(input)?;
    Ok((remaining, TypedDataLabel(output)))
}

fn inline_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("$$"), is_not("\n\r")))(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(
            tag("##="),
            take_until(alt((
                value((), typed_data_label),
                value((), untyped_data_label),
            ))),
        ),
    )(input)
}

fn text_data_set(input: &str) -> IResult<&str, TextDataSet> {
    todo!()
}

fn string_data_set(input: &str) -> IResult<&str, StringDataSet> {
    todo!()
}

fn affn_float_data_set(input: &str) -> IResult<&str, AffnFloatDataSet> {
    todo!()
}

fn affn_int_data_set(input: &str) -> IResult<&str, AffnIntDataSet> {
    todo!()
}

fn asdf_data_set(input: &str) -> IResult<&str, AsdfDataSet> {
    todo!()
}

impl Parser {
    fn new() -> Self {
        Self {}
    }

    fn parse(input: &str) -> JcampDx {
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
    fn test_untyped_data_label() {
        let (remaining, output) =
            untyped_data_label("##O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "SOLID_ANODE");
        assert_eq!(output, UntypedDataLabel("OBSERVATION232TYPE".into()));
    }

    #[test]
    fn test_typed_data_label() {
        let (remaining, output) =
            typed_data_label("##O-B  SER\\va/TiON232_TYPE=SOLID_ANODE").unwrap();
        assert_eq!(remaining, "");
        assert_eq!(output, TypedDataLabel("OBSERVATION232TYPE".into()));
    }

    #[test]
    fn test_inline_comment() {
        let (remaining, _) = inline_comment("$$SOME COMMENT\n").unwrap();
        assert_eq!(remaining, "\n");
    }
}
