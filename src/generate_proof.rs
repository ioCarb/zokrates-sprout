use std::io::{Read, Seek};

use rand::{rngs::StdRng, SeedableRng};
use zokrates_ark::Ark;
use zokrates_ast::ir::{self, ProgEnum, Witness};
use zokrates_common::helpers::{BackendParameter, CurveParameter, Parameters, SchemeParameter};
use zokrates_field::Field;
use zokrates_proof_systems::{
    Backend, Marlin, Proof, Scheme, SolidityCompatibleField, SolidityCompatibleScheme, G16, GM17,
};

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
            // ProgEnum::Bls12_381Program(p) => {
            //     compute_proof::<_, _, G16, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bls12_377Program(p) => {
            //     compute_proof::<_, _, G16, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bw6_761Program(p) => compute_proof::<_, _, G16, Ark>(p, proving_key, witness),
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },

        Parameters(BackendParameter::Ark, _, SchemeParameter::GM17) => match prog {
            ProgEnum::Bn128Program(p) => compute_proof::<_, _, GM17, Ark>(p, proving_key, witness),
            // ProgEnum::Bls12_381Program(p) => {
            //     compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bls12_377Program(p) => {
            //     compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bw6_761Program(p) => {
            //     compute_proof::<_, _, GM17, Ark>(p, proving_key, witness)
            // }
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },

        Parameters(BackendParameter::Ark, _, SchemeParameter::MARLIN) => match prog {
            ProgEnum::Bn128Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            // ProgEnum::Bls12_381Program(p) => {
            //     compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bls12_377Program(p) => {
            //     compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            // }
            // ProgEnum::Bw6_761Program(p) => {
            //     compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            // }
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },
        _ => unreachable!(),
    }?;

    Ok(proof)
}

fn compute_proof<
    'a,
    T: Field + SolidityCompatibleField,
    I: Iterator<Item = ir::Statement<'a, T>>,
    S: Scheme<T> + SolidityCompatibleScheme<T>,
    B: Backend<T, S>,
>(
    ir_prog: ir::ProgIterator<'a, T, I>,
    proving_key: impl Read,
    witness: impl Read,
) -> Result<String, String> {
    let witness = Witness::read(witness).map_err(|e| format!("couldnt read witness {:?}", e))?;
    let mut rng = StdRng::from_entropy(); // think about different entropy sources

    let proof = B::generate_proof(ir_prog, witness, proving_key, &mut rng);
    // let proof = serde_json::to_string_pretty(&zokrates_proof_systems::TaggedProof::<T, S>::new(
    //     proof.proof,
    //     proof.inputs,
    // ))
    // .unwrap();

    format_proof(proof)
}

pub fn format_proof<T: SolidityCompatibleField, S: SolidityCompatibleScheme<T>>(
    proof: Proof<T, S>,
) -> Result<String, String> {
    let res: <S as SolidityCompatibleScheme<T>>::Proof = S::Proof::from(proof.proof);
    let proof_object =
        serde_json::to_value(&res).map_err(|e| format!("cant get result value {}", e))?;

    let inputs =
        serde_json::to_value(&proof.inputs).map_err(|e| format!("cant get inputs value {}", e))?;

    let mut result = String::from("[");

    result.push_str(
        &proof_object
            .as_object()
            .unwrap()
            .iter()
            .map(|(_, value)| value.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    );

    if !proof.inputs.is_empty() {
        result.push(',');
        result.push_str(&inputs.to_string());
    }
    result.push(']');

    Ok(result)
}
