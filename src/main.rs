use std::{collections::HashMap, io::Write};

use anyhow::{anyhow, Ok, Result};
use clap::Parser;

use tokio::join;
use x_diff::{
    highlight_text, Action, Args, DiffConfig, DiffProfile, ExtraArgs, RequestProfile,
    ResponseProfile, RunArgs,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
        _ => panic!("Not Implement"),
    }
    Ok(())
}

async fn parse() -> Result<()> {
    use dialoguer::theme::ColorfulTheme;
    use dialoguer::{Input, MultiSelect};

    let theme = ColorfulTheme::default();

    let url1: String = Input::with_theme(&theme)
        .with_prompt("URL1")
        .interact_text()?;
    let url2: String = Input::with_theme(&theme)
        .with_prompt("URL2")
        .interact_text()?;

    let profile: String = Input::with_theme(&theme)
        .with_prompt("Profile name")
        .interact_text()?;
    let req1: RequestProfile = url1.parse()?;
    let req2: RequestProfile = url2.parse()?;
    let default_extra_args = ExtraArgs::default();
    let (response1, response2) = {
        let (res1, res2) = join!(
            req1.send(&default_extra_args),
            req2.send(&default_extra_args),
        );
        (res1?, res2?)
    };

    let headers = [response1.get_header_keys(), response2.get_header_keys()].concat();

    let chose_headers = MultiSelect::with_theme(&theme)
        .with_prompt("Select headers to skip")
        .items(headers.as_slice())
        .interact()?;
    let skip_headers = chose_headers
        .iter()
        .map(|idx| headers[*idx].to_string())
        .collect::<Vec<_>>();

    let response_profile = ResponseProfile::new(skip_headers, vec![]);

    let diff_profile = DiffProfile::new(req1, req2, response_profile);
    let config = serde_yaml::to_string(&DiffConfig {
        profiles: HashMap::from([(profile, diff_profile)]),
    })?;
    let mut stdout = std::io::stdout().lock();
    write!(stdout, "{}", highlight_text(&config, "yaml")?)?;

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
