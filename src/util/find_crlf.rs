pub fn find_crlf(buf: &Vec<u8>) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\r\n")
}

