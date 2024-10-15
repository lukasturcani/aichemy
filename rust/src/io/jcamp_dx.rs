use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, one_of},
    sequence::delimited,
    IResult,
};

struct Parser {}

struct DataLabel(String);

struct TypedDataLabel(String);

fn data_label_name(input: &str) -> IResult<&str, String> {
    one_of(" -/\\_")
}

fn data_label(input: &str) -> IResult<&str, DataLabel> {
    let (remaining, output) = delimited(tag("##"), data_label_name, tag("="))(input)?;
    Ok((remaining, DataLabel(output.to_uppercase())))
}

fn typed_data_label(input: &str) -> IResult<&str, TypedDataLabel> {
    let (remaining, output) = delimited(tag("##."), data_label_name, tag("="))(input)?;
    Ok((remaining, TypedDataLabel(output)))
}

impl Parser {
    fn new() -> Self {
        todo!()
    }

    fn parse(input: &str) {
        todo!()
    }
}
