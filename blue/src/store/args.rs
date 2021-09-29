use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Blue Client")]
pub struct Opt {
    /// Local or remote store. Must use actual IP (not localhost) to allow remote connections
    #[structopt(short = "h", long = "host", default_value = "localhost")]
    pub host: String,

    /// Host port
    #[structopt(short = "p", long = "port", default_value = "7878")]
    pub port: usize,

    #[structopt(short = "r", long = "role", default_value = "leader")]
    pub role: String,

    #[structopt(short = "r", long = "role", required_if("role", "follower"))]
    pub follow: Option<String>,
}
