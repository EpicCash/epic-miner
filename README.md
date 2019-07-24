# Epic Miner

A standalone mining implementation intended for mining epic against a running Epic node.

## Supported Platforms

At present, only mining plugins for linux-x86_64 and MacOS exist. This will likely change over time as the community creates more solvers for different platforms.

## Requirements

- The requirements for building the miner are the same from the epic server. You can check them in the topic [Requirements](https://gitlab.com/epiccash/epic/blob/master/doc/build.md#requirements) in the Epic server build instructions
- (**For cuda plugins**) Cuda toolkit 9+ (check with `nvcc --version`)
- (**For opencl plugins**) opencl-dev

For Debian-based distributions (Debian, Ubuntu, Mint, etc) you can install the opencl-dev with the following command in the terminal:

```sh
sudo apt install ocl-icd-opencl-dev
```

## Build steps

```sh
git clone https://gitlab.com/epiccash/epic-miner
cd epic-miner
git submodule update --init --recursive
```

To build the project you will have to specify if you are going to mine using `OPENCL` or `CUDA`. To mine using **CPUs/GPUs** use `OPENCL`. Execute the following line in the terminal to build with `OPENCL`:

```sh
cargo build --features opencl
```

If you have NVIDIA GPUs and your system has **the latest nvidia drivers and the Cuda toolkit 9+ installed**, you can build the cuda plugins using the following command:

```sh
cargo build --features cuda
```

## What was built

A successful build gets you:

- `target/debug/epic-miner` - the main epic-miner binary
- `target/debug/plugins/*` - mining plugins

## Running the Epic Miner

### Prerequisites

**To run the epic-miner you also need an epic server (with the stratum server enabled) running and an epic wallet listening.**

- Instruction of how to run the **epic server** and the **epic wallet** (in listening mode) using the .deb packages can be found [here](https://gitlab.com/epiccash/epic/blob/master/doc/running.org).
- If you want to build the **epic server** from source code, instructions can be found [here](https://gitlab.com/epiccash/epic/blob/master/doc/build.md).
- If you want to build and execute **epic wallet** (in listening mode) from source code, instructions can be found [here](https://gitlab.com/epiccash/epicwallet/tree/master/doc/build.md).

- To enable the stratum server in the epic server, we need to edit the server configuration file called `epic-server.toml`. If you are using the [epic server default configutation](https://gitlab.com/epiccash/epic/blob/master/doc/running.org#run_config_default), this file is under your home directory in the folder `~/.epic/main`. Open this file with your text editor and find the following line:

    ``` toml
      enable_stratum_server = false
    ```

    Then, change it to:

    ``` toml
      enable_stratum_server = true
    ```

    More information about this can be found on the topic [Configuring the Epic Server to work with the miner](https://gitlab.com/epiccash/epic/blob/master/doc/running.org#config_miner_server) in the testnet documentation.

### Executing the epic miner

**Make sure you always run epic-miner within a directory that contains an
`epic-miner.toml` configuration file.**

After you have your epic server running (with the stratum server enabled) and a wallet listening to it, to execute the epic miner follow the instructions:

1. Open a new terminal window in the root directory of your Epic miner installation.
2. To execute epic miner, you need to specify if you built it with `OPENCL` or `CUDA`. To run the epic miner compiled with `OPENCL` execute the following line in terminal:

     ```sh
     cargo run --features opencl
     ```

    And to execute your miner built with the cuda plugin, execute the command:  

    ```sh
     cargo run --features cuda
    ```

## Configuration

Epic-miner can be further configured via the `epic-miner.toml` file.
This file contains inline documentation on all configuration
options, and should be the first point of reference. Also, you can see Topic [Configuring your epic-miner](https://gitlab.com/epiccash/epic/blob/master/doc/running.org#config_miner) in the testnet documentation for further information.
