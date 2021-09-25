use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Blue Client")]
pub struct Opt {
    /// Local or remote store
    #[structopt(short = "h", long = "host", default_value = "localhost")]
    pub host: String,

    /// Host port
    #[structopt(short = "p", long = "port", default_value = "7878")]
    pub port: usize,

    #[structopt(short = "n", long = "name", default_value = "Unknown")]
    pub name: String,
}
