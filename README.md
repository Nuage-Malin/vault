# vault
Remotely controlled filesystem :
Receives procedure calls from Maestro (scheduler) and consequently executes system calls on the current machine to manage the filesystem.

## Build (docker)

```sh
docker compose build --env-file env/vault.env
```

## Run (docker)

```sh
docker compose up --env-file env/vault.env
```
