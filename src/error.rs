use std;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ColorError {
  Generic(GenericError),
  FromUtf8(std::string::FromUtf8Error),
  ParseInt(std::num::ParseIntError),
}

impl ColorError {
  pub fn generic(message: &str) -> ColorError {
    ColorError::Generic(GenericError::new(message))
  }
}

impl From<std::string::FromUtf8Error> for ColorError {
  fn from(err: std::string::FromUtf8Error) -> ColorError {
    ColorError::FromUtf8(err)
  }
}

impl From<std::num::ParseIntError> for ColorError {
  fn from(err: std::num::ParseIntError) -> ColorError {
    ColorError::ParseInt(err)
  }
}

impl fmt::Display for ColorError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ColorError::Generic(ref err) => write!(f, "{}", err),
      ColorError::FromUtf8(ref err) => write!(f, "{}", err),
      ColorError::ParseInt(ref err) => write!(f, "{}", err),
    }
  }
}

impl error::Error for ColorError {
  fn description(&self) -> &str {
    match *self {
      ColorError::Generic(ref err) => err.description(),
      ColorError::FromUtf8(ref err) => err.description(),
      ColorError::ParseInt(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match *self {
      ColorError::Generic(ref err) => Some(err),
      ColorError::FromUtf8(ref err) => Some(err),
      ColorError::ParseInt(ref err) => Some(err),
    }
  }
}

#[derive(Debug)]
pub struct GenericError { message: String}
impl GenericError {
  pub fn new(message: &str) -> GenericError {
    GenericError{message: String::from(message)}
  }
}

impl<'a> fmt::Display for GenericError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Store error: {}", self.message)
  }
}

impl<'a> error::Error for GenericError {
  fn description(&self) -> &str { self.message.as_str() }

  fn cause(&self) -> Option<&error::Error> { None }
}