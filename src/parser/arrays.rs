use crate::parser::parse_resp::parse_resp;
use crate::{model::types::RESP, util::find_crlf::find_crlf};
use anyhow::Result;

pub fn parse_array(buf: &mut Vec<u8>) -> Result<Option<RESP>> {
    let Some(pos) = find_crlf(&buf) else {
        return Ok(None);
    };

    let count = str::from_utf8(&buf[1..pos])?.parse::<usize>()?;
    buf.drain(..pos + 2);

    let mut items: Vec<RESP> = Vec::with_capacity(count);

    for _ in 0..count {
        match parse_resp(buf)? {
            Some(r) => items.push(r),
            None => return Ok(None),
        }
    }

    Ok(Some(RESP::Arrays(items)))
}
