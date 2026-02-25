pub fn get_amount_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> u64 {

    let fee_denominator = 10_000u128;

    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;

    // apply fee
    let amount_in_with_fee =
        amount_in * (fee_denominator - fee_bps as u128);

    let numerator = amount_in_with_fee * reserve_out;
    let denominator =
        reserve_in * fee_denominator + amount_in_with_fee;

    (numerator / denominator) as u64
}