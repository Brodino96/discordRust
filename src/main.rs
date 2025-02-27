mod config;
mod db;

use std::sync::Arc;

use config::Config;
use db::Database;
use serenity::all::{GuildId, Member, Ready, User};
use serenity::{all::GatewayIntents, Client};
use serenity::{async_trait, prelude::*};
use time::{OffsetDateTime, PrimitiveDateTime};
use tracing::{debug, error, info};

struct Handler {
    db: Arc<Database>,
    config: Config
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, context: Context, member: Member) {

        if member.guild_id != self.config.guild_id {
            return
        }
        
        debug!("User {} joined", member.display_name());

        let now_utc = OffsetDateTime::now_utc();
        self.db.add_user(&member.user, PrimitiveDateTime::new(now_utc.date(), now_utc.time()))
            .await;

        let result = member.add_role(context.http, self.config.role_id)
            .await;
        if let Err(e) = result {
            error!("Failed to add role: {e}")
        }
    }

    async fn guild_member_removal(&self, _context: Context, guild_id: GuildId, user: User, _member: Option<Member>) {

        if guild_id != self.config.guild_id {
            return
        }

        debug!("User {} left", user.display_name());
        self.db.remove_user(&user)
            .await;
    }
    
    async fn ready(&self, context: Context, ready: Ready) {
        info!("Bot logged {}", ready.user.name);
        tokio::spawn({
            let db = self.db.clone();
            let interval = self.config.get_interval();
            let duration = self.config.get_duration();
            let context = context.clone();
            let guild_id = self.config.guild_id;
            let role_id = self.config.role_id;
            async move {
                let mut interval = tokio::time::interval(interval);
                
                loop {
                    interval.tick().await;
                    let cutoff_date = OffsetDateTime::now_utc() + duration;
                    
                    let users = db.delete_users(PrimitiveDateTime::new(cutoff_date.date(), cutoff_date.time()))
                        .await;
                    
                    for user in users {
                        let result = context.http.remove_member_role(
                            guild_id.into(),
                            user.id.parse::<u64>().expect("Failed to parse user id").into(),
                            role_id.into(),
                            Some("Role duration expired"))
                                .await;

                        if let Err(e) = result {
                            error!("Failed to remove user role: {e}")
                        }
                    }
    
                    debug!("{cutoff_date}")
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config: Config = Config::init().await;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS;

    let db = Arc::new(Database::init(&config.database_url).await);

    let mut client: Client = Client::builder(&config.discord_token, intents)
        .event_handler(Handler{db: db.clone(), config})
        .await
        .expect("Error creating client");

    client.start()
        .await
        .expect("Unable to start bot!");
}