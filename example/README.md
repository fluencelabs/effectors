# Fluence Minimal Template

## Usage

```sh
# Generate a service template and add it to the default worker
fluence service new myService

# Deploy the default worker
fluence deploy

# Uncomment `runDeployedServices` aqua function in `src/aqua/main.aqua` and run it
fluence run -f 'runDeployedServices()'
```
