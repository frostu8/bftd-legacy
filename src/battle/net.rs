//! A networked battle using [`backroll`].

use super::{Arena, FRAMES_PER_SECOND, State};
use super::script::Scope;

use crate::input::{Buffer as InputBuffer, sampler::Handle as InputHandle, Inputs};
use crate::render::Renderer;
use crate::Context;

use backroll::{P2PSession, P2PSessionBuilder, PlayerHandle, command::{Command, Commands}};
use backroll_transport_udp::UdpManager;

use anyhow::Error;

use std::net::SocketAddr;

/// A networked battle manager with a local player and a remote peer.
pub struct NetBattle {
    arena: Arena,
    session: P2PSession<NetConfig>,
    transport: UdpManager,
    // the player at index 0 is left, index 1 is right.
    players: [Player; 2],
}

/// Config for use in initialization of a [`NetBattle`].
pub enum NetPlayer {
    /// A local player with an input handle.
    Local(InputHandle),
    /// A remote player with a [`SocketAddr`] to bind to.
    Remote(SocketAddr),
}

impl NetBattle {
    /// Creates a new `NetBattle` with a given config.
    ///
    /// This does not perform any I/O and just sets up reading and writing. The
    /// [`Arena`] passed must have been synced beforehand.
    ///
    /// # Panics
    /// Panics if more than one local player is supplied. Only give one!
    pub fn new(
        cx: &mut Context,
        arena: Arena,
        players: &[NetPlayer; 2],
    ) -> Result<NetBattle, Error> {
        // initialize transport
        let bind_addrs = players
            .into_iter()
            .filter_map(|p| match p {
                NetPlayer::Remote(s) => Some(s),
                NetPlayer::Local(_) => None,
            })
            .copied()
            .collect::<Vec<SocketAddr>>();
        let transport = UdpManager::bind(cx.task_pool.clone(), &*bind_addrs)?;

        // initialize session
        let mut session = P2PSessionBuilder::<NetConfig>::new().with_frame_delay(0);

        todo!();
    }

    /// Polls an update for the `NetBattle`.
    pub fn update(&mut self, cx: &mut Context) -> Result<(), Error> {
        self.handle_commands(cx, self.session.poll())?;

        while cx.frame_limiter.should_update(FRAMES_PER_SECOND) {
            // sample input from the local player(s)
            for player in self.players.iter() {
                if let Some(input) = player.sample_local(cx) {
                    self.session.add_local_input(player.handle, input)?;
                }
            }

            // handle commands
            self.handle_commands(cx, self.session.advance_frame())?;
        }

        Ok(())
    }
    
    /// Draws the battle to a graphics context.
    pub fn draw(&mut self, cx: &mut Renderer) -> Result<(), Error> {
        self.arena.draw(cx)
    }

    fn handle_commands(
        &mut self,
        cx: &mut Context,
        cmds: Commands<NetConfig>,
    ) -> Result<(), Error> {
        for cmd in cmds {
            match cmd {
                Command::AdvanceFrame(inputs) => {
                    // do game logic here!
                    for player in self.players.iter_mut() {
                        player.inputs.push(*inputs.get(player.handle).unwrap());
                    }

                    self.arena.update(
                        &cx.script,
                        &self.players[0].inputs,
                        &self.players[1].inputs,
                    )?;
                }
                Command::Save(save) => {
                    // take a snapshot
                    let snapshot = ArenaSnapshot::snapshot(&self.arena);

                    save.save(snapshot);
                }
                Command::Load(save) => {
                    // load snapshot.
                    save.load().impose(&mut self.arena);
                }
                Command::Event(ev) => match ev {
                    _ => (),
                }
            }
        }

        Ok(())
    }
}

impl Player {
    fn sample_local(&self, cx: &mut Context) -> Option<Inputs> {
        match self.kind {
            PlayerKind::Local(input) => Some(cx.input.sample(input).unwrap_or_default()),
            _ => None,
        }
    }
}

struct NetConfig;

impl backroll::Config for NetConfig {
    type State = ArenaSnapshot;
    type Input = Inputs;
}

struct Player {
    kind: PlayerKind,
    handle: PlayerHandle,
    inputs: InputBuffer,
}

enum PlayerKind {
    Local(InputHandle),
    Remote,
}

#[derive(Clone, Hash)]
struct ArenaSnapshot {
    p1: PlayerSnapshot,
    p2: PlayerSnapshot,
}

#[derive(Clone, Hash)]
struct PlayerSnapshot {
    scope: Scope<'static>,
    state: State,
}

impl ArenaSnapshot {
    /// Takes a snapshot of the arena.
    pub fn snapshot(arena: &Arena) -> ArenaSnapshot {
        ArenaSnapshot {
            p1: PlayerSnapshot::snapshot(&arena.p1),
            p2: PlayerSnapshot::snapshot(&arena.p2),
        }
    }

    /// Imposes this snapshot upon an arena.
    pub fn impose(self, arena: &mut Arena) {
        self.p1.impose(&mut arena.p1);
        self.p2.impose(&mut arena.p2);
    }
}

impl PlayerSnapshot {
    /// Takes a snapshot of the player.
    pub fn snapshot(player: &super::Player) -> PlayerSnapshot {
        PlayerSnapshot {
            scope: player.scope.clone_visible(),
            state: player.state.clone(),
        }
    }

    /// Imposes this snapshot upon a player.
    pub fn impose(self, player: &mut super::Player) {
        player.scope = self.scope;
        player.state = self.state;
    }
}

