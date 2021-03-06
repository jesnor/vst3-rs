# vst3-rs: easy to use Rust wrapper for the [VST3 SDK](https://github.com/steinbergmedia/vst3sdk)

**NOTE: This project is in heavy development and the code/API may change at any time**

The goal of this crate is to make it as easy as possible to create VST3 plugins using idiomatic and 100% safe Rust code.
Internally the [vst3-sys](https://github.com/RustAudio/vst3-sys) crate is used for accessing the VST3 COM objects/interfaces in
pure, but unsafe Rust. This crate wraps all the COM objects/interfaces in more idiomatic and totally safe Rust traits which are very
straightforward to use. Basically you just implement one or two traits and get pretty much all VST3 functionality for free.

I'm currently working on GUI intergration using the [egui crate](https://github.com/emilk/egui). The goal here is that you will
get an automatically generated GUI containing all your plugin parameters without writing a single line of code. However, 
if you need specific GUI functionality, you can of course override/extend the default GUI.

This crate has only been tested on Windows, but it should be possible to port to other platforms with little (or no) effort.
I'm grateful for any contributions in this area.
