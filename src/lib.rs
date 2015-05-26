extern crate serde;
#[macro_use]
extern crate log;

pub use self::de::{Deserializer, from_str};
pub use self::error::{Error, ErrorCode};

pub mod de;
pub mod error;
pub mod value;

trait IsWhitespace<T> {
    fn is_ws(&self) -> bool;
    fn is_ws_or(&self, T) -> bool;
}

impl<'a> IsWhitespace<&'a[u8]> for &'a[u8] {
    fn is_ws(&self) -> bool {
        self.iter().all(|&c| c.is_ws() )
    }
    fn is_ws_or(&self, or: &'a[u8]) -> bool {
        self.iter().all(|&c| c.is_ws_or(or) )
    }
}

impl<'a> IsWhitespace<&'a[u8]> for u8 {
    fn is_ws(&self) -> bool {
        b" \t\n\r".contains(self)
    }
    fn is_ws_or(&self, or: &'a[u8]) -> bool {
        self.is_ws() || or.contains(self)
    }
}
