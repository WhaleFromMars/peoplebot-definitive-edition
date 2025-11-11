///This module provides error handling for the bot after it has started, this includes internal errors, and user errors.
use crate::prelude::*;
use poise::{BoxFuture, FrameworkError};
use std::convert::Infallible;
use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct UserError(#[from] pub anyhow::Error);

impl From<String> for UserError {
    fn from(value: String) -> Self {
        UserError(anyhow::anyhow!(value))
    }
}

impl FromStr for UserError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string().into())
    }
}

pub fn handle_error(
    error: FrameworkError<'_, GlobalState, anyhow::Error>,
) -> BoxFuture<'_, ()> {
    Box::pin(async move {
        if let Err(err) = try_handle_error(error).await {
            error!("Failed to handle error: {err:#}");
        }
    })
}

pub async fn try_handle_error(
    error: FrameworkError<'_, GlobalState, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    match error {
        FrameworkError::Command { error, ctx, .. } => {
            let invocation_string = ctx.invocation_string();
            let description = format!("{error:?}");

            if error.is::<UserError>() {
                ctx.send(
                    CreateReply::default()
                        .content(description)
                        .reply(true)
                        .ephemeral(true),
                )
                .await?;
            } else {
                error!("An error occurred whilst executing {invocation_string:?}: {error:#}");
                ctx.send(
                    CreateReply::default()
                        .content("An internal error occurred")
                        .reply(true)
                        .ephemeral(true),
                )
                .await?;
            }
        }
        other => {
            poise::builtins::on_error(other).await?;
        }
    }

    Ok(())
}
