//! This crate contains client-only components of Playform.

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(convert)]
#![feature(main)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]
#![feature(ptr_as_ref)]

#![allow(mutex_atomic)]

#![plugin(clippy)]

extern crate bincode;
extern crate cgmath;
extern crate common;
extern crate env_logger;
extern crate gl;
#[macro_use]
extern crate log;
extern crate libc;
extern crate num;
extern crate sdl2;
extern crate sdl2_sys;
extern crate stopwatch;
extern crate rustc_serialize;
extern crate test;
extern crate thread_scoped;
extern crate time;
extern crate yaglw;

mod camera;
mod client;
mod fontloader;
mod hud;
mod light;
mod load_terrain;
mod main;
mod mob_buffers;
mod player_buffers;
mod process_event;
mod render;
mod server;
mod server_update;
mod shaders;
mod terrain_buffers;
mod ttf;
mod update_thread;
mod vertex;
mod view;
mod view_thread;
mod view_update;
