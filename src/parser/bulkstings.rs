use crate::{model::types::RESP, util::find_crlf::find_crlf};
use anyhow::Result;

pub fn parse_bulk_sting(buf: &mut Vec<u8>) -> Result<Option<RESP>> {
    let Some(pos) = find_crlf(&buf) else {
        return Ok(None);
    };

    let len = str::from_utf8(&buf[1..pos])?.parse::<i64>()?;
    let start = pos + 2;

    if len == -1 {
        buf.drain(..start);
        return Ok(Some(RESP::BulkStrings(None)));
    }

    let len = len as usize;
    if buf.len() < start + 2 + len {
        return Ok(None);
    }

    let data = buf[start..start + len].to_vec();
    buf.drain(..start + len + 2);

    Ok(Some(RESP::BulkStrings(Some(data))))
}
