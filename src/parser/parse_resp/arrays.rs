use super::parse_resp;
use crate::{model::types::RESP, util::find_crlf};
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
    let mut items_size = 0;

    for _ in 0..count {
        match parse_resp(buf, offset)? {
            Some((r, c)) => {
                items.push(r);
                items_size += c;
            }
            None => return Ok(None),
        }
    }

    Ok(Some((RESP::Arrays(items), items_size)))
}
