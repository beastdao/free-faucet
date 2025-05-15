# 0xname-sepolia-faucet


<img width="1470" alt="0xNAME SEPOLIA Faucet" src="https://github.com/user-attachments/assets/c44d8598-9c6b-43f6-9ca2-7c1c2a4bf0a0" />

### version [0.1.1] - 2025-05-14

**What's new:**
- Fully redesigned UI, with custom Themes
- Faucet logs added
- Configurable storage: ability set custom log/db limits with automatic FIFO compaction — ensuring stable storage size by pruning stale claims and logs.



### Local Development

- **Install Rust**  
   Follow the official [Rust installation guide](https://www.rust-lang.org/tools/install).
- **Install Cargo Binstall**  
   ```sh
   cargo install cargo-binstall
   ```
- **Install Dioxus CLI**  
   ```sh
   cargo install dioxus-cli
   ```
- **Install Required Dependencies (Non-macOS Users)**  
   If you are not using macOS, install the necessary dependencies. Refer to the [Dioxus installation guide](https://dioxuslabs.com/learn/0.6/getting_started/#).
- **Ensure the WASM Target is Installed**  
   ```sh
   rustup target add wasm32-unknown-unknown
   ```
- **Clone the Repository**  
   ```sh
   git clone https://github.com/beastdao/0xname-sepolia-faucet.git
   ```
- **Change dir to the Project Directory**  
   ```sh
   cd 0xname-sepolia-faucet
   ```
- **To build Bundle fro web**  
   ```sh
   dx bundle --package web
   ```
- **Set Up Environment Variables**  
   Provide the required parameters in the `.env` file.
- **To run**  
    ```sh
    dx serve --package web
    ```
- **Format code before a Pull Request**  
    Before submitting a pull request, ensure your code is formatted correctly:  
    ```sh
    dx fmt
    ```

     
