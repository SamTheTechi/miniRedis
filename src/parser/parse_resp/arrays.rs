use super::parse_resp;
use crate::{model::RESP, util::find_crlf};
use anyhow::Result;

pub fn parse_array(buf: &[u8], offset: &mut usize) -> Result<Option<(RESP, usize)>> {
    let start = *offset;

    let Some(pos) = find_crlf(&buf[start..]) else {
        return Ok(None);
    };
    let count_end = *offset + pos;
    let count = str::from_utf8(&buf[start + 1..count_end])?.parse::<usize>()?;

    *offset = count_end + 2;
    let mut items: Vec<RESP> = Vec::with_capacity(count);

    for _ in 0..count {
        match parse_resp(buf, offset)? {
            Some((r, _c)) => {
                items.push(r);
            }
            None => return Ok(None),
        }
    }

    Ok(Some((RESP::Arrays(items), *offset - start)))
}
