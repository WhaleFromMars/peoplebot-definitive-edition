use crate::prelude::*;

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
