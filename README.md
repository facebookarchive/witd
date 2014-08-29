#WitD-rust

WitD is a light executable that integrates any devices to Wit.AI servers.

##How to build

WitD is written in rust. You can easily build it with :

```bash
$ cargo build
```

##Launch the server

To launch the server :

```bash
$ HOST=0.0.0.0 PORT=9877 ./target/witd-rust
```

##Send request

###Text request 

```bash
$ curl -X GET "http://localhost:9877/text?q=Hello%20world"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

###Voice request

###Running on Raspberry Pi

The easiest way to have WitD running on a Raspberry Pi is to run the provided precompiled executable:

```bash
./witd-rust
```

If you insist on compiling it yourself:

1) Setup a Rust cross-compiler by following [these instructions]("https://wit.ai").
2) Setup sshfs on your system
3) Run the build script:
```bash
./build
```

 
