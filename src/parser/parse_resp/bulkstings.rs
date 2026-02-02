use crate::{model::RESP, util::find_crlf};
use anyhow::Result;

pub fn parse_bulk_sting(buf: &[u8], offset: &mut usize) -> Result<Option<(RESP, usize)>> {
    let start = *offset;

    let Some(pos) = find_crlf(&buf[start..]) else {
        return Ok(None);
    };

    let len_end = start + pos;
    let len = str::from_utf8(&buf[start + 1..len_end])?.parse::<i64>()?;

    if len == -1 {
        *offset = len_end + 2;
        return Ok(Some((RESP::BulkStrings(None), *offset - start)));
    }

    let len = len as usize;
    let bulk_str_end = len_end + 2 + len;
    if buf.len() < bulk_str_end + 2 {
        return Ok(None);
    }

    let data = buf[len_end + 2..bulk_str_end].to_vec();
    *offset = bulk_str_end + 2;

    Ok(Some((RESP::BulkStrings(Some(data)), *offset - start)))
}
