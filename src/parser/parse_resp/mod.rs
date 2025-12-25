mod arrays;
mod bulkstings;
mod integers;
mod simple_errors;
mod simple_strings;

use crate::model::types::RESP;
use crate::parser::parse_resp::{
    arrays::parse_array, bulkstings::parse_bulk_sting, integers::parse_integers,
    simple_errors::parse_simple_error, simple_strings::parse_simple_string,
};
use anyhow::Result;

pub fn parse_resp(buf: &[u8], offset: &mut usize) -> Result<Option<(RESP, usize)>> {
    if *offset >= buf.len() {
        return Ok(None);
    }
    let first_byte = buf[*offset];

    match first_byte {
        b'+' => parse_simple_string(buf, offset),
        b'-' => parse_simple_error(buf, offset),
        b':' => parse_integers(buf, offset),
        b'$' => parse_bulk_sting(buf, offset),
        b'*' => parse_array(buf, offset),
        _ => Err(anyhow::anyhow!("Invalid RESP type")),
    }
}
