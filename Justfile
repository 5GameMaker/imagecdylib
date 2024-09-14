build:
    rm -fr build
    mkdir -p build/include build/lib
    cargo build --release
    cp target/release/*imagecdylib* build/lib
    cbindgen -c Cbindgen.toml --lang C -o build/include/imagecdylib.h
