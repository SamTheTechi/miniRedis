use crate::{model::types::RESP, util::find_crlf::find_crlf};
use anyhow::Result;

pub fn parse_simple_string(buf: &mut Vec<u8>) -> Result<Option<RESP>> {
    if let Some(pos) = find_crlf(&buf) {
        let line = buf[1..pos].to_vec();
        buf.drain(..pos + 2);

        let s = String::from_utf8(line)?;

        return Ok(Some(RESP::SimpleStrings(s)));
    }
    Ok(None)
}
