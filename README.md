# rustyHPSDR

My first attempt at a Rust application.

This is very early code to implement a UI in Rust using gtk4-rs to implement a Radio for the OpenHPSDR Protocols.

The current code only implements a Receiver. It does work with Protocol 1 and Protocol 2 radios including the Hermes Lite.

I am using WDSP as an extern C library so you need to install my port of WDSP on [github](https://github.com/g0orx/wdsp.git). I have not currently implemented a Rust wrappper so all the calls on have to wrapped with "unsafe { ... }".

You will need to install Rust and Cargo. See [Rust install](https://www.rust-lang.org/tools/install) for information on istalling Rust and Cargo.

# pre-requisites for WDSP
<pre>
sudo apt install -y libfftw3-dev
</pre>

# Download and compile WDSP
<pre>
git clone https://github.com/g0orx/wdsp.git
cd wdsp
make clean
make
sudo make install
</pre>

# pre-requisites for rustyHPSDR
<pre>
sudo apt install -y libgtk-4-dev
</pre>

# Download and compile rustyHPSDR
<pre>
git clone https://github.com/g0orx/rustyHPSDR.git
cd rusyHPSDR
cargo clean
cargo build
</pre>

