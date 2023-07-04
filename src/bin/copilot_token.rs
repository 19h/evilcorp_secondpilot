use evilcorp_secondpilot;

#[tokio::main]
async fn main() {
    let arg =
        std::env::args()
            .nth(1)
            .expect("Please provide a token");

    println!(
        "{:?}",
        evilcorp_secondpilot::EvilcorpSecondPilotClient::new(arg)
            .get_token()
            .await
            .unwrap(),
    );
}
