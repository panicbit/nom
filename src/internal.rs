//! Basic types to build the parsers

use self::IResult::*;
use self::Needed::*;
use util::ErrorKind;

#[cfg(feature = "core")]
use std::prelude::v1::*;
use std::boxed::Box;

/// Contains the error that a parser can return
///
/// It can represent a linked list of errors, indicating the path taken in the parsing tree, with corresponding position in the input data.
/// It depends on P, the input position (for a &[u8] parser, it would be a &[u8]), and E, the custom error type (by default, u32)
#[derive(Debug,PartialEq,Eq,Clone)]
pub enum Err<P,E=u32>{
  /// An error code, represented by an ErrorKind, which can contain a custom error code represented by E
  Code(ErrorKind<E>),
  /// An error code, and the next error
  Node(ErrorKind<E>, Box<Err<P,E>>),
  /// An error code, and the input position
  Position(ErrorKind<E>, P),
  /// An error code, the input position and the next error
  NodePosition(ErrorKind<E>, P, Box<Err<P,E>>)
}

/// Contains information on needed data if a parser returned `Incomplete`
#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Needed {
  /// needs more data, but we do not know how much
  Unknown,
  /// contains the required data size
  Size(usize)
}

impl Needed {
  pub fn is_known(&self) -> bool {
    *self != Unknown
  }

  /// Maps a `Needed` to `Needed` by appling a function to a contained `Size` value.
  #[inline]
  pub fn map<F: FnOnce(usize) -> usize>(self, f: F) -> Needed {
    match self {
      Unknown => Unknown,
      Size(n) => Size(f(n)),
    }
  }
}

/// Holds the result of parsing functions
///
/// It depends on I, the input type, O, the output type, and E, the error type (by default u32)
///
#[derive(Debug,PartialEq,Eq,Clone)]
pub enum IResult<I,O,E=u32> {
   /// indicates a correct parsing, the first field containing the rest of the unparsed data, the second field contains the parsed data
  Done(I,O),
  /// contains a Err, an enum that can indicate an error code, a position in the input, and a pointer to another error, making a list of errors in the parsing tree
  Error(Err<I,E>),
  /// Incomplete contains a Needed, an enum than can represent a known quantity of input data, or unknown
  Incomplete(Needed)
}

impl<I,O,E> IResult<I,O,E> {
  pub fn is_done(&self) -> bool {
    match *self {
      Done(_,_) => true,
      _         => false
    }
  }

  pub fn is_err(&self) -> bool {
    match *self {
      Error(_) => true,
      _        => false
    }
  }

  pub fn is_incomplete(&self) -> bool {
    match *self {
      Incomplete(_) => true,
      _             => false
    }
  }

  /// Maps a `IResult<I, O, E>` to `IResult<I, N, E>` by appling a function
  /// to a contained `Done` value, leaving `Error` and `Incomplete` value
  /// untouched.
  #[inline]
  pub fn map<N, F: FnOnce(O) -> N>(self, f: F) -> IResult<I, N, E> {
    match self {
      Done(i, o)    => Done(i, f(o)),
      Error(e)      => Error(e),
      Incomplete(n) => Incomplete(n),
    }
  }

  /// Maps a `IResult<I, O, E>` to `IResult<I, O, E>` by appling a function
  /// to a contained `Incomplete` value, leaving `Done` and `Error` value
  /// untouched.
  #[inline]
  pub fn map_inc<F>(self, f: F) -> IResult<I, O, E>
   where F: FnOnce(Needed) -> Needed {
    match self {
      Error(e)      => Error(e),
      Incomplete(n) => Incomplete(f(n)),
      Done(i, o)    => Done(i, o),
    }
  }

  /// Maps a `IResult<I, O, E>` to `IResult<I, O, N>` by appling a function
  /// to a contained `Error` value, leaving `Done` and `Incomplete` value
  /// untouched.
  #[inline]
  pub fn map_err<N, F>(self, f: F) -> IResult<I, O, N>
   where F: FnOnce(Err<I, E>) -> Err<I, N> {
    match self {
      Error(e)      => Error(f(e)),
      Incomplete(n) => Incomplete(n),
      Done(i, o)    => Done(i, o),
    }
  }
}

pub trait GetInput<I> {
  fn remaining_input(&self) -> Option<I>;
}

pub trait GetOutput<O> {
  fn output(&self) -> Option<O>;
}

impl<'a,I,O,E> GetInput<&'a[I]> for IResult<&'a[I],O,E> {
  fn remaining_input(&self) -> Option<&'a[I]> {
    match *self {
      Done(ref i,_) => Some(*i),
      _             => None
    }
  }
}

impl<O,E> GetInput<()> for IResult<(),O,E> {
  fn remaining_input(&self) -> Option<()> {
    match *self {
      Done((),_) => Some(()),
      _          => None
    }
  }
}

impl<'a,O,E> GetInput<&'a str> for IResult<&'a str,O,E> {
  fn remaining_input(&self) -> Option<&'a str> {
    match *self {
      Done(ref i,_) => Some(*i),
      _          => None
    }
  }
}

impl<'a,I,O,E> GetOutput<&'a[O]> for IResult<I,&'a[O],E> {
  fn output(&self) -> Option<&'a[O]> {
    match *self {
      Done(_, ref o) => Some(*o),
      _              => None
    }
  }
}

impl<I,E> GetOutput<()> for IResult<I,(),E> {
  fn output(&self) -> Option<()> {
    match *self {
      Done(_,()) => Some(()),
      _          => None
    }
  }
}

impl<'a,I,E> GetOutput<&'a str> for IResult<I,&'a str,E> {
  fn output(&self) -> Option<&'a str> {
    match *self {
      Done(_,ref o) => Some(*o),
      _          => None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use util::ErrorKind;

  #[test]
  fn needed_map() {
    let unknown = Needed::Unknown;
    let size = Needed::Size(5);

    assert_eq!(size.map(|x| x * 2), Needed::Size(10));
    assert_eq!(unknown.map(|x| x * 2), Needed::Unknown);
  }

  #[test]
  fn iresult_map() {
    let done: IResult<&[u8], u32> = IResult::Done(&b""[..], 5);
    let error: IResult<&[u8], u32> = IResult::Error(Err::Code(ErrorKind::Tag));
    let incomplete: IResult<&[u8], u32> = IResult::Incomplete(Needed::Unknown);

    assert_eq!(done.map(|x| x * 2), IResult::Done(&b""[..], 10));
    assert_eq!(error.map(|x| x * 2), IResult::Error(Err::Code(ErrorKind::Tag)));
    assert_eq!(incomplete.map(|x| x * 2), IResult::Incomplete(Needed::Unknown));
  }

  #[test]
  fn iresult_map_inc() {
    let done: IResult<&[u8], u32> = IResult::Done(&b""[..], 5);
    let error: IResult<&[u8], u32> = IResult::Error(Err::Code(ErrorKind::Tag));
    let inc_unknown: IResult<&[u8], u32> = IResult::Incomplete(Needed::Unknown);
    let inc_size: IResult<&[u8], u32> = IResult::Incomplete(Needed::Size(5));

    assert_eq!(done.map_inc(|n| if let Needed::Size(i) = n {Needed::Size(i+1)} else {n}), IResult::Done(&b""[..], 5));
    assert_eq!(error.map_inc(|n| if let Needed::Size(i) = n {Needed::Size(i+1)} else {n}), IResult::Error(Err::Code(ErrorKind::Tag)));
    assert_eq!(inc_unknown.map_inc(|n| if let Needed::Size(i) = n {Needed::Size(i+1)} else {n}), IResult::Incomplete(Needed::Unknown));
    assert_eq!(inc_size.map_inc(|n| if let Needed::Size(i) = n {Needed::Size(i+1)} else {n}), IResult::Incomplete(Needed::Size(6)));
  }

  #[test]
  fn iresult_map_err() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Error(u32);

    let error_kind = Err::Code(ErrorKind::Custom(Error(5)));

    let done: IResult<&[u8], u32> = IResult::Done(&b""[..], 5);
    let error: IResult<&[u8], u32> = IResult::Error(Err::Code(ErrorKind::Tag));
    let incomplete: IResult<&[u8], u32> = IResult::Incomplete(Needed::Unknown);

    assert_eq!(done.map_err(|_| error_kind.clone()), IResult::Done(&b""[..], 5));
    assert_eq!(error.map_err(|x| {println!("err: {:?}", x); error_kind.clone()}), IResult::Error(error_kind.clone()));
    assert_eq!(incomplete.map_err(|x| {println!("err: {:?}", x); error_kind.clone()}), IResult::Incomplete(Needed::Unknown));
  }
}
