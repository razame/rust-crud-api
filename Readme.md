To run this project
===================

1️⃣ Make sure you have Rust installed

If you haven’t installed Rust yet:

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh


Then reload your shell and check:

rustc --version
cargo --version

2️⃣ Create or check .env

Since you’re using dotenvy, you’ll likely have something like:

DATABASE_URL=mysql://username:password@localhost/db_name


Make sure .env is in your project root.

3️⃣ Build the project

From your project root (where Cargo.toml is):

cargo build

4️⃣ Run the project

Simply:

cargo run


Rocket will start, and you’ll see output like:

🚀 Rocket has launched from http://127.0.0.1:8000


By default:

Rocket reads Rocket.toml (optional) for configuration.

Without it, dev mode runs at localhost:8000.

5️⃣ Optional: Run in release mode

For better performance:

cargo run --release

6️⃣ Hot reloading (optional)

Rust doesn’t have native hot reloading, but you can use cargo-watch:

cargo install cargo-watch
cargo watch -x run


That will restart Rocket every time you change code.

✅ So in your case:

cd /path/to/rusty
cargo run


…will start your Rocket server.