# Fluence Effectors

List of supported effectors:
- IPFS: support only `get` and `add` commands
- cURL: support only `post` and `get` HTTP requests types


# How to get the effector's CID

Note that the same intruction should be correct for the all the supported effectors.

Run `./curl/build.sh 2>/dev/null`. This command will hide the debug output and will print you the CID of compiled effector:
```bash
$ ./curl/build.sh 2>/dev/null
bafkreig5td7jalnbsgff5egar7vvgsracqbpgxuxf5fjbexjzesbxii334
```

Remove `2>/dev/null` and run the command again if you encounter any problems for debug output.

# How to get the effector's CID manually

You can use the IPFS binary yourself and call the following command:
```
ipfs add --only-hash -Q --cid-version 1 --hash sha2-256 --chunker=size-262144 $WASM_FILE_PATH
```

# How to use in your marine service

Each effector provides the rust crate with relevant types and module import declaration.
- `curl-effector-imports` for the cURL effector
- `ipfs-effector-imports` for the IPFS effector

# Example

In the `example/` you may find the ways to import the module into your project and use in the facade module.
Read `example/README.md` for more details.
