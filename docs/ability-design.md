# Ability Design Proposal

## Goals

- Support many abilities without giant match statements
- Capture: resource cost, cast time, cooldown, damage, buffs, debuffs, procs
- Allow static effects (talents, gear, gems) to modify abilities
- Keep the sim loop generic—it shouldn't know ability-specific logic

---

## Core Concept: `AbilitySpec` Trait

Each ability is a type that implements a trait. The sim dispatches through the trait.

```rust
/// What the sim needs to execute an ability. Implement per ability.
pub trait AbilitySpec {
    fn id(&self) -> Ability;

    // --- Resource ---
    fn resource_cost(&self, state: &GameState) -> i32;

    // --- Timing ---
    fn cast_time_sec(&self) -> f32 { 0.0 }
    fn cooldown_sec(&self) -> f32 { 0.0 }
    fn cooldown_cdr_mult(&self) -> f32 { 1.0 }

    // --- Effects (called when ability actually fires) ---
    fn execute(&self, state: &mut GameState, target: TargetId);

    // --- Procs (optional) ---
    fn procs(&self) -> &[ProcSpec] { &[] }
}
```

**Why a trait?**
- Each ability encapsulates its own logic
- No central match—add a new ability = add a new impl
- Static effects can query `id()` to apply modifiers
- Default impls for common cases (instant, no CD, no procs)

---

## Proc System: Declarative

Procs are declared on the ability; the sim rolls for them generically.

```rust
pub struct ProcSpec {
    pub proc_id: &'static str,
    pub proc_type: ProcType,
    pub effect: ProcEffect,
}

pub enum ProcType {
    /// Hasted rPPM: proc_chance = ppm × haste × min(time_since, 10) / 60
    Rppm(f32),
    /// Flat % per trigger
    FlatChance(f32),
    /// Classic PPM (weapon-speed normalized, no haste)
    Ppm(f32),
}

pub enum ProcEffect {
    GrantBuff { buff_id: &'static str, duration_ticks: i32, max_charges: Option<u32> },
    ApplyDebuff { debuff_id: &'static str, duration_ticks: i32, stacks: u32 },
    // Extensible: Custom(Box<dyn Fn(&mut GameState)>)
}
```

The sim loop: after `execute()`, iterate `ability.procs()`, roll per `ProcType`, apply `ProcEffect`.

---

## Buff/Debuff IDs and Effects

Standardize IDs so procs and abilities reference the same things:

```rust
pub mod buff_ids {
    pub const CELESTIAL_IMPETUS: &str = "CelestialImpetus";
    // ...
}

pub mod debuff_ids {
    pub const LUNARLIGHT_MARK: &str = "LunarlightMark";
    // ...
}
```

`ProcEffect::GrantBuff` / `ApplyDebuff` use these. The sim has generic `grant_buff(id, duration, charges)` and `apply_debuff(enemy, id, duration, stacks)`.

---

## Static Effects: Modifiers

Static effects (talents, gems, gear) modify abilities rather than define them:

```rust
pub enum StaticEffectModifier {
    /// +X% damage to specific ability
    AbilityDamagePct { ability: Ability, pct: f32 },
    /// Add a proc to an ability (e.g. weapon trait)
    AddProc { ability: Ability, proc: ProcSpec },
    /// Global CDR
    CooldownReductionPct(f32),
    /// Reduce resource cost
    ResourceCostReduction { ability: Ability, flat: i32 },
}
```

`StaticEffect` could grow from `{ name, description }` to `{ id, modifiers: Vec<StaticEffectModifier> }`. The sim (or an "effect resolver") applies modifiers when computing cost, damage, procs.

---

## Registry vs Enum Dispatch

**Option A: Enum + impl AbilitySpec for each variant**
```rust
impl AbilitySpec for Ability {
    fn id(&self) -> Ability { *self }
    fn resource_cost(&self, state: &GameState) -> i32 {
        match self {
            Ability::CelestialShot => if state.celestial_impetus_charges() > 0 { 0 } else { 15 },
            Ability::FocusedShot => -20,
            // ...
        }
    }
    // ...
}
```
- Pro: Single enum, no indirection
- Con: One big impl block (but each method is a match—still centralized)

**Option B: Per-type impls + registry**
```rust
struct FocusedShot;
impl AbilitySpec for FocusedShot { ... }

// Registry: Ability -> &'static dyn AbilitySpec
lazy_static! { static ref ABILITY_REGISTRY: HashMap<Ability, &'static dyn AbilitySpec> = ... }
```
- Pro: Logic fully per-ability
- Con: More boilerplate, need to wire registry

**Recommendation:** Start with **Option A**—keep `Ability` enum, implement `AbilitySpec` for it. Each method is a match, but the *structure* is clear and the sim loop is generic. Migrate to Option B later if impl blocks get unwieldy.

---

## Summary: What Goes Where

| Concern | Where |
|---------|-------|
| Cost, cast, CD | `AbilitySpec` |
| Damage, buff consume, debuff apply | `AbilitySpec::execute()` |
| Proc triggers | `AbilitySpec::procs()` → `ProcSpec` |
| Proc roll logic | Sim (generic `try_proc_rppm`, etc.) |
| Modifiers from gear/talents | `StaticEffect` + resolver when querying cost/damage/procs |

---

## Migration Path

1. Define `AbilitySpec` trait and `ProcSpec` / `ProcType` / `ProcEffect`
2. Implement `AbilitySpec` for `Ability` (move logic from match fns into impl)
3. Refactor sim loop to call `ability_spec.resource_cost()`, `execute()`, then roll `procs()`
4. Extend `StaticEffect` with modifiers when ready
