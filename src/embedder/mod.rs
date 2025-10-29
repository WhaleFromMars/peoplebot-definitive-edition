use crate::prelude::*;

inventory::submit! {
    CommandRegistry(commands)
}

fn commands() -> Vec<Command<GlobalState, Error>> {
    vec![]
}
