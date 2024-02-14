mod config;

use config::Config;
use serenity::{
    all::{ChannelId, GatewayIntents, Message, Ready},
    async_trait,
    client::{Context, EventHandler},
    gateway::ActivityData,
    Client,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, message: Message) {
        println!("{}", message.content)
    }

    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        let name = data_about_bot
            .user
            .global_name
            .as_ref()
            .unwrap_or(&data_about_bot.user.name);

        println!("{} is ready!", name);
    }
}

#[tokio::main]
async fn main() {
    let config = Config::load().expect("Failed to load config!");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(config.bot_token, intents)
        .event_handler(Handler)
        .activity(ActivityData::watching("chatbridges"))
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(e) = client.start().await {
        println!("An error occurred while running the client: {:?}", e);
    }
}
