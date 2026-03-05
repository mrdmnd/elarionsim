
// Static configuration - does not change over life of sim rollout

use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
enum Stat {
    Agility,
    Crit,
    Haste,
    Expertise,
    Spirit,
}

enum Talent {
    FocusedExpanse,
    PiercingSeekers,
    FinalCrescendo,
    SkylitGrace,
    Fusillade,
    SkywardMunitions,
    RepeatingStars,
    LunarFury,
    LethalShots,
    LunarlightAffinity,
    FerventSupremacy,
    ImpendingHeartseeker,
    ResurgentWinds,
    LastLights,
}

enum Ability {
    // CDs
    EventHorizon,
    SkystridersGrace,
    LunarlightMark,
    StarfallVolley,
    SkystridersSupremacy,
    // Rotational
    HeartseekerBarrage,
    HighwindArrow,
    Multishot,
    CelestialShot,
    FocusedShot,
}

// These are things like weapon traits, gem passives, gear tier sets, etc.
struct StaticEffect {
    name: String
}

enum GemColor {
    Red,
    Purple,
    Yellow,
    Green,
    Blue,
    Diamond,
}

struct Gem {
    color: GemColor,
    power: i32 // 120, 240, 360, 480
}
struct EquippedItem {
    name: String,
    stat_ratings: HashMap<Stat, i32>,

    // percentage increase to gem power, assumed 1.0 by default.
    // some items can be 1.35, legendaries are 2.0
    gem_power_increase: f32,
    slotted_gem: Option<Gem>,

    // some items have static effects
    // model these later?
    static_effects: Vec<StaticEffect>,
}

struct PlayerConfig {
    talents: Vec<Talent>,
    equipped_items: Vec<EquippedItem>,
    weapon_traits: Vec<StaticEffect>,
}

struct EnemyConfig {
    name: String,
    base_health: i32,
}

struct EncounterConfig {
    enemies: Vec<EnemyConfig>,
}

struct SimConfig {
    player_config: PlayerConfig,
    encounter_config: EncounterConfig,
}

fn main() {
    let sim_config = SimConfig {
        player_config: PlayerConfig {
            talents: vec![
                Talent::FocusedExpanse,
                Talent::PiercingSeekers,
                Talent::LethalShots,
                Talent::LunarlightAffinity,
                Talent::FerventSupremacy,
            ],
            equipped_items: vec![
                EquippedItem {
                    name: "Starforged Longbow".to_string(),
                    stat_ratings: HashMap::from([
                        (Stat::Agility, 450),
                        (Stat::Crit, 210),
                        (Stat::Haste, 140),
                    ]),
                    gem_power_increase: 2.0,
                    slotted_gem: Some(Gem {
                        color: GemColor::Diamond,
                        power: 480,
                    }),
                    static_effects: vec![],
                },
                EquippedItem {
                    name: "Windrunner's Pauldrons".to_string(),
                    stat_ratings: HashMap::from([
                        (Stat::Agility, 300),
                        (Stat::Expertise, 120),
                    ]),
                    gem_power_increase: 1.0,
                    slotted_gem: Some(Gem {
                        color: GemColor::Purple,
                        power: 360,
                    }),
                    static_effects: vec![],
                },
            ],
            weapon_traits: vec![
                StaticEffect { name: "Celestial Precision".to_string() },
            ],
        },
        encounter_config: EncounterConfig {
            enemies: vec![
                EnemyConfig {
                    name: "Training Dummy".to_string(),
                    base_health: 1_000_000,
                },
            ],
        },
    };

    println!("SimConfig ready: {} talent(s), {} item(s), {} enemy(ies)",
        sim_config.player_config.talents.len(),
        sim_config.player_config.equipped_items.len(),
        sim_config.encounter_config.enemies.len(),
    );
}
