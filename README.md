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
Tests are numbered because their execution has to follow a specific order (the remove file can't work if the create file has not been executed previously)

```sh
./scripts/launch_unit_tests.sh
```
