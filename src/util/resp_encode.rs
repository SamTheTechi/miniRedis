pub fn array_len(len: usize) -> Vec<u8> {
    format!("*{}\r\n", len).into_bytes()
}

pub fn bulk_str(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut out = format!("${}\r\n", bytes.len()).into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\r\n");
    out
}

pub fn integer(n: i64) -> Vec<u8> {
    format!(":{}\r\n", n).into_bytes()
}
