<h1 align="center"> DRink! CLI </h1>

https://github.com/use-ink/drink/assets/27450471/4a45ef8a-a7ec-4a2f-84ab-0a2a36c1cb4e

We provide this simple command line tool to help you play with your local contracts in a convenient way.

# Dependencies

The only requirement for running DRInk! is having Rust installed. The code was tested with version `1.70`.
All other dependencies are managed by Cargo and will be installed upon running `cargo build` or `cargo run`.

# Running DRInk! CLI

When you run the binary (`cargo run --release`) you'll see a DRink! TUI.
You can also choose to start from a specific path by supplying the `--path` argument like:
```bash
cargo run --release -- --path <absolute path to e.g. example/flipper>
```

## CLI modes

In a somewhat Vim-inspired way, the `drink-cli` allows you to work in two modes: the Managing mode and the Drinking mode.

### Managing mode

This is the default mode, facilitating high-level interactions with the TUI itself.
At any point, you can enter it by pressing the `Esc` key. Once in the Managing mode:
- Press `h` to see a list of available commands with their brief descriptions;
- Press `q` to quit the TUI;
- Press `i` to enter the Drinking mode.

### Drinking mode

This is the mode where you can interact with your environment and type your commands inside the `User input` field.
When in Managing mode, you can enter the Drinking mode by pressing 'i' on your keyboard.

You have several commands at your disposal (you can list them in the Managing mode by pressing the 'h' key):
- `cd` and `clear` will, just like their Bash counterparts, change the directory and clear the output, respectively. You will see the current working directory as the first entry in the `Current environment` pane;
- `build` command will build a contract from the sources in the current directory;
- `deploy` command will deploy a contract from the current directory. Note that if your constructor takes arguments, you will need to supply them to this command, like: `deploy true` in the case of the Flipper example;
- by pressing `Tab` you can switch between all deployed contracts (with automatic directory change);
- `call` command will call a contract with the given message. Again, if the message takes arguments, they need to be supplied here;
- `next-block` command will advance the current block number;
- `add-tokens` command will add tokens to the given account.
