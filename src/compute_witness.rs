use std::io::{Read, Seek};

use log::info;
use zokrates_ast::ir::{self, ProgEnum};
use zokrates_field::Field;

pub fn compute_witness_wrapper(
    mut content: impl Read + Seek,
    inputs: Vec<String>,
) -> Result<impl AsRef<[u8]>, String> {
    let witness = match ProgEnum::deserialize(&mut content)? {
        ProgEnum::Bn128Program(p) => compute_witness(p, inputs),
        ProgEnum::Bls12_377Program(p) => compute_witness(p, inputs),
        ProgEnum::Bls12_381Program(p) => compute_witness(p, inputs),
        ProgEnum::Bw6_761Program(p) => compute_witness(p, inputs),
        ProgEnum::PallasProgram(p) => compute_witness(p, inputs),
        ProgEnum::VestaProgram(p) => compute_witness(p, inputs),
    }?;

    Ok(witness)
}
// TODO: should return Witness<T> but doesn't work, bc of missmatichng arms
fn compute_witness<'a, T: Field, I: Iterator<Item = ir::Statement<'a, T>>>(
    ir_prog: ir::ProgIterator<'a, T, I>,
    inputs: Vec<String>,
) -> Result<Vec<u8>, String> {
    info!("computing witness");

    // info!("inputs: {}", inputs);

    let datas = inputs
        .iter()
        .flat_map(|s| s.split(' '))
        .map(|x| T::try_from_dec_str(x))
        .map(|x| x.unwrap())
        .collect::<Vec<T>>();

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
