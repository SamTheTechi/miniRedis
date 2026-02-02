use crate::{model::RESP, util::find_crlf};
use anyhow::Result;

pub fn parse_integers(buf: &[u8], offset: &mut usize) -> Result<Option<(RESP, usize)>> {
    let start = *offset;
    let Some(pos) = find_crlf(&buf[start..]) else {
        return Ok(None);
    };

    let s = str::from_utf8(&buf[start + 1..start + pos])?;
    let value = s.parse::<i64>()?;

    *offset = start + pos + 2;

    Ok(Some((RESP::Integers(value), *offset - start)))
}
