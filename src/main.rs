use miette::{IntoDiagnostic, Result};
use watchexec::{
    Watchexec,
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    handler::{Handler as _, PrintDebug},
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut init = InitConfig::default();
    init.on_error(PrintDebug(std::io::stderr()));

    let mut runtime = RuntimeConfig::default();
    runtime.pathset(["watchexec.conf"]);

    let conf = YourConfigFormat::load_from_file("watchexec.conf").await.into_diagnostic()?;
    conf.apply(&mut runtime);

    let we = Watchexec::new(init, runtime.clone())?;
    let w = we.clone();

    let c = runtime.clone();
    runtime.on_action(move |action: Action| {
        let mut c = c.clone();
        let w = w.clone();
        async move {
            for event in action.events.iter() {
                if event.paths().any(|(p, _)| p.ends_with("/watchexec.conf")) {
                    let conf = YourConfigFormat::load_from_file("watchexec.conf").await?;

                    conf.apply(&mut c);
                    let _ = w.reconfigure(c.clone());
                    // tada! self-reconfiguring watchexec on config file change!

                    break;
                }
            }

            action.outcome(Outcome::if_running(
                Outcome::DoNothing,
                Outcome::both(Outcome::Clear, Outcome::Start),
            ));

            Ok(())

            // (not normally required! ignore this when implementing)
            as std::result::Result<_, MietteStub>
        }
    });

    we.reconfigure(runtime);
    we.main().await.into_diagnostic()?;
    Ok(())
}

// ignore this! it's stuff to make the above code get checked by cargo doc tests!
struct YourConfigFormat; impl YourConfigFormat { async fn load_from_file(_: &str) -> std::result::Result<Self, MietteStub> { Ok(Self) } fn apply(&self, _: &mut RuntimeConfig) {} } use miette::Diagnostic; use thiserror::Error; #[derive(Debug, Error, Diagnostic)] #[error("stub")] struct MietteStub;