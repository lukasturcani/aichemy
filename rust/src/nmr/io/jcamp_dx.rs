use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alphanumeric1, anychar, char, line_ending, multispace0, multispace1, not_line_ending,
        one_of, space0, u64,
    },
    combinator::{consumed, opt, peek, value},
    multi::{many0, many1, many_till, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use super::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Text(String),
    Number(f64),
    Array(Vec<f64>),
}

fn data_label_name(input: &str) -> IResult<&str, String> {
    let (remaining, (prefix, (_, label_name))) = pair(
        opt(one_of("$.")),
        consumed(many1(terminated(alphanumeric1, many0(one_of(" -/\\_"))))),
    )(input)?;
    let label_name = label_name.join("").to_uppercase();
    Ok((
        remaining,
        format!(
            "{}{}",
            prefix.map_or("".into(), |x| x.to_string()),
            label_name
        ),
    ))
}

fn data_label(input: &str) -> IResult<&str, String> {
    delimited(tag("##"), data_label_name, tag("="))(input)
}

fn labeled_data_record(input: &str) -> IResult<&str, (String, Value)> {
    let (remaining, (label, value)) = separated_pair(
        data_label,
        space0,
        alt((
            asdf_data_set,
            array_data_set,
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
    let (remaining, (output, _)) = preceded(char('<'), many_till(anychar, char('>')))(input)?;
    Ok((remaining, Value::Text(String::from_iter(output))))
}

fn affn_number_data_set(input: &str) -> IResult<&str, Value> {
    let (remaning, output) = double(input)?;
    Ok((remaning, Value::Number(output)))
}

fn asdf_data_set_line(input: &str) -> IResult<&str, Vec<f64>> {
    let (remaining, (_, ys)) =
        separated_pair(double, space0, many1(delimited(space0, double, space0)))(input)?;
    Ok((remaining, ys))
}

fn asdf_data_set_value_block(input: &str) -> IResult<&str, Value> {
    let (remaining, output) = separated_list1(multispace1, asdf_data_set_line)(input)?;
    Ok((
        remaining,
        Value::Array(output.into_iter().flatten().collect()),
    ))
}

fn asdf_data_set(input: &str) -> IResult<&str, Value> {
    let (remaining, output) = preceded(
        preceded(tag("(X++(Y..Y))"), multispace1),
        asdf_data_set_value_block,
    )(input)?;
    Ok((remaining, output))
}

fn array_data_set(input: &str) -> IResult<&str, Value> {
    let (remaining, output) = preceded(
        preceded(
            delimited(char('('), delimited(u64, tag(".."), u64), char(')')),
            multispace0,
        ),
        separated_list0(multispace0, double),
    )(input)?;
    Ok((remaining, Value::Array(output)))
}

fn parser(input: &str) -> IResult<&str, Vec<(String, Value)>, nom::error::Error<String>> {
    delimited(
        multispace0,
        separated_list0(
            delimited(
                multispace0,
                opt(alt((inline_comment, multi_line_comment))),
                multispace0,
            ),
            labeled_data_record,
        ),
        multispace0,
    )(input)
    .map_err(|source| source.to_owned())
}

/// A parser for JCAMP-DX files.
///
/// This parser is based on the JCAMP-DX specification, defined
/// [here](http://www.jcamp-dx.org/protocols/dxir01.pdf),
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_NMR_1993.pdf) and
/// [here](https://iupac.org/wp-content/uploads/2021/08/JCAMP-DX_MS_1994.pdf)
/// TODO: stuff
#[derive(Clone, Debug, Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    pub fn parse(self, input: &str) -> Result<HashMap<String, Value>, Error> {
        let (_, output) = parser(input).map_err(|source| Error::Parse { source })?;
        Ok(output.into_iter().collect())
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
        assert_eq!(output, ".OBSERVATION232TYPE".to_string());

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

        let (remaining, output) = multi_line_text_data_set("<>  \n").unwrap();
        assert_eq!(remaining, "  \n");
        assert_eq!(output, Value::Text("".into()));
    }

    #[test]
    fn test_labeled_data_record() {
        let (remaining, (label, value)) =
            labeled_data_record("##.OBSERVATION232TYPE= SOLID_ANODE\n").unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(label, ".OBSERVATION232TYPE");
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

        let (remaining, (label, value)) =
            labeled_data_record("##$O-B  SER\\va/TiON232_TYPE= (0..3)  \n  1 2 3 4  ").unwrap();
        assert_eq!(remaining, "  ");
        assert_eq!(label, "$OBSERVATION232TYPE");
        assert_eq!(value, Value::Array(vec![1., 2., 3., 4.]));

        let (remaining, (label, value)) = labeled_data_record(
            "##XYPOINTS= (X++(Y..Y))
                -0.001 -0.001 0.001
                0.001 0.001 0.001
                0.002 0.002 0.002
                0.001 0.001 0.001 \n",
        )
        .unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(label, "XYPOINTS");
        assert_eq!(
            value,
            Value::Array(vec![
                -0.001, 0.001, 0.001, 0.001, 0.002, 0.002, 0.001, 0.001
            ])
        );
    }

    #[test]
    fn test_asdf_data_set_line() {
        let (remaining, output) = asdf_data_set_line(
            "16383 +2259260   -5242968  -7176216
            16374 +1757248   +3559312   1108422
            16365 -5429568   -7119772   -2065758 \n",
        )
        .unwrap();
        assert_eq!(
            remaining,
            "
            16374 +1757248   +3559312   1108422
            16365 -5429568   -7119772   -2065758 \n",
        );
        assert_eq!(output, vec![2259260., -5242968., -7176216.]);
    }

    #[test]
    fn test_asdf_data_set_value_block() {
        let (remaining, output) = asdf_data_set_value_block(
            "16383 +2259260   -5242968  -7176216
            16374 +1757248   +3559312   1108422
            16365 -5429568   -7119772   -2065758 \n",
        )
        .unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            output,
            Value::Array(vec![
                2259260., -5242968., -7176216., 1757248., 3559312., 1108422., -5429568., -7119772.,
                -2065758.
            ])
        );
    }

    #[test]
    fn test_asdf_data_set() {
        let (remaining, output) = asdf_data_set(
            "(X++(Y..Y))\n\
            16383 +2259260   -5242968  -7176216 \n\
            16374 +1757248   +3559312   1108422 \n\
            16365 -5429568   -7119772   -2065758 \n",
        )
        .unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            output,
            Value::Array(vec![
                2259260., -5242968., -7176216., 1757248., 3559312., 1108422., -5429568., -7119772.,
                -2065758.
            ])
        );

        let (remaining, output) = asdf_data_set(
            "(X++(Y..Y))
            16383 +2259260   -5242968  -7176216
            16374 +1757248   +3559312   1108422 \n\
            16365 -5429568   -7119772   -2065758 \n",
        )
        .unwrap();
        assert_eq!(remaining, "\n");
        assert_eq!(
            output,
            Value::Array(vec![
                2259260., -5242968., -7176216., 1757248., 3559312., 1108422., -5429568., -7119772.,
                -2065758.
            ])
        );
    }

    #[test]
    fn test_array() {
        let (remaining, output) = array_data_set("(0..3)  \n 1 2 3 4   \n").unwrap();
        assert_eq!(remaining, "   \n");
        assert_eq!(output, Value::Array(vec![1., 2., 3., 4.]));

        let (remaining, output) = array_data_set(
            "(0..7)
                1 2 3 4
                1 2 3 4
            ",
        )
        .unwrap();
        assert_eq!(remaining, "\n            ");
        assert_eq!(output, Value::Array(vec![1., 2., 3., 4., 1., 2., 3., 4.]));
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

    #[test]
    fn test_parser() {
        let (remaining, output) = parser(
            "
                ##TITLE= diff
                ##JCAMP-DX= 5.00   $$ ISDF V5.00
                ##DATA TYPE= MASS SPECTRUM
                ##$D= (0..5)
                10 11 12 13
                14 15 16 17
                ##.OBSERVE NUCLEUS= ^1H
                $$ something
                $$ ---------
                ##XYPOINTS= (X++(Y..Y))
                    -0.001 -0.001 0.001
                    0.002 0.003 0.001
                    0.001 0.001 0.001
                ##END=
            ",
        )
        .unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            output,
            vec![
                ("TITLE".into(), Value::Text("diff".into())),
                ("JCAMPDX".into(), Value::Number(5.)),
                ("DATATYPE".into(), Value::Text("MASS SPECTRUM".into())),
                (
                    "$D".into(),
                    Value::Array(vec![10., 11., 12., 13., 14., 15., 16., 17.])
                ),
                (".OBSERVENUCLEUS".into(), Value::Text("^1H".into())),
                (
                    "XYPOINTS".into(),
                    Value::Array(vec![-0.001, 0.001, 0.003, 0.001, 0.001, 0.001])
                ),
                ("END".into(), Value::Text("".into())),
            ]
        );
    }
}
