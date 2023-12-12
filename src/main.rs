use JEmulator::dissasembler::Dissasembler;
use JEmulator::cpu::Cpu;
use JEmulator::joypad::{ ButtonEvent, ButtonEventWrapper };

use std::time::Instant;
use std::sync::{ Arc, Mutex };
use std::sync::mpsc::channel;
use std::thread;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit::dpi::LogicalSize;
use winit::event::{ Event, WindowEvent, VirtualKeyCode, ElementState };

use pixels::{ SurfaceTexture, Pixels };

const MAX_CYCLES: u64 = 69905;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;
const PIXEL_SIZE: u32 = 2;

#[derive(Default, Clone, Copy)]
pub struct ColorPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::new().filter_or("", "info")).init();
    let event_loop = EventLoop::new();
    
    let window = {
        let size = LogicalSize::new((WIDTH*PIXEL_SIZE) as f64, (HEIGHT*PIXEL_SIZE) as f64);
        WindowBuilder::new()
            .with_title("Jack's Emulator!")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH*PIXEL_SIZE, HEIGHT*PIXEL_SIZE, surface_texture).unwrap()
    };
    

    let pixel_array1 = Arc::new(Mutex::new([[ColorPixel::default(); WIDTH as usize]; HEIGHT as usize]));
    let pixel_array2 = Arc::clone(&pixel_array1);
    
    let (event_sender, event_receiver) = channel::<ButtonEventWrapper>();
    let (render_sender, render_receiver) = channel::<()>();
    
    thread::spawn(move || {
        let d = Dissasembler::new().unwrap();
        let mut cpu = Cpu::from_rom(include_bytes!("../roms/drMario.gb").to_vec());
        
        loop {
            
            let start = Instant::now();
            let mut cycles = 0;

            while cycles < MAX_CYCLES {
                for event in event_receiver.try_iter() {
                }
            
                let tick_cycles = cpu.tick(&d);
                cycles += tick_cycles as u64;

                cpu.mmu.tick(tick_cycles);
            }

            'here: loop {
                if start.elapsed().as_nanos() >= 16_666_667 {
                    break 'here;
                }
            }

            render_sender.send(()).unwrap();
            
        }
    });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        
        match event {
            Event::WindowEvent { window_id, event } => {
                match event {
                    WindowEvent::CloseRequested => std::process::exit(0),
                    WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                        if let Some(code) = input.virtual_keycode {
                            let event = match code {
                                VirtualKeyCode::A => ButtonEvent::A,
                                VirtualKeyCode::S => ButtonEvent::B,
                                VirtualKeyCode::Return => ButtonEvent::Start,
                                VirtualKeyCode::Space => ButtonEvent::Select,
                                VirtualKeyCode::Right => ButtonEvent::Right,
                                VirtualKeyCode::Left => ButtonEvent::Left,
                                VirtualKeyCode::Up => ButtonEvent::Up,
                                VirtualKeyCode::Down => ButtonEvent::Down,
                                _ => ButtonEvent::None,
                            };
                            
                            event_sender.send(ButtonEventWrapper { event, new_state: input.state }).unwrap();
                        }
                    }
                    _ => ()
                }
            },
            Event::MainEventsCleared => {
                let _ = render_receiver.try_recv();
            }
            _ => ()
        }
    })
    
}
