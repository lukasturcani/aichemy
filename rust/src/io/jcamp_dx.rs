use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of},
    combinator::{consumed, recognize},
    multi::{many0, many1},
    sequence::{delimited, terminated},
    IResult,
};

pub struct Parser {}

#[derive(Debug, PartialEq, Eq, Clone)]
struct DataLabel(String);

#[derive(Debug, PartialEq, Eq, Clone)]
struct TypedDataLabel(String);

fn data_label_name(input: &str) -> IResult<&str, String> {
    let (remaining, (_, output)) =
        consumed(many1(terminated(alphanumeric1, many0(one_of(" -/\\_")))))(input)?;
    Ok((remaining, output.join("")))
}

fn data_label(input: &str) -> IResult<&str, DataLabel> {
    let (remaining, output) = delimited(tag("##"), data_label_name, tag("="))(input)?;
    Ok((remaining, DataLabel(output.to_uppercase())))
}

fn typed_data_label(input: &str) -> IResult<&str, TypedDataLabel> {
    let (remaining, output) = delimited(tag("##."), data_label_name, tag("="))(input)?;
    Ok((remaining, TypedDataLabel(output.into())))
}

impl Parser {
    fn new() -> Self {
        todo!()
    }

    fn parse(input: &str) {
        todo!()
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
    }
}
