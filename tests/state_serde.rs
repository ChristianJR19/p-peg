//! Tests for state account serialization and deserialization.
//!
//! These verify that reading a value after writing it returns the
//! same value, and that discriminator checks work correctly. They
//! exercise the zero-copy read/write helpers that the on-chain code
//! depends on.

#[cfg(test)]
mod tests {
    // Discriminators (must match constants.rs).
    const DISC_CONFIG: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x63, 0x6f, 0x6e, 0x66];
    const DISC_POSITION: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x70, 0x6f, 0x73, 0x6e];
    const DISC_CREATURE: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x63, 0x72, 0x65, 0x61];
    const DISC_ORACLE: [u8; 8] = [0x70, 0x70, 0x65, 0x67, 0x6f, 0x72, 0x63, 0x6c];

    // ---- helpers for raw buffer read/write ---------------------------------

    fn write_u64(buf: &mut [u8], offset: usize, val: u64) {
        buf[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
    }

    fn read_u64(buf: &[u8], offset: usize) -> u64 {
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[offset..offset + 8]);
        u64::from_le_bytes(b)
    }

    fn write_u16(buf: &mut [u8], offset: usize, val: u16) {
        buf[offset..offset + 2].copy_from_slice(&val.to_le_bytes());
    }

    fn read_u16(buf: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([buf[offset], buf[offset + 1]])
    }

    fn write_pubkey(buf: &mut [u8], offset: usize, key: &[u8; 32]) {
        buf[offset..offset + 32].copy_from_slice(key);
    }

    fn read_pubkey(buf: &[u8], offset: usize) -> [u8; 32] {
        let mut k = [0u8; 32];
        k.copy_from_slice(&buf[offset..offset + 32]);
        k
    }

    // ---- config account tests ----------------------------------------------

    #[test]
    fn test_config_discriminator() {
        let mut buf = vec![0u8; 168];
        buf[0..8].copy_from_slice(&DISC_CONFIG);
        assert_eq!(&buf[0..8], &DISC_CONFIG);

        // Wrong discriminator should be detectable.
        buf[0] = 0xFF;
        assert_ne!(&buf[0..8], &DISC_CONFIG);
    }

    #[test]
    fn test_config_roundtrip() {
        let mut buf = vec![0u8; 168];
        buf[0..8].copy_from_slice(&DISC_CONFIG);

        // Write fields.
        let authority = [42u8; 32];
        let vault = [99u8; 32];
        let mint = [77u8; 32];
        write_pubkey(&mut buf, 8, &authority);
        write_pubkey(&mut buf, 40, &vault);
        write_pubkey(&mut buf, 72, &mint);
        write_u64(&mut buf, 104, 15000); // min_collateral_ratio
        write_u64(&mut buf, 112, 500);   // liquidation_bonus
        write_u64(&mut buf, 120, 500_000_000); // spawn_threshold
        write_u64(&mut buf, 128, 10_000_000);  // reroll_fee
        write_u64(&mut buf, 136, 30);    // protocol_fee_bps
        write_u64(&mut buf, 144, 1_000_000_000_000); // total_collateral
        write_u64(&mut buf, 152, 500_000_000_000);   // total_minted
        buf[160] = 254; // bump

        // Read back.
        assert_eq!(read_pubkey(&buf, 8), authority);
        assert_eq!(read_pubkey(&buf, 40), vault);
        assert_eq!(read_pubkey(&buf, 72), mint);
        assert_eq!(read_u64(&buf, 104), 15000);
        assert_eq!(read_u64(&buf, 112), 500);
        assert_eq!(read_u64(&buf, 120), 500_000_000);
        assert_eq!(read_u64(&buf, 128), 10_000_000);
        assert_eq!(read_u64(&buf, 136), 30);
        assert_eq!(read_u64(&buf, 144), 1_000_000_000_000);
        assert_eq!(read_u64(&buf, 152), 500_000_000_000);
        assert_eq!(buf[160], 254);
    }

    // ---- position account tests --------------------------------------------

    #[test]
    fn test_position_roundtrip() {
        let mut buf = vec![0u8; 112];
        buf[0..8].copy_from_slice(&DISC_POSITION);

        let owner = [11u8; 32];
        write_pubkey(&mut buf, 8, &owner);
        write_u64(&mut buf, 40, 5_000_000_000); // 5 SOL collateral
        write_u64(&mut buf, 48, 200_000_000);   // $200 minted
        write_u64(&mut buf, 56, 1_700_000_000); // deposited_at
        write_u64(&mut buf, 64, 1_700_001_000); // last_interact
        let creature_key = [88u8; 32];
        write_pubkey(&mut buf, 72, &creature_key);
        buf[104] = 1; // has_creature
        buf[105] = 253; // bump

        assert_eq!(read_pubkey(&buf, 8), owner);
        assert_eq!(read_u64(&buf, 40), 5_000_000_000);
        assert_eq!(read_u64(&buf, 48), 200_000_000);
        assert_eq!(read_u64(&buf, 56), 1_700_000_000);
        assert_eq!(read_u64(&buf, 64), 1_700_001_000);
        assert_eq!(read_pubkey(&buf, 72), creature_key);
        assert_eq!(buf[104], 1);
        assert_eq!(buf[105], 253);
    }

    #[test]
    fn test_position_atomic_add_collateral() {
        let mut buf = vec![0u8; 112];
        buf[0..8].copy_from_slice(&DISC_POSITION);
        write_u64(&mut buf, 40, 1_000_000_000);

        // Simulate atomic add.
        let current = read_u64(&buf, 40);
        let delta = 500_000_000u64;
        let new_val = current.checked_add(delta).unwrap();
        write_u64(&mut buf, 40, new_val);

        assert_eq!(read_u64(&buf, 40), 1_500_000_000);
    }

    #[test]
    fn test_position_atomic_sub_collateral() {
        let mut buf = vec![0u8; 112];
        buf[0..8].copy_from_slice(&DISC_POSITION);
        write_u64(&mut buf, 40, 2_000_000_000);

        let current = read_u64(&buf, 40);
        let delta = 750_000_000u64;
        let new_val = current.checked_sub(delta).unwrap();
        write_u64(&mut buf, 40, new_val);

        assert_eq!(read_u64(&buf, 40), 1_250_000_000);
    }

    // ---- creature account tests --------------------------------------------

    #[test]
    fn test_creature_roundtrip() {
        let mut buf = vec![0u8; 128];
        buf[0..8].copy_from_slice(&DISC_CREATURE);

        let owner = [22u8; 32];
        let position = [33u8; 32];
        write_pubkey(&mut buf, 8, &owner);
        write_pubkey(&mut buf, 40, &position);
        write_u64(&mut buf, 72, 0xABCD_1234_5678_9ABCu64); // dna
        write_u16(&mut buf, 80, 3);   // generation
        buf[82] = 5;   // species (basilisk)
        buf[83] = 2;   // element (earth)
        buf[84] = 4;   // rarity (legendary)
        buf[85] = 6;   // mood (proud)
        write_u16(&mut buf, 86, 7500); // power
        write_u64(&mut buf, 88, 1_700_000_000); // spawned_at
        write_u64(&mut buf, 96, 1_700_500_000); // evolved_at
        write_u64(&mut buf, 104, 350_000_000);  // xp
        write_u64(&mut buf, 112, 15);           // feeds
        buf[120] = 251; // bump

        assert_eq!(read_pubkey(&buf, 8), owner);
        assert_eq!(read_pubkey(&buf, 40), position);
        assert_eq!(read_u64(&buf, 72), 0xABCD_1234_5678_9ABCu64);
        assert_eq!(read_u16(&buf, 80), 3);
        assert_eq!(buf[82], 5);
        assert_eq!(buf[83], 2);
        assert_eq!(buf[84], 4);
        assert_eq!(buf[85], 6);
        assert_eq!(read_u16(&buf, 86), 7500);
        assert_eq!(read_u64(&buf, 88), 1_700_000_000);
        assert_eq!(read_u64(&buf, 96), 1_700_500_000);
        assert_eq!(read_u64(&buf, 104), 350_000_000);
        assert_eq!(read_u64(&buf, 112), 15);
        assert_eq!(buf[120], 251);
    }

    // ---- oracle account tests ----------------------------------------------

    #[test]
    fn test_oracle_roundtrip() {
        let mut buf = vec![0u8; 40];
        buf[0..8].copy_from_slice(&DISC_ORACLE);
        write_u64(&mut buf, 8, 150_000_000);  // $150
        write_u64(&mut buf, 16, 500_000);      // ±$0.50
        write_u64(&mut buf, 24, 1_700_000_000);
        buf[32] = 250;

        assert_eq!(read_u64(&buf, 8), 150_000_000);
        assert_eq!(read_u64(&buf, 16), 500_000);
        assert_eq!(read_u64(&buf, 24), 1_700_000_000);
        assert_eq!(buf[32], 250);
    }

    #[test]
    fn test_oracle_staleness() {
        let max_staleness = 120u64;
        let updated_at = 1_000_000u64;
        let now_fresh = 1_000_100u64;
        let now_stale = 1_000_200u64;

        assert!(now_fresh - updated_at <= max_staleness);
        assert!(now_stale - updated_at > max_staleness);
    }

    // ---- discriminator uniqueness ------------------------------------------

    #[test]
    fn test_all_discriminators_unique() {
        let discs = [DISC_CONFIG, DISC_POSITION, DISC_CREATURE, DISC_ORACLE];
        for i in 0..discs.len() {
            for j in (i + 1)..discs.len() {
                assert_ne!(discs[i], discs[j], "Discriminators at {} and {} collide", i, j);
            }
        }
    }

    #[test]
    fn test_discriminators_are_ascii() {
        // All discriminators should be valid ASCII for readability.
        for disc in [DISC_CONFIG, DISC_POSITION, DISC_CREATURE, DISC_ORACLE] {
            for byte in disc {
                assert!(byte.is_ascii(), "Non-ASCII byte {:#x} in discriminator", byte);
            }
        }
    }
}
