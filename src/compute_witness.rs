use std::io::{Cursor, Read, Seek};

use log::info;
use zokrates_ast::{
    ir::{self, ProgEnum, Witness},
    typed::{types::GTupleType, ConcreteSignature, ConcreteType},
};
use zokrates_field::Field;

pub fn compute_witness_wrapper(mut content: impl Read + Seek) -> Result<impl AsRef<[u8]>, String> {
    let witness = match ProgEnum::deserialize(&mut content)? {
        ProgEnum::Bn128Program(p) => compute_witness(p),
        ProgEnum::Bls12_377Program(p) => compute_witness(p),
        ProgEnum::Bls12_381Program(p) => compute_witness(p),
        ProgEnum::Bw6_761Program(p) => compute_witness(p),
        ProgEnum::PallasProgram(p) => compute_witness(p),
        ProgEnum::VestaProgram(p) => compute_witness(p),
    }?;

    Ok(witness)
}
// TODO: should return Witness<T> but doesn't work, bc of missmatichng arms
pub fn compute_witness<'a, T: Field, I: Iterator<Item = ir::Statement<'a, T>>>(
    ir_prog: ir::ProgIterator<'a, T, I>,
) -> Result<Vec<u8>, String> {
    info!("computing witness");

    let signature = ConcreteSignature::new()
        .inputs(vec![ConcreteType::FieldElement; ir_prog.arguments.len()])
        .output(ConcreteType::Tuple(GTupleType::new(
            vec![ConcreteType::FieldElement; ir_prog.return_count],
        )));

    let raw_arguments = "337 113569";
    info!("inputs: {}", raw_arguments);
    let arguments = raw_arguments
        .split(' ')
        .map(|x| T::try_from_dec_str(x))
        .map(|x| x.unwrap())
        .collect::<Vec<T>>();

    let interpreter = zokrates_interpreter::Interpreter::default();
    let public_inputs = ir_prog.public_inputs();

    let witness = interpreter
        .execute(
            &arguments,
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
