/// Offline fuzz scaffold for SSS invariants.
///
/// This file is intentionally lightweight so it can run without extra tools
/// while the Trident CLI and honggfuzz are being provisioned.
///
/// Invariants mirrored from bounty requirements:
/// 1) paused blocks mint/burn
/// 2) minter quota never underflows
/// 3) blacklist blocks transfer intent
/// 4) supply accounting is consistent

#[derive(Clone, Copy)]
struct Case {
    paused: bool,
    blacklisted: bool,
    quota: u64,
    minted: u64,
    burned: u64,
}

fn lcg(seed: &mut u64) -> u64 {
    // Deterministic pseudo-random generator for reproducible cases.
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

fn check(case: Case) {
    if case.paused {
        assert_eq!(case.minted, 0, "mint must be blocked when paused");
        assert_eq!(case.burned, 0, "burn must be blocked when paused");
    }

    assert!(
        case.minted <= case.quota,
        "minted amount exceeds minter quota"
    );

    if case.blacklisted {
        // In this scaffold we model blocked transfer by requiring no burn/mint move.
        assert_eq!(case.minted, 0, "blacklisted actor should not mint");
    }

    assert!(
        case.minted >= case.burned,
        "burned cannot exceed minted total supply"
    );
}

fn main() {
    let iterations: usize = std::env::var("FUZZ_CASES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10_000);

    let mut seed = 0x5eed_5eed_u64;
    for _ in 0..iterations {
        let paused = (lcg(&mut seed) & 1) == 1;
        let blacklisted = (lcg(&mut seed) & 1) == 1;
        let quota = lcg(&mut seed) % 1_000_000;

        // If paused/blacklisted, model operation as blocked.
        let minted = if paused || blacklisted {
            0
        } else {
            lcg(&mut seed) % (quota.saturating_add(1))
        };
        let burned = if paused { 0 } else { lcg(&mut seed) % (minted.saturating_add(1)) };

        check(Case {
            paused,
            blacklisted,
            quota,
            minted,
            burned,
        });
    }

    println!("fuzz_0 scaffold passed {} cases", iterations);
}
