# Effectors

List of supported effectors:
Effector | Latest Effector Package| Rust Binding Crate | Rust CID Crate |
---------|------------------------|--------------------|----------------|
IPFS     | [latest](https://github.com/fluencelabs/ipfs-effector/releases/latest) | [ipfs-effector-imports](https://crates.io/crates/ipfs-effector-imports) | [ipfs-effector-cid](https://crates.io/crates/ipfs-effector-cid) |
cURL     | [latest](https://github.com/fluencelabs/curl-effector/releases/latest) | [curl-effector-imports](https://crates.io/crates/curl-effector-imports) | [curl-effector-cid](https://crates.io/crates/curl-effector-cid) |


# How to add an effector to your project

Let's add the cURL effector to your [Fluence CLI](https://fluence.dev/docs/build/setting-up/installing_cli) project.

All information you need to add an effector to your project is located in the latest release description of the effector:
![cURL effector release page]

The *CIDv1* field can help you find providers in the explorer for this particular effector.

The *archive* in the assets (`curl_effector.tar.gz` in the example) is an effector package that can be used to add the module to your project:
```
LATEST_RELEASE_TAG=effector-v0.1.1
fluence module add https://github.com/fluencelabs/curl-effector/releases/download/$LATEST_RELEASE_TAG/curl_effector.tar.gz --service myService
```

Check `Cargo.toml` is your project root directory. Now, it should contain the *binding crate* for the effector
```toml
[workspace.dependencies.curl-effector-imports]
version = "0.1.1"
```
This crate contains the effector's definition, so you can start using it by importing this crate into your module.
Add this to the `Cargo.toml` of your module:
```
curl-effector-imports = { workspace = true }
```
So you can import it in your module:
```
use curl_effector_imports as curl;
```

You can see more in the [example](./example/README.md).


# How to verify the effector's CID

The CID of the effector module can be found on the release page for the chosen effector.

If you want to manually validate the CID of the module, you can:
1. Download and unpack the effector archive located on the release page.
2. Download IPFS CLI 
3. Call this IPFS command to evaluate the CID
   ```
   ipfs add --only-hash -Q --cid-version 1 --hash sha2-256 --chunker=size-262144 curl_effector.wasm
   ```

# Example

In the `example/`, you may find the ways to import the module into your project and use in the facade module.
Read `example/README.md` for more details.

# How to start developing the effectors

We prepared the [effector-template](https://github.com/fluencelabs/effector-template/) repository to serve as a basic effector template. We build our cURL and IPFS effector on top of this template as an example.
