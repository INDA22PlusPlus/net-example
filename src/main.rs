//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::{
    event,
    graphics::{self, Color},
    input::keyboard::KeyCode,
    Context, GameResult,
};
use glam::*;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(PartialEq)]
enum State {
    Playing,
    WaitingForOpponent,
}

const TILE_SIZE: f32 = 1000.0 / 8.0;

struct MainState {
    // Logic
    player_pos: (u8, u8),
    enemy_pos: (u8, u8),
    state: State,

    // Networking
    stream: TcpStream,

    // Rendering
    arena_image: graphics::Image,
    rect: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // A stream and a boolean indicating wether or not the program is a host or a client
        let (stream, client) = {
            let mut args = std::env::args();
            // Skip path to program
            let _ = args.next();

            // Get first argument after path to program
            let host_or_client = args
                .next()
                .expect("Expected arguments: --host or --client 'ip'");

            match host_or_client.as_str() {
                // If the program is running as host we listen on port 8080 until we get a
                // connection then we return the stream.
                "--host" => {
                    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
                    (listener.incoming().next().unwrap().unwrap(), false)
                }
                // If the program is running as a client we connect to the specified IP address and
                // return the stream.
                "--client" => {
                    let ip = args.next().expect("Expected ip address after --client");
                    let stream = TcpStream::connect(ip).expect("Failed to connect to host");
                    (stream, true)
                }
                // Only --host and --client are valid arguments
                _ => panic!("Unknown command: {}", host_or_client),
            }
        };

        // Set TcpStream to non blocking so that we can do networking in the update thread
        stream
            .set_nonblocking(true)
            .expect("Failed to set stream to non blocking");

        let arena_image = graphics::Image::from_path(ctx, "/arena.png", true)?;
        let rect = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect {
                x: 0.0,
                y: 0.0,
                w: TILE_SIZE,
                h: TILE_SIZE,
            },
            Color::WHITE,
        )?;

        Ok(MainState {
            player_pos: if client { (7, 7) } else { (0, 0) },
            enemy_pos: if client { (0, 0) } else { (7, 7) },
            // Host starts playing and the client waits
            state: if client {
                State::WaitingForOpponent
            } else {
                State::Playing
            },
            stream,
            arena_image,
            rect,
        })
    }

    /// Checks if a move packet is available in returns the new positions otherwise it returns none
    fn recieve_move_packet(&mut self) -> Option<(u8, u8)> {
        let mut buf = [0u8; 2];
        match self.stream.read(&mut buf) {
            Ok(_) => Some((buf[0], buf[1])),
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => None,
                _ => panic!("Error: {}", e),
            },
        }
    }

    /// Sends a move packet of the current position and sets the state to waiting
    fn send_move_packet(&mut self) {
        let mut buf = [self.player_pos.0, self.player_pos.1];
        self.stream
            .write(&mut buf)
            .expect("Failed to send move packet");
        self.state = State::WaitingForOpponent;
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        if self.state == State::Playing {
            if let Some(keycode) = input.keycode {
                match keycode {
                    KeyCode::Left | KeyCode::A => {
                        if self.player_pos.0 > 0 {
                            self.player_pos.0 -= 1;
                            self.send_move_packet();
                        }
                    }
                    KeyCode::Right | KeyCode::D => {
                        if self.player_pos.0 < 8 {
                            self.player_pos.0 += 1;
                            self.send_move_packet();
                        }
                    }
                    KeyCode::Down | KeyCode::S => {
                        if self.player_pos.1 < 8 {
                            self.player_pos.1 += 1;
                            self.send_move_packet();
                        }
                    }
                    KeyCode::Up | KeyCode::W => {
                        if self.player_pos.1 > 0 {
                            self.player_pos.1 -= 1;
                            self.send_move_packet();
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        match self.state {
            State::Playing => {}
            State::WaitingForOpponent => {
                // If we recieved at move packet we first set the enemy pos to the recieved
                // position and then set the state to playing
                if let Some(pos) = self.recieve_move_packet() {
                    self.state = State::Playing;
                    self.enemy_pos = pos;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
        );

        canvas.draw(&self.arena_image, graphics::DrawParam::new());

        // Draw self
        canvas.draw(
            &self.rect,
            graphics::DrawParam::new()
                .offset(-Vec2::new(self.player_pos.0 as f32, self.player_pos.1 as f32) * TILE_SIZE)
                .color(Color::BLUE),
        );

        canvas.draw(
            &self.rect,
            graphics::DrawParam::new()
                .offset(-Vec2::new(self.enemy_pos.0 as f32, self.enemy_pos.1 as f32) * TILE_SIZE)
                .color(Color::RED),
        );

        canvas.finish(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };
    let cb = ggez::ContextBuilder::new("net-example", "antlilja")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("net-example"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1000.0, 1000.0));
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
