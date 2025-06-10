use std::{ffi::OsString, fmt::{self, Display}};
use nom::{error::{ErrorKind, Error as NomError, ParseError as NomParserError}, IResult};
use thiserror::Error;
use nom::Err as NomErr;

// #[derive(Debug)]
// pub enum ChangesParserError<'a>
// {
//     Error(String),
//     ///Если не распознана последовательность токенов
//     TokensParseErrors(Vec<String>),
//     TargetDocumentInfo(&'a str),
//     ChangesMapFilePath(&'a str),
//     OperationError(String),
//     JsonpathNotFound(String),
//     FromIoError(std::io::Error),
//     FromSerdeError(serde_json::Error),
//     FromNomError(nom::Err<nom::error::Error<&'a str>>)
// }




// impl<'a> fmt::Display for ChangesParserError<'a>
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
//     {
//         match self 
//         {
//             ChangesParserError::Error(val) => write!(f, "{}", val),
//             ChangesParserError::TokensParseErrors(val) => write!(f, "{:?}", val),
//             ChangesParserError::TargetDocumentInfo(val) => write!(f, "Не обнаружена информация о документе в который необходимо внести изменения->{}", val),
//             ChangesParserError::OperationError(val) => write!(f, "{}", val),
//             ChangesParserError::ChangesMapFilePath(p) => write!(f, "Файл карты изменений по пути {} не найден", p),
//             ChangesParserError::JsonpathNotFound(p) => write!(f, "Объект не обнаружен по адресу {}", p),
//             ChangesParserError::FromIoError(e) => write!(f, "Ошибка IO: {}", e),
//             ChangesParserError::FromSerdeError(e) => write!(f, "Ошибка serde: {}", e.to_string()),
//             ChangesParserError::FromNomError(e) => write!(f, "Ошибка nom: {}", e.to_string()),
//         }
//     }
// }
// impl<'a> From<std::io::Error> for ChangesParserError<'a>
// {
//     fn from(error: std::io::Error) -> Self 
//     {
//         ChangesParserError::FromIoError(error)
//     }
// }

// impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for ChangesParserError<'a>
// {
//     fn from(error: nom::Err<nom::error::Error<&'a str>>) -> Self 
//     {
//         ChangesParserError::FromNomError(error)
//     }
// }

// impl<'a> From<serde_json::Error> for ChangesParserError<'a>
// {
//     fn from(error: serde_json::Error) -> Self 
//     {
//         ChangesParserError::FromSerdeError(error)
//     }
// }


// #[derive(Debug, PartialEq)]
// pub enum CustomNomError<'a, I: ToString> 
// {
//   Error(String, &'a str),
//   Nom(I, ErrorKind),
// }

// impl<'a, I: ToString> Display for CustomNomError<'a, I>
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
//     {
//         match self
//         {
//             CustomNomError::Error(error, remains) => write!(f, "Ошибка: {}. Остаток ввода->{}", error, remains),
//             CustomNomError::Nom(input, kind) => write!(f, "Ошибка nom: {}. Остаток ввода->{:?}", input.to_string(), kind)
//         }    
//     }
// }
// #[macro_export]
// macro_rules! nom_error
// {
//     ($err:tt, $remains:expr) => 
//     {
//         {
//             crate::logger::error!("{} |Остаток токенов->{}",$err, $remains);
//             nom::Err::Error(CustomNomError::Error($err.to_owned(), $remains))
//         }
//     };
// }
// #[macro_export]
// macro_rules! nom_warn
// {
//     ($err:tt, $remains:expr) => 
//     {
//         {
//             crate::logger::warn!("{} |Остаток токенов->{}",$err, $remains);
//             nom::Err::Error(CustomNomError::Error($err.to_owned(), $remains))
//         }
//     };
// }

// #[macro_export]
// macro_rules! nom_error2
// {
//     ($err:tt, $remains:expr) => 
//     {
//         {
//             crate::logger::error!("{} |Остаток токенов->{}",$err, $remains);
//             nom::Err::Error(CustomNomError::Error($err.to_owned(), $remains))
//         }
//     };
// }


// impl<'a, I: ToString> ParseError<I> for CustomNomError<'a, I> 
// {
//   fn from_error_kind(input: I, kind: ErrorKind) -> Self 
//   {
//     //let err = nom::Err::Error("dfsdfss");
//     CustomNomError::Nom(input, kind)
//   }

//   fn append(_: I, _: ErrorKind, other: Self) -> Self 
//   {
//     other
//   }
// }

// #[test]
// fn test_macro()
// {
    
//     let s = "321";
//     let e : nom::Err<CustomNomError::<&str>> = nom_error!("123", s);
// }
impl<I: ToString> NomParserError<I> for ParserError
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self 
    {
        Self::NomError
        {
            input: input.to_string(),
            code: kind
        }
    }

    fn append(input: I, kind: ErrorKind, other: Self) -> Self 
    {
        other
    }
}


#[derive(Error, Debug)]
pub enum ParserError
{
    #[error("Ошибка `{0}`")]
    Error(String),
    #[error("Заголовок `{0}` не распознан")]
    ParseHeaderError(String),
    #[error("Номер `{0}` не распознан")]
    ParseNumberError(String),
    #[error("Неправильная последовательность токенов номера абзаца -> {0}")]
    IndentQueueTokensError(String),
    ///Если не распознана последовательность токенов
    #[error("Не распознана последовательность токенов->{0:?}")]
    TokensParseErrors(Vec<String>),
    #[error("Не обнаружена информация о документе в который необходимо внести изменения->{0:?}")]
    TargetDocumentInfo(OsString),
    #[error("{0}")]
    ChangesMapFilePath(String),
    #[error("{0}")]
    OperationError(String),
    #[error(transparent)]
    FromIoError(#[from] std::io::Error),
    #[error(transparent)]
    FromSerdeError(#[from] serde_json::Error),
    #[error("nom error: {input} -> {code:?}")]
    NomError {input: String, code: ErrorKind},

}
// impl ParserError
// {
//     pub fn err<I, O>(e: ParserError) -> IResult<I, O, ParserError>
//     {
//         let err = nom::Err::Error(e);
//         Err(err)
//     }
// }

impl<I, O> Into<IResult<I, O, ParserError>> for ParserError
{
    fn into(self) -> IResult<I, O, ParserError> 
    {
       Err(nom::Err::Error(self))
    }
}
// impl From<nom::Err<Error>> for Error
// {
//     fn from(value: nom::Err<Error>) -> Self 
//     {
//         let err = match value
//         {
//             nom::Err::Error(e) => e,
//             nom::Err::Incomplete(_) => Error::NomError { input: "Недостаточно данных для разбора строки".to_owned(), code: ErrorKind::Fail },
//             nom::Err::Failure(f) => f
//         };
//         err
//     }
// }

// impl Into<ParserError> for nom::Err<ParserError>
// {
//     fn into(self) -> ParserError
//     {
//       let err = match self
//       {
//         nom::Err::Error(e) => e,
//         nom::Err::Incomplete(_) => ParserError::NomError { input: "Недостаточно данных для разбора строки".to_owned(), code: ErrorKind::Fail },
//         nom::Err::Failure(f) => f
//       };
//       err
//     }
// }
impl Into<nom::Err<ParserError>> for ParserError
{
    fn into(self) -> nom::Err<ParserError> 
    {
        nom::Err::Error(self)
    }
}
impl From<NomError<&str>> for ParserError 
{
    fn from(err: NomError<&str>) -> Self 
    {
        ParserError::NomError 
        {
            input: err.input.to_string(),
            code: err.code,
        }
    }
}

// Реализация From для nom::Err
impl From<nom::Err<NomError<&str>>> for ParserError 
{
    fn from(err: nom::Err<NomError<&str>>) -> Self 
    {
        match err 
        {
            nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
            nom::Err::Incomplete(_) => ParserError::Error("Неожиданный конец ввода".to_string()),
        }
    }
}

impl From<NomErr<ParserError>> for ParserError 
{
    fn from(err: NomErr<ParserError>) -> Self 
    {
        match err 
        {
            NomErr::Error(e) | NomErr::Failure(e) => e,
            NomErr::Incomplete(_) => ParserError::Error("Неожиданный конец ввода".to_string()),
        }
    }
}


impl serde::Serialize for ParserError 
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
	S: serde::ser::Serializer,
	{
		serializer.serialize_str(self.to_string().as_ref())
	}
}


// pub fn parse(_input: &str) -> IResult<&str, &str, CustomNomError<&str>> 
// {
//   Err(nom::Err::Error(CustomNomError::MyError))
// }