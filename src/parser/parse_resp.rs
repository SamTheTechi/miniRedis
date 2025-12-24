use crate::model::types::RESP;
use crate::parser::{
    arrays::parse_array, bulkstings::parse_bulk_sting, integers::parse_integers,
    simple_errors::parse_simple_error, simple_strings::parse_simple_string,
};
use anyhow::Result;

pub fn parse_resp(buf: &mut Vec<u8>) -> Result<Option<RESP>> {
    let first_byte = buf[0];
    match first_byte {
        b'+' => parse_simple_string(buf),
        b'-' => parse_simple_error(buf),
        b':' => parse_integers(buf),
        b'$' => parse_bulk_sting(buf),
        b'*' => parse_array(buf),
        _ => Err(anyhow::anyhow!("Invalid RESP type")),
    }
}
