use quote::quote;
use syn::parse_quote;
use std::io::Write;
use syn::Stmt;

mod auxfuncs;
mod helpers;
mod hyperbolic;
mod inv_trig;
mod log_exp;
mod recip_sqrt;
mod test;
mod trig;

use auxfuncs::*;
use hyperbolic::*;
use inv_trig::*;
use log_exp::*;
use recip_sqrt::*;
use trig::*;
use doctor_syn::codegen::c;

fn generate_libm(path: &str, num_bits: usize, number_type: &str, language: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut file = std::fs::File::create(path)?;

    let (trig, trig_tests) = gen_single_pass_trig(num_bits, number_type);
    let (inv_trig, inv_trig_tests) = gen_inv_trig(num_bits, number_type);
    let (log_exp, log_exp_tests) = gen_log_exp(num_bits, number_type);
    let (hyperbolic, hyperbolic_tests) = gen_hyperbolic(num_bits, number_type);
    let (recip_sqrt, recip_sqrt_tests) = gen_recip_sqrt(num_bits, number_type);
    let (aux, aux_tests) = gen_aux(num_bits, number_type);

    let functions : Vec<Stmt> = parse_quote!(
        #trig
        #inv_trig
        #log_exp
        #hyperbolic
        #recip_sqrt
        #aux
    );

    let tests : Vec<Stmt> = parse_quote!(
        #trig_tests
        #inv_trig_tests
        #log_exp_tests
        #hyperbolic_tests
        #recip_sqrt_tests
        #aux_tests
    );

    match language {
        "rust" => {
            for stmt in functions.iter().chain(tests.iter()) {
                let tokens = quote!(#stmt);
                file.write_all(tokens.to_string().as_bytes())?;
            }
        }
        "c" => {
            file.write_all(b"#include<math.h>\n")?;
            file.write_all(b"\n")?;
            file.write_all(b"inline float mul_add(float a, float b, float c) {\n")?;
            file.write_all(b"    return a * b + c;\n")?;
            file.write_all(b"}\n")?;
            file.write_all(b"\n")?;
            file.write_all(b"inline float from_bits(unsigned x) {\n")?;
            file.write_all(b"    union {\n")?;
            file.write_all(b"        float f;\n")?;
            file.write_all(b"        unsigned x;\n")?;
            file.write_all(b"    } u;\n")?;
            file.write_all(b"    u.x = x;\n")?;
            file.write_all(b"    return u.f;\n")?;
            file.write_all(b"}\n")?;
            file.write_all(b"\n")?;
            file.write_all(b"typedef float f32;\n")?;
            file.write_all(b"\n")?;

            for stmt in functions.iter().chain(tests.iter()) {
                if let Stmt::Item(item) = stmt {
                    use c::AsC;
                    let context = c::Context::new();
                    let code = item.as_c(&context)?;
                    file.write_all(code.as_bytes())?;
                }
            }
        }
        language => panic!("language {} not supported", language),
    }

    Ok(())
}

fn main() {
    // let val = doctor_syn::expr!((2.16065388452409390396).cos()).eval(60).unwrap();
    // let bd : doctor_syn::bigdecimal::BigDecimal = val.into();
    //let val = doctor_syn::expr!(123456789.123456789123456789123456789123456789).eval(60).unwrap();
    //let bd : bigdecimal::BigDecimal = "123456789.123456789123456789123456789123456789".parse().unwrap();
    //let val = doctor_syn::expr!(123456789.123456789123456789123456789123456789);
    //let val = doctor_syn::Expression::from(bd);
    // println!("val={}", val);
    // println!("bd={}", bd);

    // generate_libm("tests/libm32.rs", 32, "f32_hex", "rust").unwrap();
    // generate_libm("tests/libm64.rs", 64, "f64_hex", "rust").unwrap();

    generate_libm("tests/libm32.c", 32, "f32_hex", "c").unwrap();
    // generate_libm("tests/libm64.c", 64, "f64_hex", "c").unwrap();
}
