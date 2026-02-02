use crate::{model::RESP, util::find_crlf};
use anyhow::Result;

pub fn parse_simple_string(buf: &[u8], offset: &mut usize) -> Result<Option<(RESP, usize)>> {
    let start = *offset;

    if let Some(pos) = find_crlf(&buf[start..]) {
        let line = buf[start + 1..start + pos].to_vec();
        let s = String::from_utf8(line)?;

        *offset = start + pos + 2;

        return Ok(Some((RESP::SimpleStrings(s), *offset - start)));
    }
    Ok(None)
}
