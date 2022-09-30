#[derive(Debug)]
pub enum Error {
    IllegalElement,
    MissingOperator,
    UnknownOperator(String),
    OperandMismatch,
}

pub type ParseResult<T> = Result<T, Error>;

pub const fn illegal_element<T>() -> ParseResult<T> {
    Err(Error::IllegalElement)
}

pub const fn unknown_operator<T>(name: String) -> ParseResult<T> {
    Err(Error::UnknownOperator(name))
}

pub const fn operand_mismatch<T>() -> ParseResult<T> {
    Err(Error::OperandMismatch)
}

#[macro_export]
macro_rules! parse_diary_func {
    ($name:ident (|$($param_name:ident : $param_type:path),+| $generator:expr) -> $rtype:ty) => {
        crate::parse_func!(
            $name(
                |$($param_name : $param_type),+| $generator,
                crate::syntax_error::illegal_element(),
                crate::syntax_error::operand_mismatch(),
                crate::syntax_error::operand_mismatch()
            ) -> ParseResult<$rtype>
        );
    };
}

#[macro_export]
macro_rules! get_rand_diary {
    ($iter:expr, $typ:path) => {
        crate::get_rand!(
            $iter,
            $typ,
            crate::syntax_error::operand_mismatch(),
            crate::syntax_error::illegal_element()
        )
    };
}

#[macro_export]
macro_rules! match_keyword {
    ($ve:expr, |$rand:ident| {$($patt:pat => $then:expr),+}) => {
        match crate::sexp::expect_application($ve) {
            Ok((rator, $rand)) => {
                match rator.as_str() {
                    $($patt => $then,)*
                    _ => crate::syntax_error::unknown_operator(rator.to_owned()),
                }
            },
            Err(crate::sexp::ApplicationError::MissingOperator) => Err(Error::MissingOperator),
            Err(crate::sexp::ApplicationError::HeadIsNotLiteral) => Err(Error::IllegalElement),
        }
    }
}

#[macro_export]
macro_rules! match_keyword_mut {
    ($ve:expr, |$rand:ident| {$($patt:pat => $then:expr),+}) => {
        match crate::sexp::expect_application($ve) {
            Ok((rator, mut $rand)) => {
                match rator.as_str() {
                    $($patt => $then,)*
                    _ => crate::syntax_error::unknown_operator(rator.to_owned()),
                }
            },
            Err(crate::sexp::ApplicationError::MissingOperator) => Err(Error::MissingOperator),
            Err(crate::sexp::ApplicationError::HeadIsNotLiteral) => Err(Error::IllegalElement),
        }
    }
}
