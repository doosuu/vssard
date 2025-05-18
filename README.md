# VSSard 🧙🏼

VSSard (spelled: ˈwɪzəd') is a [COVESA Vehicle Signal Specification](https://github.com/COVESA/vehicle_signal_specification) server based on [SpacetimeDB](https://github.com/ClockworkLabs/SpacetimeDB).

## Getting started

To get started you need VSCode and a container runtime such as Docker Desktop to make use of VSCode's dev containers feature.
Once you have that set-up, simply fire up the repository using the provided dev container settings and you are good to go!

## Building the server

In the dev container, navigate to the `server` directory and run:

```sh
spacetime build
```

## Deploying the server code to Spacetime DB

To deploy the server code to a running Spacetime DB instance, run:

```sh
spacetime publish vssard .
```

## Connecting the (pseudo-)CAN client

This repo ships with a pseudo CAN client which does not actually connect to any CAN bus but just simulates some incoming VSS values. To run this application navigate to the `can_client` directory and run:

```sh
cargo run
```

This pseudo client will now continously publish VSS data to the server.
