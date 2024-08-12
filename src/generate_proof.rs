use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::{JsonAbi, Param};
use alloy_primitives::U256;
use alloy_sol_types::sol;
use rand::{rngs::StdRng, SeedableRng};
use ruint::{uint, Uint};
use serde_json::Value;
use std::str::{self, FromStr};
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};
use zokrates_ark::Ark;
use zokrates_ast::ir::{self, ProgEnum, Witness};
use zokrates_common::helpers::{BackendParameter, CurveParameter, Parameters, SchemeParameter};
use zokrates_field::{Bn128Field, Field};
use zokrates_proof_systems::{
    Backend, Marlin, Proof, Scheme, SolidityCompatibleField, SolidityCompatibleScheme, G16, GM17,
};
fn encode_single(ty: &str, input: &serde_json::Value) -> DynSolValue {
    // TODO handle more types
    match ty {
        "uint256" => {
            let input = input.to_string();
            // println!("asdf: {}", input);
            return DynSolValue::from(
                U256::from_str_radix(&input.trim_matches('"')[2..], 16).unwrap(),
            );
        }
        _ => unreachable!(),
    }
}
fn abi_encode(param: &Param, inputs: &serde_json::Value) -> DynSolValue {
    // println!("inputs: {:?}", inputs);
    // println!("param: {:?}", param);
    match param.ty.as_str() {
        "tuple" => {
            match inputs {
                Value::Array(arr) => {
                    return DynSolValue::Tuple(
                        param
                            .components
                            .iter()
                            .zip(inputs.as_array().unwrap().iter())
                            .map(|(component, input)| abi_encode(component, input))
                            .collect(),
                    );
                }
                Value::Object(o) => {
                    return DynSolValue::Tuple(
                        param
                            .components
                            .iter()
                            .zip(inputs.as_object().unwrap().iter())
                            .map(|(component, (_, input))| abi_encode(component, input))
                            .collect(),
                    );
                }
                _ => unreachable!(),
            };
        }
        // encode array like uint256[4]
        s if s.contains("[") => {
            let (ty, _) = s.split_once('[').unwrap_or(("",s));
            // TODO: check for matching size etc.
            return DynSolValue::FixedArray(
                inputs
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|input| encode_single(ty, input))
                    .collect(),
            );
        }
        _ => return encode_single(&param.ty, &inputs),
    };
}

pub fn compute_proof_wrapper(
    mut circuit: impl Read + Seek,
    proving_key: impl Read,
    witness: impl Read,
    scheme: &str,
) -> Result<String, String> {
    let abi = BufReader::new(File::open("abi.json").unwrap());
    let abi: JsonAbi = serde_json::from_reader(abi).unwrap();

    let program = ProgEnum::deserialize(&mut circuit)?;
    let curve_parameter = CurveParameter::try_from(program.curve())?;
    let backend_parameter = BackendParameter::Ark;
    let scheme_parameter = SchemeParameter::try_from(scheme)?;
    let parameters = Parameters(backend_parameter, curve_parameter, scheme_parameter);

    let proof = match parameters {
        Parameters(BackendParameter::Ark, _, SchemeParameter::G16) => match program {
            ProgEnum::Bn128Program(p) => compute_proof::<_, _, G16, Ark>(p, proving_key, witness),
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },
        Parameters(BackendParameter::Ark, _, SchemeParameter::GM17) => match program {
            ProgEnum::Bn128Program(p) => compute_proof::<_, _, GM17, Ark>(p, proving_key, witness),
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },
        Parameters(BackendParameter::Ark, _, SchemeParameter::MARLIN) => match program {
            ProgEnum::Bn128Program(p) => {
                compute_proof::<_, _, Marlin, Ark>(p, proving_key, witness)
            }
            _ => Err("Unsupported curve! currently only bn128 is supported".to_string()),
        },
        _ => unreachable!(),
    }?;
    println!("proof: {:?}", proof);

    let function_name = "verifyTx";
    let func = &abi.function(function_name).unwrap()[0];

    let inputs1 = abi_encode(&func.inputs[0], &proof["proof"]);
    let inputs2 = abi_encode(&func.inputs[1], &proof["inputs"]);

    println!("in: {:?}", inputs1);
    println!("in: {:?}", inputs2);

    let asdf = abi.function(function_name).unwrap()[0]
        .abi_encode_input(&[inputs1, inputs2])
        .unwrap();

    let enc = hex::encode(asdf);

    println!("enc: {}", enc);

    Ok(enc.to_string())
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
) -> Result<serde_json::Value, String> {
    let witness = Witness::read(witness).map_err(|e| format!("couldnt read witness {:?}", e))?;
    let mut rng = StdRng::from_entropy(); // think about different entropy sources

    let proof = B::generate_proof(ir_prog, witness, proving_key, &mut rng);

    let p: serde_json::Value =
        serde_json::to_value(&proof) // FIXME: this might be enough, no tagged proof and params needed?
            .map_err(|why| format!("Could not deserialize proof: {}", why))?;

    let proof: serde_json::Value = serde_json::to_value(
        &zokrates_proof_systems::TaggedProof::<T, S>::new(proof.proof, proof.inputs),
    )
    .map_err(|why| format!("Could not deserialize proof: {}", why))?;

    return Ok(p);

    let curve = proof
        .get("curve")
        .ok_or_else(|| "Field `curve` not found in proof".to_string())?
        .as_str()
        .ok_or_else(|| "`curve` should be a string".to_string())?;

    let scheme = proof
        .get("scheme")
        .ok_or_else(|| "Field `scheme` not found in proof".to_string())?
        .as_str()
        .ok_or_else(|| "`scheme` should be a string".to_string())?;

    let parameters: (CurveParameter, SchemeParameter) =
        (curve.try_into().unwrap(), scheme.try_into().unwrap());

    match parameters {
        (CurveParameter::Bn128, SchemeParameter::G16) => {
            format_proof::<Bn128Field, G16>(proof)
        }
        (CurveParameter::Bn128, SchemeParameter::GM17) => {
            format_proof::<Bn128Field, GM17>( proof)
        }
        (CurveParameter::Bn128, SchemeParameter::MARLIN) => {
            format_proof::<Bn128Field, Marlin>( proof)
        }
        _ => Err(format!("Could not print proof with given parameters (curve: {}, scheme: {}): only bn128 is supported", curve, scheme))
    }
}

pub fn format_proof<T: SolidityCompatibleField, S: SolidityCompatibleScheme<T>>(
    proof: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let proof: Proof<T, S> = serde_json::from_value(proof).map_err(|why| format!("{:?}", why))?;

    let res = S::Proof::from(proof.proof);
    let proof_object =
        serde_json::to_value(&res).map_err(|e| format!("cant get result value {}", e))?;

    let inputs =
        serde_json::to_value(&proof.inputs).map_err(|e| format!("cant get inputs value {}", e))?;

    // let encoded = [parse_input(&proof_object), (parse_input(&inputs))].concat();

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
    result.push(']');

    if !proof.inputs.is_empty() {
        result.push(',');
        result.push_str(&inputs.to_string());
    }

    // println!("result: {}", result);

    // let result = result.replace("\"", "'");

    Ok(serde_json::Value::from_str(&result).unwrap())
}

// format_proof(proof)

// pub fn format_proof<T: SolidityCompatibleField, S: SolidityCompatibleScheme<T>>(
//     proof: Proof<T, S>,
// ) -> Result<String, String> {
//     let res: <S as SolidityCompatibleScheme<T>>::Proof = S::Proof::from(proof.proof);
//     let proof_object =
//         serde_json::to_value(&res).map_err(|e| format!("cant get result value {}", e))?;

//     let inputs =
//         serde_json::to_value(&proof.inputs).map_err(|e| format!("cant get inputs value {}", e))?;

//     let mut result = String::new();

//     result.push_str(
//         &proof_object
//             .as_object()
//             .unwrap()
//             .iter()
//             .map(|(_, value)| value.to_string())
//             .collect::<String>(),
//     );

//     if !proof.inputs.is_empty() {
//         result.push_str(&inputs.to_string());
//     }

//     result.retain(|c| c != ',');
//     result.retain(|c| c != '[' && c != ']');
//     result.retain(|c| c != '"');

//     Ok(result)
// }
