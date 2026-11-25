use std::{process::ExitCode, sync::Arc};

use api::{
    application::use_cases::users::{SetRoleByUsernameUseCase, SetRoleByUsernameUseCaseParams},
    domain::value_objects::Role,
    infrastructure::adapters::repositories::DynamodbUserRepository,
};
use aws_config::Region;

const SERVICE: &str = "gym-api";
const DEFAULT_STAGE: &str = "dev";
const DEFAULT_REGION: &str = "eu-west-1";

struct Args {
    username: String,
    role: Role,
    stage: String,
    region: String,
}

fn usage() -> String {
    format!(
        "Usage: set-role <username> <admin|user> [stage] [region]\n  stage   default: {DEFAULT_STAGE}\n  region  default: {DEFAULT_REGION}"
    )
}

fn parse_role(value: &str) -> Result<Role, String> {
    match value {
        "admin" => Ok(Role::Admin),
        "user" => Ok(Role::User),
        _ => Err(format!(
            "Error: role must be 'admin' or 'user', got '{value}'"
        )),
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::User => "user",
    }
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    match args {
        [username, role, rest @ ..] if rest.len() <= 2 => Ok(Args {
            username: username.clone(),
            role: parse_role(role)?,
            stage: rest
                .first()
                .cloned()
                .unwrap_or_else(|| DEFAULT_STAGE.to_string()),
            region: rest
                .get(1)
                .cloned()
                .unwrap_or_else(|| DEFAULT_REGION.to_string()),
        }),
        _ => Err(usage()),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    if raw_args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{}", usage());
        return ExitCode::SUCCESS;
    }

    let args = match parse_args(&raw_args) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let table_name = format!("{SERVICE}-{}-users", args.stage);
    let aws_config = aws_config::from_env()
        .region(Region::new(args.region.clone()))
        .load()
        .await;
    let user_repository = DynamodbUserRepository::new(
        aws_sdk_dynamodb::Client::new(&aws_config),
        table_name.clone(),
    );

    println!(
        "==> Setting role on '{}' in {table_name} ({})",
        args.username, args.region
    );

    let result = SetRoleByUsernameUseCase {
        user_repository: Arc::new(user_repository),
    }
    .execute(SetRoleByUsernameUseCaseParams {
        username: args.username,
        role: args.role,
    })
    .await;

    match result {
        Ok(user) => {
            println!(
                "==> {} ({}) is now {}",
                user.username,
                user.id,
                role_name(user.role)
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {}", error.message);
            ExitCode::FAILURE
        }
    }
}
