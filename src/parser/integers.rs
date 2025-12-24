use crate::{model::types::RESP, util::find_crlf::find_crlf};
use anyhow::Result;

pub fn parse_integers(buf: &mut Vec<u8>) -> Result<Option<RESP>> {
    let Some(pos) = find_crlf(&buf) else {
        return Ok(None);
    };
    buf.drain(..pos + 2);

    let s = str::from_utf8(&buf[1..pos])?;
    let value = s.parse::<i64>()?;

    Ok(Some(RESP::Integers(value)))
}
