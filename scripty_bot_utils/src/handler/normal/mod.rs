use poise::serenity_prelude::EventHandler;
use serenity::{
	all::{Interaction, VoiceState},
	client::Context as SerenityContext,
	model::{channel::Message, event::ResumedEvent, gateway::Ready, id::GuildId},
};

mod cache_ready;
mod interaction_create;
mod message;
mod ready;
mod resume;
mod voice_state_update;

pub struct BotEventHandler;

#[async_trait]
impl EventHandler for BotEventHandler {
	#[inline]
	async fn cache_ready(&self, ctx: SerenityContext, guilds: Vec<GuildId>) {
		cache_ready::cache_ready(ctx, guilds).await;
	}

	#[inline]
	async fn message(&self, ctx: SerenityContext, new_message: Message) {
		message::message(ctx, new_message).await;
	}

	#[inline]
	async fn ready(&self, ctx: SerenityContext, ready: Ready) {
		ready::ready(ctx, ready).await;
	}

	#[inline]
	async fn resume(&self, ctx: SerenityContext, resume: ResumedEvent) {
		resume::resume(ctx, resume).await;
	}

	#[inline]
	async fn voice_state_update(
		&self,
		ctx: SerenityContext,
		old: Option<VoiceState>,
		new: VoiceState,
	) {
		voice_state_update::voice_state_update(ctx, old, new).await;
	}

	#[inline]
	async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
		interaction_create::interaction_create(ctx, interaction).await;
	}
}
