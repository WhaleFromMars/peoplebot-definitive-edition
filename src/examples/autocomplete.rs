use crate::prelude::*;
use std::fmt::Write as _;

inventory::submit! {
    CommandRegistry(commands)
}

fn commands() -> Vec<Command<GlobalState, Error>> {
    vec![greet()]
}

async fn autocomplete_name(_ctx: Context<'_>, partial: &str) -> CreateAutocompleteResponse {
    let choices = ["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"]
        .into_iter()
        .filter(move |name| name.starts_with(partial))
        .map(AutocompleteChoice::from)
        .collect();

    CreateAutocompleteResponse::new().set_choices(choices)
}

async fn autocomplete_number(_ctx: Context<'_>, _partial: &str) -> CreateAutocompleteResponse {
    // Dummy choices
    let choices = [1_u32, 2, 3, 4, 5].iter().map(|&n| {
        AutocompleteChoice::new(
            format!("{n} (why did discord even give autocomplete choices separate labels)"),
            n,
        )
    });

    CreateAutocompleteResponse::new().set_choices(choices.collect())
}

/// Greet a user. Showcasing autocomplete!
#[command(slash_command)]
pub async fn greet(
    ctx: Context<'_>,
    #[description = "Who to greet"]
    #[autocomplete = "autocomplete_name"]
    name: String,
    #[description = "A number... idk I wanted to test number autocomplete"]
    #[autocomplete = "autocomplete_number"]
    number: Option<u32>,
) -> Result<(), Error> {
    let mut response = format!("Hello {}", name);
    if let Some(number) = number {
        let _ = write!(response, "#{}", number);
    }
    response += "!";

    ctx.say(response).await?;
    Ok(())
}
