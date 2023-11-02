use clap::Parser;

#[derive(Parser)]
pub struct AppConfig {
    //Bot Token
    #[clap(env)]
    pub teloxide_token: String,

    #[clap(env)]
    pub tbank_url: String,

    #[clap(env)]
    pub redis_url: String,

    #[clap(env)]
    pub chart_generator_url: String,

}
