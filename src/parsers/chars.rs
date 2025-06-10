use nom::
{
  bytes::complete::tag, error::{ ErrorKind, ParseError}, AsChar, Compare, IResult, Input,
};

use crate::error::{ParserError};

///:
pub fn definition(s: &str) -> IResult<&str, &str, ParserError>
{
  tag(":")(s)
}
///;
pub fn end_indent_char(s: &str) -> IResult<&str, &str, ParserError>
{
  tag(";")(s)
}
// pub fn tag<T, I, Error>(tag: T) -> impl Fn(I) -> IResult<I, I, Error>
// where
//     Error: ParseError<I>,
//     I: Input + Compare<T>,
//     T: Input + Clone,
///Такой же как space0 только дополнительно есть неразрывный пробел
pub fn space0<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: Input,
    <T as Input>::Item: AsChar + Clone,
{
  input.split_at_position_complete(|item| 
  {
    let c = item.as_char();
    !(c == ' ' || c == '\t' || c == ' ')
  })
}

///Такой же как space1 только дополнительно есть неразрывный пробел
pub fn space1<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: Input,
    <T as Input>::Item: AsChar + Clone,
{
  input.split_at_position1_complete(
    |item| 
    {
      let c = item.as_char();
      !(c == ' ' || c == '\t' || c == ' ')
    },
    ErrorKind::Space,
  )
}