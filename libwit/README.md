# libwit

`libwit` is a small library that makes it easy to integrate Wit.ai with many programming languages. It manages client audio recording and communication with Wit.ai.

To compile the C-compatible library, first run

```bash
cargo build
```

and then

```bash
./build_c.sh
```

This will create a `libwit.a` file in this folder.

To compile the example, run

```bash
cd example
gcc -Wall -o test test.c -I ../include -L ../lib -lwit -lsox -lcurl -lcurl -lcurl -lcurl -lSystem -lpthread -lc -lm
```
