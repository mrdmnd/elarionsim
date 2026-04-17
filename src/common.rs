// Domain types shared across the sim (stats, talents, abilities, gear vocabulary)

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Stat {
    Stamina,
    Armor,
    Agility,
    Crit,
    Haste,
    Expertise,
    Spirit,
}

#[derive(Debug)]
pub enum Talent {
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

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Ability {
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
#[derive(Clone, Debug)]
pub struct StaticEffect {
    pub name: String,
    pub description: String,
}

/// Identifies a gear set (e.g. tier set). Items in the same set can grant set bonuses.
pub type SetId = String;
