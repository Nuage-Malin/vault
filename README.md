# vault
Remotely controlled filesystem :
Receives procedure calls from Maestro (scheduler) and consequently executes system calls on the current machine to manage the filesystem.

## Run (docker)

```sh
./scripts/launch_vault.sh
######### or
./scripts/launch_vault-cache.sh
```

## Test
```sh
./scripts/launch_unit_tests.sh
```
