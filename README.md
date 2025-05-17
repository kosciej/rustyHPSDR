# rustyHPSDR

My first attempt at a Rust application.

This is very early code to implement a UI in Rust using gtk4-rs to implement a Radio for the OpenHPSDR Protocols.

The current code only implements a Receiver. It does work with Protocol 1 and Protocol 2 radios including the Hermes Lite. I will be adding more features including Transmit over the next few weeks.

I am using WDSP as an extern C library so you need to install my port of WDSP on [github](https://github.com/g0orx/wdsp.git). I have not currently implemented a Rust wrappper so all the calls to it have to be wrapped with "unsafe { ... }".

You will need to install Rust and Cargo. See [Rust install](https://www.rust-lang.org/tools/install) for information on installing Rust and Cargo.

# When the application is run it will first discovery all the HPSDR compatable devices on the nework interfaces.

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/discovery.png">

# If you get this after clicking on Start and before the Radio starts running

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/wait.png">

You can either just wait or click on Wait to close the dialog or you can change the timeout value by installing dconf-editor and change the /org/gnome/mutter/check-alive-timeout value from the default of 5000 to 60000.

# When a device is selected and Start button clicked the radio will start running

Currently Split, Mic Gain and Drive do not do anything. They will once transmit is implemented.

Most of the other buttons are self explanitary.

To change frequency you can use the scroll wheel in the VFO window, the Spectrum window and Waterfall window or you can click on the frequency to move to.

The CTUN button enables Click Tuning. When active it allows you to click within the Spectrum or Waterfall to QSY without shifting the Spectrum or Waterfall display. When "CTUN" is inactive, clicking on the Spectrum re-centers the Spectrum, thereby shifting the entire Spectrum. 

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/screenshot1.png">

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/screenshot2.png">

# Adjusting spectrum and waterfall display

If you move the mouse over to the left side of the spectrum display the cursor will change to an up/down arrow that will allow you to use the scroll wheel to move the limits up and down for the current band.

You can do the same fo the waterfall display to adjust the limits up and down for the current band.

Note that the Configure button currently brings up a dialog that lets you set the limits for all bands.

<img src="https://github.com/g0orx/rustyHPSDR/blob/main/images/cursor.png">

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

# Running rustyHPSDR
<pre>
cargo run
</pre>

