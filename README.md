# stuffed
A little CLI and library for pulling and pushing Wasm components as OCI artifacts

## How to use stuffed

```shell
# start your local registry
docker run -d -p 5000:5000 --restart=always --name registry registry:2

# push an image to the local registry with reference wasm/my-component:v1.0.0.
$ cargo run -- push --reference localhost:5000/wasm/my-component:v1.0.0 --root-path simple-grep-1.0.0.wasm -i
    Finished dev [unoptimized + debuginfo] target(s) in 0.35s
     Running `target/debug/stuffed push --reference 'localhost:5000/wasm/my-component:v1.0.0' --root-path ../../bca/registry/demo/simple-grep-1.0.0.wasm -i`
pushed: localhost:5000/wasm/my-component:v1.0.0 with digest: 4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37

# pull that same image and save the content of the first layer in the current directory.
$ cargo run -- pull --reference localhost:5000/wasm/my-component:v1.0.0 -i
    Finished dev [unoptimized + debuginfo] target(s) in 0.24s
     Running `target/debug/stuffed pull --reference 'localhost:5000/wasm/my-component:v1.0.0' -i`
pulled: ./sha256-4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37

# Sure enough, it is in our local directory
$ ls -la | grep sha
-rw-r--r--@  1 david  staff  2158774 Sep 19 21:31 sha256-4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37

# Check and see if that file is really a Wasm module (yeah... it's not a component, but we'll fix that).
$ file ./sha256-4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37
./sha256-4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37: WebAssembly (wasm) binary module version 0x1 (MVP)

# Use regctl to inspct the local registry and list the images.
$ regctl repo ls localhost:5000
wasm/my-component

# Use regctl to list the tags for our image.
$ regctl tag ls localhost:5000/wasm/my-component
v1.0.0

# Use regctl to inspect the manifest for our image. Note the single layer. That is our Wasm module.
$ regctl manifest get localhost:5000/wasm/my-component:v1.0.0
Name:         localhost:5000/wasm/my-component:v1.0.0
MediaType:    application/vnd.oci.image.manifest.v1+json
ArtifactType: application/vnd.bytecodealliance.component.v1+wasm
Digest:       sha256:f7e89cb4ceb71fbf13fc7e356df34e18df25ea2a4ba89238fcf398985943cb39
Total Size:   2.159MB

Config:
  Digest:     sha256:e10a56793af6896f902c20855284a09d1d3056a4eb762593d6dd17649472b97b
  MediaType:  application/vnd.oci.image.config.v1+json
  Size:       170B

Layers:

  Digest:     sha256:4470bd2867f49ef7119691d6e1cb9a4ecd46b79c394a7587dc78874c9382ca37
  MediaType:  application/vnd.bytecodealliance.wasm.component.layer.v0+wasm
  Size:       2.159MB
  
 
```

## Status
This is just a quick hack to put flow back and forth with an OCI registry. 
