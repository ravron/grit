use anyhow::Result;
use std::str;

/// Some nice helper functions for working with buffers, allowing us to add methods to the buffers
/// rather than having to pass the buffers into a function (nice syntactic sugar).
///
/// Inspired by the bites::Buf trait. This should probably extend Buf and use its `chunk`,
/// `advance`, etc methods, but this was quick and easy for now.
pub trait BufUtils<'a> {
    fn get_str_until(&mut self, delim: u8) -> Result<&'a str>;
    fn get_until(&mut self, delim: u8) -> Result<&'a [u8]>;
    fn get_n_exact(&mut self, n: usize) -> Result<&'a [u8]>;
}

impl<'a> BufUtils<'a> for &'a [u8] {
    fn get_str_until(&mut self, delim: u8) -> Result<&'a str> {
        Ok(str::from_utf8(self.get_until(delim)?)?)
    }

    fn get_until(&mut self, delim: u8) -> Result<&'a [u8]> {
        let mut split = self.splitn(2, |b| *b == delim);
        let until = split
            .next()
            .expect("Split should always return at least one");
        let remaining = split.next();
        if let Some(remaining) = remaining {
            *self = remaining;
            Ok(until)
        } else {
            Err(anyhow!("buf didn't have delimiter {:?}", delim))
        }
    }

    fn get_n_exact(&mut self, n: usize) -> Result<&'a [u8]> {
        let (until, rest) = self.split_at(n);
        if until.len() == n {
            *self = rest;
            Ok(until)
        } else {
            Err(anyhow!("buf didn't have at least {} items", n))
        }
    }
}
