# witd

*Compiling witd yourself is highly experimental and will likely not work. Please use the HTTP API or the iOS/Android/Python/Ruby SDKs for non-experimental needs (https://wit.ai/docs). This message will be removed once the code gets updated.*

`witd` is a light executable that makes it easy to use Wit.ai on a wide range of devices. It manages client audio recording and communication with Wit.ai.

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

`witd` also depends on `sox` and `curl`, which you can install using the package manager for
your system.

## Start the server

Start the server:

```bash
$ HOST=0.0.0.0 PORT=9877 ./target/witd
```
The `HOST` and `PORT` variables are optional, but you can override them to use different values, or use the `-a` and `-p` options instead. Run `./target/witd --help` for more details.

## Send requests

### Speech request

For the moment, witd requires that you tell it when to start recording. For stopping, you can either do it manually or automatically detect when the user stops speaking. In the future, we plan to automatically detect when the user starts talking as well.

#### Without end-of-speech detection

Start recording audio and streaming it to wit.ai:

```bash
$ curl -X GET "http://localhost:9877/start?access_token=<YOUR_ACCESS_TOKEN>"
```

Stop recording audio and receive the wit.ai response:
```bash
$ curl -X GET "http://localhost:9877/stop"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

#### With end-of-speech detection

Start recording audio and streaming it to wit.ai, and return a response when the user stops speaking:

```bash
$ curl -X GET "http://localhost:9877/start?autoend=true&access_token=<YOUR_ACCESS_TOKEN>"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

Note the use of the argument `autoend=true`.

If you want all your queries to use end-of-speech detection, you can also start witd with "-e true":

```bash
$ HOST=0.0.0.0 PORT=9877 ./target/witd -e true
```

Note that the `curl` call blocks until the user has stopped speaking and a response is available. If the end of speech detection fails, you can still use `curl -X GET "http://localhost:9877/stop` to stop streaming audio and get the response immediately. 

### Text request

witd can also directly send text requests to wit.ai:

```bash
$ curl -X GET "http://localhost:9877/text?q=Hello%20world&access_token=<YOUR_ACCESS_TOKEN>"
{"_text":"Hello world","msg_id":"fbe2a1ff-3869-49d8-885d-67e23357ffdc","outcomes":[{"_text":"Hello world","confidence":0.263,"entities":{"location":[{"suggested":true,"value":"Hello world"}]},"intent":"get_weather"}]}
```

## Running on Raspberry Pi

The easiest way to have witd running on a Raspberry Pi is to trust us and run the provided ARM binary:

```bash
wget https://github.com/wit-ai/witd/releases/download/v0.1/witd-armv6
chmod +x witd-armv6
./witd-armv6
```

### Building witd-arm for Raspberry Pi (for the brave)

The procedure below describes how to cross-compile witd-arm on a Debian host targeting Raspbian. It may work with other configurations but has not been tested yet.

*Disclaimer: it is very ugly. The idea is to install the required libraries on the Rasperry Pi, and mount the Raspberry's /lib folders on the Debian host remotely, so that the linker running on the Debian host can access the Pi's libraries. Another approach would be to manually copy the libraries from the Raspberry Pi to the Debian host, but this often ends up in a dependency hell.*

* Setup a Rust cross-compiler by following [these instructions](https://github.com/npryce/rusty-pi/blob/master/doc/compile-the-compiler.asciidoc). However, make sure to pass the extra `--enable-rpath` argument to the configure script:
```bash
./configure --target=arm-unknown-linux-gnueabihf --prefix=$HOME/pi-rust --enable-rpath && make && make install
```
* Install the required libraries on the Raspberry Pi
```bash
pi@raspberrypi ~$ sudo apt-get install libssl-dev libcurl4-openssl-dev libcrypto++-dev libsox
pi@raspberrypi ~$ cp /usr/lib/libsox.so /usr/lib/arm-linux-gnueabihf
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
where `pi` is a user on the Raspberry Pi and 192.168.1.54 is the IP of the Raspberry Pi. The script will mount the /usr/lib/arm-linux-gnueabihf and /lib/arm-linux-gnueabihf folders of your Raspberry Pi at the same locations on your Debian host, so you need to:
* have read access to /usr/lib/arm-linux-gnueabihf and /lib/arm-linux-gnueabihf on the Raspberry Pi (which is the case for the default user on Raspbian)
* be able to sudo on the Debian host
* make sure you don't already have a /usr/lib/arm-linux-gnueabihf or /lib/arm-linux-gnueabihf on your Debian host

The resulting executable should be in `target/arm-unknown-linux-gnueabihf/witd`.

[rust]: http://rust-lang.org
[cargo]: http://crates.io
