//! Simulation event loop: tick-based, policy-driven action selection.
//!
//! Each tick:
//! 1. Bookkeeping (buffs, cooldowns)
//! 2. Policy chooses (action, target)
//! 3. Action executes (with non-determinism: procs, etc.)
//! 4. State evolves

use std::collections::HashMap;

use crate::common::Ability;
use crate::config::SimConfig;

/// Index into the encounter's enemy list. Used as target.
pub type TargetId = usize;

/// Action chosen by the policy. Wraps Ability with room for "wait" or other meta-actions later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    UseAbility(Ability),
    /// Do nothing this tick (e.g. waiting for focus or cooldown)
    Wait,
}

/// Policy: given game state, returns (action, target). Target may be None → use current target.
pub type PolicyFn = fn(&GameState) -> (Action, Option<TargetId>);

/// A buff on the player. Has duration (ticks remaining) and optional charges.
#[derive(Clone, Debug)]
pub struct Buff {
    pub id: String,
    pub duration_ticks: i32,
    pub charges: Option<u32>,
}

/// Per-ability cooldown state. Ticks remaining before usable.
#[derive(Clone, Debug, Default)]
pub struct CooldownState {
    pub remaining_ticks: f32,
    /// Multiplier: 1.0 = normal (1 tick per sim tick), 2.0 = 2 ticks of CDR per sim tick
    pub cdr_multiplier: f32,
}

/// Debuff on an enemy. Has duration and optional stacks.
#[derive(Clone, Debug)]
pub struct Debuff {
    pub id: String,
    pub duration_ticks: i32,
    pub stacks: u32,
}

/// Per-enemy runtime state (health, debuffs, etc.)
#[derive(Clone, Debug)]
pub struct EnemyState {
    pub health: i32,
    pub max_health: i32,
    pub debuffs: Vec<Debuff>,
}

/// Mutable game state at a point in time.
#[derive(Debug)]
pub struct GameState {
    pub tick: u64,
    pub config: SimConfig,

    /// Primary resource. Abilities may cost focus.
    pub focus: i32,
    pub max_focus: i32,

    /// Current target. Policy can override via (action, Some(other_target)).
    pub current_target: TargetId,

    /// Player buffs. Updated each tick (duration decay).
    pub buffs: Vec<Buff>,

    /// Ability cooldowns. Each tick, remaining_ticks -= 1.0 * cdr_multiplier (or similar).
    pub cooldowns: HashMap<Ability, CooldownState>,

    /// Encounter enemies and their runtime state.
    pub enemies: Vec<EnemyState>,

    /// RNG seed or handle for non-determinism (procs, etc.). Placeholder.
    pub rng_seed: u64,

    /// Casting state: (ability, target, ticks_remaining). When set, player is locked until cast completes.
    pub casting: Option<(Ability, TargetId, u32)>,

    /// Channeling state: (ability, target, ticks_remaining). When set, player is locked until channel completes.
    /// Unlike casting, channeled abilities can fire multiple hits over the duration; we resolve at channel end.
    pub channeling: Option<(Ability, TargetId, u32)>,

    /// rPPM: tick of last proc chance per proc id. Used for time-since-last-chance in hasted rPPM formula.
    pub rppm_last_check_tick: HashMap<String, u64>,
}

/// Tick duration in seconds. 100ms per sim tick. Used for cast/channel times, cooldowns, buffs, rPPM.
pub const TICK_DURATION_SEC: f32 = 0.1;

/// Buff/debuff IDs.
pub const CELESTIAL_IMPETUS: &str = "CelestialImpetus";
pub const LUNARLIGHT_MARK: &str = "LunarlightMark";
pub const MAX_CELESTIAL_IMPETUS_CHARGES: u32 = 2;

/// rPPM: time since last chance is capped at this (seconds). Prevents first hit from being guaranteed proc.
pub const RPPM_TIME_SINCE_CAP_SEC: f32 = 10.0;

/// Heartseeker Barrage: damage per projectile (physical). Placeholder; integrate with stats later.
pub const HEARTSEEKER_PROJECTILE_DAMAGE: i32 = 25;
/// Heartseeker Barrage: 20% chance for extra damage when hitting Lunarlight Mark. Multiplier on base damage.
pub const HEARTSEEKER_MARK_PROC_CHANCE: f32 = 0.20;
pub const HEARTSEEKER_MARK_EXTRA_DAMAGE_MULT: f32 = 1.5;

impl GameState {
    /// Build initial state from config.
    pub fn from_config(config: SimConfig) -> Self {
        let max_focus = 100; // TODO: derive from config
        let enemies = config
            .encounter_config
            .enemies
            .iter()
            .map(|e| EnemyState {
                health: e.base_health,
                max_health: e.base_health,
                debuffs: Vec::new(),
            })
            .collect();

        GameState {
            tick: 0,
            config,
            focus: max_focus,
            max_focus,
            current_target: 0,
            buffs: Vec::new(),
            cooldowns: HashMap::new(),
            enemies,
            rng_seed: 0,
            casting: None,
            channeling: None,
            rppm_last_check_tick: HashMap::new(),
        }
    }

    /// Resolve target: use explicit target or fall back to current.
    pub fn resolve_target(&self, target: Option<TargetId>) -> TargetId {
        target.unwrap_or(self.current_target)
    }

    /// Check if an ability is usable (focus, cooldown, buff charges, casting, channeling, etc.).
    pub fn is_ability_usable(&self, ability: Ability) -> bool {
        // Cannot use abilities while casting or channeling
        if self.casting.is_some() || self.channeling.is_some() {
            return false;
        }
        let cost = ability_focus_cost(self, ability);
        if self.focus < cost {
            return false;
        }
        if let Some(cd) = self.cooldowns.get(&ability) {
            if cd.remaining_ticks > 0.0 {
                return false;
            }
        }
        true
    }

    /// Count charges of Celestial Impetus buff.
    pub fn celestial_impetus_charges(&self) -> u32 {
        self.buffs
            .iter()
            .filter(|b| b.id == CELESTIAL_IMPETUS)
            .flat_map(|b| b.charges)
            .sum()
    }

    /// Consume one Celestial Impetus charge. Returns true if consumed.
    pub fn consume_celestial_impetus(&mut self) -> bool {
        for b in &mut self.buffs {
            if b.id == CELESTIAL_IMPETUS {
                if let Some(ref mut c) = b.charges {
                    if *c > 0 {
                        *c -= 1;
                        if *c == 0 {
                            b.duration_ticks = 0; // Will be removed in bookkeeping
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Add or refresh Celestial Impetus (up to max charges). Duration ~30s at 100ms ticks = 300 ticks.
    pub fn grant_celestial_impetus(&mut self) {
        const IMPETUS_DURATION_TICKS: i32 = 300;
        let existing = self
            .buffs
            .iter_mut()
            .find(|b| b.id == CELESTIAL_IMPETUS);
        if let Some(b) = existing {
            if let Some(ref mut c) = b.charges {
                if *c < MAX_CELESTIAL_IMPETUS_CHARGES {
                    *c += 1;
                    b.duration_ticks = IMPETUS_DURATION_TICKS;
                }
            }
        } else {
            self.buffs.push(Buff {
                id: CELESTIAL_IMPETUS.to_string(),
                duration_ticks: IMPETUS_DURATION_TICKS,
                charges: Some(1),
            });
        }
    }

    /// Get the target enemy, if valid.
    pub fn target_enemy(&self, target_id: TargetId) -> Option<&EnemyState> {
        self.enemies.get(target_id)
    }

    pub fn target_enemy_mut(&mut self, target_id: TargetId) -> Option<&mut EnemyState> {
        self.enemies.get_mut(target_id)
    }

    /// Check if enemy has Lunarlight Mark debuff.
    pub fn enemy_has_lunarlight_mark(&self, target_id: TargetId) -> bool {
        self.enemies
            .get(target_id)
            .map(|e| e.debuffs.iter().any(|d| d.id == LUNARLIGHT_MARK))
            .unwrap_or(false)
    }
}

/// Focus cost for an ability. Negative = generates focus.
fn ability_focus_cost(state: &GameState, ability: Ability) -> i32 {
    match ability {
        crate::common::Ability::CelestialShot => {
            if state.celestial_impetus_charges() > 0 {
                0
            } else {
                15
            }
        }
        crate::common::Ability::FocusedShot => -20, // Generates 20 focus
        crate::common::Ability::HeartseekerBarrage => 30,
        crate::common::Ability::HighwindArrow => 30,
        crate::common::Ability::Multishot => 40,
        crate::common::Ability::EventHorizon => 0,
        crate::common::Ability::SkystridersGrace => 0,
        crate::common::Ability::LunarlightMark => 0,
        crate::common::Ability::StarfallVolley => 0,
        crate::common::Ability::SkystridersSupremacy => 0,
    }
}

/// Cast time in ticks. 0 = instant. Casts lock the player until complete, then fire once.
pub fn ability_cast_ticks(ability: Ability) -> u32 {
    match ability {
        crate::common::Ability::FocusedShot => (1.22 / TICK_DURATION_SEC).round() as u32,
        _ => 0,
    }
}

/// Channel time in ticks. 0 = not a channel. Channels lock the player until complete, then fire (possibly multiple hits).
pub fn ability_channel_ticks(ability: Ability) -> u32 {
    match ability {
        crate::common::Ability::HeartseekerBarrage => (2.5 / TICK_DURATION_SEC).round() as u32,
        _ => 0,
    }
}

/// Base cooldown (ticks) and CDR multiplier for an ability. Stub: fill in per-ability.
pub fn ability_cooldown_info(ability: Ability) -> (u32, f32) {
    let (base_ticks, cdr_mult) = match ability {
        crate::common::Ability::EventHorizon => (180, 1.0),
        crate::common::Ability::SkystridersGrace => (90, 1.0),
        crate::common::Ability::LunarlightMark => (60, 1.0),
        crate::common::Ability::StarfallVolley => (120, 1.0),
        crate::common::Ability::SkystridersSupremacy => (300, 1.0),
        crate::common::Ability::HeartseekerBarrage => ((20.0 / TICK_DURATION_SEC) as u32, 1.0), // 20s CD
        _ => (0, 1.0), // rotational, no CD
    };
    (base_ticks, cdr_mult)
}

/// Bookkeeping: decay buffs, debuffs, tick down cooldowns.
fn tick_bookkeeping(state: &mut GameState) {
    // Decay buff durations
    state.buffs.retain_mut(|b| {
        b.duration_ticks -= 1;
        b.duration_ticks > 0
    });

    // Decay debuffs on enemies
    for enemy in &mut state.enemies {
        for d in &mut enemy.debuffs {
            d.duration_ticks -= 1;
        }
        enemy.debuffs.retain(|d| d.duration_ticks > 0);
    }

    // Tick down cooldowns (with CDR multiplier)
    for cd in state.cooldowns.values_mut() {
        if cd.remaining_ticks > 0.0 {
            cd.remaining_ticks = (cd.remaining_ticks - cd.cdr_multiplier).max(0.0);
        }
    }

    // TODO: focus regen, other per-tick effects
}

/// Execute the chosen action. Returns whether the action was valid/executed.
/// Non-determinism (procs) happens here.
fn execute_action(state: &mut GameState, action: Action, target_id: TargetId) -> bool {
    let ability = match action {
        Action::UseAbility(a) => a,
        Action::Wait => return true,
    };

    let cast_ticks = ability_cast_ticks(ability);
    let channel_ticks = ability_channel_ticks(ability);

    // Handle casting: continue or complete
    if let Some((cast_ability, cast_target, remaining)) = state.casting {
        if cast_ability != ability || cast_target != target_id {
            return false;
        }
        if remaining > 1 {
            state.casting = Some((cast_ability, cast_target, remaining - 1));
            return true;
        }
        // Completing cast (remaining == 1)
        state.casting = None;
        return execute_ability_effects(state, ability, target_id);
    }

    // Handle channeling: continue or complete
    if let Some((chan_ability, chan_target, remaining)) = state.channeling {
        if chan_ability != ability || chan_target != target_id {
            return false;
        }
        if remaining > 1 {
            state.channeling = Some((chan_ability, chan_target, remaining - 1));
            return true;
        }
        // Completing channel (remaining == 1)
        state.channeling = None;
        return execute_ability_effects(state, ability, target_id);
    }

    // Starting a cast
    if cast_ticks > 0 {
        if !state.is_ability_usable(ability) {
            return false;
        }
        state.casting = Some((ability, target_id, cast_ticks));
        return true;
    }

    // Starting a channel
    if channel_ticks > 0 {
        if !state.is_ability_usable(ability) {
            return false;
        }
        state.channeling = Some((ability, target_id, channel_ticks));
        return true;
    }

    // Instant ability
    if !state.is_ability_usable(ability) {
        return false;
    }
    execute_ability_effects(state, ability, target_id)
}

/// Apply focus cost, damage, buffs, debuffs for an ability. Called when ability actually executes (instant or cast complete).
fn execute_ability_effects(state: &mut GameState, ability: Ability, target_id: TargetId) -> bool {
    let cost = ability_focus_cost(state, ability);
    state.focus = (state.focus - cost).min(state.max_focus);

    // Put on cooldown
    let (base_ticks, cdr_mult) = ability_cooldown_info(ability);
    if base_ticks > 0 {
        state.cooldowns.insert(
            ability,
            CooldownState {
                remaining_ticks: base_ticks as f32,
                cdr_multiplier: cdr_mult,
            },
        );
    }

    // Celestial Shot: consume Celestial Impetus if used, apply 3 LunarlightMark when cast with Impetus
    let had_impetus = if ability == Ability::CelestialShot {
        state.celestial_impetus_charges() > 0
    } else {
        false
    };
    if had_impetus {
        state.consume_celestial_impetus();
    }

    // Apply damage to target
    let damage = if ability == Ability::HeartseekerBarrage {
        compute_heartseeker_barrage_damage(state, target_id)
    } else {
        compute_ability_damage(state, ability, target_id)
    };
    if let Some(enemy) = state.target_enemy_mut(target_id) {
        enemy.health = (enemy.health - damage).max(0);

        // Celestial Shot with Impetus: apply 3 stacks of LunarlightMark
        if ability == Ability::CelestialShot && had_impetus {
            apply_lunarlight_mark(enemy, 3);
        }
    }

    // Focused Shot: chance to grant Celestial Impetus
    if ability == Ability::FocusedShot {
        try_proc_celestial_impetus(state);
    }

    true
}

/// Add or refresh LunarlightMark stacks on enemy. Duration ~15s at 100ms ticks.
fn apply_lunarlight_mark(enemy: &mut EnemyState, stacks: u32) {
    const LUNARLIGHT_MARK_DURATION_TICKS: i32 = 150;
    if let Some(d) = enemy.debuffs.iter_mut().find(|d| d.id == LUNARLIGHT_MARK) {
        d.stacks += stacks;
        d.duration_ticks = LUNARLIGHT_MARK_DURATION_TICKS;
    } else {
        enemy.debuffs.push(Debuff {
            id: LUNARLIGHT_MARK.to_string(),
            duration_ticks: LUNARLIGHT_MARK_DURATION_TICKS,
            stacks,
        });
    }
}

/// Hasted rPPM proc check. Returns true if the proc triggered.
/// Formula: proc_chance = PPM × haste_mult × min(time_since_last_sec, 10) / 60
fn try_proc_rppm(
    state: &mut GameState,
    proc_id: &str,
    ppm: f32,
    on_proc: impl FnOnce(&mut GameState),
) -> bool {
    let haste_mult = state.config.player_config.haste_multiplier();
    let last_tick = state
        .rppm_last_check_tick
        .get(proc_id)
        .copied()
        .unwrap_or(0);
    let time_since_sec = ((state.tick.saturating_sub(last_tick)) as f32 * TICK_DURATION_SEC)
        .min(RPPM_TIME_SINCE_CAP_SEC);

    let proc_chance = ppm * haste_mult * time_since_sec / 60.0;

    state.rppm_last_check_tick
        .insert(proc_id.to_string(), state.tick);

    let roll = (state.rng_seed as f32 / u64::MAX as f32) % 1.0;
    state.rng_seed = state.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);

    if roll < proc_chance {
        on_proc(state);
        true
    } else {
        false
    }
}

/// Focused Shot proc: grants Celestial Impetus via hasted rPPM.
const CELESTIAL_IMPETUS_RPPM: f32 = 2.0;

fn try_proc_celestial_impetus(state: &mut GameState) {
    try_proc_rppm(state, CELESTIAL_IMPETUS, CELESTIAL_IMPETUS_RPPM, |s| {
        s.grant_celestial_impetus();
    });
}

/// Compute damage for an ability. Placeholder values; integrate with stats later.
fn compute_ability_damage(_state: &GameState, ability: Ability, _target_id: TargetId) -> i32 {
    match ability {
        Ability::CelestialShot => 150,   // Magical damage, placeholder
        Ability::FocusedShot => 200,    // Physical damage, placeholder
        _ => 0,
    }
}

/// Heartseeker Barrage: 10*(1+haste) projectiles, each does phys damage. 20% chance for extra damage if target has Lunarlight Mark.
fn compute_heartseeker_barrage_damage(state: &mut GameState, target_id: TargetId) -> i32 {
    let haste_mult = state.config.player_config.haste_multiplier();
    let projectiles = (10.0 * haste_mult).round() as u32;
    let has_mark = state.enemy_has_lunarlight_mark(target_id);

    let mut total = 0i32;
    for _ in 0..projectiles {
        let mut damage = HEARTSEEKER_PROJECTILE_DAMAGE;
        if has_mark {
            let roll = (state.rng_seed as f32 / u64::MAX as f32) % 1.0;
            state.rng_seed = state.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
            if roll < HEARTSEEKER_MARK_PROC_CHANCE {
                damage = (damage as f32 * HEARTSEEKER_MARK_EXTRA_DAMAGE_MULT).round() as i32;
            }
        }
        total += damage;
    }
    total
}

/// Run the simulation for a fixed number of ticks, or until encounter ends.
pub fn run_simulation(
    config: SimConfig,
    policy: PolicyFn,
    max_ticks: u64,
) -> (GameState, u64) {
    let mut state = GameState::from_config(config);
    let mut ticks_run = 0u64;

    while ticks_run < max_ticks {
        // 1. Bookkeeping
        tick_bookkeeping(&mut state);

        // 2. Policy chooses (action, target), or continue casting/channeling
        let (action, target_opt) = if let Some((ability, target_id, _)) = state.casting {
            (Action::UseAbility(ability), Some(target_id))
        } else if let Some((ability, target_id, _)) = state.channeling {
            (Action::UseAbility(ability), Some(target_id))
        } else {
            policy(&state)
        };
        let target_id = state.resolve_target(target_opt);

        // 3. Execute action
        if !execute_action(&mut state, action, target_id) {
            // Policy chose invalid action; treat as Wait
            // Could log or count as error
        }

        state.tick += 1;
        ticks_run += 1;

        // 4. Check encounter end (all enemies dead?)
        if state.enemies.iter().all(|e| e.health <= 0) {
            break;
        }
    }

    (state, ticks_run)
}
