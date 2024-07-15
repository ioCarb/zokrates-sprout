use std::io::{Read, Seek};

use log::info;
use zokrates_ast::ir::{self, ProgEnum};
use zokrates_field::Field;

pub fn compute_witness_wrapper<'a>(
    mut content: impl Read + Seek,
    inputs: impl Iterator<Item = &'a str>,
) -> Result<impl AsRef<[u8]>, String> {
    let witness = match ProgEnum::deserialize(&mut content)? {
        ProgEnum::Bn128Program(p) => compute_witness(p, inputs),
        // ProgEnum::Bls12_377Program(p) => compute_witness(p, inputs),
        // ProgEnum::Bls12_381Program(p) => compute_witness(p, inputs),
        // ProgEnum::Bw6_761Program(p) => compute_witness(p, inputs),
        // ProgEnum::PallasProgram(p) => compute_witness(p, inputs),
        // ProgEnum::VestaProgram(p) => compute_witness(p, inputs),
        _ => Err(
            "sorry this curve isnt currently supported. zokrates can only export bn128 to eth"
                .to_string(),
        ),
    }?;

    Ok(witness)
}
// TODO: should return Witness<T> but doesn't work, bc of missmatichng arms
fn compute_witness<'a, T: Field, I: Iterator<Item = ir::Statement<'a, T>>>(
    ir_prog: ir::ProgIterator<'a, T, I>,
    inputs: impl Iterator<Item = &'a str>,
) -> Result<Vec<u8>, String> {
    info!("computing witness");

    let datas: Vec<T> = inputs
        .map(|x| T::try_from_dec_str(x))
        .collect::<Result<Vec<T>, _>>()
        .map_err(|e| format!("couldnt decode inputs:{:?}", e))?;

    let interpreter = zokrates_interpreter::Interpreter::default();

    let witness = interpreter
        .execute(
            &datas,
            ir_prog.statements,
            &ir_prog.arguments,
            &ir_prog.solvers,
        )
        .map_err(|e| format!("Execution failed: {}", e))?;

    let mut writer: Vec<u8> = Vec::new();
    witness
        .write(&mut writer)
        .map_err(|why| format!("Could not save witness: {:?}", why))?;

    Ok(writer)
}
