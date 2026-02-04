pub fn bulk_to_string(binary: &Vec<u8>) -> Option<String> {
    String::from_utf8(binary.to_vec()).ok()
}
