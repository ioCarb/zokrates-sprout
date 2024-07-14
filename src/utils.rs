use std::io::Read;

/// groups elements from string by place in a separator separated list
/// with separator: ',': given ["a1,b1,c1 d1", "a2,b2,c2 d2"] -> "a1 a2 b1 b2 c1 d1 c2 d2"
/// if a later element contains more commas then the first element, the data will be rejected
// we could find the minimum do a alloc a correctly sized vec, or branch once in every loop and resize, but its just to find the size is too slow
pub fn group_by(input: Vec<String>, separator: char) -> Result<String, String> {
    if input.is_empty() {
        return Err("empty array".to_owned());
    }

    let segments_count = input[0].split(',').count();

    let mut segments: Vec<Vec<String>> = vec![Vec::new(); segments_count];

    for s in input {
        let parts = s.split(separator);
        for (i, part) in parts.enumerate() {
            segments[i].push(part.to_owned());
        }
    }

    let result: String = segments
        .iter()
        .map(|segment| segment.join(" "))
        .collect::<Vec<String>>()
        .join(" ");

    Ok(result)
}

// if db or fs, goes here
/// convert `&str` from hex to zlib compressed bytes `Vec<u8>` and decompress, returns tonic error
pub async fn convert(input: &str, context: &str) -> Result<Vec<u8>, tonic::Status> {
    convert_zlib_hex_to_bytes(input).map_err(|e| {
        tonic::Status::invalid_argument(format!(
            "couldn't decode project_config {}. should have been handled prior {}",
            context, e
        ))
    })
}

/// convert `&str` from hex to zlib compressed bytes `Vec<u8>` and decompress
pub fn convert_zlib_hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let bytes = match hex::decode(hex_str) {
        Ok(bytes) => bytes,
        Err(e) => return Err(format!("couldn't convert hex string to bytes {}", e)),
    };

    let mut d = flate2::read::ZlibDecoder::new(bytes.as_slice());

    let mut out = Vec::new();

    match d.read_to_end(&mut out) {
        Ok(_) => {}
        Err(e) => return Err(format!("couldn't decompress string {}", e)),
    };

    Ok(out)
}
