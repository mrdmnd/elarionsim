// Gem types and gem power tree (thresholds per color → static effects)

use std::collections::HashMap;

use crate::common::StaticEffect;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum GemColor {
    Red,
    Purple,
    Yellow,
    Green,
    Blue,
    Diamond,
}

#[derive(Debug)]
pub struct Gem {
    pub color: GemColor,
    pub power: i32, // 120, 240, 360, 480
}

/// A single threshold in the gem power tree: when total power for a color meets or exceeds `power`, the player gains these effects.
/// If `slot` is Some, only the highest-met threshold per slot is applied (e.g. "Might of the Minotaur II" replaces "Might of the Minotaur").
#[derive(Clone)]
pub struct GemPowerThreshold {
    pub power: i32,
    pub effects: Vec<StaticEffect>,
    /// When set, only the effect from the highest power threshold in this slot is granted.
    pub slot: Option<u8>,
}

/// Per-color thresholds: at each power level you gain the listed static effects. Thresholds are typically ordered ascending by power.
#[derive(Clone, Default)]
pub struct GemPowerTree {
    pub by_color: HashMap<GemColor, Vec<GemPowerThreshold>>,
}

/// How to resolve gem power → static effects: no gems, built-in Fellowship tree (const data), or a custom tree.
#[derive(Copy, Clone)]
pub enum GemTreeSource<'a> {
    None,
    FellowshipDefault,
    Custom(&'a GemPowerTree),
}

// Fellowship built-in tree: (power, name, description, slot). Source: Method guide.
type FellowshipThreshold = (i32, &'static str, &'static str, u8);

const RED: [FellowshipThreshold; 10] = [
    (120, "Might Of The Minotaur", "+3% primary stat while you are above 80% health.", 0),
    (240, "Champion's Heart", "+140 Stamina and +8 Strength / Intellect / Agility", 1),
    (480, "Unyielding Vitality", "You heal for 0.7% of your Maximum Health every 2 seconds while in combat.", 2),
    (720, "Titan's Blood", "+3% Stamina", 3),
    (960, "Blessing Of The Conqueror", "While in combat with a boss your Damage, Healing, and Absorption is increased by 4%.", 4),
    (1200, "Might Of The Minotaur II", "+9% primary stat while you are above 80% health.", 0),
    (1560, "Champion's Heart II", "+420 Stamina and +24 Strength / Intellect / Agility", 1),
    (1920, "Unyielding Vitality II", "You heal for 2.1% of your Maximum Health every 2 seconds while in combat.", 2),
    (2280, "Titan's Blood II", "+9% Stamina", 3),
    (2640, "Blessing Of The Conqueror II", "While in combat with a boss your Damage, Healing, and Absorption is increased by 12%.", 4),
];

const YELLOW: [FellowshipThreshold; 10] = [
    (120, "Adrenaline Rush", "+3% Haste for 10 seconds each time you deal damage to or heal a character that has 30% or less health.", 0),
    (240, "Thief's Alacrity", "+100 Stamina and +100 Haste", 1),
    (480, "Rogue's Resurgence", "When you are below 50% health you are instantly healed for 8% of your maximum health. This effect can only occur once every 20 seconds.", 2),
    (720, "Feline's Grace", "+3% Haste", 3),
    (960, "Blessing Of The Virtuoso", "You retain +3% Haste from 'Spirit of Heroism' while it is not active.", 4),
    (1200, "Adrenaline Rush II", "+9% Haste for 10 seconds each time you deal damage to or heal a character that has 30% or less health.", 0),
    (1560, "Thief's Alacrity II", "+300 Stamina and +300 Haste", 1),
    (1920, "Rogue's Resurgence II", "When you are below 50% health you are instantly healed for 24% of your maximum health. This effect can only occur once every 20 seconds.", 2),
    (2280, "Feline Grace II", "+9% Haste", 3),
    (2640, "Blessing Of The Virtuoso II", "You retain +9% Haste from 'Spirit of Heroism' while it is not active.", 4),
];

const BLUE: [FellowshipThreshold; 10] = [
    (120, "Ancestral Surge", "While Spirit of Heroism is active, your Strength / Agility / Intellect is increased by 8%.", 0),
    (240, "Mystic's Intuition", "+100 Stamina and +100 Spirit", 1),
    (480, "Resonating Soul", "For every 10% Health you are missing, all damage you take is reduced by 1%.", 2),
    (720, "Oracle's Foresight", "+3% Spirit", 3),
    (960, "Blessing Of The Prophet", "The duration of your Spirit of Heroism is increased by 6 seconds.", 4),
    (1200, "Ancestral Surge II", "While Spirit of Heroism is active, your Strength / Agility / Intellect is increased by 24%.", 0),
    (1560, "Mystic's Intuition II", "+300 Stamina and +300 Spirit", 1),
    (1920, "Resonating Soul II", "For every 10% Health you are missing, all damage you take is reduced by 3%.", 2),
    (2280, "Oracle's Foresight II", "+9% Spirit", 3),
    (2640, "Blessing Of The Prophet II", "The duration of your Spirit of Heroism is increased by 18 seconds.", 4),
];

const GREEN: [FellowshipThreshold; 10] = [
    (120, "First Strike", "+5% Expertise for 15 seconds when dealing damage to any enemy for the first time.", 0),
    (240, "Vanguard's Resolve", "+100 Stamina and +100 Expertise", 1),
    (480, "Sentinel's Bastion", "Every 60 seconds, you gain a shield that absorbs damage equal to 10% of your maximum health. The shield lasts for 60 seconds.", 2),
    (720, "Tactician's Acumen", "+3% Expertise", 3),
    (960, "Blessing Of The Commander", "Your ability Cooldowns are reduced by 4%.", 4),
    (1200, "First Strike II", "+15% Expertise for 15 seconds when dealing damage to any enemy for the first time.", 0),
    (1560, "Vanguard's Resolve II", "+300 Stamina and +300 Expertise", 1),
    (1920, "Sentinel's Bastion II", "Every 60 seconds you gain a shield that absorbs damage equal to 30% of your maximum health. The shield lasts for 60 seconds.", 2),
    (2280, "Tactician's Acumen II", "+9% Expertise", 3),
    (2640, "Blessing Of The Commander II", "+12% Ability Cooldown Reduction", 4),
];

const DIAMOND: [FellowshipThreshold; 10] = [
    (120, "Harmonious Soul", "+0.5% Critical Strike, +0.5% Haste, +0.5% Expertise and +0.5% Spirit", 0),
    (240, "Stoic's Teachings", "+60 Stamina and +15 Strength / Intellect / Agility", 1),
    (480, "Tranquil Resolve", "When you take Magic Damage you gain 'Tranquil Resolve', reducing all Magic Damage you take by 8% for 6 seconds. This effect cannot occur more than once every 20 seconds.", 2),
    (720, "Ancient's Wisdom", "+2% Strength / Intellect / Agility", 3),
    (960, "Blessing Of The Artisan", "Your Relic cooldowns are reduced by 8%.", 4),
    (1200, "Harmonious Soul II", "+1.5% Critical Strike, +1.5% Haste, +1.5% Expertise and +1.5% Spirit", 0),
    (1560, "Stoic's Teachings II", "+180 Stamina and +45 Strength / Intellect / Agility", 1),
    (1920, "Tranquil Resolve II", "When you take Magic Damage you gain 'Tranquil Resolve', reducing all Magic Damage you take by 24% for 6 seconds. This effect cannot occur more than once every 20 seconds.", 2),
    (2280, "Ancient's Wisdom II", "+6% Strength / Intellect / Agility", 3),
    (2640, "Blessing Of The Artisan II", "Your Relic cooldowns are reduced by 24%.", 4),
];

const PURPLE: [FellowshipThreshold; 10] = [
    (120, "Sealed Fate", "Your Damage and Healing effects on characters with more than 80% health have +10% critical strike chance.", 0),
    (240, "Berserker's Zeal", "+100 Stamina and +100 Critical Strike", 1),
    (480, "Reaper's Reprieve", "Each time an enemy that you dealt damage to dies, you gain Reaper's Reprieve, healing you for 12% of your Maximum Health over 12 seconds.", 2),
    (720, "Killer Instinct", "+3% Critical Strike chance", 3),
    (960, "Blessing Of The Deathdealer", "Your critical strikes do 4% more damage and healing.", 4),
    (1200, "Sealed Fate II", "Your Damage and Healing effects on characters with more than 80% health have +30% critical strike chance.", 0),
    (1560, "Berserker's Zeal II", "+300 Stamina and +300 Critical Strike", 1),
    (1920, "Reaper's Reprieve II", "Each time an enemy that you dealt damage to dies, you gain Reaper's Reprieve, healing you for 36% of your Maximum Health over 18 seconds.", 2),
    (2280, "Killer Instinct II", "+9% Critical Strike chance", 3),
    (2640, "Blessing Of The Deathdealer II", "Your critical strikes do 12% more damage and healing.", 4),
];

fn fellowship_effects_for_color(total_power: i32, thresholds: &[FellowshipThreshold]) -> Vec<StaticEffect> {
    let mut by_slot: HashMap<u8, (i32, &'static str, &'static str)> = HashMap::new();
    for &(power, name, desc, slot) in thresholds {
        if total_power >= power {
            by_slot
                .entry(slot)
                .and_modify(|e| {
                    if power > e.0 {
                        *e = (power, name, desc);
                    }
                })
                .or_insert((power, name, desc));
        }
    }
    by_slot
        .values()
        .map(|(_, name, desc)| StaticEffect {
            name: (*name).to_string(),
            description: (*desc).to_string(),
        })
        .collect()
}

/// Fellowship built-in gem power tree: lookup from const data. No heap allocation of the tree.
pub fn fellowship_effects_from_tree(
    power_by_color: &HashMap<GemColor, i32>,
) -> Vec<StaticEffect> {
    let mut out = Vec::new();
    for (color, &total_power) in power_by_color {
        let thresholds: &[FellowshipThreshold] = match color {
            GemColor::Red => &RED,
            GemColor::Yellow => &YELLOW,
            GemColor::Blue => &BLUE,
            GemColor::Green => &GREEN,
            GemColor::Diamond => &DIAMOND,
            GemColor::Purple => &PURPLE,
        };
        out.extend(fellowship_effects_for_color(total_power, thresholds));
    }
    out
}

/// Returns all static effects granted by the gem power tree for the given per-color totals.
/// When thresholds have a `slot`, only the effect from the highest-met threshold in each slot is included (upgraded powers replace base).
pub fn effects_from_tree(
    power_by_color: &HashMap<GemColor, i32>,
    tree: &GemPowerTree,
) -> Vec<StaticEffect> {
    let mut effects = Vec::new();
    for (color, total_power) in power_by_color {
        if let Some(thresholds) = tree.by_color.get(color) {
            let met: Vec<_> = thresholds
                .iter()
                .filter(|t| *total_power >= t.power)
                .collect();
            if met.is_empty() {
                continue;
            }
            // Group by slot: if any threshold has a slot, only take the highest power per slot
            let has_slots = met.iter().any(|t| t.slot.is_some());
            if has_slots {
                let mut by_slot: std::collections::HashMap<u8, &GemPowerThreshold> =
                    std::collections::HashMap::new();
                for t in &met {
                    if let Some(slot) = t.slot {
                        by_slot
                            .entry(slot)
                            .and_modify(|e| {
                                if t.power > e.power {
                                    *e = t;
                                }
                            })
                            .or_insert(t);
                    }
                }
                for t in by_slot.values() {
                    effects.extend(t.effects.clone());
                }
            } else {
                for t in &met {
                    effects.extend(t.effects.clone());
                }
            }
        }
    }
    effects
}
