use rand::Rng;

use super::mob_template::{
    CombatStats, DamageType, FoodChainTier, LootEntry, MobRegistry, MonsterTemplate,
};
use super::zone::Zone;
use super::area::Area;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAX_MOB_TYPES_PER_AREA: usize = 5;
const BASE_SPAWN_RATE: f32 = 0.50;

// Standard drop template IDs — these must exist in the object registry.
// Defined in base data files; generator references them by ID.
const DROP_MEAT:         &str = "raw_meat";
const DROP_BONE:         &str = "trihelix_bone";
const DROP_HIDE_SOFT:    &str = "hide_soft";
const DROP_FIBER:        &str = "alien_fiber";
const DROP_CHITIN:       &str = "chitin_plate";
const DROP_CRYSTAL:      &str = "coherence_crystal";
const DROP_MEMBRANE:     &str = "resonance_membrane";
const DROP_TOXIC:        &str = "toxic_extract";
const DROP_COH_GLAND:    &str = "coherence_gland";

// ---------------------------------------------------------------------------
// Feature enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Integument { Soft, Fibrous, Chitin, Crystal, Membrane }

#[derive(Debug, Clone, Copy)]
enum SecondaryLimb { Grasping, Bladed, Impact, Spike, SensoryStalk }

#[derive(Debug, Clone, Copy)]
enum SensoryOrgan { Optical, Compound, CoherenceSensitive, Electromagnetic }

#[derive(Debug, Clone, Copy)]
enum Size { Tiny, Small, Medium, Large, Massive }

impl Size {
    fn label(self) -> &'static str {
        match self {
            Size::Tiny    => "tiny",
            Size::Small   => "small",
            Size::Medium  => "medium-sized",
            Size::Large   => "large",
            Size::Massive => "massive",
        }
    }
    fn hp_base(self) -> u32 {
        match self { Size::Tiny => 8, Size::Small => 18, Size::Medium => 35,
                     Size::Large => 65, Size::Massive => 120 }
    }
    fn damage_mult(self) -> f32 {
        match self { Size::Tiny => 0.5, Size::Small => 0.8, Size::Medium => 1.0,
                     Size::Large => 1.5, Size::Massive => 2.5 }
    }
}

// ---------------------------------------------------------------------------
// Generation context
// ---------------------------------------------------------------------------

pub struct GenContext<'a> {
    pub visit_count:    u32,
    pub coherence:      u8,
    pub biome:          &'a str,
    pub existing_tiers: Vec<FoodChainTier>,
    pub existing_count: usize,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns true if the formula decides a new mob should be generated here.
pub fn should_generate(ctx: &GenContext, rng: &mut impl Rng) -> bool {
    if ctx.existing_count >= MAX_MOB_TYPES_PER_AREA {
        return false;
    }
    let density   = 1.0 - (ctx.existing_count as f32 / MAX_MOB_TYPES_PER_AREA as f32);
    let coherence = ctx.coherence as f32 / 100.0;
    let visit     = 1.0 / (ctx.visit_count as f32).sqrt().max(1.0);
    let chain     = food_chain_weight(&ctx.existing_tiers);
    let chance    = BASE_SPAWN_RATE * density * coherence * visit * chain;
    rng.gen_range(0.0f32..1.0) < chance
}

/// Build a complete MonsterTemplate. Call only after `should_generate` returns true.
pub fn generate(ctx: &GenContext, rng: &mut impl Rng) -> MonsterTemplate {
    let tier        = choose_tier(&ctx.existing_tiers, rng);
    let hexaradial  = matches!(tier, FoodChainTier::Apex)
                        || (matches!(tier, FoodChainTier::Predator) && rng.gen_bool(0.25));
    let size        = choose_size(&tier, rng);
    let integument  = choose_integument(&tier, ctx.coherence, rng);
    let secondary   = choose_secondary(&tier, rng);
    let sensory     = choose_sensory(ctx.coherence, rng);
    let coh_pulse   = ctx.coherence >= 70 && rng.gen_bool(0.40);
    let aggressive  = matches!(tier, FoodChainTier::Predator | FoodChainTier::Apex | FoodChainTier::Coherence);

    let name        = gen_name(ctx.biome, &size, rng);
    let short       = format!("a {name}");
    let room_look   = gen_room_look(&name, &tier, &secondary, rng);
    let description = gen_description(&size, hexaradial, &integument, &secondary, &sensory);

    let (hp_min, hp_max) = hp_range(&size, &integument, &tier);
    let combat      = derive_combat(&tier, &size, &secondary, &integument, coh_pulse);
    let loot_table  = derive_loot(&integument, &secondary, &sensory, coh_pulse);
    let chance_loot = match tier {
        FoodChainTier::Grazer    => 50,
        FoodChainTier::Scavenger => 45,
        FoodChainTier::Predator  => 65,
        FoodChainTier::Apex      => 80,
        FoodChainTier::Coherence => 70,
    };
    let respawn = match tier {
        FoodChainTier::Grazer | FoodChainTier::Scavenger => 120,
        FoodChainTier::Predator                          => 240,
        FoodChainTier::Apex | FoodChainTier::Coherence   => 480,
    };

    MonsterTemplate {
        id:               name.clone(),
        names:            vec![name.clone()],
        short,
        description,
        room_look,
        health_min:       hp_min,
        health_max:       hp_max,
        combat,
        stationary:       false,
        wanders:          matches!(tier, FoodChainTier::Grazer | FoodChainTier::Scavenger),
        aggressive,
        follows_aggressive: matches!(tier, FoodChainTier::Apex),
        calls_for_help:   matches!(tier, FoodChainTier::Predator | FoodChainTier::Apex) && rng.gen_bool(0.4),
        detection_range:  if coh_pulse || matches!(sensory, SensoryOrgan::CoherenceSensitive) { 2 } else { 1 },
        flee_threshold:   match tier {
            FoodChainTier::Grazer | FoodChainTier::Scavenger => rng.gen_range(25..=45),
            FoodChainTier::Predator                          => rng.gen_range(10..=20),
            _                                                => 0,
        },
        faction:          None,
        respawn_secs:     respawn,
        chance_of_loot:   chance_loot,
        loot_table,
        food_chain_tier:  tier,
        generated:        true,
    }
}

/// Collect generation context from a zone + area + current mob registry.
pub fn build_context<'a>(
    zone:     &'a Zone,
    area:     &Area,
    registry: &MobRegistry,
    monsters: &std::collections::HashMap<u32, crate::mob::MonsterInstance>,
) -> GenContext<'a> {
    use super::hex::PlayerLocation;

    let loc = PlayerLocation::area(zone.coord, area.id);

    // Collect unique template IDs currently living in this area.
    let area_template_ids: std::collections::HashSet<&str> = monsters.values()
        .filter(|m| !m.dead && m.core.location == loc)
        .map(|m| m.template_id.as_str())
        .collect();

    let existing_tiers: Vec<FoodChainTier> = area_template_ids.iter()
        .filter_map(|id| registry.get(*id))
        .map(|t| t.food_chain_tier.clone())
        .collect();

    GenContext {
        visit_count:    area.visit_count.max(1),
        coherence:      zone.coherence,
        biome:          &zone.biome_origin,
        existing_tiers,
        existing_count: area_template_ids.len(),
    }
}

// ---------------------------------------------------------------------------
// Food chain helpers
// ---------------------------------------------------------------------------

fn food_chain_weight(existing: &[FoodChainTier]) -> f32 {
    let has_base = existing.iter().any(|t| matches!(t, FoodChainTier::Grazer | FoodChainTier::Scavenger));
    let has_pred = existing.iter().any(|t| matches!(t, FoodChainTier::Predator));
    if !has_base { 1.5 }
    else if !has_pred { 1.2 }
    else { 0.8 }
}

fn choose_tier(existing: &[FoodChainTier], rng: &mut impl Rng) -> FoodChainTier {
    let has_base = existing.iter().any(|t| matches!(t, FoodChainTier::Grazer | FoodChainTier::Scavenger));
    let has_pred = existing.iter().any(|t| matches!(t, FoodChainTier::Predator));

    let roll: f32 = rng.gen_range(0.0..1.0);
    if !has_base {
        if roll < 0.75 { FoodChainTier::Grazer } else { FoodChainTier::Scavenger }
    } else if !has_pred {
        if roll < 0.60 { FoodChainTier::Predator }
        else if roll < 0.80 { FoodChainTier::Grazer }
        else { FoodChainTier::Coherence }
    } else {
        if roll < 0.30 { FoodChainTier::Apex }
        else if roll < 0.55 { FoodChainTier::Predator }
        else if roll < 0.75 { FoodChainTier::Grazer }
        else { FoodChainTier::Coherence }
    }
}

// ---------------------------------------------------------------------------
// Feature selection
// ---------------------------------------------------------------------------

fn choose_size(tier: &FoodChainTier, rng: &mut impl Rng) -> Size {
    let roll: f32 = rng.gen_range(0.0..1.0);
    match tier {
        FoodChainTier::Grazer | FoodChainTier::Scavenger =>
            if roll < 0.20 { Size::Tiny } else if roll < 0.60 { Size::Small } else { Size::Medium },
        FoodChainTier::Predator =>
            if roll < 0.20 { Size::Small } else if roll < 0.65 { Size::Medium } else { Size::Large },
        FoodChainTier::Apex =>
            if roll < 0.30 { Size::Large } else { Size::Massive },
        FoodChainTier::Coherence =>
            if roll < 0.40 { Size::Small } else if roll < 0.80 { Size::Medium } else { Size::Large },
    }
}

fn choose_integument(tier: &FoodChainTier, coherence: u8, rng: &mut impl Rng) -> Integument {
    let roll: f32 = rng.gen_range(0.0..1.0);
    let coh_bias = coherence as f32 / 100.0;
    if coh_bias > 0.65 && roll < coh_bias * 0.30 { return Integument::Membrane; }
    if coh_bias > 0.75 && roll < coh_bias * 0.20 { return Integument::Crystal; }
    match tier {
        FoodChainTier::Grazer | FoodChainTier::Scavenger =>
            if roll < 0.50 { Integument::Soft } else if roll < 0.80 { Integument::Fibrous } else { Integument::Chitin },
        FoodChainTier::Predator =>
            if roll < 0.30 { Integument::Soft } else if roll < 0.60 { Integument::Fibrous } else { Integument::Chitin },
        FoodChainTier::Apex =>
            if roll < 0.20 { Integument::Fibrous } else if roll < 0.55 { Integument::Chitin } else { Integument::Crystal },
        FoodChainTier::Coherence =>
            if roll < 0.50 { Integument::Membrane } else { Integument::Crystal },
    }
}

fn choose_secondary(tier: &FoodChainTier, rng: &mut impl Rng) -> SecondaryLimb {
    let roll: f32 = rng.gen_range(0.0..1.0);
    match tier {
        FoodChainTier::Grazer =>
            if roll < 0.60 { SecondaryLimb::Grasping } else { SecondaryLimb::SensoryStalk },
        FoodChainTier::Scavenger =>
            if roll < 0.50 { SecondaryLimb::Grasping } else if roll < 0.75 { SecondaryLimb::Spike } else { SecondaryLimb::SensoryStalk },
        FoodChainTier::Predator =>
            if roll < 0.40 { SecondaryLimb::Bladed } else if roll < 0.70 { SecondaryLimb::Impact } else { SecondaryLimb::Spike },
        FoodChainTier::Apex =>
            if roll < 0.45 { SecondaryLimb::Bladed } else if roll < 0.75 { SecondaryLimb::Impact } else { SecondaryLimb::Spike },
        FoodChainTier::Coherence =>
            if roll < 0.50 { SecondaryLimb::SensoryStalk } else { SecondaryLimb::Grasping },
    }
}

fn choose_sensory(coherence: u8, rng: &mut impl Rng) -> SensoryOrgan {
    let roll: f32 = rng.gen_range(0.0..1.0);
    let coh = coherence as f32 / 100.0;
    if coh > 0.60 && roll < coh * 0.45 { return SensoryOrgan::CoherenceSensitive; }
    if roll < 0.40 { SensoryOrgan::Optical }
    else if roll < 0.70 { SensoryOrgan::Compound }
    else { SensoryOrgan::Electromagnetic }
}

// ---------------------------------------------------------------------------
// Stat derivation
// ---------------------------------------------------------------------------

fn hp_range(size: &Size, integument: &Integument, tier: &FoodChainTier) -> (u32, u32) {
    let base = size.hp_base();
    let int_bonus: u32 = match integument {
        Integument::Soft     => 0,
        Integument::Fibrous  => 5,
        Integument::Chitin   => 12,
        Integument::Crystal  => 18,
        Integument::Membrane => 8,
    };
    let tier_bonus: u32 = match tier {
        FoodChainTier::Grazer | FoodChainTier::Scavenger => 0,
        FoodChainTier::Predator  => 10,
        FoodChainTier::Apex      => 30,
        FoodChainTier::Coherence => 15,
    };
    let min = (base + int_bonus + tier_bonus).saturating_sub(base / 4);
    let max = base + int_bonus + tier_bonus + base / 3;
    (min, max)
}

fn derive_combat(
    tier:       &FoodChainTier,
    size:       &Size,
    secondary:  &SecondaryLimb,
    integument: &Integument,
    coh_pulse:  bool,
) -> CombatStats {
    let mult = size.damage_mult();
    let (base_atk, base_def, base_dmg_min, base_dmg_max, base_xp) = match tier {
        FoodChainTier::Grazer    => (2, 1, 1, 3, 8),
        FoodChainTier::Scavenger => (3, 1, 1, 4, 10),
        FoodChainTier::Predator  => (6, 3, 3, 8, 30),
        FoodChainTier::Apex      => (10, 5, 6, 14, 80),
        FoodChainTier::Coherence => (5, 2, 2, 7, 40),
    };
    let def_bonus: u32 = match integument {
        Integument::Soft     => 0,
        Integument::Fibrous  => 1,
        Integument::Chitin   => 3,
        Integument::Crystal  => 5,
        Integument::Membrane => 2,
    };
    let (attack_type, atk_bonus, dmg_bonus) = match secondary {
        SecondaryLimb::Bladed      => (DamageType::Physical, 2, 2),
        SecondaryLimb::Impact      => (DamageType::Physical, 1, 3),
        SecondaryLimb::Spike       => (DamageType::Physical, 1, 1),
        SecondaryLimb::Grasping    => (DamageType::Physical, 0, 0),
        SecondaryLimb::SensoryStalk => (DamageType::Physical, 0, 0),
    };
    let final_type = if coh_pulse { DamageType::Coherence } else { attack_type };
    CombatStats {
        attack:      base_atk + atk_bonus,
        defense:     base_def + def_bonus,
        damage_min:  ((base_dmg_min + dmg_bonus) as f32 * mult) as u32,
        damage_max:  ((base_dmg_max + dmg_bonus) as f32 * mult) as u32,
        attack_type: final_type,
        xp_value:    (base_xp as f32 * mult) as u32,
        resistances: if matches!(integument, Integument::Crystal | Integument::Membrane) {
            vec![DamageType::Coherence]
        } else { vec![] },
        immunities:  vec![],
    }
}

fn derive_loot(
    integument: &Integument,
    secondary:  &SecondaryLimb,
    sensory:    &SensoryOrgan,
    coh_pulse:  bool,
) -> Vec<LootEntry> {
    let mut loot = vec![
        LootEntry { template_id: DROP_MEAT.into(),  chance: 90, qty_min: 1, qty_max: 3 },
        LootEntry { template_id: DROP_BONE.into(),  chance: 70, qty_min: 1, qty_max: 2 },
    ];
    match integument {
        Integument::Soft     => loot.push(LootEntry { template_id: DROP_HIDE_SOFT.into(), chance: 75, qty_min: 1, qty_max: 1 }),
        Integument::Fibrous  => loot.push(LootEntry { template_id: DROP_FIBER.into(),    chance: 70, qty_min: 1, qty_max: 2 }),
        Integument::Chitin   => loot.push(LootEntry { template_id: DROP_CHITIN.into(),   chance: 65, qty_min: 1, qty_max: 2 }),
        Integument::Crystal  => loot.push(LootEntry { template_id: DROP_CRYSTAL.into(),  chance: 60, qty_min: 1, qty_max: 3 }),
        Integument::Membrane => loot.push(LootEntry { template_id: DROP_MEMBRANE.into(), chance: 55, qty_min: 1, qty_max: 1 }),
    }
    if matches!(secondary, SecondaryLimb::Spike) {
        loot.push(LootEntry { template_id: DROP_TOXIC.into(), chance: 45, qty_min: 1, qty_max: 1 });
    }
    if matches!(sensory, SensoryOrgan::CoherenceSensitive) || coh_pulse {
        loot.push(LootEntry { template_id: DROP_COH_GLAND.into(), chance: 40, qty_min: 1, qty_max: 1 });
    }
    loot
}

// ---------------------------------------------------------------------------
// Name generation
// ---------------------------------------------------------------------------

fn gen_name(biome: &str, size: &Size, rng: &mut impl Rng) -> String {
    let syllable_count = match size {
        Size::Tiny | Size::Small  => 1,
        Size::Medium              => if rng.gen_bool(0.5) { 1 } else { 2 },
        Size::Large               => 2,
        Size::Massive             => if rng.gen_bool(0.5) { 2 } else { 3 },
    };

    // Biome-weighted onset consonants
    let onsets: &[&str] = if biome.contains("coast") || biome.contains("shore") {
        &["l", "r", "n", "v", "s", "ln", "rv", "nl"]
    } else if biome.contains("ruin") || biome.contains("precursor") {
        &["vr", "thr", "kr", "gr", "zr", "vl", "thr", "gl"]
    } else if biome.contains("debris") || biome.contains("waste") {
        &["k", "g", "x", "gr", "kl", "gx", "kv", "xr"]
    } else {
        &["v", "k", "th", "r", "p", "s", "gr", "n", "m", "z", "gl", "pl"]
    };

    let nuclei: &[&str] = &["a", "e", "i", "o", "ae", "or", "ur", "ar"];
    let codas:  &[&str] = &["x", "th", "n", "r", "k", "s", "m", "l", ""];

    let mut name = String::new();
    for _ in 0..syllable_count {
        name.push_str(onsets[rng.gen_range(0..onsets.len())]);
        name.push_str(nuclei[rng.gen_range(0..nuclei.len())]);
        let coda = codas[rng.gen_range(0..codas.len())];
        name.push_str(coda);
    }
    name
}

// ---------------------------------------------------------------------------
// Description generation
// ---------------------------------------------------------------------------

fn gen_room_look(name: &str, tier: &FoodChainTier, secondary: &SecondaryLimb, rng: &mut impl Rng) -> String {
    let verb = match tier {
        FoodChainTier::Grazer    => ["moves through", "picks its way across", "grazes near"][rng.gen_range(0..3)],
        FoodChainTier::Scavenger => ["scavenges along", "circles through", "roots through"][rng.gen_range(0..3)],
        FoodChainTier::Predator  => ["stalks through", "crouches at the edge of", "moves low across"][rng.gen_range(0..3)],
        FoodChainTier::Apex      => ["dominates", "moves heavily through", "stands at the centre of"][rng.gen_range(0..3)],
        FoodChainTier::Coherence => ["pulses near", "drifts through", "clusters at"][rng.gen_range(0..3)],
    };
    let location = ["the area", "the undergrowth", "the terrain", "the open ground"][rng.gen_range(0..4)];
    let weapon_note = match secondary {
        SecondaryLimb::Bladed   => " Its weapon limbs catch the light.",
        SecondaryLimb::Impact   => " Heavy impact limbs hang ready.",
        SecondaryLimb::Spike    => " Spine-tipped limbs twitch slowly.",
        _                       => "",
    };
    format!("A {name} {verb} {location}.{weapon_note}")
}

fn gen_description(
    size:       &Size,
    hexaradial: bool,
    integument: &Integument,
    secondary:  &SecondaryLimb,
    sensory:    &SensoryOrgan,
) -> String {
    let symmetry = if hexaradial { "six-fold" } else { "three-fold" };
    let limb_n   = if hexaradial { "six" } else { "three" };
    let size_str = size.label();

    let int_desc = match integument {
        Integument::Soft     => "soft, dark-pigmented tissue",
        Integument::Fibrous  => "a mat of dense alien fiber",
        Integument::Chitin   => "irregular chitin-analogue plating",
        Integument::Crystal  => "crystalline growths across its integument",
        Integument::Membrane => "a thin resonance membrane that shimmers faintly",
    };
    let limb_desc = match secondary {
        SecondaryLimb::Grasping    => format!("{limb_n} grasping limbs"),
        SecondaryLimb::Bladed      => format!("{limb_n} bladed weapon limbs ending in serrated ridges"),
        SecondaryLimb::Impact      => format!("{limb_n} heavy impact limbs"),
        SecondaryLimb::Spike       => format!("{limb_n} spine-tipped limbs"),
        SecondaryLimb::SensoryStalk => format!("{limb_n} long sensory stalks"),
    };
    let sense_desc = match sensory {
        SensoryOrgan::Optical           => "simple optical organs",
        SensoryOrgan::Compound          => "compound optical organs covering a wide arc",
        SensoryOrgan::CoherenceSensitive => "coherence-sensitive organs that track field fluctuations",
        SensoryOrgan::Electromagnetic   => "electromagnetic sensory organs",
    };

    format!(
        "A {size_str} organism with {symmetry} body symmetry. Its surface is covered in {int_desc}. \
         {limb_desc} extend from the central body, and {sense_desc} sweep the surroundings continuously."
    )
}

// ---------------------------------------------------------------------------
// ID collision avoidance
// ---------------------------------------------------------------------------

/// Make the generated name unique within the current registry.
pub fn unique_id(base: &str, registry: &MobRegistry) -> String {
    if !registry.contains_key(base) {
        return base.to_string();
    }
    for i in 2..=99 {
        let candidate = format!("{base}{i}");
        if !registry.contains_key(&candidate) {
            return candidate;
        }
    }
    format!("{base}_{}", uuid::Uuid::new_v4().simple())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn rng() -> StdRng { StdRng::seed_from_u64(42) }

    fn empty_ctx(coherence: u8) -> GenContext<'static> {
        GenContext {
            visit_count: 1, coherence,
            biome: "forest",
            existing_tiers: vec![], existing_count: 0,
        }
    }

    #[test]
    fn should_generate_returns_true_on_first_pristine_visit() {
        let ctx = empty_ctx(70);
        let mut rng = rng();
        // With coherence 70, first visit, empty area: chance = 0.5 * 1.0 * 0.7 * 1.0 * 1.5 = 0.525
        // Run 20 trials — expect at least one true.
        let any = (0..20).any(|_| should_generate(&ctx, &mut rng));
        assert!(any, "expected at least one generation in 20 trials");
    }

    #[test]
    fn should_generate_false_at_max_density() {
        let ctx = GenContext {
            visit_count: 1, coherence: 90, biome: "forest",
            existing_tiers: vec![
                FoodChainTier::Grazer, FoodChainTier::Scavenger,
                FoodChainTier::Predator, FoodChainTier::Predator, FoodChainTier::Apex,
            ],
            existing_count: MAX_MOB_TYPES_PER_AREA,
        };
        let mut rng = rng();
        assert!(!should_generate(&ctx, &mut rng));
    }

    #[test]
    fn generate_produces_valid_template() {
        let ctx = empty_ctx(60);
        let mut rng = rng();
        let tmpl = generate(&ctx, &mut rng);
        assert!(!tmpl.id.is_empty());
        assert!(!tmpl.names.is_empty());
        assert!(!tmpl.room_look.is_empty());
        assert!(!tmpl.description.is_empty());
        assert!(tmpl.health_max >= tmpl.health_min);
        assert!(tmpl.combat.damage_max >= tmpl.combat.damage_min);
        assert!(tmpl.generated);
        assert!(!tmpl.loot_table.is_empty());
    }

    #[test]
    fn generate_grazer_is_not_aggressive() {
        let ctx = GenContext {
            visit_count: 1, coherence: 50, biome: "forest",
            existing_tiers: vec![], existing_count: 0,
        };
        // Run many times — any grazer generated must be non-aggressive.
        let mut rng = rng();
        for _ in 0..50 {
            let tmpl = generate(&ctx, &mut rng);
            if matches!(tmpl.food_chain_tier, FoodChainTier::Grazer) {
                assert!(!tmpl.aggressive, "grazers must not be aggressive");
            }
        }
    }

    #[test]
    fn generate_apex_is_aggressive() {
        // Force apex tier by providing full food chain.
        let ctx = GenContext {
            visit_count: 1, coherence: 80, biome: "ruins",
            existing_tiers: vec![
                FoodChainTier::Grazer, FoodChainTier::Predator,
            ],
            existing_count: 2,
        };
        let mut rng = rng();
        for _ in 0..30 {
            let tmpl = generate(&ctx, &mut rng);
            if matches!(tmpl.food_chain_tier, FoodChainTier::Apex) {
                assert!(tmpl.aggressive);
                assert!(tmpl.follows_aggressive);
            }
        }
    }

    #[test]
    fn unique_id_avoids_collision() {
        let mut registry = MobRegistry::new();
        registry.insert("velk".into(), generate(&empty_ctx(50), &mut rng()));
        let id = unique_id("velk", &registry);
        assert_ne!(id, "velk");
        assert!(!registry.contains_key(&id));
    }

    #[test]
    fn generated_template_always_has_meat_and_bone() {
        let ctx = empty_ctx(60);
        let mut rng = rng();
        for _ in 0..20 {
            let tmpl = generate(&ctx, &mut rng);
            assert!(tmpl.loot_table.iter().any(|e| e.template_id == DROP_MEAT));
            assert!(tmpl.loot_table.iter().any(|e| e.template_id == DROP_BONE));
        }
    }

    #[test]
    fn name_is_non_empty_for_all_sizes() {
        let mut rng = rng();
        for size in [Size::Tiny, Size::Small, Size::Medium, Size::Large, Size::Massive] {
            let name = gen_name("forest", &size, &mut rng);
            assert!(!name.is_empty(), "name should never be empty");
        }
    }
}
