use super::*;
use ckb_testtool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use ckb_testtool::context::Context;

const MAX_CYCLES: u64 = 30_000_000;

#[test]
fn test_customized_rule_tx() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("rhai-demo");
    let out_point = context.deploy_cell(contract_bin);

    // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from("ABC"))
        .expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity((20000 * 100_000_000u64).pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![CellOutput::new_builder()
        .capacity((10000 * 100_000_000u64).pack())
        .lock(lock_script.clone())
        .build()];
    let outputs_data = vec![Bytes::new()];

    // prepare witnesses
    let customized_rule = r#"
    let price_tiers = [50000, 20000, 10000, 5000, 2000, 1000];
    let len = account_chars.len();
    if len > 6 {
        100
    } else {
        price_tiers[len - 1]
    }"#;

    let witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(customized_rule)).pack())
        .build();

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .cell_dep(lock_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
