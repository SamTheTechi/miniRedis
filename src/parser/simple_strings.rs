use crate::{
    util::find_crlf::find_crlf,
    model::types::RESP
}
use anyhow::{ Result};

fn parse_simple_string(mut buf: Vec<u8>) -> Result<Option<RESP>> {
    if let Some(pos) = find_crlf(&buf) {
        buf.drain(..pos + 2);
    }

    print!()
}
