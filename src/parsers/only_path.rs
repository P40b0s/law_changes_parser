use nom::
{
    IResult,
    branch::{permutation, alt},
    bytes::complete::{tag, tag_no_case, take_until, take_till, is_a},
    character::complete::{digit0, char, alphanumeric1, space0, alpha1, one_of, anychar},
    sequence::{tuple, pair, separated_pair, terminated, preceded, delimited},
    multi::{many0, many1, many_till},
    combinator::{value, not, eof, map, opt}
};
use crate::{error::CustomNomError, json_path_creator::JsonPathCreator};

use super::{HEADER_NUMBER, ITEM_NUMBER, INDENT_NUMBERS, paths, numbers::list_number, next_is_content, apply, in_new_edition, lost_power, words::number_items_with_number_lost_power, chars::definition};
use super::{space0, space1};

///`1) в статье 2^2:`
pub fn in_path_definition(s: &str) -> IResult<&str, JsonPathCreator, CustomNomError<&str>>
{
    let (remains, (_,_,_,_,p,_,_)) = tuple((list_number, space1, tag("в"), space1, paths, definition, eof))(s)?;
    Ok((remains, p))
}

#[cfg(test)]
mod tests
{
    use format_constructor::DateTimeFormat;
    use format_structure::Date;
    use logger::info;
    use nom::{branch::{permutation, alt}, bytes::complete::{take_until, tag, is_a, tag_no_case}, combinator::map, IResult, multi::{many0, many1, many_till}, sequence::{pair, tuple}, character::complete::{anychar, alpha0, alpha1, alphanumeric1, digit1}};
    use crate::{error::CustomNomError, parsers::{space1, space0, target_document::DocumentType}};

    #[test]
    fn test_check_add()
    {
        logger::StructLogger::initialize_logger();
        let check_all_strings : Vec<&str> = vec![
            r#"1) в статье 40:"#,
            r#"п) в пункте 16:"#,
            r#"1) в статье 4^12:"#,
            r#"п) в пункте 1_6:"#,
            r#"п) в пункте 1^1-1:"#,
        ];
        for s in check_all_strings
        {
            let p = super::in_path_definition(s).unwrap();
            info!("тест строки ->{} остаток токенов ->{}", p.0, p.1);
        }
    }
}