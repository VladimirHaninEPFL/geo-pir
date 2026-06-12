use spiral_rs::params::Params;

use crate::db_settings::LogicalDatabase;

// params for the protocol
const POLY_LEN: usize = 2048;
const CRT_MODULI: [u64; 2] = [268369921u64, 249561089u64];
const NOISE_WIDTH: f64 = 6.4;
const PIR_N: usize = 1;
const PT_MODULUS: u64 = 512;
const Q2_BITS: u64 = 21;
const T_CONV: usize = 4;
const T_EXP_LEFT: usize = 16;
const T_EXP_RIGHT: usize = 56;
const T_GSW: usize = 10;
const EXPAND_QUERIES: bool = true;
const MAX_DB_DIM_1: usize = 10;

fn pir_item_capacity_bytes(n: usize, instances: usize, pt_modulus: u64) -> usize {
    let pt_modulus_bits = pt_modulus.ilog2() as usize;
    instances * n * n * POLY_LEN * pt_modulus_bits / 8
}

fn bytes_per_chunk(db_item_size: usize, instances: usize, n: usize) -> usize {
    let trials = n * n;
    db_item_size.div_ceil(instances * trials)
}

fn modp_words_per_chunk(db_item_size: usize, instances: usize, n: usize, pt_modulus: u64) -> usize {
    let pt_modulus_bits = pt_modulus.ilog2() as usize;
    let chunk_bytes = bytes_per_chunk(db_item_size, instances, n);
    (chunk_bytes * 8).div_ceil(pt_modulus_bits)
}

fn decoded_bytes_per_chunk(
    db_item_size: usize,
    instances: usize,
    n: usize,
    pt_modulus: u64,
) -> usize {
    let pt_modulus_bits = pt_modulus.ilog2() as usize;
    let words = modp_words_per_chunk(db_item_size, instances, n, pt_modulus);
    (words * pt_modulus_bits) / 8
}

fn packed_item_roundtrips(db_item_size: usize, instances: usize) -> bool {
    let words = modp_words_per_chunk(db_item_size, instances, PIR_N, PT_MODULUS);
    let chunk_bytes = bytes_per_chunk(db_item_size, instances, PIR_N);
    let decoded_chunk_bytes = decoded_bytes_per_chunk(db_item_size, instances, PIR_N, PT_MODULUS);

    words <= POLY_LEN && decoded_chunk_bytes == chunk_bytes
}

fn choose_db_dimensions(packed_items_needed: usize) -> (usize, usize, usize) {
    let packed_items_capacity = usize::max(2, packed_items_needed.next_power_of_two());
    let total_log2 = packed_items_capacity.ilog2() as usize;
    let db_dim_1 = MAX_DB_DIM_1.min(total_log2.saturating_sub(1));
    let db_dim_2 = total_log2 - db_dim_1;

    (packed_items_capacity, db_dim_1, db_dim_2)
}

pub struct DerivedPirLayout {
    pub params: Params,
    pub records_per_pir_item: usize,
}

pub fn  make_params(logical_db: &LogicalDatabase) -> DerivedPirLayout {
    assert!(logical_db.num_records > 0);
    assert!(logical_db.record_size_bytes > 0);

    let single_instance_capacity_bytes = pir_item_capacity_bytes(PIR_N, 1, PT_MODULUS);

    let mut instances = logical_db
        .record_size_bytes
        .div_ceil(single_instance_capacity_bytes);

    let (records_per_pir_item, packed_db_item_size) = loop {

        let item_capacity_bytes = pir_item_capacity_bytes(PIR_N, instances, PT_MODULUS);
        let records_per_pir_item =
            usize::max(1, item_capacity_bytes / logical_db.record_size_bytes);
        let packed_db_item_size = records_per_pir_item * logical_db.record_size_bytes;

        if packed_item_roundtrips(packed_db_item_size, instances) {
            break (
                records_per_pir_item,
                packed_db_item_size,
            );
        }

        instances += 1;
    };

    assert!(records_per_pir_item > 0);

    let packed_items_needed = logical_db.num_records.div_ceil(records_per_pir_item);
    let (_packed_items_capacity, db_dim_1, db_dim_2) = choose_db_dimensions(packed_items_needed);

    let params = Params::init(
        POLY_LEN,
        &CRT_MODULI,
        NOISE_WIDTH,
        PIR_N,
        PT_MODULUS,
        Q2_BITS,
        T_CONV,
        T_EXP_LEFT,
        T_EXP_RIGHT,
        T_GSW,
        EXPAND_QUERIES,
        db_dim_1,
        db_dim_2,
        instances,
        packed_db_item_size,
    );

    DerivedPirLayout {
        params,
        records_per_pir_item,
    }
}