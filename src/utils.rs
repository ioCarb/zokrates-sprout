use std::io::Read;

// if db or fs, goes here
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
