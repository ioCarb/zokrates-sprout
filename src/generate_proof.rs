use std::io::{Read, Seek};

use rand::{rngs::StdRng, SeedableRng};
use zokrates_ark::Ark;
use zokrates_ast::ir::{self, ProgEnum, Witness};
use zokrates_common::helpers::{BackendParameter, CurveParameter, Parameters, SchemeParameter};
use zokrates_field::Field;
use zokrates_proof_systems::{Backend, Marlin, Scheme, G16, GM17};

pub fn compute_proof_wrapper(
    mut prog: impl Read + Seek,
    proving_key: impl Read,
    witness: impl Read,
    scheme: &str,
) -> Result<String, String> {
    let prog = ProgEnum::deserialize(&mut prog)?;

    let curve_parameter = CurveParameter::try_from(prog.curve())?;

    let backend_parameter = BackendParameter::Ark;

    let scheme_parameter = SchemeParameter::try_from(scheme)?;

    let parameters = Parameters(backend_parameter, curve_parameter, scheme_parameter);

    let proof = match parameters {
        Parameters(BackendParameter::Ark, _, SchemeParameter::G16) => match prog {
            ProgEnum::Bn128Program(p) => compute_proof::<_, _, G16, Ark>(p, proving_key, witness),
            ProgEnum::Bls12_381Program(p) => {
                compute_proof::<_, _, G16, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bls12_377Program(p) => {
                compute_proof::<_, _, G16, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bw6_761Program(p) => compute_proof::<_, _, G16, Ark>(p, proving_key, witness),
            _ => unreachable!(),
        },

        Parameters(BackendParameter::Ark, _, SchemeParameter::GM17) => match prog {
            ProgEnum::Bn128Program(p) => compute_proof::<_, _, GM17, Ark>(p, proving_key, witness),
            ProgEnum::Bls12_381Program(p) => {
                compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bls12_377Program(p) => {
                compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bw6_761Program(p) => {
                compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            }
            _ => unreachable!(),
        },

        Parameters(BackendParameter::Ark, _, SchemeParameter::MARLIN) => match prog {
            ProgEnum::Bn128Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bls12_381Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bls12_377Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            ProgEnum::Bw6_761Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }?;

    Ok(proof)
}

pub fn compute_proof<
    'a,
    T: Field,
    I: Iterator<Item = ir::Statement<'a, T>>,
    S: Scheme<T>,
    B: Backend<T, S>,
>(
    ir_prog: ir::ProgIterator<'a, T, I>,
    proving_key: impl Read,
    witness: impl Read,
) -> Result<String, String> {
    let witness = Witness::read(witness).map_err(|e| format!("couldnt read witness {:?}", e))?;
    let mut rng = StdRng::from_entropy(); // think about different entropy sources

    let proof = B::generate_proof(ir_prog, witness, proving_key, &mut rng);
    let proof = serde_json::to_string_pretty(&zokrates_proof_systems::TaggedProof::<T, S>::new(
        proof.proof,
        proof.inputs,
    ))
    .unwrap();

    Ok(proof)
}
