use crate::{Context, Error};
use scripty_bot_utils::checks::is_guild;
use scripty_bot_utils::globals::DM_SUPPORT_GLOBAL;

#[poise::command(prefix_command, hide_in_help)]
pub async fn ps(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!("subcommands: `{}ps close`", ctx.prefix()))
        .await?;
    Ok(())
}

#[poise::command(prefix_command, hide_in_help, check = "is_guild")]
pub async fn ps_close(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(st) = DM_SUPPORT_GLOBAL.get() {
        let dctx = ctx.discord();
        st.close_ticket(
            dctx,
            ctx.channel_id()
                .to_channel(dctx)
                .await?
                .guild()
                .expect("should be in guild"),
        )
        .await;
    } else {
        ctx.say("error").await?;
    }
    Ok(())
}