# witd

`witd` is a light executable that makes it easy to use Wit.ai on a wide range of devices. It manages client audio recording and communication wit Wit.ai.

## Why?

Wit.ai allows developers to build smart apps and devices. By streaming end-users' voice to Wit's speech-to-action API, it is possible to determine the intent of the end users as well as all the details required to fulfill their requests.

However, developers are often in trouble when it comes to recording the audio and streaming it to Wit.ai API.

We provide clients that do exactly that for iOS and Android.
`witd` is an attempt to do the same for more devices, e.g. Raspberry Pi, BeagleBone, etc.

## How to build

`witd` is written in [Rust][rust]. We recommend you use [Cargo][cargo] to build it:

```bash
$ cargo build
```

## Start the server

Start the server:

```bash
$ HOST=0.0.0.0 PORT=9877 ./target/witd
```

## Send requests

### Speech request

For the moment, witd requires that you tell it when to start and stop recording. Moving forward, 2 new modes will be added:

- "silence detection" to stop the recording
- "hands-free" to start and stop the recording when user is speaking

Start recording audio and streaming it to wit.ai:

```bash
$ curl -X GET "http://localhost:9877/start&access_token=<YOUR_ACCESS_TOKEN>"
```

Stop recording audio and receive the wit.ai response:
```bash
$ curl -X GET "http://localhost:9877/stop"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

### Text request

witd can also directly send text requests to wit.ai:

```bash
$ curl -X GET "http://localhost:9877/text?q=Hello%20world&access_token=<YOUR_ACCESS_TOKEN>"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

## Running on Raspberry Pi

The easiest way to have witd running on a Raspberry Pi is to run the provided ARM binaries:

```bash
./witd-arm
```

### Building witd-arm for Raspberry Pi (for the brave)

The procedure below describes how to cross-compile witd-arm on a Debian host targeting Raspbian. It may work with other configurations but has not been tested yet.

* Setup a Rust cross-compiler by following [these instructions](https://github.com/npryce/rusty-pi/blob/master/doc/compile-the-compiler.asciidoc). However, make sure to pass the extra `--enable-rpath` argument to the configure script:
```bash
./configure --target=arm-unknown-linux-gnueabihf --prefix=$HOME/pi-rust --enable-rpath && make && make install
```
* Install the required libraries on the Raspberry Pi, and create a libportaudio.so symlink:
```bash
pi@raspberrypi ~$ sudo apt-get install libssl-dev libcurl4-openssl-dev libcrypto++-dev libportaudio-dev
pi@raspberrypi ~$ ln -s /usr/lib/arm-linux-gnueabihf/libportaudio.so.2 /usr/lib/arm-linux-gnueabihf/libportaudio.so
```
* Install sshfs so that the build script running on the host can access the precompiled libraries on the Raspberry Pi by mounting a remote filesystem:
```bash
sudo apt-get install sshfs
sudo modprobe fuse
sudo adduser `whoami` fuse
sudo chown root:fuse /dev/fuse
sudo chmod +x /bin/fusermount
newgrp fuse (or logout/login)
newgrp
```
* Run the build script:
```bash
./raspbuild pi@192.168.1.54
```
where `pi` is a user on the Raspberry Pi and 192.168.1.54 is the IP of the Raspberry Pi. You need read access to /usr/lib/arm-linux-gnueabihf and /lib/arm-linux-gnueabihf on the Raspberry Pi (which is the case for the default user on Raspbian). You may be prompted for your Debian and/or Raspberry Pi password.

The resulting executable should be in `target/arm-unknown-linux-gnueabihf/witd`.

[rust]: http://rust-lang.org
[cargo]: http://crates.io
