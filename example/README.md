# Effectors usage example

This project provides an example of effector usage in a real project.

# How to build the project

First, you should call `./update.sh` with command will build all required effector modules and move them into the project.
Then call `fluence build` to actually build them.

You may find it troublesome to test the project in the repl since the effectors operates via the Particle Vault which doesn't have reliable support in the repl.
So you should use the local network to test it:
1. `fluence default env local` to set up the default enviroment
2. `fluence local up` to set up the local network
3. `fluence deal deploy`
4. `fluence run -f 'runDeployedServicesHttp()'` to run the cURL effector example
5. `fluence run -f 'runDeployedServicesIpfs()'` to run the IPFS effector example


### Example
```
$ fluence service repl myRPC
...
1> call myRPC simple_get_ipfs ["/dns4/ipfs.fluence.dev/tcp/5001", "bafkreibezaflnu3lp34vpgvear4ni2ggdpbkcif7o5vrll7a6ldlfeiura"]
result: "<3\n"
 elapsed time: 279.169667ms
2> call myRPC simple_get_http ["http://google.com"]
result: "<HTML><HEAD><meta http-equiv=\"content-type\" content=\"text/html;charset=utf-8\">\n<TITLE>301 Moved</TITLE></HEAD><BODY>\n<H1>301 Moved</H1>\nThe document has moved\n<A HREF=\"http://www.google.com/\">here</A>.\r\n</BODY></HTML>\r\n"
 elapsed time: 124.668617ms

```

# How we use effector in this project

## File layout

First we need to add the modules into the service file structure:
```
$ tree src/services
src/services
└── myRPC
    ├── modules
    │   ├── curl_effector
    │   │   ├── curl_effector.wasm
    │   │   └── module.yaml
    │   ├── ipfs_effector
    │   │   ├── ipfs_effector.wasm
    │   │   └── module.yaml
    │   └── myRPC
    │       ├── Cargo.toml
    │       ├── module.yaml
    │       └── src
    │           └── main.rs
    └── service.yaml
```

Note that later we may add a better support for compiled modules in CLI, so you don't need to copy them in your project.

Then, we should add them in the service definition `service.yaml`:
```
name: myRPC

modules:
  facade:
    get: modules/myRPC
  curl_effector:
    get: modules/curl_effector/
  ipfs_effector:
    get: modules/ipfs_effector/
```

## Import module definition

To use an external module from your module, you need to import its declaration. For effectors, we provided the crates `<name>-effector-imports` that imports all the required types and the definition of the module.
So, in your module, it's enough just to import `use curl_effector_imports::*;` and you can use the exported module functions like `curl_get` and `curl_post`.


# Troubleshootings

## Linking
If you see some unpleasant `marine-rs-sdk` errors during linking stage of the service compilation, you may need to check that the `marine-rs-sdk` versions used to compile your modules and the same.
