use std::env;
use wny_core::mode::{Language, Mode, Slot, SlotChoice};
use wny_core::orchestrator::Orchestrator;
use wny_egress::{EgressGuard, PlatformGuard};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        args.push("status".into());
    }
    std::env::set_var("WNY_DEV", "1");
    let mut orch = Orchestrator::open_default().expect("orchestrator");
    match args[0].as_str() {
        "status" => {
            let s = orch.snapshot();
            println!("{}", serde_json::to_string_pretty(&s).unwrap());
        }
        "on" => {
            let mode = orch.snapshot().mode;
            let m = Mode::parse(&mode).unwrap_or(Mode::Protection);
            if let Ok(mut g) = PlatformGuard::new(m) {
                let _ = g.engage(m);
            }
            let s = orch.set_on(true).unwrap();
            println!("{}", s.status);
        }
        "off" => {
            if let Ok(mut g) = PlatformGuard::new(Mode::Protection) {
                let _ = g.disengage();
            }
            let s = orch.set_on(false).unwrap();
            println!("{}", s.status);
        }
        "mode" => {
            let m = Mode::parse(args.get(1).map(|s| s.as_str()).unwrap_or("protection"))
                .expect("mode");
            let s = orch.set_mode(m).unwrap();
            println!("{}", s.mode);
        }
        "lang" => {
            let l = Language::parse(args.get(1).map(|s| s.as_str()).unwrap_or("en"));
            let s = orch.set_language(l).unwrap();
            println!("{}", s.language);
        }
        "pin" => {
            let slot = Slot::parse(args.get(1).map(|s| s.as_str()).unwrap_or("")).expect("slot");
            let choice = SlotChoice::parse(args.get(2).map(|s| s.as_str()).unwrap_or("auto"));
            let s = orch.set_slot(slot, choice).unwrap();
            println!("{} -> {:?}", slot.as_str(), s);
        }
        "new-identity" => {
            orch.new_identity().unwrap();
            println!("ok");
        }
        "captive" => {
            orch.open_captive_portal().unwrap();
            println!("captive");
        }
        other => {
            eprintln!("unknown command {other}");
            eprintln!("wny status|on|off|mode|lang|pin|new-identity|captive");
            std::process::exit(2);
        }
    }
}
