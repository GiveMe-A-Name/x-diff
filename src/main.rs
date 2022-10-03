use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use x_diff::{Action, Args, DiffConfig, RunArgs};

// TODO: main logical
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("Not Implement"),
    }
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_filepath = args.config.unwrap_or("./x_diff.yml".to_string());
    let config = DiffConfig::load_yaml(&config_filepath).await?;
    let profile = config.get_profile(&args.profile).ok_or(anyhow!(
        "Profile {} not found in config file {}",
        args.profile,
        config_filepath,
    ))?;
    let extra_args = args.extra_params.into();
    profile.diff(extra_args).await?;
    Ok(())
}
