mod common;
mod config;
mod gem;
mod sim;

use config::SimConfig;
use gem::GemTreeSource;
use sim::{run_simulation, Action, PolicyFn};

fn dummy_policy(_state: &sim::GameState) -> (Action, Option<sim::TargetId>) {
    // Stub: always wait. Replace with real policy that picks abilities.
    (Action::Wait, None)
}

fn main() {
    let sim_config = SimConfig::sample();

    println!(
        "SimConfig ready: {} talent(s), {} item(s), {} enemy(ies)",
        sim_config.player_config.talents.len(),
        sim_config.player_config.equipped_items.len(),
        sim_config.encounter_config.enemies.len(),
    );

    let effects = sim_config
        .player_config
        .all_static_effects(GemTreeSource::FellowshipDefault);
    println!("Static effects: {}", effects.len());
    for e in &effects {
        println!("  - {}: {}", e.name, e.description);
    }

    let (final_state, ticks) = run_simulation(sim_config, dummy_policy as PolicyFn, 100);
    println!(
        "Sim ran {} ticks. Final focus: {}/{}",
        ticks, final_state.focus, final_state.max_focus
    );
}
