pub fn parse_pcwstr(data: &[u8]) -> String {
    let d = data
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|&x| x != 0);

    char::decode_utf16(d)
        .map(|it| it.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

pub fn parse_multi_sz(data: &[u8]) -> Vec<String> {
    let mut vec = Vec::new();
    let mut iter = data
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .peekable();

    while let Some(&(1..)) = iter.peek() {
        vec.push(
            char::decode_utf16(iter.by_ref().take_while(|&x| x != 0))
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect(),
        );
    }

    vec
}

pub fn encode_utf16_null(input: &str) -> Vec<u8> {
    input
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(|u16| u16.to_le_bytes())
        .collect()
}
