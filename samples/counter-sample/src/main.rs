use counter_sample::store::AppState;
use streamdeck_plugin::Plugin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    counter_sample::actions::force_link();
    Plugin::builder(std::env::args())
        .state(AppState::default())
        .add_registered_actions()
        .run()
        .await?;
    Ok(())
}
