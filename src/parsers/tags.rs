use nom::{branch::alt, bytes::complete::{tag, tag_no_case}, combinator::{eof, map}, sequence::{delimited, pair}, IResult, Parser};
use crate::error::{ParserError};

use super::{chars::{end_indent_char, definition}};
use super::{space0, space1};

///следующего содержания:
pub fn next_is_content(s: &str) -> IResult<&str, bool, ParserError>
{
    let (remains, _ ) = ((tag("следующего") ,space1, tag("содержания"), definition, eof)).parse(s)?;
    Ok((remains, true))
}
///`следующего содержания:` но без признака окочания строки
pub fn next_is_content_not_eof(s: &str) -> IResult<&str, bool, ParserError>
{
    let (remains, _ ) = ((tag("следующего") ,space1, tag("содержания"), definition)).parse(s)?;
    Ok((remains, true))
}
///изложить (его) в новой редакции
pub fn in_new_edition(s: &str) -> IResult<&str, &str, ParserError>
{
    let n = ((
        tag("изложить"),
        space1,
        alt((map(pair(tag("его "), tag("в")), |_m|""), tag("в"))),
        space1,
        tag("следующей"),
        space1,
        tag("редакции"),
        definition,
        eof
    )).parse(s)?;
    Ok((n.0, ""))
}
///изложить (его) в новой редакции (без признака окончания строки)
pub fn in_new_edition_not_eof(s: &str) -> IResult<&str, &str, ParserError>
{
    let n = ((
        tag("изложить"),
        space1,
        alt((map(pair(tag("его "), tag("в")), |_m|""), tag("в"))),
        space1,
        tag("следующей"),
        space1,
        tag("редакции"),
        definition,
    )).parse(s)?;
    Ok((n.0, ""))
}

///дополнить новыми\дополнить новым\дополнить
pub fn apply(s: &str) -> IResult<&str, &str, ParserError>
{
    let mut parser = delimited(
        space0,
        alt((
            tag_no_case("дополнить новыми"),
            tag_no_case("дополнить новым"),
            tag_no_case("дополнить")
        )), space0);
        let v = parser.parse(s)?;
    // let v = delimited(
    //     space0,
    //     alt((
    //         tag_no_case("дополнить новыми"),
    //         tag_no_case("дополнить новым"),
    //         tag_no_case("дополнить")
    //     )), space0)(s)?;
    Ok((v.0, ""))
}

/// признать утратившим силу;
pub fn lost_power(s: &str) -> IResult<&str, &str, ParserError>
{
    let lost = ((
        space0, tag("признать"), space1, alt((tag("утратившими"), tag("утратившей"), tag("утратившим"))), space1, tag("силу"), end_indent_char, eof
    )).parse(s)?;
    Ok((lost.0, ""))
}

#[cfg(test)]
mod tests
{
    use super::{in_new_edition, next_is_content, lost_power};

    #[test]
    fn test_in_new_edition_fn()
    {
        logger::StructLogger::new_default();
        let new_1 = "изложить в следующей редакции:";
        let new_2 = "изложить его в следующей редакции:";
        let new_3 = "следующего содержания:";
        let new_4 = "признать утратившим силу;";
        let new_5 = "признать утратившими силу;";
        let t1 = in_new_edition(new_1).unwrap();
        let t2 = in_new_edition(new_2).unwrap();
        let t3 = next_is_content(new_3).unwrap();
        let t4 = lost_power(new_4).unwrap();
        let t5 = lost_power(new_5).unwrap();
    }
}