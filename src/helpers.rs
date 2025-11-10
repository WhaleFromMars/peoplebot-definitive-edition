use crate::prelude::*;

/// Returns the maximum attachment size limit for a guild.
pub fn attachment_byte_limit(ctx: &Context, guild_id: Option<GuildId>) -> u64 {
    let tier = guild_id
        .and_then(|id| {
            ctx.serenity_context()
                .cache
                .guild(id)
                .map(|guild| guild.premium_tier)
        })
        .unwrap_or(PremiumTier::Tier0);

    match tier {
        PremiumTier::Tier0 | PremiumTier::Tier1 => 10 * 1_000_000,
        PremiumTier::Tier2 => 50 * 1_000_000,
        PremiumTier::Tier3 => 100 * 1_000_000,
        _ => 10 * 1_000_000,
    }
}

/// Formats a byte count into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    //least overengineered helper, thanks claude
    const UNITS: [(&str, u64); 6] = [
        ("PB", 1_000_000_000_000_000),
        ("TB", 1_000_000_000_000),
        ("GB", 1_000_000_000), // if anything larger than this gets hit someone dies
        ("MB", 1_000_000),
        ("KB", 1_000),
        ("B", 1),
    ];

    if bytes == 0 {
        return "0 B".to_string();
    }

    for &(unit, factor) in UNITS.iter() {
        if bytes >= factor {
            // For bytes, keep it as a plain integer.
            if factor == 1 {
                return format!("{bytes} {unit}");
            }

            // Scale by 100 to keep two decimal places using integer math.
            let decimals: u32 = 2;
            let scale: u128 = 10u128.pow(decimals);
            let factor128 = factor as u128;
            let bytes128 = bytes as u128;

            // Rounded value with two decimal digits: round(bytes/factor * 10^decimals)
            let scaled = (bytes128 * scale + factor128 / 2) / factor128;

            let int_part = (scaled / scale) as u64;
            let frac_part = (scaled % scale) as u64;

            // Format with up to two decimals (trim trailing zeros).
            if frac_part == 0 {
                // No decimals needed
                return format!("{int_part} {unit}");
            } else if frac_part % 10 == 0 {
                // One decimal place is enough, e.g. 1.20 -> 1.2
                return format!("{}.{} {unit}", int_part, frac_part / 10);
            } else {
                // Full two decimal places
                return format!("{int_part}.{:02} {unit}", frac_part);
            }
        }
    }

    // Should be unreachable, as "B" with factor 1 catches everything > 0
    "0 B".to_string()
}
