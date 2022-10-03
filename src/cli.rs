use crate::ExtraArgs;
use anyhow::{anyhow, Ok};
use clap::{Parser, Subcommand};

/// Diff two http request and compare the difference of the responses
#[derive(Debug, Clone, Parser)]
#[command(version, author, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
// TODO: ?? what is `none_exhaustive`?
#[non_exhaustive]
pub enum Action {
    /// Diff two Api Response based on given profile
    Run(RunArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// Profile name
    #[arg(short, long)]
    pub profile: String,

    /// Override args. Could be used to override the query
    /// For query params, use `-e key=value`
    /// For headers, use `-e %key=value`
    /// For body, use `-e #key=value`
    #[arg(short, long, value_parser = parse_key_val, number_of_values = 1)]
    pub extra_params: Vec<KeyVal>,

    /// Configuration to use.
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    val: String,
}

fn parse_key_val<'a>(s: &'a str) -> anyhow::Result<KeyVal> {
    let mut pair = s.splitn(2, '=');
    let retrieve = |v: Option<&'a str>| -> anyhow::Result<&str> {
        Ok(v.ok_or(anyhow!("Invalid key=value pair: {}", s))?.trim())
    };
    let (key, value) = (retrieve(pair.next())?, retrieve(pair.next())?);

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, &key[1..]),
        Some('#') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("Invalid key")),
    };

    Ok(KeyVal {
        key_type,
        key: key.to_owned(),
        val: value.to_owned(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut new_extra_args = ExtraArgs {
            headers: vec![],
            query: vec![],
            body: vec![],
        };
        // diffrence about args.into_iter(), for x in args
        for arg in args {
            let arg_item = (arg.key, arg.val);
            match arg.key_type {
                KeyValType::Query => new_extra_args.query.push(arg_item),
                KeyValType::Header => new_extra_args.headers.push(arg_item),
                KeyValType::Body => new_extra_args.body.push(arg_item),
            }
        }
        new_extra_args
    }
}
