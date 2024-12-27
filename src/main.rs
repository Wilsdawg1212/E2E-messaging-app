mod crypto;
mod p2p;
mod fallback;

fn main() {


    #[tokio::main]
    async fn main() {
        // Example: Start the fallback server
        tokio::spawn(async {
            fallback::run_server().await;
        });

        // Example: Start P2P listening
        p2p::start_p2p_listener().await;
    }

}
