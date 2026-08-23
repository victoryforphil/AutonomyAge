# vs-broker

The Victory Suite broker. A broker node system with commanders, task
definitions, and TCP/channel adapters, built on `vs-data-store` and `vs-wtf`.

Part of the [Victory Suite](../) workspace (under `libs/`).

```sh
protoc --proto_path=../admin-proto pubsub_admin.proto --grpc-web_out=import_style=typescript,mode=grpcwebtext:src --js_out=import_style=commonjs,binary:src
```
