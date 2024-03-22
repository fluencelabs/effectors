# Effectors usage example

This project provides an example of effector usage in a real project.

# How to build the project

You can just call `fluence build` to actually build the `myRPC` service.

You may find it troublesome to test the project in the repl since the effectors operate via the Particle Vault which doesn't have reliable support in the repl.
So you should use the local network to test it:
1. `fluence default env local` to set up the default enviroment
2. `fluence local up` to set up the local network
3. `fluence deal deploy`
4. `fluence run -f 'runDeployedServicesHttp()'` to run the cURL effector example
5. `fluence run -f 'runDeployedServicesIpfs()'` to run the IPFS effector example


### Example

You can check how the service is run via REPL:
```
$ fluence service repl myRPC
...

1> call myRPC simple_get_ipfs [ "/ip4/127.0.0.1/tcp/5001", "bafkreibezaflnu3lp34vpgvear4ni2ggdpbkcif7o5vrll7a6ldlfeiura" ] { "particle": {"id": "id", "token": "token"} }
result: "<3\n"
 elapsed time: 26.496958ms

2> call myRPC simple_get_http ["http://google.com"] { "particle": { "id": "id", "token": "token"} }
result: "<HTML><HEAD><meta http-equiv=\"content-type\" content=\"text/html;charset=utf-8\">\n<TITLE>301 Moved</TITLE></HEAD><BODY>\n<H1>301 Moved</H1>\nThe document has moved\n<A HREF=\"http://www.google.com/\">here</A>.\r\n</BODY></HTML>\r\n"
 elapsed time: 75.998552ms

```

Note that when calling a service, we also set call parameters that define a particle that is calling the service: `{ "particle": { "id": "id", "token": "token"} }`.
It's required to use the effectors since they operate via the Particle Vault.

If you see errors when calling the functions, you may need to manually create a temporary directory for your particle vault (note that the directory name consist of the `id` and `token` of the call parameters).
```
mkdir .fluence/tmp/volumes/particles/id-token
```


# How we use effector in this project

After project initialization, you see a basic Fluence CLI project layout:
```
$ tree src/services
src
├── aqua
│   └── main.aqua
└── services
    └── myRPC
        ├── modules
        │   └── myRPC
        │       ├── Cargo.toml
        │       ├── module.yaml
        │       └── src
        │           └── main.rs
        └── service.yaml
```

First, we added the effector modules with `fluence module add $EFFECTOR_PACKAGE_URL --service myRPC` command. This command DIDN'T change the layout of your project, but add the links to the `service.yaml` file:
```yaml
name: myRPC

modules:
  facade:
    get: modules/myRPC
  curl_effector:
    get: https://github.com/fluencelabs/curl-effector/releases/download/effector-v0.1.1/curl_effector.tar.gz
  ipfs_effector:
    get: https://github.com/fluencelabs/ipfs-effector/releases/download/effector-v0.1.1/ipfs_effector.tar.gz
```

These modules will be downloaded on `fluence service build`. You can find the downloaded modules in your `.fluence/` directory:
```
$ tree .fluence/modules
.fluence/modules
├── curl_effector.tar.gz_5841c1968f8a4f4f75c000d9c7600373
│   ├── curl_effector.wasm
│   └── module.yaml
└── ipfs_effector.tar.gz_d28bf5174afc4c6aaf5f44cf406c6792
    ├── ipfs_effector.wasm
    └── module.yaml
```

The `fluence module add` command also adds *binding crates* of the added effector to the project's root `Cargo.toml`:
```toml
[workspace.dependencies.curl-effector-imports]
version = "0.1.1"

[workspace.dependencies.ipfs-effector-imports]
version = "0.1.1"
```

These crates can be used to import the effector module into your module:
```rust
// From src/services/myRPC/modules/myRPC/src/main.rs
use curl_effector_imports as curl;
use curl_effector_imports::CurlRequest;

use ipfs_effector_imports as ipfs;
```

However, do not forget to use this crates in your module's `Cargo.toml`:
```
[dependencies]
curl-effector-imports = { workspace = true }
ipfs-effector-imports = { workspace = true }
```

## Import module definition

To use an external module from your module, you need to import its declaration. For effectors, we provided the crates `<name>-effector-imports` that import all the required types and the definition of the module.
So, in your module, it's enough just to import `use curl_effector_imports::*;`, and you can use the exported module functions like `curl_get` and `curl_post`.


# Troubleshootings

## Linking

If you see some unpleasant `marine-rs-sdk` errors during the linking stage of the service compilation, you may need to check that the `marine-rs-sdk` versions used to compile your modules are the same among the modules. To check the version using the Marine CLI (installed by Fluence CLI) like this:
```
$ marine info .fluence/modules/curl_effector.tar.gz_5841c1968f8a4f4f75c000d9c7600373/curl_effector.wasm
it version:  0.27.0
sdk version: 0.14.0 # <<--- this is the version of `marine-rs-sdk`
```

## Example Error

If running this example you encounter such error:
```
2> call myRPC simple_get_http ["http://google.com"] { "particle": { "id": "id", "token": "token"}}
result: "curl cli call failed \n\"http://google.com/ -X GET -o /.fluence/tmp/volumes/particles/id-token/some_path --connect-timeout 4 --no-progress-meter --retry 0\": error: , stderr: Warning: Failed to open the file \nWarning: .fluence/tmp/volumes/part\nWarning: icles/id-token/some_path: No such file or directory\ncurl: (23) Failure writing output to destination\n"
```
You may need to create a directory for the particle vault manually:
```
mkdir .fluence/tmp/volumes/particles/id-token
```
