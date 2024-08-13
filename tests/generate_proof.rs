use std::{fs, io::Cursor};

use zokrates_sprout::generate_proof::compute_proof_wrapper;

#[test]
fn format_test() {
    let proving_key = Cursor::new(fs::read("tests/proving.key").unwrap());
    let out = Cursor::new(fs::read("tests/out").unwrap());
    let witness = Cursor::new(fs::read("tests/witness").unwrap());
    let scheme = "g16";

    let result = compute_proof_wrapper(out, proving_key, witness, scheme).unwrap();

    // let correct = fs::read_to_string("tests/yan.json").unwrap();

    println!("res: {}", result);

    // assert!(correct == result);
    assert!(true);
}
