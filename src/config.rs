// Simulation configuration types and defaults

use std::collections::HashMap;

use crate::common::*;
use crate::gem::{effects_from_tree, fellowship_effects_from_tree, Gem, GemColor, GemTreeSource};

/// Defines a set bonus: when at least `threshold` pieces from this set are equipped, the player gains `effect`.
#[derive(Clone, Debug)]
pub struct SetBonus {
    pub set_id: SetId,
    pub threshold: u32,
    pub effect: StaticEffect,
}

#[derive(Debug)]
pub struct EquippedItem {
    pub name: String,
    pub stat_ratings: HashMap<Stat, i32>,

    // percentage increase to gem power, assumed 1.0 by default.
    // some items can be 1.35, legendaries are 2.0
    pub gem_power_increase: f32,
    pub slotted_gem: Option<Gem>,

    // items can have individual static effects
    pub static_effects: Vec<StaticEffect>,

    /// If present, this item counts toward the given set. With enough pieces equipped (see `SetBonus`), the player gains the set's static effect(s).
    pub set: Option<SetId>,
}

#[derive(Debug)]
pub struct PlayerConfig {
    pub talents: Vec<Talent>,
    pub equipped_items: Vec<EquippedItem>,
    pub weapon_traits: Vec<StaticEffect>,
    /// Set bonus rules: when N+ items from a set are equipped, the corresponding static effect applies.
    pub set_bonuses: Vec<SetBonus>,
}

/// Haste rating required for 1% haste at level 70 (WoW-style).
pub const HASTE_RATING_PER_PERCENT: f32 = 170.0;

impl PlayerConfig {
    /// Total haste rating from all equipped items.
    pub fn total_haste_rating(&self) -> i32 {
        self.equipped_items
            .iter()
            .filter_map(|i| i.stat_ratings.get(&Stat::Haste))
            .sum()
    }

    /// Haste as a decimal multiplier (1.0 = 0%, 1.22 = 22% haste). Used for hasted rPPM.
    pub fn haste_multiplier(&self) -> f32 {
        1.0 + (self.total_haste_rating() as f32 / HASTE_RATING_PER_PERCENT / 100.0)
    }

    /// Total effective gem power per color (gem power × item's gem_power_increase, summed across all slotted gems).
    pub fn total_gem_power_by_color(&self) -> HashMap<GemColor, i32> {
        let mut by_color: HashMap<GemColor, i32> = HashMap::new();
        for item in &self.equipped_items {
            if let Some(gem) = &item.slotted_gem {
                let effective = (gem.power as f32 * item.gem_power_increase) as i32;
                *by_color.entry(gem.color.clone()).or_insert(0) += effective;
            }
        }
        by_color
    }

    /// Collects all static effects from talents, slotted gems, and equipment (including set bonuses when applicable).
    pub fn all_static_effects(&self, gem_tree: GemTreeSource<'_>) -> Vec<StaticEffect> {
        let mut effects = Vec::new();

        // From talents
        for _talent in &self.talents {
            // TODO: map talent -> static effect(s)
        }

        // From slotted gems: total power per color, then lookup (const fellowship data or custom tree)
        match gem_tree {
            GemTreeSource::None => {}
            GemTreeSource::FellowshipDefault => {
                effects.extend(fellowship_effects_from_tree(&self.total_gem_power_by_color()));
            }
            GemTreeSource::Custom(tree) => {
                effects.extend(effects_from_tree(&self.total_gem_power_by_color(), tree));
            }
        }

        // From equipment: weapon traits and per-item effects
        effects.extend(self.weapon_traits.clone());
        for item in &self.equipped_items {
            effects.extend(item.static_effects.clone());
        }

        // From set bonuses (when threshold met)
        for _set_bonus in &self.set_bonuses {
            // TODO: count equipped pieces for this set, add effect if threshold met
        }

        effects
    }
}

#[derive(Debug)]
pub struct EnemyConfig {
    pub name: String,
    pub base_health: i32,
}

#[derive(Debug)]
pub struct EncounterConfig {
    pub enemies: Vec<EnemyConfig>,
}

#[derive(Debug)]
pub struct SimConfig {
    pub player_config: PlayerConfig,
    pub encounter_config: EncounterConfig,
}

impl SimConfig {
    /// Default sample config for development and testing.
    pub fn sample() -> Self {
        SimConfig {
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
                        set: None,
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
                        set: None,
                    },
                ],
                set_bonuses: vec![],
                weapon_traits: vec![StaticEffect {
                    name: "Celestial Precision".to_string(),
                    description: String::new(),
                }],
            },
            encounter_config: EncounterConfig {
                enemies: vec![EnemyConfig {
                    name: "Training Dummy".to_string(),
                    base_health: 1_000_000,
                }],
            },
        }
    }
}
