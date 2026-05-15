mod services {
    pub mod auth;
}

fn main() {
    services::auth::login();
}